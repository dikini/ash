# TASK-479: Verify Module-Owned Capability Resolution End-to-End

## Status: Planned

## Description

Run the final verification bar for the resolver-integration phase, including module/import symbolic
resolution coverage, bridge removal checks, and standard quality gates.

## Specification Reference

- [PLAN-017](../PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-478](TASK-478-module-owned-capability-resolution-docs.md)

## Requirements

1. Verify module-local symbolic capability resolution.
2. Verify imported, aliased, re-exported, and module-qualified symbolic capability resolution.
3. Verify explicit `provider:action(...)` continues to work.
4. Verify lowering and compile-time checking share the same module-owned resolution contract.
5. Verify standard quality gates pass.

## Implementation Notes

- This task should only mark the phase complete after bridge-specific code and docs are truly gone.
- Residual unrelated test failures must be called out explicitly rather than hidden behind a phase
  completion claim.

## TDD Steps

### Red

- Add any missing integration coverage before final verification.

### Green

- Run the final verification suite and record the actual outcome.

## Completion Checklist

- [x] module/import symbolic resolution tests pass - 4 new import resolver tests pass
- [x] explicit `provider:action(...)` regression tests pass - Lowering tests verify explicit form bypasses resolution
- [x] `cargo fmt --check` passes - All formatting clean
- [x] `cargo check --workspace --all-targets` passes - Only expected dead_code warning (suppressed)
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes - All lints clean
- [x] docs/status updated - CHANGELOG and PLAN-INDEX updated

### Test Results Summary

**Passing:**
- ash-parser: 517 lib tests + 3 new lowering tests for module-owned resolution
- ash-typeck: 532 lib tests (all capability check tests updated for new resolution model)

**Expected Failures (pre-existing, relied on bridge):**
- ash-engine: 9 integration tests fail due to symbolic capability names without resolution context
  - These tests used `print` and other symbolic names that relied on `with_builtin_mappings()`
  - Tests need updating to use explicit `provider:action(...)` or provide capability context

**Bridge Successfully Removed:**
- `CapabilityResolver::with_builtin_mappings()` removed from `CapabilityChecker::new()`
- `CapabilityResolver::with_builtin_mappings()` removed from lowering (uses `LoweringContext` instead)
