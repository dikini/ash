# TASK-713: join and gather wait-for-all observation

## Status: ✅ Complete

## Description

Implement wait-for-all `join` and `gather` process observation barriers.

## Specification Reference

- SPEC-048
- SPEC-049
- SPEC-050

## Dependencies

- ✅ TASK-712: completed prerequisite task

## Requirements

### Functional Requirements

1. Consume all input handles before waiting.
2. Wait for all observed children to reach terminal state; do not fail fast.
3. Return ordered success values when all children succeed.
4. Aggregate one or more child failures while preserving every source `ProcessId`.

### Property Requirements (proptest)

```rust
// Add property-based tests for identity preservation, handle linearity,
// failure aggregation, environment projection, or typing invariants where
// this task manipulates those semantics.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target PLAN-098 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the semantic tower split.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not collapse Act, Proc, and Workflow boundaries.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic, typing, runtime identity, failure, or ordering invariants where appropriate.

## Verification Steps

- [x] Join waits for both children even when one fails early.
- [x] Gather preserves input ordering for successes.
- [x] Aggregate failure tests cover multiple failing children and source identity preservation.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
