# PLAN-190: Surface Do Expression Statements

**Status:** Complete
**Depends on:** Phase 189 Surface Match Ordinary Scrutinees.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `PLAN-185`, and `PLAN-182`.

## Goal

Make target `do { ... }` sequencing accept ordinary expression statements (`expr;`) as specified,
so function bodies can sequence calls or value expressions without inventing temporary bindings.

## Scope

This phase closes the next unified-`do` surface gap:

- parse `do { call(); return value; }`;
- parse `do { expr; return value; }`;
- lower/typecheck/evaluate expression statements as direct-style sequencing that discards the
  statement result;
- keep existing `let`, `<-`, and `return` do statements working;
- prove the behavior through function-first engine and CLI paths without workflow syntax.

## Non-Goals

- No new runtime mode.
- No authority grant or handler/provider behavior.
- No broad block-expression statement overhaul outside `do`.
- No change to `let`, `<-`, or `return` semantics.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1883](tasks/TASK-1883-surface-do-expression-statement-plan-packet.md) | Create the Phase 190 plan packet | Complete |
| [TASK-1884](tasks/TASK-1884-do-expression-statement-execution.md) | Parse, check, and execute do expression statements | Complete |

## Verification Evidence

- RED probe: `ash check` on `do { log_unit(); return 41; }` failed with a parse error before
  implementation.
- RED probe: `ash check` on `do { 1 + 1; return 41; }` failed with a parse error before
  implementation.
- RED test: `cargo test -p ash-engine --test task_1884_do_expression_statements` failed at parse
  before implementation.
- GREEN test: `cargo test -p ash-parser target_ambient_do_block_parses_expression_statements`.
- GREEN test: `cargo test -p ash-engine --test task_1884_do_expression_statements`.
- GREEN CLI probe: `cargo run -p ash-cli -- check`, `cargo run -p ash-cli -- run --dry-run`,
  and `cargo run -p ash-cli -- run` passed on a `do { touch(); 1 + 1; return 41; }` fixture.

## Acceptance Criteria

- [x] Phase 190 plan and task packet exist and are indexed.
- [x] `do { call(); return value; }` parses, checks, and executes.
- [x] `do { expr; return value; }` parses, checks, and executes.
- [x] Existing `do` `let`, `<-`, and `return` coverage remains green.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the expression-statement fixture.
- [x] Changelog and target specs record the surface-language change.
