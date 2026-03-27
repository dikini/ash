# TASK-307: Fix ash check Exit Codes (SPEC-005 Compliance)

## Status: 🔴 Critical - Phase 47/48 Gap

## Problem

`ash check` returns exit code 1 for parse errors instead of the SPEC-005 mandated exit code 2.

SPEC-005 requires:
- `0`: No errors
- `1`: Type errors or policy violations  
- `2`: Parse errors
- `3`: I/O errors

Current behavior:
- Parse errors return exit code 1 (wrapped in anyhow! then converted to CliError::General)

## Root Cause

`crates/ash-cli/src/commands/check.rs`:
```rust
Err(e) => Err(anyhow::anyhow!("Parse error: {e}")),
```

This wraps parse errors in a generic anyhow error, which `CliError::from` may not classify correctly.

## Files to Modify

- `crates/ash-cli/src/commands/check.rs` - Lines 77-83
- `crates/ash-cli/src/error.rs` - Verify From<anyhow::Error> classification
- `crates/ash-cli/tests/cli_spec_compliance_test.rs` - Add exit code assertions

## Implementation

1. Preserve error types through check() instead of wrapping in anyhow!
2. Or ensure CliError::from correctly detects "Parse error:" prefix
3. Add test assertions for exit codes in cli_spec_compliance_test.rs

## Verification

```bash
$ ash check /tmp/parse-error.ash; echo "Exit: $?"
# Should be Exit: 2, not Exit: 1
```

## Completion Checklist

- [ ] Parse errors return exit code 2
- [ ] Type errors return exit code 1
- [ ] I/O errors return exit code 3
- [ ] Tests verify exit codes
- [ ] CHANGELOG.md updated

**Estimated Hours:** 4
**Priority:** Critical (Phase 47/48 compliance gap)
