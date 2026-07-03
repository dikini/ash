# PLAN-188: Surface Match Constructor Scrutinees

**Status:** Complete
**Depends on:** Phase 187 Surface Record Expressions.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `SPEC-020`, `SPEC-004`, and `PLAN-185`.

## Goal

Make ordinary ADT constructor expressions usable directly as `match` scrutinees in the
function-first target language, so pattern matching does not require users to bind a temporary
variable just to avoid parser ambiguity with the match body delimiter.

## Scope

This phase closes the next surface-function expression gap after structural records:

- parse match scrutinees such as `match Some { value: 41 } { ... }` inside ordinary `fn` bodies;
- keep existing `match name { ... }` parsing intact, where the brace after a non-constructor
  scrutinee remains the match body delimiter;
- check and execute a function-first ADT constructor/match fixture through the engine and CLI paths;
- update target specs and indexes so ADT constructor scrutinees are represented as ordinary
  expressions in pattern-matching contexts.

## Non-Goals

- No new ADT runtime representation.
- No new exhaustiveness model.
- No changes to variant-pattern syntax.
- No removal of legacy workflow syntax.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1879](tasks/TASK-1879-surface-match-constructor-scrutinee-plan-packet.md) | Create the Phase 188 plan packet | Complete |
| [TASK-1880](tasks/TASK-1880-adt-constructor-match-scrutinee-execution.md) | Parse, check, and execute constructor-expression match scrutinees | Complete |

## Verification Evidence

- RED: `ash check` on a function-first source containing `match Some { value: 41 } { ... }`
  failed with a parse error before implementation.
- RED: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees` failed before
  implementation because `engine.parse` returned `Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees` passed after
  function-body match scrutinee parsing accepted ADT constructor payloads.
- Parser regression: `cargo test -p ash-parser parse_fn_match_constructor_expression_scrutinee`
  passed.
- Non-interference regression: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow`
  passed.
- CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the constructor-scrutinee
  fixture, with execution returning `41`.

## Acceptance Criteria

- [x] Phase 188 plan and task packet exist and are indexed.
- [x] Function-body `match` scrutinees accept ADT record-constructor expressions.
- [x] Existing variable/literal match scrutinees keep parsing.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the constructor-scrutinee fixture.
- [x] Changelog and target specs record the surface-language change.
