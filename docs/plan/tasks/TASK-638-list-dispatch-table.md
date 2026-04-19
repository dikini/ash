# TASK-638: Complete List-Op Qualified Dispatch Wiring

**Status:** Planned
**Dependencies:** TASK-637
**Spec:** SPEC-044
**Estimated hours:** 1

## Objective

Add qualified dispatch aliases (list::len, list::head, etc.) to builtin_dispatch_table().
The runtime already supports these as unqualified builtins; this task adds the
qualified-name entries and ensures the imported builtin path uses them consistently.
This is wiring/consistency work, not new runtime semantics.

## TDD Steps

1. **Red:** Write failing test per objective.
2. **Green:** Implement to pass.
3. **Verify:** `cargo test` passes, clippy clean.
