# TASK-643: Delete add_builtin_functions()

**Status:** Planned
**Dependencies:** TASK-640
**Spec:** SPEC-034
**Estimated hours:** 1

## Objective

Remove hardcoded list-op type registrations now covered by .ash declarations. Contains only list ops (predicates removed by TASK-631A). Depends only on list op completion, not predicate work.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
