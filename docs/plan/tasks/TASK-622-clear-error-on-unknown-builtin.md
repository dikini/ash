# TASK-622: Clear Error on Unknown Builtin

**Status:** Planned
**Dependencies:** TASK-621 (runtime dispatch table)
**Spec:** SPEC-BUILTIN-FN Section 7.2

## Objective

When a `builtin fn` is called but has no Rust implementation in the dispatch table, produce a clear, distinct error message.

## Context

TASK-621 adds the dispatch table. The `EvalError::UnimplementedBuiltin` variant is already added. This task ensures the fallback path produces the correct error.

## Requirements

1. When the dispatch table lookup fails for a builtin fn, return `EvalError::UnimplementedBuiltin { name }`.
2. The error message must be: `"builtin function '{name}' declared but not implemented in runtime"`.
3. This error is distinct from `EvalError::UnknownFunction` (which is for completely unknown names).

## TDD Steps

1. **Red:** Write a test that:
   - Declares a fictional builtin (e.g., `"mystery_module::mystery"`) 
   - Calls it through the dispatch path
   - Asserts the error is `EvalError::UnimplementedBuiltin` with the correct name

2. **Green:** Ensure the fallback in the dispatch table produces `UnimplementedBuiltin` for qualified names that match known builtin patterns but aren't in the table.

3. **Verify:** Test passes. Error is distinct from `UnknownFunction`.

## Files

- Modify: `crates/ash-interp/src/eval.rs` (error path in dispatch)
- Modify: test file from TASK-621

## Estimated Hours

1-2
