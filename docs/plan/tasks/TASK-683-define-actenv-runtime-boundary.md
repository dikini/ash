# TASK-683: Define ActEnv runtime struct and boundary

## Status: ✅ Complete

## Description

Introduce the runtime-only `ActEnv` carrier needed to thread capability context, policy stack, provenance, and effect-log state through expression-level effectful computation.

## Specification Reference

- SPEC-047 §7.3

## Dependencies

- 📝 TASK-672: prerequisite task

## Requirements

### Functional Requirements

1. Define `ActEnv` in the interpreter/runtime layer only.
2. Do not expose `ActEnv` as an Ash value.
3. Make the boundary explicit enough for later runtime primitive integration.

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

- [x] Interpreter/runtime compiles with the new `ActEnv` carrier.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
