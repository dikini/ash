# TASK-675: Lower ActBlock into existing core expressions

## Status: ✅ Complete

## Description

Implement Phase-97 lowering for `Expr::ActBlock` by desugaring it into existing core expression forms such as `Call`, `FnDef`, and `FnApply`. No new core AST variants are introduced.

## Specification Reference

- SPEC-047 §6

## Dependencies

- 📝 TASK-673: prerequisite task
- 📝 TASK-674: prerequisite task

## Requirements

### Functional Requirements

1. Add lowering support for `surface::Expr::ActBlock` in `crates/ash-parser/src/lower.rs`.
2. Desugar return statements to `unit(...)` calls.
3. Desugar bind statements into nested `bind(...)` applications using closure values.

### Property Requirements (proptest)

```rust
// Add property-based tests where the task manipulates syntax lowering,
// typing invariants, or runtime sequencing that should hold across broad inputs.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target Phase-97 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the additive Phase-97 architecture.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not introduce core-IR expansion beyond the frozen Phase-97 plan.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic/lowering/runtime invariants where appropriate.

## Verification Steps

- [x] `crates/ash-parser/src/lower.rs` lowers `surface::Expr::ActBlock` through `lower_act_block()`.
- [x] Return-only act blocks lower to `unit(...)` calls; bind statements lower to nested `bind(...)` calls with `FnDef` continuations.
- [x] Lowering rejects empty blocks and invalid statement order, and introduces no `ActBlock`-specific core IR variant.

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
- The landed slice is the additive surface-to-core desugaring promised by PLAN-097: `ActBlock` is erased during lowering into existing `Call`/`FnDef` forms rather than expanding core AST surface area.
