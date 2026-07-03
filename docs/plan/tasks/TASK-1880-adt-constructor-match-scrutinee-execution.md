# TASK-1880: ADT Constructor Match Scrutinee Execution

**Status:** Complete
**Plan:** [PLAN-188](../PLAN-188-SURFACE-MATCH-CONSTRUCTOR-SCRUTINEES.md)

## Description

Parse, check, and execute a function-first source where a `match` scrutinee is an ordinary ADT
record-constructor expression, such as `match Some { value: 41 } { ... }`.

## Requirements

1. Function-body match parsing must accept record-constructor scrutinees.
2. Existing variable, literal, parenthesized, and binary scrutinees must keep parsing.
3. The match body delimiter must remain unambiguous for `match value { ... }`.
4. The engine and CLI paths must check and execute the fixture without requiring a `workflow` block.
5. The change must not add a new runtime mode or privileged workflow syntax.

## TDD Steps

1. RED: add an engine regression that parses/checks/runs a local ADT constructor-scrutinee source.
2. Verify the regression fails with the current parse error.
3. GREEN: adjust function-body match scrutinee parsing to recognize constructor expressions only
   when the brace content is a constructor field payload, not a match arm list.
4. Verify the focused regression and existing Phase 185 rich fixture pass.
5. Probe `ash check`, `ash run --dry-run`, and `ash run` for the same fixture.

## Completion Checklist

- [x] RED failure captured.
- [x] Parser accepts constructor-expression match scrutinees.
- [x] Engine regression passes.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.

## Evidence

- RED: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees` failed with
  `source should parse: Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees` passed.
- Parser regression: `cargo test -p ash-parser parse_fn_match_constructor_expression_scrutinee`
  passed.
- Non-interference regression: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow`
  passed.
- CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the constructor-scrutinee
  fixture and printed `41`.
