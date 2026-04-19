# TASK-641: Create std/src/predicate.ash with Generic Builtin Declarations

**Status:** Planned
**Dependencies:** TASK-635
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Create std/src/predicate.ash with 6 generic builtin fn declarations (is_int, is_string, is_bool, is_list, is_record, is_null). Verify whether std root module export changes are needed.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
