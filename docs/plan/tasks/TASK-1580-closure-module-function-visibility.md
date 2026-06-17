# TASK-1580: Fix Module-Level Function Visibility Inside Closures

**Status:** 📝 Planned
**Phase:** [PLAN-158](../PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Owner:** Phase 158

## Problem

Module-level functions are not accessible from within closures defined in workflows. When a closure calls a module-level function, the interpreter fails with `UndefinedVariable`.

## Root Cause

The closure environment capture (`EnvFrame`) is lexical but only captures local scopes. Module-level function bindings are stored in the module's symbol table, not in the runtime environment frame. When the closure's `EnvFrame` is searched, it walks the parent chain of local scopes but doesn't reach back to the module's exported names.

## Example Failure

```ash
fn add_one(x: Int) -> Int { x + 1 }
fn mul_two(x: Int) -> Int { x * 2 }
fn compose(x: Int) -> Int { add_one(mul_two(x)) }  // UndefinedVariable("add_one")

workflow main() -> Bool {
    let list = [1, 2, 3]
    let mapped = map(list, compose)  // compose fails internally
    ret mapped == [2, 3, 4]
}
```

## Proposed Fix

Option A: Inject module-level function bindings into the closure capture environment at closure creation time.

Option B: Modify variable lookup to fall back to module exports after checking local scopes.

Option C: Store module-level functions in the top-level `EnvFrame` so they're naturally in the parent chain.

## Files to Modify

- `crates/ash-interp/src/eval.rs` - Closure creation and application
- `crates/ash-core/src/env_frame.rs` - Environment frame lookup
- `crates/ash-engine/src/lib.rs` - Module execution setup

## Verification

- Test that module-level functions can be called from closures
- Test nested function composition
- Ensure no regressions in existing closure behavior

## Notes

This is the deepest fix of the three issues. May require changes to how module-level bindings are stored in the runtime environment.
