# TASK-1581: Distinguish Function Calls from Capability Calls in Lowerer

**Status:** 📝 Planned
**Phase:** [PLAN-158](../PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Owner:** Phase 158

## Problem

The lowerer conflates function calls with capability calls. When the parser sees a call like `reverse(list)`, it checks if `reverse` is a "symbolic capability" via `ctx.resolve_capability()`. If the name is not registered as a capability, it fails with "unresolved symbolic capability" instead of falling back to function lookup.

## Root Cause

In `crates/ash-parser/src/lower.rs` (around line 962), the lowering code for `SurfaceWorkflow::Act` always treats symbolic calls as capability lookups:

```rust
crate::surface::OperationalTarget::Symbolic { capability_name } => {
    match ctx.resolve_capability(capability_name.as_ref()) {
        Some((provider, action)) => (provider, action),
        None => {
            return Err(LoweringError::UnresolvedCapability { ... })
        }
    }
}
```

The lowerer doesn't check if the name is a known function/import before falling back to capability resolution.

## Example Failure

```ash
use list::{reverse as rev}

workflow main() -> Bool {
    let list = [1, 2, 3]
    let r = rev(list)  // Error: unresolved symbolic capability 'rev'
    ret r == [3, 2, 1]
}
```

## Proposed Fix

Modify the lowering pipeline to:
1. First check if the name is a known function in the module's imports/exports
2. Only if not found, fall back to capability resolution
3. If neither, then report the error

## Files to Modify

- `crates/ash-parser/src/lower.rs` - Workflow lowering logic
- `crates/ash-parser/src/lower.rs` - Expression lowering logic (if similar issue exists)

## Verification

- Test that imported functions work when called
- Test that capability calls still work
- Test that `reverse` (and other list functions) work when imported

## Notes

This may require passing module import information to the lowering context, or checking the module's exported names before capability resolution.
