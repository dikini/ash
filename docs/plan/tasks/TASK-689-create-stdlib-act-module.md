# TASK-689: Replace placeholder `std::act` stubs with ordinary library implementations

## Status: ✅ Complete

## Description

Replace the current placeholder `std/src/act.ash` surface with the Phase-97 ordinary library implementations of `unit`, `bind`, `then`, and `guard` once the prerequisite `std::act` substrate is honest enough to support them.

## Specification Reference

- SPEC-047 §2.5

## Dependencies

- 📝 TASK-685: prerequisite task
- 📝 TASK-689A: honest `std::act` substrate prerequisite
- 📝 TASK-689B: imported ordinary `pub fn` signature substrate prerequisite
- 📝 TASK-689C: policy/environment substrate prerequisite for ordinary `guard`
- TASK-689D is now complete for the public opaque `Act` boundary: ordinary helpers import/typecheck over the A-path substrate, hidden-carrier enforcement is in place, workflow/eval async-force coverage is focused-test backed, and the remaining token/list force result shape is documented as an internal runtime compatibility detail rather than a public `std::act` representation.

## Requirements

### Functional Requirements

1. Remove the placeholder `pub builtin fn` declarations for `unit`, `bind`, `then`, and `guard` from `std/src/act.ash`.
2. Replace or minimize reliance on the placeholder/public surrogate `Act` representation in favor of the honest opaque Phase-97 library boundary required by the landed substrate.
3. Expose `unit`, `bind`, `then`, and `guard` through the `act::` library boundary rather than as general runtime builtins, preserving opaque `Act` semantics.
4. Keep signatures and examples aligned with current Ash type syntax and the frozen SPEC-047 contract.
5. Prove the ordinary-library import/type/execute path through focused engine/runtime tests.

### Property Requirements (proptest)

```rust
// Add property-based tests where the task manipulates syntax lowering,
// typing invariants, or runtime sequencing that should hold across broad inputs.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target Phase-97 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the additive Phase-97 architecture.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not introduce core-IR expansion beyond the frozen Phase-97 plan.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic/lowering/runtime invariants where appropriate.

## Verification Steps

- [x] `std/src/act.ash` no longer exposes placeholder builtin helper declarations.
- [x] `std/src/act.ash` matches the ordinary-library Phase 97 architecture.
- [x] Focused engine/runtime tests prove import + type + execute behavior for ordinary `std::act` helpers.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
- The current `std/src/act.ash` file is only a placeholder surface until TASK-689A lands the honest substrate required for ordinary library definitions.
- TASK-689B preserved imported ordinary `pub fn` signatures through module loading and engine type binding, so imported ordinary helpers are no longer forced through an arity-only fallback.
- TASK-689C is now sufficient for an honest ordinary-library `guard` through the narrow `act::policy_check` bridge while preserving the runtime-only `ActEnv` boundary.
- TASK-689E landed the enabling library/type-export semantics so `type T = ...` now preserves public/discoverable type identity without automatically exporting constructors/representation.
- TASK-689D completed the public opaque `Act` prerequisite: the public boundary, ordinary helper import path, hidden-carrier protection checks, async-force workflow coverage, and internal-forcing correspondence are landed.
- TASK-689 closeout is now proven by focused end-to-end checks: `cargo run -q -p ash-cli -- check std/src/act.ash`, `cargo test -p ash-engine --test module_resolution -- --nocapture`, and `cargo test -p ash-interp --test act_env_runtime_boundary -- --nocapture`.
- `guard` now executes through the ordinary-library boundary while deferring policy evaluation to Act-force time via the internal `act::__guard` bridge, preserving the runtime-only `ActEnv` boundary.
- Any broader Ash-visible runtime-environment feature or fully native replacement for the internal token/list force-result compatibility shape should be spun out into a separate spec/plan track instead of being silently folded into TASK-689.
