# TASK-1890: Tuple ADT Expression Execution

**Status:** Complete
**Plan:** [PLAN-193](../PLAN-193-SURFACE-TUPLE-ADT-EXPRESSIONS.md)

## Description

Implement tuple-payload ADT construction and matching in ordinary function-first sources.

## Requirements

1. Preserve uppercase `Constructor(args...)` as `Expr::Constructor` in parsed function bodies rather
   than reclassifying it as a function call.
2. Preserve tuple-payload variant patterns in `match` arms and lower positional fields through the
   existing `_0`, `_1`, ... convention.
3. Typecheck and execute a function-first source that constructs and matches a tuple-payload ADT.
4. Preserve existing record-payload ADT, ordinary match-scrutinee, and postfix projection behavior.
5. Execute the tuple-ADT fixture through engine and CLI paths without requiring `workflow`.

## TDD Steps

1. RED: add parser and engine tests for a tuple-payload ADT fixture.
2. Verify the tests fail with the current unresolved-constructor/function-call behavior.
3. GREEN: route tuple-payload constructor calls through the existing constructor expression path.
4. Verify focused tuple-ADT tests and Phase 188/189/192 regressions pass.
5. Probe `ash check`, `ash run --dry-run`, and `ash run`.

## Completion Checklist

- [x] RED failures captured.
- [x] Parser preserves tuple-payload constructor expressions in function bodies.
- [x] Engine regression passes for tuple-payload constructor matching.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.

## Verification Evidence

- RED: `cargo test -p ash-parser parse_fn_match_tuple_constructor_expression_scrutinee` failed
  because `RuntimeError(2, "missing config")` parsed as `Expr::Call`.
- RED: `cargo test -p ash-engine --test task_1890_tuple_adt_expressions` failed with unresolved
  function-call typing for `RuntimeError`.
- GREEN: `cargo test -p ash-parser parse_fn_match_tuple_constructor_expression_scrutinee` passed.
- GREEN: `cargo test -p ash-engine --test task_1890_tuple_adt_expressions` passed.
- CLI: `ash check`, `ash run --dry-run`, and `ash run` passed on the tuple-ADT fixture; `run`
  printed `2`.
- REGRESSION: `cargo test -p ash-parser --test fn_parser_tests` passed.
- REGRESSION: `cargo test -p ash-parser --test tuple_variant_parser` passed.
- REGRESSION: `cargo test -p ash-engine --test task_1880_match_constructor_scrutinees --test
  task_1882_match_ordinary_scrutinees --test task_1888_postfix_field_projection --test
  task_1890_tuple_adt_expressions` passed.
