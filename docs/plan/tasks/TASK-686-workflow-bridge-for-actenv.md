# TASK-686: Workflow bridge: construct/apply ActEnv from workflow context

## Status: 🟡 Ready

## Description

Bridge expression-level `Act<T>` values into workflow execution by constructing an `ActEnv` from the existing workflow runtime context when needed.

## Specification Reference

- SPEC-047 §7.4
- SPEC-047 §8.3

## Dependencies

- 📝 TASK-683: prerequisite task
- 📝 TASK-685: prerequisite task

## Requirements

### Functional Requirements

1. Build `ActEnv` from workflow capability/policy/provenance state.
2. Apply expression-level `Act<T>` values when encountered from workflow execution contexts that support the bridge.
3. Leave `Workflow::Act` unchanged in Phase 97.

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

- [ ] Workflow/expression interop cases execute without regressing workflow-level act semantics.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
