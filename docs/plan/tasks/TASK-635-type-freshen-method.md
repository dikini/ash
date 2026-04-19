# TASK-635: Add `freshen` to Type (Conditional)

**Status:** Planned
**Dependencies:** TASK-634
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Only if TASK-634 shows freshening is needed. Add Type::freshen() that replaces all Type::Var with fresh variables via memo HashMap. Must preserve variable identity (same var maps to same fresh var).

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
