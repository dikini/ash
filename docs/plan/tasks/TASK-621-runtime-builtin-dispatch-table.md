# TASK-621: Runtime Builtin Dispatch Table

**Status:** Planned
**Dependencies:** TASK-618 (module loader registers builtin fn)
**Spec:** SPEC-BUILTIN-FN Section 7

## Objective

Add a builtin dispatch mechanism to the evaluator. When a `builtin fn` call reaches the evaluator, dispatch to the correct Rust implementation by qualified name.

## Context

Tracks A and B are complete. The evaluator already has hardcoded dispatch in `eval_function_call` (in `crates/ash-interp/src/eval.rs`) that handles calls like `(Some("string"), "concat")` and `(_, "len")`. The module loader already registers `CallableKind::Builtin` for builtin fn declarations, but the runtime currently skips them (see `build_imported_closures` in `crates/ash-engine/src/lib.rs` line 1285).

The goal is to create a proper **builtin dispatch table** -- a static mapping from qualified names to Rust function implementations -- and route `CallableKind::Builtin` calls through it.

## Requirements

1. Create a `BuiltinTable` (or similar) in `ash-interp` that maps qualified function names (e.g., `"string::concat"`, `"string::starts_with"`) to Rust implementations.
2. When `build_imported_closures` encounters a `CallableKind::Builtin`, instead of just registering param count and continuing, register a `Value::BuiltinFn` (or similar marker) in the closures table.
3. In the evaluator's call dispatch (`Expr::Call`), when the callable is a builtin marker, look up the implementation in the dispatch table and invoke it.
4. The existing `eval_function_call` match arms for known builtins (string ops, list ops, record ops, type predicates) should remain as the implementations behind the dispatch table.
5. All existing tests must continue to pass unchanged.

## TDD Steps

1. **Red:** Write a test in `crates/ash-interp/tests/builtin_dispatch.rs` that:
   - Constructs a builtin dispatch table
   - Calls `string::concat("hello ", "world")` through the new dispatch path
   - Verifies the result is `"hello world"`
   - Also test `string::starts_with("hello", "he")` → `true`
   
2. **Green:** Implement the dispatch table and wire it into the evaluator:
   - Add `EvalError::UnimplementedBuiltin { name: String }` (needed by TASK-622 but we add the variant now)
   - Create the dispatch table as a `HashMap<String, fn(Vec<Value>) -> EvalResult<Value>>` or similar
   - Wire the builtin marker into `build_imported_closures`
   - In `eval_function_call`, after checking hardcoded match arms, check the dispatch table as a fallback before returning `UnknownFunction`

3. **Verify:** `cargo test -p ash-interp` passes. All existing behavior preserved.

## Files

- Modify: `crates/ash-interp/src/eval.rs` (add dispatch table lookup)
- Modify: `crates/ash-interp/src/error.rs` (add `UnimplementedBuiltin` variant)
- Modify: `crates/ash-engine/src/lib.rs` (update `build_imported_closures` for builtin)
- Create: `crates/ash-interp/tests/builtin_dispatch.rs`

## Estimated Hours

3-4
