# TASK-691: Performance baseline for desugared act-block execution

## Status: 🟡 Ready

## Description

Establish an honest initial performance baseline for the lowered/desugared act-block execution path before any later optimization work.

## Specification Reference

- PLAN-097 Track D

## Dependencies

- 📝 TASK-690: prerequisite task

## Requirements

### Functional Requirements

1. Measure representative desugared act-block execution paths.
2. Record baseline numbers and obvious hotspots.
3. Avoid premature optimization; this task is observational.

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

- [ ] A reproducible baseline exists for future Phase-97 optimization discussions.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
