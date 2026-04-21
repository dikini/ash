# TASK-688: Finalize SPEC-047 amendments and targeted spec updates

## Status: 🟡 Ready

## Description

Finalize the documentation corpus so Phase 97 planning and implementation surfaces all describe the same additive architecture.

## Specification Reference

- SPEC-047
- SPEC-002
- SPEC-003
- SPEC-004
- SPEC-027
- SPEC-031

## Dependencies

- 📝 TASK-672: prerequisite task

## Requirements

### Functional Requirements

1. Ensure SPEC-047 and PLAN-097 stay aligned.
2. Apply targeted supporting amendments to related specs as needed.
3. Avoid reopening SPEC-025 or SPEC-BUILTIN-FN scope in this phase.

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

- [ ] Phase 97 docs form a consistent packet with no leftover contradictory wording.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
