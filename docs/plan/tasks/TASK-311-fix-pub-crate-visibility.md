# TASK-311: Fix pub(crate) Visibility Enforcement (SPEC-009)

## Status: 🟡 Medium - Phase 48 Gap

## Problem

`pub(crate)` visibility is effectively unenforced. The visibility checker returns `true` for all `Visibility::Crate` lookups.

`crates/ash-typeck/src/visibility.rs:132-136`:
```rust
Visibility::Crate => {
    // For now, we assume same crate if both paths start with the same root
    // In a real implementation, this would check crate roots
    true
}
```

This is a SPEC-009 compliance hole, not just a refactor TODO.

## Root Cause

No crate-root tracking in the visibility system. All items are visible to all other items.

## Files to Modify

- `crates/ash-typeck/src/visibility.rs` - Lines 132-136
- May need ModulePath to track crate root

## Implementation

Options:
1. Track crate root in ModulePath
2. Use a simple heuristic (same top-level directory)
3. Document as known limitation for now

## Verification

```rust
// In crate A
pub(crate) fn internal() {}  // Should not be visible from crate B

// In crate B  
use crate_a::internal;  // Should fail
```

## Completion Checklist

- [ ] pub(crate) properly restricts visibility
- [ ] Tests verify cross-crate restrictions
- [ ] CHANGELOG.md updated

**Estimated Hours:** 6
**Priority:** Medium (SPEC-009 compliance gap)
