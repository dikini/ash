# TASK-676: Property and integration tests for act-block parsing and lowering

## Status: 🟡 Ready

## Description

Add focused parser/lowering tests for expression-level act blocks, including sequencing, nesting, and dual-context `act` parsing.

## Specification Reference

- SPEC-047 §2
- SPEC-047 §6

## Dependencies

- 📝 TASK-674: prerequisite task
- 📝 TASK-675: prerequisite task

## Requirements

### Functional Requirements

1. Cover minimal block, multi-bind block, and nested act-block cases.
2. Cover preservation of workflow-level `act` syntax.
3. Use property tests where useful for lowering structure invariants.

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

- [ ] New parser/lowering tests pass consistently.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
