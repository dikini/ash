# TASK-690: Cross-layer validation for parse -> type -> execute

## Status: ✅ Complete

## Description

Run end-to-end validation that expression-level act blocks parse, lower, type-check, and execute coherently across the Ash substrate.

## Specification Reference

- SPEC-047 cross-layer scope

## Dependencies

- 📝 TASK-682: prerequisite task
- 📝 TASK-687: prerequisite task
- 📝 TASK-688: prerequisite task
- 📝 TASK-689: prerequisite task

## Requirements

### Functional Requirements

1. Exercise parse -> lower -> type -> execute end-to-end examples.
2. Validate both pure and effectful boundary behavior.
3. Confirm docs, task files, and implementation behavior all align.

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

- [x] End-to-end validation examples pass cleanly.
- [x] `cargo test --all` passed during Phase 97 baseline verification.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passed during Phase 97 baseline verification.
- [x] `cargo fmt --check` passed during Phase 97 baseline verification.

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
- Cross-layer validation now includes ordinary-library `std::act::guard` permit/deny execution, async-force workflow boundary coverage for `Expr::Unary`, and the broader `cargo test -p ash-engine --test module_resolution -- --nocapture` / `cargo test -p ash-interp --test act_env_runtime_boundary -- --nocapture` focused suites.
