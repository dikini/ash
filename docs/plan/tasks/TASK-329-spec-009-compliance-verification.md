# TASK-329: SPEC-009 Visibility Compliance Verification

## Status: 🟡 Medium

## Problem

Review identified potential SPEC-009 compliance gaps in visibility enforcement. Need to verify implementation matches specification across both type checking and import resolution.

## Spec Reference

SPEC-009 Section 3.2 (Visibility Levels), Section 7.1 (Access Rules)

## Scope

This task is **verification only**. If gaps are found, document them as new tasks.

## Verification Steps

### 1. Review Implementation

Check `crates/ash-typeck/src/visibility.rs`:
- `Visibility::Crate` variant exists
- `is_visible_path()` handles `Crate` variant correctly
- `Visibility::Super { levels }` properly handles multi-level parent visibility

Check `crates/ash-parser/src/import_resolver.rs`:
- Import visibility is enforced for `pub(crate)`
- `pub(super)` is not treated as universally visible
- `pub(in path)` / restricted visibility is not treated as universally visible

### 2. Review Test Coverage

Check `crates/ash-typeck/tests/visibility_test.rs`:
- Tests for `pub(crate)` visibility
- Tests for `pub(super)` with various levels
- Edge cases (root module, nested modules)

Check `crates/ash-parser/src/import_resolver.rs` tests:
- Coverage for private item rejection
- Coverage for `pub(super)` import rejection/allow cases
- Coverage for restricted-path import rejection/allow cases

### 3. Identify Gaps

Document any missing test coverage or implementation gaps.

## Potential Gaps to Check

- [ ] `pub(crate)` visibility across module boundaries
- [ ] `pub(super)` with levels > 1
- [ ] `pub(in path)` restricted visibility (if implemented)
- [ ] Visibility at root module (should behave as `pub(crate)`)
- [ ] Import resolver applies the same restricted visibility rules as the type checker

## Verification

```bash
cargo test --package ash-typeck visibility --quiet
cargo test --package ash-parser import_resolver --quiet
```

## Completion Checklist

- [ ] Implementation reviewed against SPEC-009
- [ ] Test coverage assessed
- [ ] Import resolution path assessed against SPEC-009
- [ ] Gaps documented (if any)
- [ ] New tasks created for any gaps found
- [ ] Report generated

**Estimated Hours:** 2
**Priority:** Medium (compliance verification)
**Note:** This is a verification task. Implementation fixes (if needed) will be separate tasks.
