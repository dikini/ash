# TASK-1888: Postfix Field Projection Execution

**Status:** Complete
**Plan:** [PLAN-192](../PLAN-192-SURFACE-POSTFIX-PROJECTION.md)

## Description

Implement postfix field projection on ordinary primary expressions, including structural record
literals and parenthesized ADT constructor expressions.

## Requirements

1. Parse `{ field: value }.field` as `Expr::FieldAccess`.
2. Parse `(Constructor { field: value }).field` as `Expr::FieldAccess`.
3. Preserve existing variable, nested-field, call-result, and match-scrutinee parsing.
4. Typecheck and execute the projected values through the existing field-access path.
5. Execute function-first sources with postfix projection through engine and CLI paths.

## TDD Steps

1. RED: add parser and engine tests for record-literal and constructor field projection.
2. Verify the tests fail on the current parse error.
3. GREEN: make postfix field projection apply after ordinary primary expressions.
4. Verify existing record, match, block, and `do` regressions remain green.
5. Probe `ash check`, `ash run --dry-run`, and `ash run`.

## Completion Checklist

- [x] RED failures captured.
- [x] Parser accepts record-literal field projection.
- [x] Parser accepts constructor-expression field projection.
- [x] Engine regression passes.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.
