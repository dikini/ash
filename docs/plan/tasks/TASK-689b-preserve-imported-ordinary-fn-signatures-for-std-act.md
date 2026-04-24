# TASK-689B: Preserve imported ordinary `pub fn` signatures for `std::act`

## Status: ✅ Complete

## Description

Before `std/src/act.ash` can replace its placeholder builtin declarations with ordinary library implementations, the engine must preserve and use declared signatures for imported ordinary `pub fn` callables instead of collapsing them to arity-only synthetic types.

## Specification Reference

- SPEC-047 §1.1
- SPEC-047 §2.5
- SPEC-047 §5

## Dependencies

- 📝 TASK-689A: prerequisite task

## Requirements

### Functional Requirements

1. Preserve declared signatures for imported ordinary `pub fn` callables exported from module files.
2. Ensure the engine type-binding path uses those preserved signatures instead of the current non-builtin arity-only fallback.
3. Add focused tests proving imported ordinary `std::act` helpers can cross the parse -> import -> type boundary honestly.
4. Document any remaining blocker specific to `guard` semantics if policy/environment access is still unavailable after the signature fix.

### Property Requirements (proptest)

```rust
// Prefer focused regression tests for imported-callable signature preservation.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing engine/type tests that prove imported ordinary `pub fn` signatures are currently discarded or weakened to arity-only fallback.

### Step 2: Implement (Green)

Preserve and bind imported ordinary-function signatures through the existing module loader and engine type environment with minimal scope.

### Step 3: Integration (Green)

Verify the real `std::act` import/type path, not only local parser acceptance.

### Step 4: Verification

Re-run focused engine/type checks and update task/plan surfaces honestly.

## Verification Steps

- [x] Imported ordinary `pub fn` signatures are preserved through module loading.
- [x] Engine type binding no longer reduces imported ordinary `std::act` helpers to arity-only synthetic types.
- [x] Focused engine/type tests prove the ordinary `std::act` import/type boundary.
- [x] `cargo test -p ash-engine -- --nocapture` targeted to the new coverage passes.
- [x] `cargo fmt --check` passes.

## Dependencies for Next Task

This task outputs the honest imported-signature substrate needed before TASK-689 can replace placeholder builtin `std::act` stubs with ordinary library implementations.

## Notes

- Phase 97 is additive.
- This task is about imported ordinary-function signatures and type binding, not yet about final `guard` policy semantics.
- Completed slice: `Workflow` now preserves imported ordinary `pub fn` signatures, `build_imported_closures(...)` carries them through the engine boundary, `bind_imported_callable_types(...)` uses `ash_typeck::fn_signature_type(...)`, and focused ash-engine tests cover both preserved ordinary-function signatures and the upgraded internal binding path.
- Remaining blocker moved to TASK-689C: honest ordinary-library `guard` still needs enough policy/environment surface to match SPEC-047 without faking runtime-only details.
