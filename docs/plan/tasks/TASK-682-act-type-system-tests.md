# TASK-682: Type-system tests for purity rejection and Act inference

## Status: 🟡 Ready

## Description

Add comprehensive type-system tests for `Act<T>` inference, purity rejection, and `invoke(...)` typing.

## Specification Reference

- SPEC-047 §5

## Dependencies

- 📝 TASK-678: prerequisite task
- 📝 TASK-679: prerequisite task
- 📝 TASK-680: prerequisite task
- 📝 TASK-681: prerequisite task

## Requirements

### Functional Requirements

1. Cover inferred and annotated `Act<T>` results.
2. Cover rejection paths in pure functions.
3. Cover coexistence with existing closure typing and callable resolution.

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

- [ ] Phase-97 type-system test suite passes.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
