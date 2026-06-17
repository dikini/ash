# TASK-1583: Verification and Regression Tests for Language Surface Fixes

**Status:** 📝 Planned
**Phase:** [PLAN-158](../PLAN-158-LANGUAGE-SURFACE-FIXES.md)
**Owner:** Phase 158

## Goal

Add comprehensive tests for the three language surface fixes and ensure no regressions in existing functionality.

## Tests to Add

### For TASK-1580 (Module-level function visibility)

1. `test_module_function_in_closure` - Module-level function called from closure
2. `test_nested_module_function_calls` - `compose(x) { add_one(mul_two(x)) }`
3. `test_multiple_module_functions_in_closure` - Multiple module functions used

### For TASK-1581 (Function vs capability resolution)

1. `test_imported_function_not_capability` - Imported function works
2. `test_renamed_import_not_capability` - `use list::{reverse as rev}` works
3. `test_capability_calls_still_work` - Existing capability calls unaffected

### For TASK-1582 (Closure expression parsing)

1. `test_fn_literal_as_argument` - `map(list, fn(x) { x + 1 })`
2. `test_fn_literal_in_list` - `[fn(x) { x }, fn(x) { x + 1 }]`
3. `test_fn_literal_in_record` - `{ f: fn(x) { x } }`
4. `test_fn_literal_in_let` - `let f = fn(x) { x }` (existing, ensure no regression)

## Verification

- `cargo test --workspace` passes (or only pre-existing failures)
- `cargo clippy --workspace --all-targets -- -D warnings` passes
- `cargo fmt --check` passes

## Notes

Tests should be added to appropriate test files:
- `crates/ash-engine/tests/` for engine-level tests
- `crates/ash-parser/tests/` for parser-level tests
- `crates/ash-interp/tests/` for interpreter-level tests
