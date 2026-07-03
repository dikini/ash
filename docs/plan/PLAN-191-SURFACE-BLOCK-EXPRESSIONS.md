# PLAN-191: Surface Block Expressions

**Status:** Complete
**Depends on:** Phase 190 Surface Do Expression Statements.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `PLAN-185`, and `PLAN-190`.

## Goal

Make ordinary `{ ... }` block expressions usable in function-first Ash wherever an expression is
expected, including nested blocks and expression statements whose values are discarded before the
tail expression.

## Scope

This phase closes the next ordinary-expression ergonomics gap:

- parse nested block expressions such as `{ let x = 40 + 1; x }`;
- parse block expression statements such as `{ touch(); 41 }` and `{ 1 + 1; 41 }`;
- lower/typecheck/evaluate block expression statements as direct-style sequencing that discards
  the statement result;
- keep existing function-body `let`, local `fn`, and tail-expression parsing working;
- prove the behavior through function-first engine and CLI paths without workflow syntax.

## Non-Goals

- No new runtime mode.
- No workflow/profile syntax changes.
- No broad destructuring or exhaustiveness change.
- No change to `do` sequencing semantics.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1885](tasks/TASK-1885-surface-block-expression-plan-packet.md) | Create the Phase 191 plan packet | Complete |
| [TASK-1886](tasks/TASK-1886-nested-block-expression-execution.md) | Parse, check, and execute nested block expressions and block expression statements | Complete |

## Verification Evidence

- RED probe: `ash check` on `fn main() -> Int { { let x = 40 + 1; x } }` failed with a parse
  error before implementation.
- RED probe: `ash check` on `fn main() -> Int { { touch(); 41 } }` failed with a parse error
  before implementation.
- RED test: `cargo test -p ash-engine --test task_1886_nested_block_expressions` failed at parse
  before implementation.
- GREEN test: `cargo test -p ash-parser parse_fn_nested_block_with_expression_statements`.
- GREEN test: `cargo test -p ash-engine --test task_1886_nested_block_expressions`.
- GREEN CLI probe: `cargo run -p ash-cli -- check`, `cargo run -p ash-cli -- run --dry-run`,
  and `cargo run -p ash-cli -- run` passed on a nested-block fixture returning `41`.

## Acceptance Criteria

- [x] Phase 191 plan and task packet exist and are indexed.
- [x] Nested block expressions parse, check, and execute in function-first sources.
- [x] Block expression statements parse, check, and execute with their values discarded.
- [x] Existing function-body `let`, local `fn`, and target `do` coverage remains green.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the nested-block fixture.
- [x] Changelog and target specs record the surface-language change.
