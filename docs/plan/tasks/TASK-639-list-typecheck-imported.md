# TASK-639: Typecheck List Ops Through Imported .ash Declarations

**Status:** Planned
**Dependencies:** TASK-635, TASK-637
**Spec:** SPEC-034
**Estimated hours:** 2-3

## Objective

Verify list ops typecheck with correct polymorphic types through the engine import path. Test len([1,2]) as Int, map([1,2],|x|=>x+1) as List<Int>, len('not a list') as type error.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
