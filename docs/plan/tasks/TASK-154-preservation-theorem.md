# TASK-154: Preservation Theorem

## Status: 🟡 Ready to Start

## Description

Prove that evaluation preserves types.

## Specification Reference

- SPEC-004 Section 3
- Type Safety standard theory

## Theorem Statement

```lean
theorem preservation {env : Env} {e : Expr} {τ : TypeExpr} {r : EvalResult}
  (h_welltyped : WellTyped env e τ)
  (h_eval : eval env e = .ok r) :
  WellTyped env (.literal r.value) τ
```

## TDD Steps

### Step 1: State Theorem (Red)

Add to `Ash/Proofs/Preservation.lean`.

### Step 2: Induction on Expression (Green)

Complex case analysis for each expression type.

## Completion Checklist

- [ ] Theorem stated
- [ ] All cases proven
- [ ] Build passes

## Estimated Effort

32 hours

## Dependencies

TASK-152

## Blocked By

- TASK-152

## Blocks

- TASK-155 (Type Safety)
