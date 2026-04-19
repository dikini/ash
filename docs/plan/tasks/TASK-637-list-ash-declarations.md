# TASK-637: Create std/src/list.ash with Generic Builtin Declarations

**Status:** Planned
**Dependencies:** TASK-635
**Spec:** SPEC-034
**Estimated hours:** 2-3

## Objective

Create std/src/list.ash with 7 generic builtin fn declarations (len, head, tail, append, concat, filter, map). Verify whether std root module export changes are actually needed.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
