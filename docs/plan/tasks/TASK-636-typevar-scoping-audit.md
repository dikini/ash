# TASK-636: Audit Type-Variable Scoping at Call Sites

**Status:** Planned
**Dependencies:** None
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Verify that polymorphic builtin calls with different concrete types at different call sites typecheck correctly. Bind len as Fn([List<Var(0)>], Int), check len([1,2,3]) and len(['a','b']) both succeed independently.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
