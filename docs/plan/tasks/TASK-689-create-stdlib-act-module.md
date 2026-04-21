# TASK-689: Create std/src/act.ash library module

## Status: 🟡 Ready

## Description

Add the Phase-97 standard-library module defining the ordinary library functions `unit`, `bind`, `then`, and `guard`.

## Specification Reference

- SPEC-047 §2.5

## Dependencies

- 📝 TASK-685: prerequisite task

## Requirements

### Functional Requirements

1. Create `std/src/act.ash`.
2. Define `unit`, `bind`, `then`, and `guard` as library functions, not runtime builtins.
3. Keep signatures and examples aligned with current Ash type syntax.

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

- [ ] Stdlib act module exists and matches the Phase 97 architecture.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
