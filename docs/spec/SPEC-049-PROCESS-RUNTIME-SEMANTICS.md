# SPEC-049: Process Runtime Semantics

**Status:** Draft
**Date:** 2026-04-24
**Related:** DESIGN-030, SPEC-048, SPEC-050, SPEC-051, SPEC-047, SPEC-025, SPEC-004, NOTE-007, NOTE-008
**Promotes:** NOTE-007 for overlapping process identity, runtime environment component, and child-environment projection semantics

## Summary

This specification defines the first normative process-runtime semantics for `Proc<A>` execution.

`SPEC-048` owns the public proc library/type surface. This spec owns the runtime meaning of process identity, process handles, child-environment projection, cooperative scheduling, async `par`, `await`, `join`, `scatter`, and `gather`.

The first normative model is intentionally conservative:

- `par` creates child `ProcessId`s.
- `P<A>` is an opaque child-process handle, not a branch token.
- `P<A>` is affine/linear in the first model.
- `await`, `join`, and `gather` consume handles.
- `join` and `gather` wait for all observed children and aggregate failures.
- `yield : Proc<Unit>` is the explicit cooperative scheduling point.
- `BranchId` is internal/subordinate and not the public identity of a process handle.

## 1. Scope and Authority

### 1.1 In scope

This spec defines:

1. process runtime identities;
2. process lifecycle states;
3. the semantic meaning of `P<A>` handles;
4. child environment projection for `par`/`scatter`;
5. cooperative `yield` behavior;
6. async `par` start/admission semantics;
7. `await`, `join`, and `gather` observation semantics;
8. process-failure observation and aggregation hooks;
9. the boundary between process runtime and workflow governance.

### 1.2 Out of scope

This spec does not define:

1. the public type/library surface of `Proc<A>`; see [SPEC-048](SPEC-048-PROC-LIBRARY.md);
2. the general operational-bottom and `with_error` surface; see [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md);
3. workflow admission, roles, contracts, and `WorkflowFailure`; see [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md);
4. mailbox/channel syntax and queue layout;
5. supervisors, monitors, `dup`, shared handles, replayable observations, or multiple observers;
6. concrete scheduler implementation, fairness theorem, thread model, or executor API;
7. concrete Rust structs or storage layout.

### 1.3 Normative vs informative

Unless a section is explicitly marked informative, it is normative.

Implementation examples, Rust-like pseudocode, and storage sketches are informative. Semantic laws, lifecycle states, failure timing rules, and handle-consumption rules are normative.

## 2. Semantic Tower Position

Ash uses the current semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

`Proc<A>` adds process identity, child-process creation, process-local runtime components, scheduling points, and process observation above sequential `Act<A>` computation. It does not add workflow governance; governance belongs to `Workflow` and [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md).

Top-down runtime admission flows from outside runtime or workflow into process runtime, then into effectful computation:

```text
outside runtime / workflow
  admits Proc runtime execution
Proc
  invokes Effectful / Act computation
Effectful / Act
  calls Pure functions
```

Lower strata must not manufacture higher-stratum authority. A process may use only the capabilities, providers, policies, resources, and role/capability context projected into its environment by its parent process or workflow admission boundary.

## 3. Runtime Domains

### 3.1 Identities

Minimum identity vocabulary:

```text
RunId
WorkflowId
ProcessId
BranchId
EffectScopeId
LexicalFrameId
```

A process runtime execution is rooted in a `RunId` and usually a `WorkflowId` or host-created process root. Each running process has exactly one `ProcessId`.

`BranchId` may be used internally for scheduling, tracing, effect-scope grouping, or runtime bookkeeping. It is subordinate to a `ProcessId` and is not the public identity of `P<A>`.

### 3.2 Process handle

The public handle type from [SPEC-048](SPEC-048-PROC-LIBRARY.md) is:

```text
P<A>
```

Runtime meaning:

```text
P<A> = opaque handle to ProcessId expected to complete normally with A
```

A handle includes, or references, at least:

```text
ProcessHandle<A> {
  process_id: ProcessId,
  expected_result_type: A,
  observation_state: Unobserved | Consumed,
}
```

The structure above is informative. Normatively, the handle identifies a process and tracks whether it remains available for consuming observation.

### 3.3 First-pass ownership discipline

In the first normative model, `P<A>` is affine/linear:

1. A handle may be moved.
2. A handle may be consumed by exactly one observation operation.
3. `await`, `join`, and `gather` consume the handles they observe.
4. There is no implicit cloning, sharing, replay, or second observation.
5. Dropping an unobserved handle is not specified as success, cancellation, or detach by this spec; explicit detach/drop/cancel semantics remain a future process-runtime/supervision extension.

A conforming implementation must reject or prevent use-after-consume at the relevant static or dynamic boundary.

## 4. Process Lifecycle

### 4.1 Lifecycle states

A process is in one of these semantic lifecycle states:

```text
Admitting
Running
Yielded
Succeeded(value)
Failed(OperationalFailure)
Cancelled(OperationalFailure)
```

`Admitting` is the pre-running phase where the runtime creates identity, derives child environment, registers handles, and asks the scheduler/runtime to accept the process.

`Running` means the process computation is active or ready to be scheduled.

`Yielded` means the process cooperatively gave control back to the scheduler and may later resume.

`Succeeded(value)` is normal completion.

`Failed(failure)` is operational non-completion as specified by [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md).

`Cancelled(failure)` is operational non-completion caused by cancellation. Detailed cancellation initiation, propagation, and cleanup are deferred; cancellation failures still carry process identity.

### 4.2 Terminal states

The terminal process states are:

```text
Succeeded(value)
Failed(failure)
Cancelled(failure)
```

A terminal process must not resume.

Observation operations may observe terminal state immediately or wait until terminal state is reached.

## 5. Process Environment Model

### 5.1 Identity-indexed components

Process runtime context is not one ambient cloneable map. It is an identity-indexed component set, approximately:

```text
(TowerLevel, EntityId, ComponentType, Key) -> Component
```

This spec relies on the model recorded in NOTE-007 and hardens the process-runtime requirements needed by `par`, `await`, `join`, `scatter`, and `gather`.

### 5.2 Parent process environment

A running process has a process environment containing or indexing at least:

1. current `ProcessId`;
2. optional parent `ProcessId`;
3. child-process registry;
4. scheduler/runtime placement metadata;
5. process-local cancellation scope;
6. process-local failure scope;
7. handle observation registry;
8. process-local resources;
9. effect-environment projection hooks for `Act` execution;
10. provenance/audit append sinks.

### 5.3 Child environment projection

`par` and `scatter` create child process environments by typed projection, not by monolithic context cloning.

Normative operation shape:

```text
derive_child_env(parent_env, child_process_id, child_index) -> ChildEnv | OperationalFailure
```

Requirements:

1. Child environments must be equal-or-less-authorized than the parent environment, never wider.
2. Read-only components may be shared as read-only.
3. Child-local components must be freshly allocated or identity-indexed to the child `ProcessId`.
4. Append-only provenance/audit components may append into a parent-visible sink if ordering/evidence rules are preserved.
5. Linear or exclusive resources must not be duplicated; they must be explicitly partitioned, moved, or rejected during admission.
6. Process failure channels are child-local until observed or reported through an enclosing boundary.
7. Effect invocation scopes are child-local even if they reference shared provider/capability definitions.

### 5.4 Projection failure

Child environment projection may fail before a child process starts. Examples:

- parent lacks authority to start a child;
- capability surface cannot be projected;
- exclusive resource cannot be split;
- scheduler refuses admission;
- handle allocation or child registry registration fails.

Such failures are start/admission/handle-creation failures. They are failures of the parent process's current operation and may be caught by a `with_error` scope around `par` or `scatter`, as specified by [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md).

## 6. `yield : Proc<Unit>`

### 6.1 Meaning

`yield` is an explicit cooperative scheduling point in the current process.

Normal behavior:

```text
yield : Proc<Unit>
```

It preserves:

1. current `ProcessId`;
2. current process-local environment identity;
3. lexical bindings;
4. process handle ownership state.

It does not create child processes, split environments, observe handles, or complete the process.

### 6.2 Failure/cancellation point

An implementation may use `yield` as a point where cancellation or scheduler refusal is surfaced, if cancellation has already been requested or the scheduler cannot continue the process.

If surfaced, the result is an operational failure at the current `ProcessId`, not a domain-level `Result` value.

## 7. `par` Semantics

### 7.1 Signature

From [SPEC-048](SPEC-048-PROC-LIBRARY.md):

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

### 7.2 Identity law

When evaluated in parent process `P0`, `par(pa, pb)` creates two child process identities:

```text
P1 = fresh_child_process_id(parent = P0, child_index = 0)
P2 = fresh_child_process_id(parent = P0, child_index = 1)
```

It then derives child environments:

```text
env1 = derive_child_env(parent_env, P1, 0)
env2 = derive_child_env(parent_env, P2, 1)
```

`par` admission is all-or-none in the first normative model. A conforming implementation must complete all pre-run admission work before either child can execute user code:

1. allocate both child identities;
2. derive both child environments;
3. allocate and register both child records;
4. allocate both public handles;
5. only then make both children runnable.

If any pre-run admission step fails, neither child may have executed. This prevents orphaned children with no returned affine handle.

If all admission/projection/registration steps succeed, `par` starts or schedules `pa` under `P1` and `pb` under `P2`, and returns:

```text
(P<A>{process_id = P1}, P<B>{process_id = P2})
```

### 7.3 Failure timing

A `with_error` around the lexical `par` call catches only failures that occur before handles are successfully returned:

1. child identity allocation failure;
2. child environment projection failure;
3. resource split/partition failure;
4. scheduler/admission refusal;
5. child registry or handle allocation failure.

After handles are returned, failures inside `P1` or `P2` belong to those child `ProcessId`s. They do not retroactively fail the already-returned lexical `par` call. They are observed by `await`, `join`, `gather`, cancellation/supervision, or workflow boundary reporting.

### 7.4 No public branch handles

`par` may create internal branch identities for traces, scheduling, or implementation artifacts. Those identities are not public handles. `P<A>` denotes process identity, not branch identity.

## 8. `await` Semantics

### 8.1 Signature

```text
await : P<A> -> Proc<A>
```

`await` is the single-handle observation primitive. If a different surface name is chosen later, it must preserve this semantic role.

### 8.2 Consuming observation

`await(h)` consumes `h`.

If `h` identifies process `P1`:

1. If `P1` has already succeeded with value `a`, `await(h)` returns `a`.
2. If `P1` has failed or was cancelled, `await(h)` raises an observed process failure carrying `P1` as source identity.
3. If `P1` is not terminal, `await(h)` waits/suspends the observing process until `P1` reaches terminal state.

The observing process may be blocked/suspended while waiting. Such waiting is not semantic stuckness.

### 8.3 Observed process failure

If `P1` fails with operational failure `f`, `await(h)` raises an operational failure at the observing process whose payload/cause preserves `f` and `P1` identity.

The exact failure object shape is owned by [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), but the preservation requirement belongs here.

### 8.4 Observer cancellation while waiting

Entering `await`, `join`, or `gather` consumes the relevant handles immediately.

If the observing process is cancelled while blocked in an observation operation, the observing process terminates with cancellation at the observer `ProcessId`. Children are not implicitly cancelled by this first-pass rule unless an explicit future cancellation policy says so.

Because the public handles have already been consumed, the runtime must retain enough child completion/failure evidence for workflow-boundary or future supervision reporting. This evidence retention is not replayable public observation and does not create a second `P<A>` observer.

## 9. `join` Semantics

### 9.1 Signature

```text
join : P<A> -> P<B> -> Proc<(A, B)>
```

### 9.2 Wait-for-both rule

`join(h1, h2)` consumes both handles and waits until both target processes are terminal.

It is not equivalent to sequentially awaiting `h1` and then `h2` with fail-fast short-circuiting.

### 9.3 Outcomes

Let `h1` identify `P1` and `h2` identify `P2`.

1. If `P1` succeeds with `a` and `P2` succeeds with `b`, `join(h1, h2)` returns `(a, b)`.
2. If exactly one child fails or is cancelled, `join(h1, h2)` raises an observed process failure preserving that child's `ProcessId` and lower failure evidence.
3. If both children fail or are cancelled, `join(h1, h2)` raises an aggregate observed process failure preserving both child `ProcessId`s and both lower failures.

A conforming implementation must not abandon a still-running sibling merely because the other sibling failed, unless an explicit future cancellation policy says so. First-pass `join` waits for both terminal states.

## 10. `scatter` and `gather`

### 10.1 `scatter`

```text
scatter : List<A> -> (A -> Proc<B>) -> Proc<List<P<B>>>
```

`scatter(xs, f)` creates one child `ProcessId` per input element, using stable list order as child index, and returns handles in the same order.

Admission/projection failure before handles return is failure of the `scatter` operation in the parent process.

`scatter` admission is all-or-none in the first normative model. A conforming implementation must allocate identities, derive environments, register child records, and allocate handles for every element before any child can execute user code. If any pre-run admission step fails, no child may have executed and no partial handle list is returned.

### 10.2 `gather`

```text
gather : List<P<A>> -> Proc<List<A>>
```

`gather(handles)` consumes all handles, waits for all target processes to terminate, and then:

1. returns values in handle-list order if all succeed;
2. raises one observed process failure if exactly one target fails/cancels;
3. raises an aggregate observed process failure if more than one target fails/cancels.

Failure aggregation must preserve each failed child's `ProcessId` and lower failure evidence.

## 11. Process Failure and Scoped Handling

Process computations may fail operationally. `fail e` inside a process terminates that process unsuccessfully unless handled inside the process before it escapes.

A `with_error` scope inside a child process can catch failures routed to that child process's current dynamic scope.

A `with_error` scope around `await`, `join`, or `gather` in an observing process can catch observed process failures raised by those observation operations.

A `with_error` scope around `par` catches only start/admission/handle-creation failure before handles are returned.

The syntax, typing, and failure object model for `fail` and `with_error` are defined in [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md).

## 12. Relation to Workflow

A workflow may admit and govern process execution, but `Proc` does not itself own workflow governance.

This spec owns process identity and observation semantics below workflow governance. [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md) owns:

1. workflow admission;
2. role/capability context admission;
3. workflow `requires`/`ensures`;
4. workflow reporting;
5. `WorkflowFailure` construction;
6. reinterpretation of unhandled lower-level failures at workflow boundaries.

A process failure observed inside workflow execution remains a process-level failure until the workflow boundary reinterprets it or reports it.

## 13. Conformance Requirements

A conforming implementation of this first process-runtime model must:

1. treat `P<A>` as an opaque identity-bearing process handle;
2. prevent or reject repeated consuming observation of the same handle;
3. create child `ProcessId`s for `par` operands;
4. derive child environments by component-wise projection, not monolithic context cloning;
5. preserve child failure identity through `await`, `join`, and `gather`;
6. implement `join`/`gather` as wait-for-all observation barriers;
7. aggregate multiple observed failures rather than silently discarding all but the first;
8. keep `BranchId` internal/subordinate;
9. keep workflow governance semantics out of `Proc` except through explicit admission/projection inputs from a workflow boundary.

## 14. Deferred Questions

1. Exact public spelling of `P<A>` vs `Process<A>`.
2. Exact `run : Proc<A> -> ?` semantics and host boundary.
3. Explicit detach/drop/cancel semantics for unobserved linear handles.
4. Supervisor/monitor/shared-handle semantics.
5. Mailbox/channel syntax and ownership model.
6. Scheduler fairness, placement, and time semantics.
7. Internal cleanup policy for partially allocated `scatter` admission records when pre-run admission fails. Semantic partial execution remains closed by the all-or-none rule in §10.1.

## Changelog

### 2026-04-24

- Initial draft defining process identities, affine/linear process handles, child-environment projection, `yield`, async `par`, `await`, wait-for-all `join`, `scatter`/`gather`, and the process/workflow/failure boundary.
