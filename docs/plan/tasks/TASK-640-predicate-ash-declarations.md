# TASK-640: Create std/src/predicate.ash with Generic Builtin Declarations

**Status:** Planned
**Dependencies:** TASK-634
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Create std/src/predicate.ash with 6 generic builtin fn declarations (is_int, is_string, is_bool, is_list, is_record, is_null). Wire pub mod predicate into stdlib.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
