# TASK-313: Fix prop_partial_discharge_scenario Proptest

## Status: 🔴 Critical - Test Regression

## Problem

`prop_partial_discharge_scenario` fails with specific minimal input:

```
role = (
    "_a",           // role name
    [],             // granted capabilities
    ["__", "__", "aa"],  // obligation patterns
)

left: `1`,
right: `2`: Should have correct number of pending obligations
```

## Root Cause

Obligation tracking in role runtime props test has a bug where the expected count (2) doesn't match actual (1).

`crates/ash-interp/tests/role_runtime_props.rs:599`

## Files to Modify

- `crates/ash-interp/tests/role_runtime_props.rs` - Lines 473, 599

## Investigation Needed

1. Is the test expectation wrong?
2. Is the obligation tracking logic wrong?
3. Is this a real bug or test artifact?

## Completion Checklist

- [ ] Root cause identified
- [ ] Test passes with all inputs
- [ ] CHANGELOG.md updated

**Estimated Hours:** 4
**Priority:** Critical (test regression)
