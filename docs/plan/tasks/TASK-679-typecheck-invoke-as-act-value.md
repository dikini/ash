# TASK-679: Type-check invoke as Act<Value>

## Status: ✅ Complete

## Description

Teach the type checker to recognize the runtime primitive callable `invoke(provider, action, args)` as producing `Act<Value>`.

## Specification Reference

- SPEC-047 §5.4

## Dependencies

- 📝 TASK-677: prerequisite task

## Requirements

### Functional Requirements

1. Recognize expression-level `invoke(...)` calls through the existing call path.
2. Require provider/action string arguments and a `List<Value>` argument list shape.
3. Return `Act<Value>` in the type checker.

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

- [x] Type-checker rejects malformed invoke calls and accepts valid ones.
- [x] `cargo test --all` passes
- [x] `cargo clippy --all-targets --all-features` passes cleanly
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
