# TASK-689A: Establish honest `std::act` substrate for ordinary library helpers

## Status: ✅ Complete

## Description

Before `std/src/act.ash` can honestly export `unit`, `bind`, `then`, and `guard` as ordinary Ash library functions, Phase 97 must close the remaining substrate gaps between the frozen SPEC-047 contract and the current parser/type/runtime reality.

## Specification Reference

- SPEC-047 §1.1
- SPEC-047 §2.5
- SPEC-047 §7
- SPEC-047 §8

## Dependencies

- 📝 TASK-688: prerequisite task
- 📝 TASK-687: prerequisite task

## Requirements

### Functional Requirements

1. Audit and document the current mismatch between the frozen `std::act` contract and the placeholder implementation in `std/src/act.ash`.
2. Establish an honest substrate boundary for `std::act` ordinary-library helpers, including the runtime/type/module conditions required before `unit`, `bind`, `then`, and `guard` can stop being placeholders.
3. Add focused tests that distinguish parser-clean placeholder declarations from the real ordinary-library import/type/execute path needed by TASK-689.
4. Update the relevant task/plan surfaces so Phase 97 no longer overclaims TASK-689 as immediately ready.

### Property Requirements (proptest)

```rust
// Add property-based tests only if this task introduces new syntax/lowering/runtime
// invariants that need broad coverage. Otherwise prefer focused regression tests.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing or gap-revealing tests that prove the current `std::act` substrate is not yet equivalent to ordinary library helpers.

### Step 2: Implement (Green)

Make the minimal substrate or validation/documentation changes needed to establish an honest contract-first boundary for TASK-689.

### Step 3: Integration (Green)

Verify the real engine/module path, not only file-local parsing, so subsequent `std::act` work rests on a truthful substrate.

### Step 4: Verification

Re-run focused parser/engine/type/runtime checks and ensure the plan/task corpus matches the implementation reality.

## Verification Steps

- [x] The current `std::act` placeholder surface and its remaining gaps are documented honestly.
- [x] Focused tests or validation artifacts prove the real import/module/runtime boundary needed by TASK-689.
- [x] `cargo test -p ash-engine --test module_resolution -- --nocapture` passes or the exact remaining blocker is documented in-task without overclaiming readiness.
- [x] `cargo test -p ash-engine --test module_file_check_tests -- --nocapture` passes
- [x] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs the honest substrate and queue state needed before TASK-689 can replace the placeholder `std::act` stubs with ordinary library implementations.

## Notes

- Phase 97 is additive.
- `Act<A>` should remain abstract and first-class at the language level; do not freeze placeholder aliases as the delivered contract.
- Do not claim TASK-689 complete while `std/src/act.ash` still depends on placeholder builtins or non-spec representations.
- TASK-689A closes the honesty gap by proving that `std/src/act.ash` now crosses the real parser/module boundary cleanly enough for subsequent work:
  - `check_module_file` accepts the current placeholder surface.
  - Real import-backed engine execution can now resolve `use act::{unit, bind, then, guard}` through the engine path as well.
  - TASK-689 remains open because the file still exposes placeholder builtin declarations and a surrogate public `Act` representation rather than the ordinary-library contract promised by SPEC-047.
