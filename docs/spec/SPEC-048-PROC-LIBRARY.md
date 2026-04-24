# SPEC-048: Proc Library

**Status:** Draft
**Date:** 2026-04-23
**Related:** DESIGN-030, NOTE-006, NOTE-007, NOTE-008, SPEC-004, SPEC-022, SPEC-047, SPEC-049, SPEC-050, SPEC-051

## Summary

Introduce `Proc<A>` as a distinct public type constructor and define a minimal `proc` library focused on process-structured composition with minimal immediate runtime interference.

This spec is intentionally narrower than a full process/workflow runtime spec. It defines the public surface consequences of the Proc model; [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) and [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md) own normative runtime and failure details. This spec focuses on:

- the public identity of `Proc<A>`
- the initial proc-library surface
- the algebraic intent of that surface
- explicit deferrals for runtime-heavy features such as mailbox mechanics, spawning, bottom/failure handling, and full execution semantics

## Motivation

Ash now has a clearer separation between:

- pure functions
- `Act<A>` for sequential effectful computation
- workflows, which are increasingly understood operationally as isolated process-like units

A dedicated `Proc<A>` layer is valuable because it provides a process-oriented library/type substrate without forcing the full workflow model, scheduler model, or supervision model to land at the same time.

## 1. Core Position

### 1.1 `Proc<A>` is distinct from `Act<A>`

Normative position:

- `Proc` and `Act` are different monads/algebras
- `Act<A>` remains the sequential effectful-computation type
- `Proc<A>` is the public type of process-structured computation

This spec does not define `Proc` as reducible to `Act`, even though `Proc<Act<A>>` remains a valid nested type.

Design-note refinement: `Act<A>` is currently best understood as the `Effectful`/sequential stratum below `Proc<A>` in the semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

This spec preserves the public `Act`/`Proc` distinction because the strata differ in both admissible power and environment model. The common embedding direction is from sequential effectful computation into process composition. A later proc surface may expose this as an explicit operation such as:

```text
from_act : Act<A> -> Proc<A>
```

This does not imply that `Proc<Act<A>>` implicitly flattens. `Proc<Act<A>>` means “a process computation whose normal result is a suspended effectful computation.”

### 1.2 Public type form

The public type form is:

```text
Proc<A>
```

This spec does **not** introduce a higher-kinded public form such as `Proc<F, A>`.

### 1.3 Opaque execution substrate

`Proc<A>` is a public type identity with an opaque runtime interpretation boundary.
This spec does not expose raw `ActEnv` structure or any lower-level process-environment representation.

## 2. Scope

In scope:

- `Proc<A>` as a draft public type constructor
- `P<A>` as a draft public running-process handle type, used by async `par`/`await`/`join`/message operations
- a `proc` library namespace/module
- initial library surface: `unit`, `bind`, `then`, `yield`, `par`, `await`, `join`, `scatter`, `gather`
- algebraic intent for sequential vs. parallel process composition
- explicit deferral of mailbox/spawn/full runtime-heavy features

Out of scope for this spec:

- workflow lowering into proc
- supervisor hierarchies
- mailbox address/capability model
- exact `run` operational semantics
- exact environment-distribution law for `par`
- role/capability/process enrichment hooks (`with_roles`, `with_capabilities`, etc.)
- process IR / runtime scheduler design
- exact operational bottom/failure handling semantics (`fail`, `with_error`) beyond compatibility constraints in this spec (see SPEC-050)

## 3. Surface Direction

### 3.1 Library namespace

The proc surface is provided through a `proc` library/module.
Inside that namespace, the ordinary unsuffixed names are preferred:

- `unit`
- `bind`
- `then`
- `par`
- `join`
- `scatter`
- `gather`

This mirrors the `act` library choice where unsuffixed names live under `act::...`.

### 3.2 Public type usage

The intended user-facing type spelling is:

```ash
Proc<A>
```

This spec intentionally keeps the public type shape simple.
Any deeper semantic relation to `Act<A>` remains specification prose or implementation detail, not a surface kind/generalization commitment.

The proc surface also reserves a public running-process handle type:

```ash
P<A>
```

`P<A>` is an opaque handle to a running child process that may eventually produce `A`. In the first normative model it is affine/linear: observation operations consume the handle, and [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) owns the first runtime behavior. Any detach/drop/cancel behavior beyond that first model must be specified by a future process-runtime or supervision extension. It is not a special join token; the same handle identity may be used by `await`, `join`, `gather`, `send`, cancellation, and later mailbox/channel operations.

## 4. Initial Library Surface

### 4.1 Core signatures

The initial proc-library surface should support at least the following shapes:

```text
unit   : A -> Proc<A>
bind   : Proc<A> -> (A -> Proc<B>) -> Proc<B>
then   : Proc<A> -> Proc<B> -> Proc<B>
yield  : Proc<Unit>
par    : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
await  : P<A> -> Proc<A>
join   : P<A> -> P<B> -> Proc<(A, B)>
```

Interpretation:

- `unit` lifts a pure value into trivial process structure
- `bind` gives dependent sequential process composition
- `then` sequences while discarding the left value
- `yield` gives the scheduler an explicit cooperative scheduling point within the current process identity
- `par` starts/adopts independent process computations as child `ProcessId`s and returns their running process handles
- `await` observes one running process handle, consuming that handle in the first affine/linear model
- `join` observes/synchronizes two running process handles and has the public shape of a wait-for-all observation combinator; [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) owns handle consumption, terminal-state waiting, and aggregate observation semantics

### 4.2 Derived or library-level combinators

A plausible initial proc library may also expose:

```text
scatter : List<A> -> (A -> Proc<B>) -> Proc<List<P<B>>>
gather  : List<P<A>> -> Proc<List<A>>
```

These are process-oriented collection combinators over running process handles. `scatter` starts/adopts one process per input element and returns handles. `gather` observes a collection of process handles and returns their completed values, or propagates observed process failure according to [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) and [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md).

This spec does not require a unique encoding yet, but they belong in the proc-library vocabulary.

## 5. Algebraic Intent

### 5.1 Sequential face

`bind` and `then` define the sequential/dependent face of `Proc`.
This is the part of the process algebra closest to ordinary monadic composition.

### 5.2 Parallel face

`yield` defines the explicit cooperative scheduling point for long-running process computations. It preserves the current `ProcessId`, does not split environments, and normally returns `Unit`. [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) owns the first normative process-runtime behavior and any later cancellation or scheduler-refusal extension at a `yield` point.

`par` defines the independent process-start face of `Proc`.
This spec treats `par` as more central to the concurrency/process story than `bind`, without removing `bind`.

Working process-composition intuition:

```text
Act . Act . Act          -- sequential effect composition via bind
Proc || Proc || Proc     -- async process start/composition via par
join(P<A>, P<B>)         -- observation/synchronization point
```

Because `par` is asynchronous, a failure inside one of the returned running processes does not retroactively fail the lexical `par` call after it has returned. Such failures are attached to the child `ProcessId` and are observed by later `await`, `join`, `gather`, cancellation, supervision, or workflow boundary operations.

The public-facing identity consequence is:

```text
par(pa, pb) in parent ProcessId P0
  creates child ProcessIds P1 and P2
  runs pa under P1 and pb under P2
  returns (P<A>{process_id = P1}, P<B>{process_id = P2})
```

`BranchId` may exist as an internal runtime identity for scheduling, traces, or effect scopes, but it is not the public identity represented by `P<A>`.

Consequently, a scoped bottom handler around `par` handles only failure to start/admit/create the running process handles:

```ash
with_error {
    par(p1, p2)
} handle {
    _ => fallback_handles
}
```

Branch/process failures should be handled around observation points such as `await`/`join` or inside the processes themselves:

```ash
handles = par(p1, p2);
with_error {
    join(handles.0, handles.1)
} handle {
    _ => fallback_result
}
```

The `with_error`/`fail` syntax above is included only to show the public consequence of async `par`. [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md) owns the normative bottom/failure and scoped-handling model. NOTE-008 is the historical design note promoted by SPEC-050 for overlapping content.

### 5.3 Applicative / monoidal reading

This spec explicitly leaves room for `Proc` to be understood not only as a monad, but also through applicative/monoidal structure.
That is likely the more natural interface for independent concurrency than monadic sequencing alone.

This spec does not require higher-kinded interface syntax to land now. The point is semantic guidance for the proc-library design.

## 6. Runtime Boundary

This spec deliberately minimizes runtime commitments.

### 6.0 Coordination / non-interference rule

Act semantics are active concurrent work. This spec must therefore avoid claiming ownership of Act runtime behavior.

This spec owns only:

- the public identity of `Proc<A>`
- the proc-library surface
- the algebraic intent of proc combinators
- explicit deferrals needed to keep later runtime integration open

This spec does **not** own:

- hidden `ActEnv` threading semantics
- the concrete Act runtime carrier meaning
- exact `run` semantics
- concurrency scheduler or mailbox runtime design

Any later proc/runtime implementation must integrate with the landed Act semantics rather than preempt them here.

### 6.1 Not required in the initial proc-library slice

This spec does not require immediate landing of:

- `spawn`
- `run`
- `send`
- `receive`
- mailbox/channel runtime structures
- scheduler/pinning policies
- direct public construction of process IDs / workflow addresses

Those operations remain explicitly deferred to a later proc/runtime slice.

### 6.2 Compatibility requirement

Even though these features are deferred, the proc-library design must remain compatible with later additions of:

- process-local mailbox support
- channel-based communication
- process execution via `run`
- workflow elaboration into proc machinery
- identity-indexed process handles usable by `join`, `gather`, message send/receive, cancellation, and workflow reporting

## 7. Relation to Workflow

This spec intentionally avoids defining workflow in terms of proc today.
However, compatibility with that future direction is a requirement.

Compatibility statement:

- workflow remains unchanged by this spec
- proc is introduced as its own library/type layer
- later workflow elaboration into `Proc` is permitted and should not be blocked by the initial proc-library design

## 8. Related Runtime and Follow-On Surfaces

The proc surface is now split across these normative owners:

1. [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) owns `P<A>` runtime representation, process identity discipline, child-environment projection, `yield`, async `par`, `await`, `join`, `scatter`, and `gather` observation semantics.
2. [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md) owns precise `par` start/admission failure vs process-observation failure rules, `fail`, `with_error`, and aggregate observed-failure representation.
3. [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md) owns the workflow boundary: admission, governance, reporting, `WorkflowFailure`, and lower-failure reinterpretation.

Remaining follow-on surfaces not defined by this SPEC-048/SPEC-049/SPEC-050/SPEC-051 batch are:

1. `run` semantics and host boundary
2. mailbox/channel model
3. `send` / `receive` process operations over process handles
4. explicit detach/drop/cancel semantics for unobserved handles
5. supervisor/monitor/shared-handle semantics

## 9. Implementation Guidance

### 9.1 Parser/type-system expectation

The intended type shape is an ordinary constructor-form type:

```text
Proc<A>
```

No new exotic syntax is required for the type itself beyond ordinary type-constructor usage.
If the current parser/typechecker cannot yet represent the necessary public type identity cleanly, that limitation should be handled as an implementation prerequisite rather than by broadening the public proc type form.

### 9.2 Keep the first slice library-first

The recommended implementation order is:

1. establish the public `Proc<A>` type identity
2. establish the public `P<A>` process-handle type identity
3. establish the proc library surface and typing intent
4. implement runtime backing according to SPEC-049/SPEC-050 only where needed by the selected surface slice, while leaving `run`, mailbox/channel, cancellation, and supervision follow-ons out of SPEC-048

This preserves the "minimal runtime interference" goal.

## 10. Current Answers and Open Questions

1. Public type surface: keep `Act<A>` and `Proc<A>` for now. This remains open to future extension with explicit `Pure`/`Workflow` computation names or indexed `Comp<K, A>` only if implementation pressure justifies it.
2. Process-handle spelling: current preference is `Process<A>`, but this spec continues to use `P<A>` as short draft notation until naming is finalized.
3. `par` identity creation: `par` creates child `ProcessId`s and returns process handles for those identities. `BranchId` remains internal/subordinate.
4. `P<A>` ownership: first normative model is affine/linear. Observation operations consume handles; `dup`, shared handles, monitors, supervisors, replayable observation, and multiple observers are deferred.
5. `join` shape: `join` is binary and waits for both handles to terminate before returning success or aggregate failure. The core primitive is `await` or an equivalent single-handle observation operation, from which `join` and `gather` may be built.
6. Which proc combinators can be landed as library declarations before full runtime backing exists? SPEC-049/SPEC-050 define the first runtime/failure contract for `yield`, async `par`, `await`, `join`, `scatter`, and `gather`.
7. How should `run` be specified later so `Proc` stays distinct from `Act` while still supporting the embedding of sequential `Act<A>` computations into process composition?
8. How should later mailbox/channel and supervision operators compose with `scatter`/`gather` without weakening the first affine/linear observation model?
9. Which additional bottom/failure affordances, if any, belong in proc-specific libraries rather than the general SPEC-050 failure model?

## Changelog

### 2026-04-24

- Aligned the proc library draft with the resolved `par` semantics slice. Added `yield` and `await`, made first-pass `P<A>` handles affine/linear, specified that `par` creates child `ProcessId`s rather than public branch handles, and linked process failure behavior to SPEC-049 process-runtime and SPEC-050 operational-bottom specs.

- Aligned the proc library draft with DESIGN-030's semantic tower. Added `P<A>` running process handles, changed `par` to asynchronous process start returning handles, added `join`, updated `scatter`/`gather` around handles, clarified `Act<A>` as the effectful/sequential stratum below `Proc<A>`, recorded current answers for public type surface/process-handle naming/`par` identity creation/`join` shape, and delegated exact `fail`/`with_error` bottom semantics to SPEC-050.

### 2026-04-23

- Initial draft capturing a minimal proc-library/type slice distinct from workflow and `Act`.
