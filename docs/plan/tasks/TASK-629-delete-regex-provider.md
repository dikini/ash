# TASK-629: Delete legacy regex carrier

## Status: ✅ Complete

## Description
Delete the legacy regex capability-carrier implementation and remove all wiring, tests, and documentation that describe regex as a capability provider. This task was gated on the positive builtin end-to-end path being proven first.

## Specification Reference
- SPEC-BUILTIN-FN: Sections 8, 9.5
- PLAN-BUILTIN-FN: Track E / TASK-629

## Dependencies
- ✅ TASK-621: Runtime builtin dispatch table
- ✅ TASK-627: Regex builtin declarations
- ✅ TASK-628: Regex evaluator builtin dispatch
- ✅ TASK-630: Positive regex end-to-end test must pass before deletion

## Requirements
1. Delete `crates/ash-engine/src/providers/regex.rs`.
2. Remove regex provider wiring from engine/provider modules.
3. Replace provider-centric regex tests with builtin-path tests.
4. Update repository documentation and changelog references away from the provider model.
5. Verify no remaining provider-era regex references remain in code/docs.

## TDD Steps
### Step 1: Write Tests / Inventory (Red)
- Verify existing regex tests/docs still reference the provider path.

### Step 2: Implement (Green)
- Delete provider, remove wiring, rewrite tests to builtin path.
- Update docs that still describe regex as a provider.

### Step 3: Verify
- `cargo test -p ash-engine --test regex_builtin` or equivalent passes.
- Repository grep for stale provider-era regex wording returns zero hits in in-scope surfaces.

## Verification Steps
- [x] provider source deleted
- [x] provider wiring removed
- [x] regex tests exercise builtin path only
- [x] repo grep over intended verification surfaces is clean

## Notes
Must not be marked complete until the positive end-to-end builtin regex path is already verified by TASK-630.
