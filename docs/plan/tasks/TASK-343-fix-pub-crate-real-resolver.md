# TASK-343: Fix pub(crate) Enforcement for Real Resolver Path

## Status: ✅ Complete

## Problem

TASK-332 diverged from the Phase 54 plan by introducing `CrateId` infrastructure, but this infrastructure is only used in tests. The real resolver path (`resolver.rs:resolve_crate`) never calls `set_crate()`, leaving `pub(crate)` enforcement non-operational on real loader-built graphs.

**Root Cause:**
```rust
// import_resolver.rs:483-491 (before fix)
Visibility::Crate => {
    match (self.module_graph.crate_for(importing_module),
           self.module_graph.crate_for(target_module)) {
        (Some(importing_crate), Some(target_crate)) => importing_crate == target_crate,
        _ => false, // <-- Always hit in production since set_crate() is never called
    }
}
```

**Plan Violation:**
PHASE-54-IMPLEMENTATION-PLAN.md:64-68 explicitly stated:
> "If the resolver still only supports one crate root, do **not** introduce `CrateId` unless it is required by the implementation. Keep the task scoped to the current import model..."

## Solution

Since Phase 54 only supports single-crate graphs, `pub(crate)` now checks that both modules exist in the graph (same-crate model), rather than using CrateId membership.

### Implementation

Changed `is_visible()` for `Visibility::Crate` from CrateId-based check to graph membership check:

```rust
Visibility::Crate => {
    // Single-crate model: pub(crate) visible to all modules in this graph
    self.module_graph.nodes.contains_key(&importing_module)
        && self.module_graph.nodes.contains_key(&target_module)
}
```

## Files Modified

1. `crates/ash-parser/src/import_resolver.rs` - Fixed `is_visible()` for Crate visibility
2. `crates/ash-parser/tests/visibility_integration_test.rs` - Added regression tests

## Regression Tests Added

### test_pub_crate_visibility_check_with_imports
Tests that pub(crate) visibility is correctly enforced when resolving imports. Creates a graph WITHOUT calling `set_crate()`, adds both exports and use statements, then verifies the import succeeds.

### test_pub_crate_rejects_external_module
Tests that imports from non-existent modules (external crates) are rejected.

## Verification

```bash
✅ cargo test --package ash-parser import_resolver --quiet (35 tests pass)
✅ cargo test --package ash-parser --test visibility_integration_test (14 tests pass)
✅ cargo test --workspace --quiet (all tests pass)
✅ cargo clippy --package ash-parser -- -D warnings
✅ cargo fmt --check
```

## Completion Checklist

- [x] `is_visible()` updated to not rely on CrateId for pub(crate)
- [x] Real resolver path correctly allows pub(crate) imports
- [x] All existing tests still pass (49 visibility tests)
- [x] Regression tests added that exercise the actual visibility check
- [x] Clippy clean
- [x] CHANGELOG.md updated with accurate description

**Estimated Hours:** 0.5
**Actual Hours:** 0.5
**Priority:** Critical (functional defect)
**Related:** TASK-332, Phase 55 cross-crate planning
**Note:** True cross-crate enforcement (external crate rejection) is Phase 55 scope
