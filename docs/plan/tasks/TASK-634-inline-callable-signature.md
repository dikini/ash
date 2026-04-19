# TASK-634: Add signature Field to InlineCallable

**Status:** Planned
**Dependencies:** None
**Spec:** SPEC-034
**Estimated hours:** 2-3

## Objective

Add signature: Option<ash_parser::surface::BuiltinFnDef> to InlineCallable in module_loader.rs. Populate in parse_builtin_fn_callable(). All existing construction sites use None.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
