# TASK-300: CLI Input Integration Tests

## Status: 📝 Planned

## Description

Unignore and verify all CLI `--input` integration tests once TASK-299 is complete. The tests were created in TASK-292 but marked as `#[ignore]` pending type checker integration.

## Current State

5 integration tests exist in `crates/ash-cli/tests/cli_input_workflow_test.rs`:

| Test | Status | Issue |
|------|--------|-------|
| `test_input_bound_to_workflow_parameters` | ignored | UnboundVariable("name") |
| `test_multiple_workflow_parameters` | ignored | UnboundVariable |
| `test_boolean_workflow_parameter` | ignored | UnboundVariable |
| `test_list_workflow_parameter` | ignored | UnboundVariable |
| `test_missing_required_parameter` | passing | Error handling works |

## Dependencies

- ✅ TASK-292: Make CLI --input functional (tests created)
- ⏳ TASK-299: Type checker parameter binding (required)

## Requirements

1. **Unignore Tests**: Remove `#[ignore]` attributes from 4 tests
2. **Verify Functionality**: All tests must pass with actual input binding
3. **Edge Cases**: Add any missing edge case tests

## Test Coverage Required

### Basic Parameter Types
- `String` parameter binding
- `Int` parameter binding  
- `Bool` parameter binding
- `List<T>` parameter binding

### Multiple Parameters
- Two parameters of different types
- Three+ parameters
- Parameter order independence

### Error Cases
- Missing required parameter
- Extra parameter in input (should be ignored or warned)
- Type mismatch (String vs Int)

### Complex Workflows
- Parameter used in condition
- Parameter used in loop
- Parameter passed to nested workflow

## Files to Modify

- `crates/ash-cli/tests/cli_input_workflow_test.rs` - Remove `#[ignore]` attributes

## Completion Checklist

- [ ] All 4 ignored tests unignored
- [ ] All 5 CLI input tests pass
- [ ] No regressions in existing CLI tests
- [ ] `cargo test -p ash-cli` passes
- [ ] `cargo clippy --all-targets` clean
- [ ] CHANGELOG.md updated with completion note
