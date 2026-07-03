# PLAN-192: Surface Postfix Projection

**Status:** Complete
**Depends on:** Phase 191 Surface Block Expressions.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `PLAN-185`, and `PLAN-187`.

## Goal

Make field projection behave like ordinary postfix expression syntax, so records and ADT
constructor values can be projected directly without forcing a temporary binding.

## Scope

This phase closes the next ordinary-expression postfix gap:

- parse structural record projection such as `{ item: 41 }.item`;
- parse constructor projection such as `(Box { item: 41 }).item`;
- keep existing variable, call-result, and nested field projections working;
- prove the behavior through function-first engine and CLI paths without workflow syntax.

## Non-Goals

- No new runtime representation for records or constructors.
- No method-call dispatch or dot-call semantics beyond existing function application.
- No change to match scrutinee parsing.
- No workflow/profile syntax changes.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1887](tasks/TASK-1887-surface-postfix-projection-plan-packet.md) | Create the Phase 192 plan packet | Complete |
| [TASK-1888](tasks/TASK-1888-postfix-field-projection-execution.md) | Parse, check, and execute postfix field projection on ordinary primary expressions | Complete |

## Verification Evidence

- RED probe: `ash check` on `fn main() -> Int { { item: 41 }.item }` failed with a parse error
  before implementation.
- RED probe: `ash check` on `fn main() -> Int { (Box { item: 41 }).item }` failed with a parse
  error before implementation.
- GREEN: `cargo test -p ash-parser parse_fn_postfix_projection_on_record_and_constructor_values`
  passed.
- GREEN: `cargo test -p ash-engine --test task_1888_postfix_field_projection` passed.
- REGRESSION: `cargo test -p ash-engine --test task_1886_nested_block_expressions` passed.
- REGRESSION: `cargo test -p ash-engine --test task_1878_surface_record_expressions` passed.
- REGRESSION: `cargo test -p ash-engine --test task_1882_match_ordinary_scrutinees` passed.
- CLI: `cargo run -p ash-cli -- check`, `run --dry-run`, and `run` passed on the
  postfix-projection fixture; `run` printed `41`.

## Acceptance Criteria

- [x] Phase 192 plan and task packet exist and are indexed.
- [x] Structural record literals accept postfix field projection.
- [x] Parenthesized ADT constructor expressions accept postfix field projection.
- [x] Existing variable/call field projection coverage remains green.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the postfix-projection fixture.
- [x] Changelog and target specs record the surface-language change.
