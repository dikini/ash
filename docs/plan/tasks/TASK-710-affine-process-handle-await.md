# TASK-710: Affine process handles and await

## Status: ✅ Complete

## Description

Implement affine `P<A>` runtime handles and the single-handle `await` observation primitive.

## Specification Reference

- SPEC-048
- SPEC-049
- SPEC-050

## Dependencies

- 📝 TASK-707: prerequisite task
- 📝 TASK-709: prerequisite task

## Requirements

### Functional Requirements

1. Represent process handles as opaque runtime values carrying `ProcessId` and result type metadata as available.
2. Consume handles at observation time and reject use-after-consume dynamically until static linear typing exists.
3. Implement `await : P<A> -> Proc<A>` against retained terminal process state.
4. Return success values or raise observed process failures preserving child `ProcessId`.

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

- [x] Use-after-consume tests fail with structured handle-consumed error.
- [x] Await success returns the child result.
- [x] Await failure preserves source `ProcessId` and lower cause.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-098.

## Notes

- Preserve existing workflow/control-link behavior unless this task explicitly changes it.
- Keep `Proc<A>` distinct from `Act<A>` and `Workflow`.
- Child workflow spawn now also records retained terminal process state in the process registry without replacing workflow `ControlLink` supervision.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
