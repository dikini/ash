# TASK-716: Workflow boundary completion and report construction

## Status: 🟡 Ready

## Description

Implement workflow completion boundary checks, ensures evaluation, lower failure reinterpretation, and in-memory report construction.

## Specification Reference

- SPEC-051
- SPEC-049
- SPEC-050

## Dependencies

- 📝 TASK-713: prerequisite task
- 📝 TASK-715: prerequisite task

## Requirements

### Functional Requirements

1. Use `ExecutionRecord` and process summaries as lower evidence inputs for `WorkflowReport`.
2. Evaluate workflow-level `ensures` predicates after normal body completion and before success reporting.
3. Check local and role obligation completion at workflow boundary.
4. Reinterpret escaping process/operational/body failures as `WorkflowFailure` while preserving lower causes.
5. Construct a minimal in-memory report for every boundary success/failure.

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

- [ ] Ensures failures map to workflow-boundary failures with evidence.
- [ ] Escaping lower failures become workflow failures with preserved causes.
- [ ] Undischarged obligations are reported as workflow-boundary failures.
- [ ] A local report exists even when an optional external report sink is absent.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
