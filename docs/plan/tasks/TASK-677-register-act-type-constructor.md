# TASK-677: Register Act type constructor

## Status: ✅ Complete

## Description

Register `Act` as a recognized type constructor of kind `* -> *` in the type environment using the existing constructor machinery.

## Specification Reference

- SPEC-047 §5.1

## Dependencies

- 📝 TASK-672: prerequisite task

## Requirements

### Functional Requirements

1. Add `Act` constructor registration to type environment initialization.
2. Reuse existing `Type::Constructor` infrastructure; do not add a new `Type` variant.
3. Ensure kind representation matches the current boxed-arrow substrate.

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

- [x] Type environment resolves `Act<T>` as a constructor application.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
