# TASK-635: Bind Imported Builtin Signatures in Engine::check()

**Status:** Planned
**Dependencies:** TASK-634
**Spec:** SPEC-034
**Estimated hours:** 3-4

## Objective

Replace arity-only synthetic type binding in all three Engine::check() paths with proper declared signature resolution. If callable has signature field, use builtin_fn_signature_type(). If not, fall back to arity-only.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
