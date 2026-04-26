# TASK-717: Semantic tower cross-layer validation

## Status: ✅ Complete

## Description

Validate the full PLAN-098 slice across parser, typechecker, interpreter, engine, docs, and examples.

## Specification Reference

- SPEC-048
- SPEC-049
- SPEC-050
- SPEC-051

## Dependencies

- ✅ TASK-713: completed prerequisite task
- ✅ TASK-716: completed prerequisite task

## Requirements

### Functional Requirements

1. Add end-to-end examples for `fail`/`with_error`, `Proc`/`P`, `par`/`await`/`join`/`gather`, and workflow boundary reporting.
2. Run doc/spec drift checks and update any canonical spec/task references found stale.
3. Verify compatibility wrappers for existing workflow execution APIs.
4. Update CHANGELOG and PLAN-INDEX task status as implementation completes.

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

- [x] Cross-layer tests pass from parse through execution.
- [x] Workspace verification commands pass.
- [x] Docs and task status match implemented behavior.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
- Added `examples/05-phase98/01-fail-with-error.ash` through `04-workflow-boundary-reporting.ash` plus CLI/engine conformance coverage. The proc example now builds source-level `await`/`join` observer procs while engine tests wait for retained terminal child state before forcing them, preserving the honest boundary that CLI `run`/`trace` still render opaque `Proc` closures.
- Verified fresh gates with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --all` in the Phase 98 worktree.
