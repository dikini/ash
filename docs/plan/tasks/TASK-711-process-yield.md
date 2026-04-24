# TASK-711: Process yield

## Status: 🟡 Ready

## Description

Implement `yield : Proc<Unit>` as an explicit cooperative process scheduling point.

## Specification Reference

- SPEC-048
- SPEC-049

## Dependencies

- 📝 TASK-709: prerequisite task

## Requirements

### Functional Requirements

1. Add process-level yield semantics distinct from workflow/proxy `Yield`.
2. Use `tokio::task::yield_now` or the runtime scheduler hook while preserving current process identity.
3. Ensure process yield does not split environment, create handles, or discharge obligations.
4. Surface cancellation/scheduler refusal only through structured operational failure when applicable.

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

- [ ] Yield tests prove current `ProcessId` is preserved.
- [ ] Yield returns `Unit` on normal scheduling.
- [ ] Existing workflow/proxy yield behavior is not changed by this task.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
