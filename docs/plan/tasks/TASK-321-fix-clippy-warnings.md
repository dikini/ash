# TASK-321: Fix Clippy Warnings in Test Code

## Status: 🟡 Medium - Code Quality

## Problem

Workspace is not clippy-clean. Running `cargo clippy --workspace --all-targets --all-features --quiet` produces warnings in Phase 49/50 test code.

**Current warnings:**
- redundant closure
- redundant clone
- temporary with significant `Drop` can be early dropped
- casting `usize` to `i64` may wrap around
- variables can be used directly in `format!` string
- this `if` statement can be collapsed

## Files with Warnings

- `crates/ash-engine/tests/role_runtime_integration_tests.rs:25`
- `crates/ash-engine/tests/e2e_capability_provider_tests.rs:64`
- `crates/ash-cli/tests/json_output_schema_test.rs:159`

## Implementation

Run `cargo clippy --workspace --all-targets --all-features --fix` and review changes.

## Verification

```bash
$ cargo clippy --workspace --all-targets --all-features --quiet
# Should produce no warnings
```

## Completion Checklist

- [ ] All clippy warnings resolved
- [ ] `cargo clippy --workspace --all-targets --all-features --quiet` clean
- [ ] All tests still pass
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2
**Priority:** Medium (code quality, verification claim)
