# TASK-314: Fix Interpreter Boolean to String Display

## Status: 📝 Planned

## Problem

The interpreter outputs "off" for boolean values instead of "true"/"false".

**Expected per user clarification:** Boolean values should display as "true" or "false"

**Current behavior:** Boolean values display as "off" (and likely "on" for true)

## Reproduction

```bash
$ echo 'workflow test() { ret true; }' > test.ash
$ ash run test.ash
off  # Should be "true"
```

Or in the ignored test:
```rust
// workflow toggle(enabled: Bool) { if enabled then { ret "on"; } ret "off"; }
// toggle(true) returns "on" via if branch, but ret true would output "off"
```

## Root Cause

The interpreter's Value to string conversion for Boolean variant uses incorrect string representations. Likely in `crates/ash-interp/src/value.rs` or similar.

## Files to Investigate

- `crates/ash-interp/src/value.rs` - Display/ToString implementation for Value::Bool
- `crates/ash-core/src/value.rs` - May also have Display impl

## Implementation

Find where boolean values are converted to strings and change:
- `true` → display as "true"
- `false` → display as "false"

## Verification

After fix:
```bash
$ ash run test.ash
true  # Correct
```

And `test_boolean_workflow_parameter` test should pass.

## Spec Impact

SPEC-004 (Interpreter) should be updated to explicitly state boolean stringification format.

## Completion Checklist

- [ ] Boolean `true` displays as "true"
- [ ] Boolean `false` displays as "false"
- [ ] `test_boolean_workflow_parameter` test passes
- [ ] All other tests still pass
- [ ] CHANGELOG.md updated
- [ ] SPEC-004 updated with boolean display rule

**Estimated Hours:** 2
**Priority:** Low (behavioral cleanup)
