# TASK-634: Audit Type-Variable Scoping in Call Resolution

**Status:** Planned
**Dependencies:** None
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Determine whether instantiate_fn_call needs freshening for polymorphic builtin fn calls. Write a test calling len([1,2,3]) then len(['a','b']) and check both typecheck correctly. If substitutions are per-call (local), no freshening needed. If accumulated, freshening required.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
