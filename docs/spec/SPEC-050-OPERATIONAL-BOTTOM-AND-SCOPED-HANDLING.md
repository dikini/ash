# SPEC-050: Operational Bottom and Scoped Handling

**Status:** Draft
**Date:** 2026-04-24
**Related:** DESIGN-030, SPEC-004, SPEC-025, SPEC-047, SPEC-048, SPEC-049, SPEC-051, NOTE-008
**Supersedes:** NOTE-008 for overlapping operational-bottom and scoped-handling semantics; SPEC-004 lines that treated surfaced `Pure` bottom as future work for bottom/failure attribution

## Summary

This specification defines Ash's first normative operational-bottom model across the current semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

The key distinction is:

```text
Result<A, E> = domain-level value protocol
fail e       = operational bottom / non-completion
```

`Result<A, E>` is an ordinary value-level return type. `fail e` terminates the current computation unsuccessfully on the operational channel. `with_error { ... } handle { ... }` is the scoped recovery form for operational failures routed to that dynamic/tower/identity scope.

## 1. Scope and Authority

### 1.1 In scope

This spec defines:

1. operational bottom as non-completion;
2. the difference between operational failure and domain `Result` values;
3. the conceptual `OperationalFailure` object;
4. `fail` typing and propagation;
5. `with_error` syntax, typing, matching, and propagation;
6. tower/entity-indexed failure attribution;
7. failure routing for effectful, process, and workflow boundaries;
8. process-observation failure hooks used by [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md);
9. workflow-boundary reinterpretation hooks used by [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md).

### 1.2 Out of scope

This spec does not define:

1. the full process runtime; see [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md);
2. workflow governance and `WorkflowFailure` reporting; see [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md);
3. all parser implementation details for `fail`/`with_error`;
4. concrete Rust error enums;
5. exception-like stack unwinding implementation strategy;
6. a user-level checked-exception system;
7. supervisor policies, retries, monitors, or compensation protocols.

### 1.3 Normative vs informative

Unless marked informative, sections are normative. Conceptual object shapes are semantic contracts, not required Rust layouts.

### 1.4 Authority relative to SPEC-004 and SPEC-025

[SPEC-004](SPEC-004-SEMANTICS.md) and [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) remain the canonical references for legacy workflow small-step outcomes such as `Return(...)` and `Reject(...)` and for effect classification over the existing effect lattice.

This spec owns the separate operational-failure channel introduced by `fail`, `with_error`, tower/entity-indexed `OperationalFailure`, and process-observation failure. Where older SPEC-004 prose says surfaced `Pure` bottom is not yet normative, this spec is the broader corpus change that makes bottom/failure attribution normative. It does not change SPEC-004's effect lattice or require legacy workflow small-step rules to be rewritten in this pass.

## 2. Operational Failure vs Domain Result

A computation returning `Result<A, E>` completes normally with a value whose shape is either `Ok(a)` or `Err(e)`.

A computation that performs `fail e` does not complete normally. It raises operational bottom carrying `e` as payload.

Example:

```ash
fn parse(s: String) -> Result<Int, ParseError> {
    if user_input_is_invalid(s) {
        return Err(ParseError {});
    }

    if internal_invariant_is_broken {
        fail InternalInvariantBroken {};
    }

    return Ok(value);
}
```

Here `Err(ParseError {})` is a normal value. `fail InternalInvariantBroken {}` is operational non-completion.

Conformance rule: an implementation must not silently encode all operational failures as user-level `Err` values unless the source program explicitly handles and returns such values.

## 3. Operational Failure Object

Operational failures are tower-indexed and entity-indexed.

Conceptual shape:

```text
OperationalFailure {
  tower: TowerLevel,
  entity: EntityId,
  payload: Value,
  payload_type: Type,
  cause: Option<OperationalFailure>,
  evidence: FailureEvidence,
}
```

Minimum tower levels:

```text
Pure
Effectful
Proc
Workflow
```

Minimum entity identities:

```text
LexicalFrameId
EffectScopeId
ProcessId
WorkflowId
RunId
```

Examples:

```text
(Pure, LexicalFrameId, DivideByZero)
(Effectful, EffectScopeId, PolicyDenied)
(Effectful, EffectScopeId, ProviderUnavailable)
(Proc, ProcessId, ProcessCancelled)
(Proc, ProcessId, ObservedProcessFailed)
(Workflow, WorkflowId, EnsuresViolation)
```

The exact internal representation may vary, but conforming implementations must preserve enough information for matching, provenance, reporting, and lower-cause evidence.

## 4. `fail`

### 4.1 Surface form

Preferred surface form:

```ash
fail error;
```

or, where the grammar admits expression-position bottom:

```ash
fail error
```

The parser may initially implement one syntactic form. The semantic contract is the same: terminate the current computation unsuccessfully with operational failure carrying `error`.

### 4.2 Typing

Typing intuition:

```text
error : E
────────────────────────
fail error : A
```

for any expected normal result type `A`.

This does not mean `fail` returns a value of type `A`. It means bottom is compatible with any expected normal type because the computation does not complete normally.

### 4.3 Attribution

`fail error` is attributed to the current semantic tower and identity:

- pure expression failure: `(Pure, current LexicalFrameId, error)`;
- effectful/Act failure: `(Effectful, current EffectScopeId, error)`;
- proc failure: `(Proc, current ProcessId, error)`;
- workflow-governance failure: `(Workflow, current WorkflowId, error)`.

If a lower-stratum failure escapes into a higher stratum, the higher stratum may wrap or reinterpret it while preserving the lower failure as cause/evidence.

At a workflow boundary, escaped lower operational failures are converted into workflow terminal outcomes as specified by [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md). Body-internal handlers catch routed `OperationalFailure` values before that boundary conversion. Workflow admission/completion governance failures are workflow terminal failures, not ordinary body-routed failures.

### 4.4 Propagation

Unmatched operational failure propagates outward through the current dynamic/tower boundary until:

1. it reaches a matching `with_error` scope;
2. it becomes a terminal process failure;
3. it becomes an observed process failure at `await`/`join`/`gather`;
4. it reaches a workflow boundary and is reported or reinterpreted as workflow failure;
5. it reaches the host boundary and is reported as uncaught operational failure.

## 5. `with_error` Scoped Handling

### 5.1 Surface form

Preferred form:

```ash
with_error {
    body
} handle {
    Pattern1 => expr1;
    Pattern2 => expr2;
    _ => fallback;
}
```

Design decisions:

1. `handle` is multi-arm and match-like.
2. Arms are tested in source order.
3. First matching arm wins.
4. `_` is the primary catch-all.
5. `*` is not a catch-all.
6. `otherwise` may be future sugar but is not the primary form.

### 5.2 Typing

Typing rule:

```text
body : A
arm_i : A for every handler arm
────────────────────────────────────────
with_error { body } handle { arms } : A
```

The handler body and every arm must produce the same normal result type. A handler arm may itself `fail`, in which case it is bottom-compatible with the expected arm type.

### 5.3 Matching target

The semantic matching target is the full `OperationalFailure` object.

Payload shorthand is allowed:

```ash
with_error {
    x / y
} handle {
    DivideByZero => 0;
    _ => fail UnknownMathFailure;
}
```

The shorthand above matches as if it inspected `failure.payload`.

Full-form matching may be used when needed:

```ash
with_error {
    invoke("fs", "read", [path])
} handle {
    Failure { tower: Effectful, payload: PolicyDenied { reason } } => fail reason;
    Failure { payload: ProviderUnavailable { provider } } => default_value;
    _ => fail UnhandledFailure;
}
```

A conforming implementation may introduce full-form matching after shorthand matching, but must not make shorthand ambiguous with ordinary value patterns.

### 5.4 Routed scope

A handler catches only failures routed through its dynamic/tower/identity scope.

It does not retroactively catch failures in already-started child processes whose handles have been returned, unless the handler surrounds an observation operation that raises an observed process failure.

## 6. Pure-Level Bottom

Pure computations may bottom even though they have no effect environment.

Examples:

1. division by zero;
2. failed partial operation;
3. non-exhaustive match, if permitted;
4. explicit `fail`;
5. implementation-defined runtime invariant failure.

Pure bottom is attributed to the current lexical/evaluation identity and propagates to the nearest routed handler or enclosing effectful/process/workflow boundary.

## 7. Effectful / Act Failure

Effect-level failures arise from effect execution:

1. provider unavailable;
2. policy denied;
3. invalid action;
4. invalid arguments;
5. timeout at provider invocation level;
6. capability violation;
7. provider execution failure.

Conceptual internal Act shape:

```text
Act<A> ~= EffEnv -> (EffEnv, A)
```

Operationally, an implementation may model this as:

```text
EffEnv -> Result<(EffEnv, A), OperationalFailure>
```

The internal `Result` above is implementation notation for the operational channel. It is not the same as Ash user-level `Result<A, E>`.

An Act-level failure is attributed to the current `EffectScopeId` unless it is immediately rewrapped by a higher stratum.

## 8. Process Failure and Observation

### 8.1 Process terminal failure

If an unhandled operational failure escapes a process computation, the process enters:

```text
Failed(failure)
```

where terminal process state must record the failed `ProcessId` and preserve the lower failure as cause/evidence.

A conforming implementation may store this as a Proc-level wrapper `OperationalFailure` or as terminal-state metadata. Regardless of representation, observation through `await`, `join`, or `gather` must expose a stable observed-process failure payload with both child `ProcessId` and lower failure evidence.

### 8.2 `par` failure timing

For async `par`:

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

A handler around the lexical `par` call catches only start/admission/handle-creation failures:

```ash
with_error {
    par(p1, p2)
} handle {
    _ => fallback_handles;
}
```

Failures inside the child processes after handles are returned are not routed to this lexical handler. They are child process failures.

### 8.3 Observation failure

`await`, `join`, and `gather` observe process terminal states.

If a child process failed, the observation operation raises an observed process failure in the observing process. That observed failure must preserve:

1. observing process identity;
2. observed child `ProcessId`;
3. lower child failure object;
4. whether the observed failure was single or aggregate.

Conceptual payloads:

```text
ObservedProcessFailed {
  child: ProcessId,
  failure: OperationalFailure,
}

ObservedProcessFailures {
  children: List<(ProcessId, OperationalFailure)>,
}
```

### 8.4 Aggregation

`join` and `gather` are wait-for-all observation barriers. If multiple observed children fail, the observation operation raises an aggregate failure preserving all failed child identities and lower failures.

A conforming implementation must not discard sibling failures merely because one failure is encountered first.

## 9. Workflow Boundary Reinterpretation

A workflow boundary may reinterpret unhandled lower-level failures as workflow failures, but it must preserve lower-level cause/evidence.

Example conceptual boundary outcome, using the shape owned by SPEC-051:

```text
WorkflowFailed(
  WorkflowFailure {
    workflow_id: WorkflowId,
    run_id: RunId,
    kind: BodyFailureEscaped,
    cause: Some(OperationalFailure),
    evidence: WorkflowFailureEvidence,
  },
  WorkflowReport,
)
```

The exact `WorkflowFailure` taxonomy, outcome shape, and reporting behavior are owned by [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md).

This spec requires only that lower-level evidence is not erased.

## 10. Handler Examples

### 10.1 Start/admission failure around `par`

```ash
with_error {
    proc::par(left, right)
} handle {
    SchedulerRefused => fallback_handles;
    ResourceSplitDenied => fallback_handles;
    _ => fail CannotStartChildren;
}
```

This catches failures before handles are returned.

### 10.2 Observation failure around `join`

```ash
handles = proc::par(left, right);
with_error {
    proc::join(handles.0, handles.1)
} handle {
    ObservedProcessFailed { child, failure } => fallback_pair;
    ObservedProcessFailures { children } => fallback_pair;
    _ => fail CannotJoinChildren;
}
```

This catches failures raised by observing already-started child processes.

### 10.3 Returning a domain result after handling operational failure

```ash
with_error {
    risky_computation()
} handle {
    KnownFailure { reason } => Err(reason);
    _ => fail UnknownOperationalFailure;
}
```

The handler may recover operational failure into a normal domain-level `Err` value if the surrounding expected type is `Result<A, E>`.

## 11. Conformance Requirements

A conforming implementation must:

1. preserve the distinction between domain `Result` values and operational failure;
2. treat `fail` as non-completion compatible with any expected normal type;
3. attribute failures to tower/entity identities;
4. preserve cause/evidence when wrapping or reinterpreting failures;
5. implement `with_error` as scoped matching over operational failures;
6. require all handler arms to produce the same normal result type;
7. propagate unmatched failures unchanged except for permitted higher-boundary wrapping with cause preservation;
8. route post-handle child-process failures to observation operations, not to the lexical `par` handler;
9. aggregate multiple child failures at wait-for-all observation barriers;
10. avoid silently converting uncaught operational failures into user-level `Err` values.

## 12. Deferred Questions

1. Exact parser representation of `fail` as expression vs statement.
2. Exact parser representation of full-form `Failure { ... }` patterns.
3. Whether pure `fail` is surfaced immediately or introduced first as a semantic lowering target.
4. Whether bottom has an explicit internal `Bottom` type or is represented by expected-type checking only.
5. Full taxonomy of standard failure payloads.
6. Source-level syntax for aggregate observed process failures.
7. Whether workflow admission/completion failures can be handled inside workflow bodies or only outside the workflow boundary.

## Changelog

### 2026-04-24

- Initial draft promoting NOTE-008 into a normative operational-bottom and scoped-handling spec, including `fail`, `with_error`, tower/entity-indexed failures, process-observation failure, aggregation, and workflow-boundary reinterpretation hooks.
