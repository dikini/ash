# TASK-638: Typecheck List Ops Through .ash Declarations

**Status:** Planned
**Dependencies:** TASK-636
**Spec:** SPEC-034
**Estimated hours:** 2-3

## Objective

Verify list ops typecheck via their .ash declarations. Test polymorphic instantiation: len([1,2]) ~ Int, map([1,2], |x| => x+1) ~ List<Int>.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
