# TASK-641: Add Type Predicates to Builtin Dispatch Table

**Status:** Planned
**Dependencies:** TASK-640
**Spec:** SPEC-034
**Estimated hours:** 1

## Objective

Add qualified entries (predicate::is_int, etc.) to builtin_dispatch_table(). Test dispatch.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
