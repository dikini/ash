# TASK-310: Fix cli_input_workflow_test Failures

## Status: 🔴 Critical - Phase 49 Test Failures

## Problem

`cargo test --workspace --quiet` fails with 3 failing tests in `cli_input_workflow_test.rs`:

1. `test_multiple_workflow_parameters` - String + Int concatenation error
2. `test_list_workflow_parameter` - Parse error for `List<Int>` syntax
3. `test_boolean_workflow_parameter` - Wrong output ("off" instead of "on")

## Root Causes

### Test 1: String concatenation
```
error: evaluation error: invalid binary operation: add on String("localhost:") and Int(8080)
```
The test expects string concatenation to work with mixed types.

### Test 2: List syntax
```
error: parse error: Parsing Error: ContextError { context: [], cause: None }
```
Parser doesn't support `List<Int>` in workflow parameters.

### Test 3: Boolean handling
Test expects "on" but gets "off" - interpreter boolean to string conversion issue.

## Files to Modify

- `crates/ash-cli/tests/cli_input_workflow_test.rs` - Fix tests or mark known issues
- May need parser/interpreter fixes for List<Int> syntax

## Options

1. **Fix the underlying issues** (parser, interpreter)
2. **Adjust tests** to match actual behavior
3. **Mark as known issues** with `#[ignore]` and document

Given these are pre-existing limitations, option 3 may be appropriate for Phase 49 closeout.

## Completion Checklist

- [ ] Root cause identified for each failure
- [ ] Tests fixed, adjusted, or marked as known issues
- [ ] `cargo test --workspace --quiet` passes
- [ ] CHANGELOG.md updated

**Estimated Hours:** 4
**Priority:** Critical (blocks phase verification)
