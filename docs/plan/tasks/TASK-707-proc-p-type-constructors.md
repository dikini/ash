# TASK-707: Register Proc and P type constructors

## Status: ✅ Complete

## Description

Register `Proc<A>` and `P<A>` as type-level constructors without enabling process operations yet.

## Specification Reference

- SPEC-048
- SPEC-049

## Dependencies

- 📝 TASK-705: prerequisite task
- 📝 TASK-706: prerequisite task

## Requirements

### Functional Requirements

1. Register `Proc` and `P` with arity/kind `* -> *` in the type environment.
2. Teach parser/type conversion paths to preserve `Proc<A>` and `P<A>` through generic constructor syntax.
3. Add diagnostics for unknown/mis-arity process constructors.
4. Do not add `par`/`await`/`join` runtime behavior in this task.

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

- [x] Typechecker accepts well-formed `Proc<Int>` and `P<String>` annotations.
- [x] Typechecker rejects malformed `Proc`/`P` arity.
- [x] Existing non-process type constructor behavior is unchanged.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

Verification evidence recorded during completion:

- Red: `cargo test -p ash-typeck --test task_707_proc_type_constructors -- --nocapture` failed before registration because `Proc` and `P` were unbound/mis-arity cases were not reported.
- Green: `cargo test -p ash-typeck --test task_707_proc_type_constructors -- --nocapture` passed with 8 tests.
- Targeted gate: `cargo fmt --check && cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings` passed.
- Final workspace gate: `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo doc --workspace --no-deps`, and `git diff --check` passed.

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
