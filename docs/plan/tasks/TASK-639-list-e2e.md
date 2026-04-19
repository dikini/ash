# TASK-639: End-to-End List Ops Verification

**Status:** Planned
**Dependencies:** TASK-637, TASK-638
**Spec:** SPEC-034
**Estimated hours:** 1-2

## Objective

Full integration: parse, typecheck, evaluate list::len([10,20,30])=3, list::map([1,2,3],|x|=>x*2)=[2,4,6].

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
