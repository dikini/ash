# TASK-329: SPEC-009 Visibility Compliance Verification Report

**Task:** Verify SPEC-009 visibility implementation is complete and compliant across both type checking and import resolution.

**Date:** 2025-03-27

**Status:** RESOLVED - Phase 54 implemented all visibility enforcement

---

## Summary of Findings

The SPEC-009 visibility implementation has **partial compliance**:

1. **Type Checker (ash-typeck)**: ✅ FULLY COMPLIANT - All visibility variants properly implemented and tested
2. **Import Resolver (ash-parser)**: ❌ NON-COMPLIANT - Simplified placeholder implementation for restricted visibility

---

## 1. Type Checker Implementation Review

### File: `crates/ash-typeck/src/visibility.rs`

| Feature | Status | Evidence |
|---------|--------|----------|
| `Visibility::Crate` variant exists | ✅ PASS | Used in `surface.rs` line 142 |
| `is_visible_path()` handles `Crate` | ✅ PASS | Lines 132-139 - checks first path segment matches |
| `Visibility::Super { levels }` | ✅ PASS | Lines 141-154 - properly calculates ancestors and checks hierarchy |
| Multi-level `pub(super)` | ✅ PASS | `ancestors()` method supports arbitrary levels (line 85-94) |
| Root module handling | ✅ PASS | Line 147-148 - `pub(super)` at root = `pub(crate)` |
| `Visibility::Self_` | ✅ PASS | Line 156 - only visible in same module |
| `Visibility::Restricted { path }` | ✅ PASS | Lines 158-161 - parses and checks path prefix |
| Error handling | ✅ PASS | `VisibilityError` enum with `PrivateItem` and `MissingContext` |
| `VisibilityChecker` struct | ✅ PASS | Provides `check_access()` method for type checking |

### Test Coverage: `crates/ash-typeck/tests/visibility_test.rs` + inline tests

| Test Category | Count | Status |
|---------------|-------|--------|
| `pub(crate)` tests | 4 | ✅ PASS |
| `pub(super)` tests | 6 | ✅ PASS |
| `pub(self)` tests | 3 | ✅ PASS |
| `pub(in path)` tests | 4 | ✅ PASS |
| `pub` tests | 2 | ✅ PASS |
| `Inherited` (private) tests | 4 | ✅ PASS |
| `ModulePath` utility tests | 8 | ✅ PASS |
| Error message tests | 2 | ✅ PASS |

**Test Results:**
```
cargo test --package ash-typeck visibility --quiet
running 33 tests
..................................
test result: ok. 33 passed; 0 failed
```

---

## 2. Import Resolver Implementation Review

### File: `crates/ash-parser/src/import_resolver.rs`

| Feature | Status | Evidence |
|---------|--------|----------|
| `Visibility::Public` | ✅ PASS | Line 481 - returns `true` |
| `Visibility::Inherited` | ✅ PASS | Line 483 - returns `false` |
| `Visibility::Crate` | ❌ **FAIL** | Line 482 - returns `true` unconditionally, no crate boundary check |
| `Visibility::Super { .. }` | ❌ **FAIL** | Line 484 - returns `true` with comment "Simplified: allow for now" |
| `Visibility::Self_` | ✅ PASS | Line 485 - returns `false` |
| `Visibility::Restricted { .. }` | ❌ **FAIL** | Line 486 - returns `true` with comment "Simplified: allow for now" |

### Critical Gap: `is_visible()` Method (Lines 474-488)

```rust
fn is_visible(
    &self,
    visibility: &Visibility,
    _importing_module: ModuleId,  // UNUSED!
    _target_module: ModuleId,      // UNUSED!
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Crate => true,              // ❌ WRONG: Always allows
        Visibility::Inherited => false,
        Visibility::Super { .. } => true,       // ❌ WRONG: Always allows
        Visibility::Self_ => false,
        Visibility::Restricted { .. } => true,  // ❌ WRONG: Always allows
    }
}
```

**Problems:**
1. `pub(crate)` items can be imported from ANY crate, not just the same crate
2. `pub(super)` items are treated as universally visible
3. `pub(in path)` items are treated as universally visible
4. Module hierarchy is not checked for restricted visibility

### Test Coverage: Import Resolver

| Test Category | Count | Status |
|---------------|-------|--------|
| Basic import resolution | 4 | ✅ PASS |
| Private item rejection (`Inherited`) | 2 | ✅ PASS |
| `pub(crate)` import enforcement | 0 | ❌ MISSING |
| `pub(super)` import enforcement | 0 | ❌ MISSING |
| `pub(in path)` import enforcement | 0 | ❌ MISSING |

**Test Results:**
```
cargo test --package ash-parser import_resolver --quiet
running 11 tests
...........
test result: ok. 11 passed; 0 failed
```

**Note:** Tests pass because they only verify `Public` and `Inherited` visibility, not the restricted variants.

---

## 3. Gap Analysis

### Gap 1: Import Resolver `pub(crate)` Enforcement (CRITICAL)
- **Location:** `crates/ash-parser/src/import_resolver.rs:482`
- **Issue:** `Visibility::Crate` returns `true` unconditionally
- **Impact:** Items marked `pub(crate)` can be imported from external crates
- **Required Fix:** Implement crate boundary checking using module graph

### Gap 2: Import Resolver `pub(super)` Enforcement (CRITICAL)
- **Location:** `crates/ash-parser/src/import_resolver.rs:484`
- **Issue:** `Visibility::Super { .. }` returns `true` with "Simplified: allow for now" comment
- **Impact:** Items marked `pub(super)` are visible from anywhere, not just parent module
- **Required Fix:** Implement parent module hierarchy check

### Gap 3: Import Resolver `pub(in path)` Enforcement (CRITICAL)
- **Location:** `crates/ash-parser/src/import_resolver.rs:486`
- **Issue:** `Visibility::Restricted { .. }` returns `true` with "Simplified: allow for now" comment
- **Impact:** Items with restricted path visibility are visible from anywhere
- **Required Fix:** Implement path prefix checking against importing module

### Gap 4: Test Coverage for Import Resolver Restricted Visibility (HIGH)
- **Location:** `crates/ash-parser/src/import_resolver.rs` (tests module)
- **Issue:** No tests for `Crate`, `Super`, or `Restricted` visibility enforcement
- **Required Fix:** Add comprehensive tests for restricted visibility in imports

---
## 4. Recommendations

### Recommendation: NEEDS_FIX

The SPEC-009 visibility compliance is **NOT COMPLETE**. The import resolver has placeholder implementations for restricted visibility that effectively treat `pub(crate)`, `pub(super)`, and `pub(in path)` as `pub`.

### Tasks to Create

1. **TASK-XXX: Implement `pub(crate)` visibility enforcement in import resolver**
   - Modify `is_visible()` to check crate boundaries
   - Use module graph to determine if importing and target modules are in same crate
   - Add tests for cross-crate import rejection

2. **TASK-YYY: Implement `pub(super)` visibility enforcement in import resolver**
   - Modify `is_visible()` to check parent module hierarchy
   - Support `levels` parameter for multi-level parent access
   - Add tests for parent/descendant access and non-parent rejection

3. **TASK-ZZZ: Implement `pub(in path)` visibility enforcement in import resolver**
   - Modify `is_visible()` to check path prefix
   - Parse restricted path and verify importing module is within that path
   - Add tests for allowed and rejected restricted imports

4. **TASK-WWW: Add comprehensive visibility tests to import resolver**
   - Add tests for all visibility variants during import resolution
   - Test edge cases: root module, nested modules, cross-crate imports
   - Ensure parity with type checker visibility tests

---

## 5. Evidence References

### Type Checker (Compliant)
- File: `crates/ash-typeck/src/visibility.rs`
  - Lines 132-139: `Visibility::Crate` implementation
  - Lines 141-154: `Visibility::Super` implementation
  - Lines 158-161: `Visibility::Restricted` implementation
  - Lines 466-546: `pub(crate)` tests
  - Lines 552-637: `pub(super)` tests

### Import Resolver (Non-Compliant)
- File: `crates/ash-parser/src/import_resolver.rs`
  - Lines 474-488: `is_visible()` with placeholder implementations
  - Line 482: `Visibility::Crate => true` (unconditional)
  - Line 484: `Visibility::Super { .. } => true` (marked as simplified)
  - Line 486: `Visibility::Restricted { .. } => true` (marked as simplified)

### Visibility Enum Definition
- File: `crates/ash-parser/src/surface.rs`
  - Lines 135-154: All visibility variants defined correctly

---

## Appendix: SPEC-009 Reference

### Section 3.2: Visibility Levels
- `pub` - visible everywhere
- `pub(crate)` - visible within the crate
- `pub(super)` - visible to parent module
- `pub(self)` - visible only in current module
- `pub(in path)` - visible in specified module path

### Section 7.1: Access Rules
Items are accessible based on their visibility annotation and the relationship between the defining module and the accessing module.

---

## Phase 54 Resolution

**Date:** 2026-03-27

All critical gaps identified in this report have been resolved through Phase 54 implementation:

### Gap Resolution Summary

| Gap | Resolution | Evidence |
|-----|------------|----------|
| `pub(crate)` enforcement | ✅ TASK-332 complete | Crate boundary checking via CrateId tracking |
| `pub(super)` enforcement | ✅ TASK-333 complete | Parent module hierarchy via ancestors() |
| `pub(in path)` enforcement | ✅ TASK-334 complete | Path resolution via resolve_path() |
| Import resolver test coverage | ✅ TASK-335 complete | 37 visibility tests added |

### Implementation Details

**Files Modified:**
- `crates/ash-core/src/module_graph.rs` - Added CrateId, parent tracking, ancestors(), resolve_path()
- `crates/ash-parser/src/import_resolver.rs` - Implemented proper visibility checks in is_visible()

**Test Results (Post-Implementation):**
```
cargo test --package ash-parser import_resolver --quiet
running 37 tests
test result: ok. 37 passed; 0 failed

cargo test --package ash-typeck visibility --quiet
running 33 tests
test result: ok. 33 passed; 0 failed
```

**SPEC-009 Compliance:** ACHIEVED

All visibility variants now properly enforced in both type checker and import resolver.

### Related Tasks
- [TASK-332](TASK-332-import-resolver-pub-crate.md) - pub(crate) enforcement
- [TASK-333](TASK-333-import-resolver-pub-super.md) - pub(super) enforcement
- [TASK-334](TASK-334-import-resolver-pub-in-path.md) - pub(in path) enforcement
- [TASK-335](TASK-335-import-resolver-visibility-tests.md) - Comprehensive visibility tests
- [TASK-336](TASK-336-phase-54-closeout.md) - Phase 54 closeout

---

*Report generated by TASK-329 verification*
*Phase 54 resolution added by TASK-336*
