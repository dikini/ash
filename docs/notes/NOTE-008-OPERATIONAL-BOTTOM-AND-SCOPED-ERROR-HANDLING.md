# NOTE-008: Operational Bottom and Scoped Error Handling

**Date:** 2026-04-24
**Status:** Draft
**Priority:** High — records the current failure model for `Pure`, `Act`, `Proc`, and workflow semantics
**Related:** DESIGN-030, SPEC-048, SPEC-047, NOTE-006, NOTE-007

## 1. Purpose

This note captures the current working model for operational failure in the Ash semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

The key distinction is:

```text
Result<A, E> = domain-level value semantics
fail e       = operational bottom / non-completion
```

Operational failure should not be silently caught and panicked. It should propagate transparently unless explicitly handled in source code.

## 2. Bottom

Every stratum may signal operational bottom.

Pure computations can bottom even though they have no effect environment:

- division by zero
- failed partial operation
- non-exhaustive match, if permitted
- explicit `fail`

At the Pure level, `fail` signals/returns bottom.

Typing intuition:

```text
fail e : A
```

for any expected normal result type `A`.

This does not mean `fail` returns a value. It means bottom is compatible with any expected result type because the computation does not complete normally.

## 3. Operational Failure vs Domain Result

A function returning `Result<A, E>` chooses a domain-level return protocol.

A function that bottoms does not return normally.

Example distinction:

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

`Err(ParseError {})` is a normal value.
`fail InternalInvariantBroken {}` is operational non-completion.

## 4. Failure Shape

Operational failures should be tower-indexed and identity-indexed.

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

Examples:

```text
(Pure, LexicalFrameId, DivideByZero)
(Effectful, EffectScopeId, PolicyDenied)
(Effectful, EffectScopeId, ProviderUnavailable)
(Proc, ProcessId, ProcessCancelled)
(Proc, ProcessId, ObservedProcessFailed)
(Workflow, WorkflowId, EnsuresViolation)
```

Workflow boundaries may reinterpret unhandled lower-level failures as `WorkflowFailure`, but the lower-level cause should remain available as evidence.

## 5. Effect-Level Failure Channel

Effect-level failures arise from effect execution:

- provider unavailable
- policy denied
- invalid action
- invalid arguments
- timeout at provider invocation level
- capability violation

Conceptual Act model:

```text
Act<A> ~= EffEnv -> (EffEnv, A)
```

Operationally:

```text
EffEnv -> Result<(EffEnv, A), EffectFailure>
```

The internal `Result` here is implementation notation for the operational channel. It is not the same as Ash user-level `Result<A, E>`.

## 6. Surface `fail`

Surface `fail` should behave like `return`, but for bottom:

```ash
fail error;
```

Meaning:

```text
terminate the current computation unsuccessfully with operational bottom carrying `error`
```

`fail` is not `Err`.
`fail` does not force every function to return `Result`.
The ordinary return type describes normal completion.

## 7. Scoped Error Handling

Preferred surface form:

```ash
with_error {
    expr
} handle {
    Pattern1 => expr1;
    Pattern2 => expr2;
    e => expr3;
}
```

A catch-all arm should use `_`:

```ash
with_error {
    expr
} handle {
    Pattern1 => expr1;
    _ => fallback;
}
```

Design decisions:

- multi-arm `handle`, mirroring `match`
- first matching arm wins
- `_` is the primary catch-all
- no `*`
- `otherwise` may be future sugar, but is not primary
- unmatched failures propagate unchanged
- every handler arm must produce the same normal result type as the body
- the handler catches only failures routed to this handler scope

Typing intuition:

```text
body : A
arm_i : A
────────────────────────────────────────
with_error { body } handle { arms } : A
```

## 8. Handler Matching Target

The semantic match target is the full operational failure object.

However, payload-oriented shorthand should be allowed:

```ash
with_error {
    x / y
} handle {
    DivideByZero => 0;
    _ => fail UnknownMathFailure;
}
```

This can desugar conceptually to matching `Failure { payload: DivideByZero }`.

Future full-form matching may look like:

```ash
with_error {
    invoke("fs", "read", [path])
} handle {
    Failure { tower: Effectful, payload: PolicyDenied { reason } } => fail reason;
    Failure { payload: ProviderUnavailable { provider } } => default_value;
    _ => fail UnhandledFailure;
}
```

The exact parser/typechecker contract is deferred.

## 9. Async `par`, `join`, and Failure Observation

For async `par`:

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

A scoped handler around `par` catches only start/admission/handle-creation failures:

```ash
with_error {
    par(p1, p2)
} handle {
    _ => fallback_handles;
}
```

Failures inside the running processes do not retroactively fail the lexical `par` call after it has returned.
They propagate along process identity toward an observation point.

Process failures should be handled around `join`, `gather`, an await-like primitive, or inside the process itself:

```ash
handles = par(p1, p2);
with_error {
    join(handles.0, handles.1)
} handle {
    _ => fallback_result;
}
```

Current direction:

- `join` is binary.
- the core primitive is probably an await-like operation over a single running process handle.
- `join` and `gather` can be built from that primitive.

## 10. Open Questions

1. Exact parser form for `fail` as expression vs statement.
2. Exact parser form for `with_error` blocks and handler arm syntax.
3. Whether handler arms match payload shorthand by default or require explicit `Failure { ... }` for nontrivial cases.
4. Whether pure `fail` is exposed immediately or only present semantically as bottom.
5. How to represent bottom in type checking: explicit `Bottom` type, expected-type checking only, or another mechanism.
6. Which operations can handle/recover from process failures vs merely observe/report them.
7. Whether workflow admission and completion failures can be handled inside workflow bodies, or only outside the workflow boundary.
