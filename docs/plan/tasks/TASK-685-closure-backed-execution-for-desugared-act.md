# TASK-685: Implement closure-backed execution path for desugared Act<T>

## Status: 🟡 Ready

## Description

Execute desugared `Act<T>` values through closure-backed runtime behavior that threads `ActEnv` through nested `bind` / `unit` structure.

## Specification Reference

- SPEC-047 §7.1-7.2

## Dependencies

- 📝 TASK-675: prerequisite task
- 📝 TASK-683: prerequisite task
- 📝 TASK-684: prerequisite task

## Requirements

### Functional Requirements

1. Evaluate lowered `bind`/`unit` structure into closure-backed computations.
2. Thread `ActEnv` left-to-right.
3. Preserve append-only effect and provenance behavior through runtime integration.

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

- [ ] Desugared `Act<T>` values execute correctly under runtime tests.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
