# TASK-1885: Surface Block Expression Plan Packet

**Status:** Complete
**Plan:** [PLAN-191](../PLAN-191-SURFACE-BLOCK-EXPRESSIONS.md)

## Description

Create the Phase 191 plan packet for ordinary nested block expressions and block expression
statements in target function-first Ash.

## Requirements

1. Add a focused plan that depends on Phase 190 and stays on the ordinary direct-style expression
   path.
2. Add implementation task coverage for parser, engine, and CLI behavior.
3. Update plan/spec/notes orientation indexes.
4. Avoid introducing workflow syntax, tower profiles, or a second semantic path.

## TDD Steps

1. Capture current parse failures for nested blocks and block expression statements.
2. Add focused parser/engine regressions.
3. Implement block expression parsing/lowering/evaluation through existing `Expr::Block`.
4. Re-run Phase 185/190 non-interference checks.

## Completion Checklist

- [x] PLAN-191 created.
- [x] TASK-1885 created.
- [x] TASK-1886 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
