# TASK-153: Progress Theorem

## Status: ✅ Complete

## Description

Prove that well-typed expressions either are values or can evaluate.

## Specification Reference

- SPEC-004 Section 3 (Big-Step Judgment)
- Type Safety standard theory

## Theorem Statement

```lean
theorem progress {env : Env} {e : Expr} {τ : TypeExpr}
  (h_welltyped : WellTyped env e τ) :
  IsValue e ∨ CanEvaluate env e
```

## TDD Steps

### Step 1: Define Type System (Red)

Create `Ash/Types/WellTyped.lean` with inductive relation.

### Step 2: State Theorem (Red)

Add to `Ash/Proofs/Progress.lean`.

### Step 3: Induction on Typing Derivation (Green)

Case analysis on typing rules.

## Completion Checklist

- [ ] `WellTyped` relation defined
- [ ] Progress theorem stated
- [ ] Progress theorem proven

## Estimated Effort

24 hours

## Dependencies

TASK-152

## Blocked By

- TASK-152

## Blocks

- TASK-155 (Type Safety)
