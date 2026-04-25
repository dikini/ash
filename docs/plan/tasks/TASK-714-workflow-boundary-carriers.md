# TASK-714: Workflow boundary carriers

## Status: ✅ Complete

## Description

Define workflow boundary outcome, admission context, failure, and report carrier types.

## Specification Reference

- SPEC-051
- SPEC-004
- SPEC-025

## Dependencies

- 📝 TASK-706: prerequisite task

## Requirements

### Functional Requirements

1. Add `WorkflowBoundaryOutcome`, `WorkflowFailure`, `WorkflowFailureKind`, `WorkflowReport`, and admission context types.
2. Keep `Return(...)`/`Reject(...)` as lower governed-body outcomes; do not replace SPEC-004/SPEC-025 terminal vocabulary.
3. Include `workflow_id`, `run_id`, status, evidence, lower causes, and report metadata fields required by SPEC-051.
4. Do not wire full admission/ensures/report sink behavior yet.

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

- [x] Carrier tests prove lower `ExecError`/process failures can be preserved as causes.
- [x] Reports can represent success and failure without external sink dependency.
- [x] Existing `ExecResult<Value>` APIs remain source-compatible.
- [ ] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
