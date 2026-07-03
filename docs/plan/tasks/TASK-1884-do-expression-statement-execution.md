# TASK-1884: Do Expression Statement Execution

**Status:** Complete
**Plan:** [PLAN-190](../PLAN-190-SURFACE-DO-EXPRESSION-STATEMENTS.md)

## Description

Implement target `do` expression statements (`expr;`) as direct-style sequencing that evaluates the
expression and discards its result before continuing.

## Requirements

1. Parse ordinary call expression statements inside `do`.
2. Parse ordinary non-call expression statements inside `do`.
3. Preserve existing `let`, `<-`, and `return` statements.
4. Typecheck and lower expression statements without adding a new runtime mode.
5. Execute function-first sources with expression statements through engine and CLI paths.

## TDD Steps

1. RED: add parser and engine tests for call and binary expression statements inside `do`.
2. Verify the tests fail on the current parse error.
3. GREEN: add a `DoStmt` expression-statement carrier and thread it through parser, lowering,
   typechecking, and interpretation.
4. Verify existing `do` regressions remain green.
5. Probe `ash check`, `ash run --dry-run`, and `ash run`.

## Completion Checklist

- [x] RED failures captured.
- [x] Parser accepts call expression statements.
- [x] Parser accepts non-call expression statements.
- [x] Engine regression passes.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.
