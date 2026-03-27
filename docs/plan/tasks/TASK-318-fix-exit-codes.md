# TASK-318: Fix ash check Exit Codes (SPEC-005 Compliance)

## Status: 🔴 Critical - Phase 50 Gap

## Problem

Exit codes are **still not SPEC-005 compliant**:

| Error Type | Current Code | SPEC-005 Code |
|------------|--------------|---------------|
| Type errors | 3 | **1** |
| I/O errors | 6 | **3** |

**Current mapping (error.rs:80-88):**
```rust
CliError::TypeError { .. } => ExitCode::from(3),  // WRONG - should be 1
CliError::IoError { .. } => ExitCode::from(6),    // WRONG - should be 3
```

**SPEC-005 Section 4:**
- `0`: No errors
- `1`: Type errors or policy violations
- `2`: Parse errors
- `3`: I/O errors

## Root Cause

The ExitCode mapping in `crates/ash-cli/src/error.rs` doesn't match SPEC-005.

## Files to Modify

- `crates/ash-cli/src/error.rs` - Lines 20, 43, 81, 84
- `crates/ash-cli/tests/cli_spec_compliance_test.rs` - Verify correct exit codes

## Implementation

Fix the exit code mapping:
```rust
CliError::TypeError { .. } => ExitCode::from(1),  // Fixed
CliError::IoError { .. } => ExitCode::from(3),    // Fixed
```

Update comments to reflect SPEC-005.

## Verification

```bash
# Type error should exit 1
$ ash check /tmp/type-error.ash; echo $?
1

# Missing file should exit 3
$ ash check /tmp/nonexistent.ash; echo $?
3
```

## Completion Checklist

- [ ] Type errors return exit code 1
- [ ] I/O errors return exit code 3
- [ ] Comments updated to reflect SPEC-005
- [ ] Tests verify correct exit codes
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2
**Priority:** Critical (SPEC-005 compliance)
