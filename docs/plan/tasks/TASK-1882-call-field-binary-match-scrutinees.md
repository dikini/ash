# TASK-1882: Call, Field, and Binary Match Scrutinees

**Status:** Complete
**Plan:** [PLAN-189](../PLAN-189-SURFACE-MATCH-ORDINARY-SCRUTINEES.md)

## Description

Extend function-body match parsing so ordinary expressions such as calls, field projections, and
binary expressions can appear directly as match scrutinees.

## Requirements

1. Parse `match make() { ... }` where `make` is an ordinary function.
2. Parse `match holder.inner { ... }` where `holder.inner` is an ordinary field projection.
3. Parse `match 40 + 1 { ... }` using the same binary expression semantics as other function
   expressions.
4. Preserve `match opt { ... }` and `match Some { value: 41 } { ... }`.
5. Keep the match body delimiter unambiguous for variable scrutinees.
6. Verify the function-first engine and CLI paths without requiring `workflow`.

## TDD Steps

1. RED: add focused parser coverage for call, field, and binary scrutinees.
2. RED: add engine coverage that executes these forms in a function-first source.
3. GREEN: extend the restricted fn-body match scrutinee parser.
4. Verify Phase 188 constructor-scrutinee coverage and Phase 185 rich fixture still pass.
5. Probe `ash check`, `ash run --dry-run`, and `ash run`.

## Completion Checklist

- [x] RED failures captured.
- [x] Parser accepts call scrutinees.
- [x] Parser accepts field-projection scrutinees.
- [x] Parser accepts binary scrutinees.
- [x] Engine regression passes.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.

## Evidence

- RED: `cargo test -p ash-parser parse_fn_match_call_field_and_binary_scrutinees` failed with
  `fn definition should parse: Backtrack(ContextError { context: [], cause: None })`.
- RED: `cargo test -p ash-engine --test task_1882_match_ordinary_scrutinees` failed with
  `source should parse: Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-parser parse_fn_match_call_field_and_binary_scrutinees` passed.
- GREEN: `cargo test -p ash-engine --test task_1882_match_ordinary_scrutinees` passed.
- Non-interference regression: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees`
  passed.
- Non-interference regression: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow`
  passed.
- CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the ordinary-scrutinee
  fixture and printed `83`.
