# TASK-643: Delete add_builtin_functions()

**Status:** Planned
**Dependencies:** TASK-639, TASK-642
**Spec:** SPEC-034
**Estimated hours:** 1

## Objective

Remove hardcoded type-env registrations for list ops now covered by .ash declarations. Unblocked portion of TASK-631B.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
