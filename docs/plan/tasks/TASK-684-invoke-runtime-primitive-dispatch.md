# TASK-684: Add invoke runtime primitive dispatch through Expr::Call

## Status: 🟡 Ready

## Description

Implement the runtime dispatch path for expression-level `invoke(...)` calls via the existing `Expr::Call` machinery rather than through a new AST variant.

## Specification Reference

- SPEC-047 §7.2

## Dependencies

- 📝 TASK-683: prerequisite task

## Requirements

### Functional Requirements

1. Route `invoke` through a clearly distinguished runtime primitive path.
2. Capture provider/action/args into closure-backed `Act<T>` execution state.
3. Preserve existing pure builtin behavior for other call targets.

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

- [ ] `invoke(...)` dispatch works without introducing a new core AST node.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
