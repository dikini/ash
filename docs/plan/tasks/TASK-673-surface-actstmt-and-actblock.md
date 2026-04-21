# TASK-673: Add surface ActStmt and Expr::ActBlock

## Status: 🟡 Ready

## Description

Add the surface-only parser/lowering carrier for expression-level act blocks. This task introduces `surface::ActStmt` and `surface::Expr::ActBlock` without modifying core IR.

## Specification Reference

- SPEC-047 §4

## Dependencies

- 📝 TASK-672: prerequisite task

## Requirements

### Functional Requirements

1. Add `ActStmt` to `crates/ash-parser/src/surface.rs`.
2. Add `Expr::ActBlock { stmts, span }` to the surface expression enum.
3. Preserve span-carrying structure suitable for diagnostics and lowering.

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

- [ ] Surface AST compiles and all downstream exhaustive matches are updated.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
