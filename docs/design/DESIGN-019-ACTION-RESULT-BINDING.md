# DESIGN-019: Action Result Binding and Continuation

## Status: Draft

## Overview

Extend `Workflow::Act` with an optional result binding and a mandatory continuation so that
capability actions can produce values that flow back into the workflow. This closes the gap where
`Act` is a terminal fire-and-forget node, and makes actions first-class value-producing workflow
steps alongside `Observe` and `Let`.

## Problem Statement

### Current State

`Workflow::Act` is a terminal workflow node:

```rust
Act {
    provider_name: Name,
    action_name: Name,
    arguments: Vec<Expr>,
    guard: Guard,
    provenance: Provenance,
}
```

The interpreter executes the action and returns its `Value` as the workflow result. There is no
continuation and no way to bind the result to a name for subsequent steps.

This means:
1. An action result cannot be used by later workflow steps.
2. To read state that an action changed, the workflow must follow up with a separate `Observe`.
3. Actions are second-class compared to `Observe` (which has continuation) and `Let` (which binds).

The `CapabilityProvider::execute()` trait method already returns `Result<Value, CapabilityError>`.
The runtime plumbing supports return values. The gap is in the workflow graph structure.

### Why This Matters

For non-trivial workflows, actions frequently produce values that downstream steps need:

- `fs:write_file(...)` might return a checksum or byte count
- `mcp:call(...)` returns an LLM response
- `http:post(...)` returns a response body
- A custom provider returns a computed result

Requiring a separate `Observe` to read back what the action just produced is semantically weak
and operationally wasteful.

## Design Goals

1. Make `Act` a value-producing, continuation-carrying workflow node — structurally equal to `Let`.
2. Support two surface continuation models: explicit `then` and lexical-scope `as`.
3. Make `let name = <cap> with ...` sugar identical to `act <cap> with ... as name`.
4. Preserve backwards compatibility: bare `act` (no continuation) remains valid.
5. Keep the core IR change minimal: two new fields on the existing variant.

## Non-Goals

1. Change the `CapabilityProvider` trait (it already returns `Value`).
2. Change `observe`, `set`, or `send` continuation models.
3. Add typed return values or effect-level constraints on action results.
4. Introduce async/streaming action results.

## Design Decisions

### Decision 1: Extend `Workflow::Act` with `result_name` and `continuation`

```rust
Act {
    provider_name: Name,
    action_name: Name,
    arguments: Vec<Expr>,
    guard: Guard,
    provenance: Provenance,
    result_name: Option<Name>,       // NEW: bind result to this name
    continuation: Box<Workflow>,     // NEW: always present (Done for terminal)
}
```

Both fields are always present. `result_name: None, continuation: Done` is the backwards-compatible
terminal form (identical semantics to current bare `act`).

### Decision 2: `then` Is Explicit Inline Continuation

```ash
act fs:write_file("/out", data) then
  observe status
```

The `then` keyword introduces an explicit inline continuation workflow. The action result is
discarded (not bound). The continuation executes after the action completes.

Core IR: `Act { result_name: None, continuation: <observed>, ... }`

### Decision 3: `as` Binds Result to Lexical Scope

```ash
act mcp:call("tools/call", params) as response
  orient response.body
  act fs:write_file("/out", response.body)
```

The `as` keyword binds the action result to a name. The name is in scope for all following
expressions in the enclosing workflow sequence. The parser lifts the remaining sequence into
the continuation, exactly as `let` already does.

Core IR: `Act { result_name: Some("response"), continuation: <rest>, ... }`

### Decision 4: `let name = <cap-call>` Desugars to `act ... as name`

```ash
let response = mcp:call("tools/call", params)
  orient response.body
```

is surface-sugar-identical to:

```ash
act mcp:call("tools/call", params) as response
  orient response.body
```

Both lower to the same core IR node. The parser recognizes `let <name> = <operational-call>` as
an `Act` with `result_name`, not a generic `Let`.

**Parser architecture constraint**: Operational calls are parsed by `action_ref()` which produces
`ActionRef` (with `OperationalTarget` variants for symbolic, qualified, and explicit forms). This
is not an `Expr` — it lives in a separate grammar path. The current `let_stmt()` parser calls
`expr()` for the RHS unconditionally. Therefore the `let <name> = <cap-call>` sugar **must be
handled at parse time** in `let_stmt()` by attempting `action_ref()` first (via lookahead or
backtracking) before falling back to generic `expr()` parsing. It cannot be deferred to lowering
because the parser has already committed to `Expr` vs `ActionRef` by then.

The concrete approach: `let_stmt()` peeks past `let <pattern> =` and tries `action_ref()`. If it
succeeds, emit `SurfaceWorkflow::Act { result_name, continuation, ... }` instead of `Let`. If it
fails (backtrack), fall through to `expr()` and emit `SurfaceWorkflow::Let` as before.

### Decision 5: Execution Semantics

The interpreter evaluates `Workflow::Act` with the new fields as follows:

1. Evaluate guard — if false, return `GuardFailed`.
2. Evaluate arguments eagerly to `Vec<Value>`.
3. Look up provider, dispatch `provider.execute(action_name, &args)`.
4. If `result_name` is `Some(name)`, bind the result value in the execution context.
5. Execute `continuation`.
6. Return the continuation's result (or the action result if continuation is `Done`).

### Decision 6: Backwards Compatibility

All existing `Act` nodes without continuation are migrated to:
```rust
Act { ..., result_name: None, continuation: Done }
```

This is semantically identical — the workflow returns the action result directly.

## Surface Syntax Summary

### New Forms

```ash
-- fire and forget (terminal, unchanged)
act fs:write_file("/out", data)

-- discard result, explicit continuation
act fs:write_file("/out", data) then
  observe status

-- bind result, lexical scope continuation
act mcp:call("tools/call", params) as response
  orient response.body

-- let sugar (identical lowering)
let response = mcp:call("tools/call", params)
  orient response.body
```

### Keyword Notes

`as` is already a contextual keyword used by `observe`, `orient`, and `propose`. No lexer or
keyword-set changes are required — `act_stmt()` extends the existing pattern.

`then` is already used by `if ... then` and `act` can reuse the same contextual parsing approach.

### Compatibility

Existing bare `act` forms continue to work unchanged.

## Architecture

### Before

```text
Surface: act <cap>(args)
  -> Surface::Act { action: ActionRef, guard, span }
  -> lower -> Core::Act { provider, action, args, guard, provenance }
  -> execute -> cap_ctx.execute(provider, action, args) -> Value (terminal)
```

### After

```text
Surface: act <cap>(args) as <name>
  <rest of workflow>
  -> Surface::Act { action, guard, result_name, continuation, span }
  -> lower -> Core::Act { provider, action, args, guard, provenance, result_name, continuation }
  -> execute -> cap_ctx.execute(provider, action, args) -> Value
            -> bind result_name in context
            -> execute continuation
            -> return continuation result

Surface: let <name> = <cap>(args)
  <rest of workflow>
  -> parser recognizes operational-call RHS
  -> lowers identically to act ... as <name>
```

## Spec Impact

Coordinated updates required for:

- **SPEC-001**: core `Workflow::Act` contract (new fields, continuation semantics)
- **SPEC-002**: surface syntax for `act ... then`, `act ... as`, and `let <name> = <cap-call>` sugar
- **SPEC-004**: big-step ACT execution (continuation, result binding)
- **SPEC-025**: small-step ACT helper (continuation step, result in environment)

## Risks

### Risk 1: Parser Ambiguity Between `let name = expr` and `let name = <cap-call>`

**Mitigation**: At parse time, the RHS determines the lowering. If the RHS is an operational call
(`ActionRef`), it lowers to `Act` with `result_name`. Otherwise it stays as `Let`. The parser already
has `OperationalTarget` to distinguish these.

### Risk 2: Migration Breaks Existing Act Tests

**Mitigation**: All existing bare `act` forms lower to `continuation: Done, result_name: None`.
Migration is mechanical. Property tests on the old shape verify semantic preservation.

### Risk 3: `as` Keyword Conflicts

**Mitigation**: `as` is already used as a contextual keyword by `observe`, `orient`, and `propose`
(see `parse_workflow.rs` lines 882, 932, 957). The `act` form extends this existing pattern —
no new keyword reservation or lexer/token changes are needed. The `act_stmt()` parser simply
checks for the `as` keyword after the guard clause, following the same `keyword("as").parse_next()`
pattern already used by `observe_stmt`, `orient_stmt`, and `propose_stmt`.

## Success Criteria

1. `Act` is structurally equal to `Let`: it can bind a name and has a continuation.
2. All three surface forms (`then`, `as`, `let = cap-call`) parse, lower, and execute correctly.
3. Existing bare `act` forms compile and run identically to before.
4. Specs agree on the new Act continuation contract.
5. Full test suite passes (`cargo test`, `cargo clippy`, `cargo fmt`).
