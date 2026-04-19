# TASK-637: Add List Ops to Builtin Dispatch Table

**Status:** Planned
**Dependencies:** TASK-636
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Add qualified entries (list::len, list::head, etc.) to builtin_dispatch_table() in eval.rs. Test dispatch through qualified names.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
