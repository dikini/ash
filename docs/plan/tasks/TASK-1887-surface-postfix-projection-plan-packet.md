# TASK-1887: Surface Postfix Projection Plan Packet

**Status:** Complete
**Plan:** [PLAN-192](../PLAN-192-SURFACE-POSTFIX-PROJECTION.md)

## Description

Create the Phase 192 plan packet for postfix field projection on ordinary primary expressions.

## Requirements

1. Add a focused plan that depends on Phase 191 and stays on the ordinary expression path.
2. Add implementation task coverage for parser, engine, and CLI behavior.
3. Update plan/spec/notes orientation indexes.
4. Avoid introducing workflow syntax, method-dispatch semantics, or a second semantic path.

## TDD Steps

1. Capture current parse failures for record-literal and parenthesized-constructor projection.
2. Add focused parser/engine regressions.
3. Implement postfix field projection through the existing `Expr::FieldAccess` path.
4. Re-run Phase 187/189/191 non-interference checks.

## Completion Checklist

- [x] PLAN-192 created.
- [x] TASK-1887 created.
- [x] TASK-1888 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
