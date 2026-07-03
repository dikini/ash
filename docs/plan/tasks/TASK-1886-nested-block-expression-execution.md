# TASK-1886: Nested Block Expression Execution

**Status:** Complete
**Plan:** [PLAN-191](../PLAN-191-SURFACE-BLOCK-EXPRESSIONS.md)

## Description

Implement ordinary nested block expressions and block expression statements as direct-style
sequencing in function-first Ash.

## Requirements

1. Parse nested block expressions in ordinary expression position.
2. Parse ordinary call expression statements inside blocks.
3. Parse ordinary non-call expression statements inside blocks.
4. Preserve existing block `let`, local `fn`, and tail-expression behavior.
5. Typecheck and lower expression statements without adding a new runtime mode.
6. Execute function-first sources with nested blocks through engine and CLI paths.

## TDD Steps

1. RED: add parser and engine tests for nested `let` blocks and expression statements.
2. Verify the tests fail on the current parse error.
3. GREEN: add a block expression-statement carrier and thread it through parser, lowering,
   typechecking, and traversal utilities.
4. Verify existing function-body and `do` regressions remain green.
5. Probe `ash check`, `ash run --dry-run`, and `ash run`.

## Completion Checklist

- [x] RED failures captured.
- [x] Parser accepts nested block expressions.
- [x] Parser accepts block expression statements.
- [x] Engine regression passes.
- [x] CLI check/dry-run/run probe passes.
- [x] Specs, indexes, and changelog updated.
