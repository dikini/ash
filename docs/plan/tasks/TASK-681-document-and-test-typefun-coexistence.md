# TASK-681: Document and test additive coexistence with Type::Fun

## Status: 🟡 Ready

## Description

Lock in the additive Phase-97 typing boundary: `Act<T>` is introduced without retiring or redefining the current `Type::Fun(...)` workflow-closure model.

## Specification Reference

- SPEC-047 §5.6
- SPEC-031

## Dependencies

- 📝 TASK-678: prerequisite task

## Requirements

### Functional Requirements

1. Add tests that preserve existing workflow-context closure typing behavior.
2. Ensure new `Act<T>` typing does not implicitly collapse `Type::Fun(...)` semantics.
3. Document the coexistence boundary in task-level notes and tests.

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

- [ ] Existing `Type::Fun(...)` behavior remains unchanged by Phase 97 typing additions.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
