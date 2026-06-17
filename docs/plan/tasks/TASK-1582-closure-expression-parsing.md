# TASK-1582: Enable `fn` Expression Parsing in All Expression Contexts

**Status:** 📝 Planned
**Phase:** [PLAN-158](../PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Owner:** Phase 158

## Problem

`fn` literals cannot be parsed in general expression positions. The parser only accepts `fn` expressions in specific contexts (like `let` bindings), not when parsing arbitrary expressions like function arguments.

## Root Cause

In `crates/ash-parser/src/parse_expr.rs`, the `primary_expr()` function doesn't include `parse_fn_expr` in the list of primary expression parsers. The `fn` keyword is only handled in statement-level parsing, not expression-level parsing.

## Example Failure

```ash
use list::{map}

workflow main() -> Bool {
    let list = [1, 2, 3]
    // This fails to parse:
    let mapped = map(list, fn(x) { x + 1 })
    ret mapped == [2, 3, 4]
}
```

## Proposed Fix

Add `parse_fn_expr` to `primary_expr()` in `parse_expr.rs` so that `fn` literals can appear anywhere a primary expression is expected.

## Files to Modify

- `crates/ash-parser/src/parse_expr.rs` - Add `parse_fn_expr` to `primary_expr()`
- `crates/ash-parser/src/parse_expr.rs` - Ensure `parse_fn_expr` handles expression context correctly

## Verification

- Test `fn` literals as function arguments
- Test `fn` literals in list literals: `[fn(x) { x }, fn(x) { x + 1 }]`
- Test `fn` literals in record fields: `{ f: fn(x) { x } }`
- Ensure no regressions in existing `fn` parsing

## Notes

This is likely the simplest fix of the three. The `parse_fn_expr` function already exists; it just needs to be wired into the expression parser.
