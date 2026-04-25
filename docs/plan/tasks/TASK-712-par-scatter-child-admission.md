# TASK-712: par and scatter child admission

## Status: ✅ Complete

## Description

Implement `par` and `scatter` all-or-none child process admission returning handles.

## Specification Reference

- SPEC-048
- SPEC-049
- SPEC-050

## Dependencies

- ✅ TASK-710: prerequisite task
- ✅ TASK-711: prerequisite task

## Requirements

### Functional Requirements

1. Implement two-child `par` admission so both children are registered before either child user code can run.
2. Implement `scatter` as ordered all-or-none generalized admission.
3. Return affine handles before child failures affect the parent.
4. Ensure `with_error { par(...) }` catches only start/admission/handle-creation failures.

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

- [x] Admission failure leaves no partially runnable children.
- [x] Returned handles preserve child order and child `ProcessId`s.
- [x] Child body failure after handle return is observed only through `await`/`join`/`gather`.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Completion Notes

- Added `proc::par` and `proc::scatter` builtin/typechecker/stdlib surface support for ordered all-or-none child admission.
- Runtime admission now batch-registers child processes with parent linkage and stable `child_index` ordering before child user code is observed.
- `proc::par` currently returns a list-backed pair of handles, and evaluator numeric field projection (`.0` / `.1`) now works consistently for that tuple-style surface in both sync and async paths.
- Added focused parser/typechecker/interpreter coverage for ordered handle return, deferred child-failure observation via later `proc::await`, and rollback on admission failure.

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
