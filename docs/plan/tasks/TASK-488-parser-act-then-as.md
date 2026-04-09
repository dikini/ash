# TASK-488: Parser Support for `act ... then`, `act ... as`, and `let = cap-call` Sugar

## Status: Planned

## Description

Add parser support for three new surface continuation forms:
1. `act <cap>(args) then <workflow>` — explicit inline continuation, result discarded
2. `act <cap>(args) as <name>` — bind result, continuation is the rest of the sequence
3. `let <name> = <cap>(args)` — sugar identical to `act <cap>(args) as <name>`

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)

## Dependencies

- [TASK-487](TASK-487-surface-act-continuation.md) — surface AST must have new fields first

## Requirements

1. Extend the existing contextual `as` handling to `act`:
   - `as` is already used by `observe`, `orient`, and `propose` via `keyword("as").parse_next()`.
   - Extend `act_stmt()` to check for `as` after the guard clause, following the same pattern.
   - **No lexer or keyword-set changes** unless proven necessary by implementation.

2. `act <cap>(args) then <workflow>`:
   - Parse `then` followed by a workflow expression (reuse `parse_single_stmt_or_block`).
   - Surface AST: `Act { result_name: None, continuation: Some(<workflow>), ... }`.

3. `act <cap>(args) as <name>`:
   - Parse `as` followed by a name (simple identifier pattern, not full pattern).
   - The continuation is the rest of the workflow sequence (same tail-capture as `let`).
   - Surface AST: `Act { result_name: Some(name), continuation: Some(<rest>), ... }`.

4. `let <name> = <cap>(args)` sugar — **must be handled at parse time**:
   - **Background**: The current `let_stmt()` (parse_workflow.rs:1064) calls `expr()` for the
     RHS unconditionally. Operational calls are parsed by `action_ref()` which produces
     `ActionRef` (not `Expr`). These live in separate grammar paths. By the time lowering runs,
     the parser has already committed to `SurfaceWorkflow::Let` with an `Expr` RHS.
   - **Approach**: In `let_stmt()`, after parsing `let <pattern> =`, attempt `action_ref()`
     first (via lookahead or backtracking). If it succeeds and the pattern is a simple name,
     emit `SurfaceWorkflow::Act { result_name: Some(name), continuation: None, ... }` instead
     of `Let`. If `action_ref()` fails (backtrack), fall through to `expr()` and emit
     `SurfaceWorkflow::Let` as before.
   - **Do not** try to recognize this at lowering time — the information is lost by then.

5. `act <cap>(args)` without `then` or `as`:
   - Surface AST: `Act { result_name: None, continuation: None, ... }` (terminal, unchanged).

6. `cargo test -p ash-parser` passes with new parse tests.

## TDD Steps

### Red

- Write parse tests for each of the three new forms. They should fail before implementation.

### Green

- Implement parser changes. Verify all parse tests pass.
- Verify existing parse tests for bare `act` still pass.

### Refactor

- Factor out the `act` continuation parsing into a helper if it gets complex.
- Ensure the `let_stmt()` action_ref try-before-expr is clean and backtracks safely.

## Completion Checklist

- [ ] `then` keyword recognized after act (contextual, no keyword-set change)
- [ ] `as` keyword recognized after act (extending existing observe/orient/propose pattern)
- [ ] `let <name> = <cap-call>` recognized at parse time in `let_stmt()` via `action_ref()` try
- [ ] Bare `act` still parses correctly (regression)
- [ ] Parse tests for all four forms (then, as, let-sugar, bare)
- [ ] `cargo test -p ash-parser` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md entry added
