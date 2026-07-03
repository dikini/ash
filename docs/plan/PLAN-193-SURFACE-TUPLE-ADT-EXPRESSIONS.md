# PLAN-193: Surface Tuple ADT Expressions

**Status:** Complete
**Depends on:** Phase 192 Surface Postfix Projection.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `PLAN-185`, and `PLAN-188`.

## Goal

Make tuple-payload ADTs usable as ordinary expressions and patterns in the function-first target
language, so `Constructor(a, b)` behaves like the tuple-payload counterpart to the record-shaped
`Constructor { field: value }` path already accepted by function bodies.

## Scope

This phase closes the next ADT surface gap:

- parse and check tuple-payload constructor expressions such as `RuntimeError(2, "missing config")`
  inside ordinary `fn` bodies;
- parse and check tuple-payload variant patterns such as `RuntimeError(code, message)` inside
  ordinary `match` arms;
- lower tuple payload positions through the existing stable positional field names (`_0`, `_1`,
  ...), matching the isolated parser/lowering contract;
- execute function-first engine and CLI fixtures without requiring workflow syntax.

## Non-Goals

- No new runtime value representation for variants.
- No tuple literal or general tuple value expansion beyond tuple-payload ADTs.
- No new pattern exhaustiveness model.
- No workflow/profile syntax changes.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1889](tasks/TASK-1889-surface-tuple-adt-plan-packet.md) | Create the Phase 193 plan packet | Complete |
| [TASK-1890](tasks/TASK-1890-tuple-adt-expression-execution.md) | Parse, check, lower, and execute tuple-payload ADTs in function-first Ash | Complete |

## Verification Evidence

- RED probe: `ash check` on a function-first `RuntimeError(2, "missing config")` match fixture
  currently fails because the tuple constructor is treated as an unresolved function call.
- RED: `cargo test -p ash-parser parse_fn_match_tuple_constructor_expression_scrutinee` failed
  because the match scrutinee parsed as `Expr::Call`.
- RED: `cargo test -p ash-engine --test task_1890_tuple_adt_expressions` failed because the
  tuple constructor reached type checking as an unresolved function call.
- GREEN: both focused tests pass after tuple constructor classification and positional field mirror
  preservation.
- CLI: `ash check`, `ash run --dry-run`, and `ash run` passed on the tuple-ADT fixture; `run`
  printed `2`.
- REGRESSION: `cargo test -p ash-parser --test fn_parser_tests`,
  `cargo test -p ash-parser --test tuple_variant_parser`, and adjacent Phase 188/189/192/193 engine
  regressions passed.

## Acceptance Criteria

- [x] Phase 193 plan and task packet exist and are indexed.
- [x] Tuple-payload constructor expressions are preserved as constructors in ordinary `fn` bodies.
- [x] Tuple-payload variant patterns bind positional payload fields in ordinary `match` arms.
- [x] Engine parsing/checking/execution passes for a function-first tuple-ADT match fixture.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the same fixture.
- [x] Changelog and target specs record the surface-language change.
