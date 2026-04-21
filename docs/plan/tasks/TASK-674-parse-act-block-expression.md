# TASK-674: Parse act block in expression context

## Status: 🟡 Ready

## Description

Extend `parse_expr.rs` so `act { ... }` is accepted in expression position while leaving workflow-level `act ...` parsing unchanged.

## Specification Reference

- SPEC-047 §2.1
- SPEC-047 §4.3

## Dependencies

- 📝 TASK-673: prerequisite task

## Requirements

### Functional Requirements

1. Parse `act { ... }` into `surface::Expr::ActBlock`.
2. Parse `IDENTIFIER = expr;` and `ret expr;` statement forms inside the block.
3. Preserve the existing workflow-context `act_stmt()` behavior in `parse_workflow.rs`.

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

- [ ] Parser tests cover dual-context `act` disambiguation.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
