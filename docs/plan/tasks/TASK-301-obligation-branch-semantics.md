# TASK-301: Verify Obligation Branch/Merge Semantics

## Status: 📝 Planned

## Description

Verify that the obligation tracking branch/merge semantics implemented in TASK-290 correctly match SPEC-022's discharge rules. The sub-agent implemented intersection semantics, but we need to confirm this is the correct interpretation.

## Current Implementation

From `crates/ash-typeck/src/obligations.rs`:

```rust
pub fn merge(&mut self, branch: Self, _parent: &Self) {
    // An obligation is discharged only if ALL branches discharged it
    // Intersection keeps obligations that are pending in BOTH contexts
    self.obligations = self.obligations.intersection(&branch.obligations);
}
```

**Question**: Is intersection correct? If an obligation is discharged in one branch but pending in another, should it:
- (A) Remain pending (intersection) - current implementation
- (B) Be considered discharged (union)?

## SPEC-022 Analysis

Per SPEC-022 Section 4.2 "Branch Discipline":
> "Obligations must be discharged on ALL execution paths. If any path leaves an obligation pending, the workflow is incomplete."

This suggests **intersection is correct** - an obligation must be discharged in ALL branches to be considered discharged overall.

## Verification Requirements

1. **Confirm Semantics**: Verify intersection matches SPEC-022
2. **Test Coverage**: Add explicit tests for:
   - Both branches discharge → obligation cleared ✓
   - One branch discharges, one pending → obligation remains ✓
   - Neither discharges → obligation remains ✓
   - Nested branches
   - Sequential workflows with obligations

3. **Edge Cases**:
   - Obligation created inside one branch only
   - Obligation discharged before branching
   - Multiple obligations with partial discharge

## Files to Review

- `crates/ash-typeck/src/obligations.rs` - `ObligationContext::merge()`
- `crates/ash-typeck/tests/obligation_tracking_test.rs` - Existing tests
- `docs/spec/SPEC-022.md` - Discharge rules specification

## Completion Checklist

- [ ] SPEC-022 discharge rules reviewed and documented
- [ ] Branch/merge semantics confirmed correct (or fixed)
- [ ] Comprehensive test coverage for branch scenarios
- [ ] Edge case tests added and passing
- [ ] Documentation comments updated in code
- [ ] `cargo test -p ash-typeck` passes
- [ ] `cargo clippy` clean
