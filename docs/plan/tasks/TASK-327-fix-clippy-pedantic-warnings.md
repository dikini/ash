# TASK-327: Fix Clippy Pedantic Warnings

## Status: 🟡 High

## Problem

Clippy produces pedantic warnings in test code that indicate potential code quality issues:

```
warning: casting `usize` to `i64` may wrap around the value
  --> crates/ash-engine/tests/e2e_capability_provider_tests.rs:67:28

warning: variables can be used directly in the `format!` string
  --> crates/ash-engine/tests/e2e_capability_provider_tests.rs:478:34
```

## Files to Modify

- `crates/ash-engine/tests/e2e_capability_provider_tests.rs` (9 warnings total)

## Implementation

### Fix cast_possible_wrap warnings (lines 67, 84)

Add `#[allow(clippy::cast_possible_wrap)]` with justification comment since test counts won't exceed i64 range.

### Fix uninlined_format_args warnings (lines 478, 499, 646, 647, 833, 834, 851)

Convert `format!("...{}", var)` to `format!("...{var}")`.

## Verification

```bash
# Should produce no warnings
cargo clippy --package ash-engine --tests -- -W clippy::pedantic --quiet

# All tests still pass
cargo test --package ash-engine --quiet
```

## Completion Checklist

- [ ] cast_possible_wrap warnings fixed (2 occurrences)
- [ ] uninlined_format_args warnings fixed (7 occurrences)
- [ ] `cargo clippy --package ash-engine --tests -- -W clippy::pedantic` clean
- [ ] All tests pass
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2
**Priority:** High (code quality)
