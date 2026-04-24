# TASK-708: Operational fail and with_error

## Status: 🟡 Ready

## Description

Implement `fail` as operational bottom and `with_error` as scoped operational failure handling.

## Specification Reference

- SPEC-050
- SPEC-004

## Dependencies

- 📝 TASK-706: prerequisite task

## Requirements

### Functional Requirements

1. Add explicit surface/core carriers for `fail` and `with_error` or document why a different representation is required before coding.
2. Type `fail e` as bottom-compatible in expression/workflow positions covered by SPEC-050.
3. Implement scoped dynamic handling that catches operational failures, not ordinary `Result` values.
4. Preserve lower failure identity/cause when handlers reinterpret failures.

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

- [ ] Failing tests cover bottom typing and branch unification.
- [ ] Handlers catch failures raised in dynamic scope and do not catch ordinary `Err` values.
- [ ] Existing `panic` behavior is reconciled or explicitly separated from `fail`.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
