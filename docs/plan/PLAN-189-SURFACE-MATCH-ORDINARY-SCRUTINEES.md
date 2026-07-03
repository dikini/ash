# PLAN-189: Surface Match Ordinary Scrutinees

**Status:** Complete
**Depends on:** Phase 188 Surface Match Constructor Scrutinees.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `SPEC-020`, `SPEC-004`, and `PLAN-185`.

## Goal

Make function-body `match` scrutinees behave like ordinary expressions instead of a special
identifier-only parser island, while preserving the match body delimiter ambiguity fix from
Phase 188.

## Scope

This phase continues the surface-function pattern-matching cleanup:

- parse call scrutinees such as `match make() { ... }`;
- parse field-projection scrutinees such as `match holder.inner { ... }`;
- parse literal/binary scrutinees such as `match 40 + 1 { ... }`;
- preserve existing variable and ADT constructor scrutinees;
- execute the new forms through function-first engine and CLI paths without `workflow` syntax.

## Non-Goals

- No new pattern syntax.
- No new exhaustiveness model.
- No record spread/update or structural record subtyping.
- No broad replacement of the fn-body parser in this phase.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1881](tasks/TASK-1881-surface-match-ordinary-scrutinee-plan-packet.md) | Create the Phase 189 plan packet | Complete |
| [TASK-1882](tasks/TASK-1882-call-field-binary-match-scrutinees.md) | Parse, check, and execute call/field/binary match scrutinees | Complete |

## Verification Evidence

- RED probe: `ash check` on `match make() { ... }` failed with a parse error before implementation.
- RED probe: `ash check` on `match holder.inner { ... }` failed with a parse error before implementation.
- RED probe: `ash check` on `match 40 + 1 { ... }` failed with a parse error before implementation.
- RED: `cargo test -p ash-parser parse_fn_match_call_field_and_binary_scrutinees` failed before
  implementation because the function definition parser returned `Backtrack(ContextError { context: [], cause: None })`.
- RED: `cargo test -p ash-engine --test task_1882_match_ordinary_scrutinees` failed before
  implementation because `engine.parse` returned `Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-parser parse_fn_match_call_field_and_binary_scrutinees` passed after
  function-body match scrutinee parsing accepted call, field-projection, and binary expressions.
- GREEN: `cargo test -p ash-engine --test task_1882_match_ordinary_scrutinees` passed after the
  same parser change.
- Non-interference regression: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees`
  passed.
- Non-interference regression: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow`
  passed.
- CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the ordinary-scrutinee
  fixture, with execution returning `83`.

## Acceptance Criteria

- [x] Phase 189 plan and task packet exist and are indexed.
- [x] Function-body `match` scrutinees accept ordinary call expressions.
- [x] Function-body `match` scrutinees accept field-projection expressions.
- [x] Function-body `match` scrutinees accept literal/binary expressions.
- [x] Existing variable and constructor scrutinees keep parsing.
- [x] CLI `check`, `run --dry-run`, and `run` pass for the ordinary-scrutinee fixture.
- [x] Changelog and target specs record the surface-language change.
