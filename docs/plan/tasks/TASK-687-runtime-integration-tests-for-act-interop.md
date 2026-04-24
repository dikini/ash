# TASK-687: Runtime integration tests for effectful fn composition and interop

## Status: ✅ Complete

## Description

Add runtime integration coverage for effectful function composition, nested act blocks, and workflow + expression interop.

## Specification Reference

- SPEC-047 §7-§8

## Dependencies

- 📝 TASK-685: prerequisite task
- 📝 TASK-686: prerequisite task

## Requirements

### Functional Requirements

1. Cover direct expression-level effectful composition.
2. Cover nested act blocks.
3. Cover workflow-facing interop scenarios that rely on the `ActEnv` bridge.

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

- [x] Runtime integration suite passes with real provider-path execution.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
