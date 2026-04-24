# TASK-689E: Refine library type-export semantics for opaque `Act`

## Status: ✅ Complete

## Description

Enable the preferred Option D direction for Phase 97 by refining the library interface semantics around `type` vs `pub type` so that a type constructor name and kind/arity can be public/discoverable to the typechecker while its constructors/associated symbols remain private unless explicitly exported.

This task is the enabling substrate for an honest opaque `Act` library boundary without introducing a new `opaque` keyword.

## Specification Reference

- SPEC-047 §1.1
- SPEC-047 §2.5
- SPEC-047 §7.4

## Dependencies

- 📝 TASK-689C: prerequisite task

## Requirements

### Functional Requirements

1. Specify the semantic rule for `type T = ...` versus `pub type T = ...` in library/module boundaries.
2. Ensure the rule generalizes beyond `Act` to other optionally exportable artifacts where appropriate, while keeping the implementation scope minimal and honest.
3. Add focused parser/module/type tests proving that a type constructor can be public/discoverable without automatically exporting its constructors/representation symbols.
4. Implement the minimal parser, engine/module-loader, and type-environment changes required so imported signatures can name such types honestly.
5. Document any runtime implications explicitly; do not add runtime exposure machinery unless it is strictly required by the refined export semantics.
6. Update TASK-689D and TASK-689 surfaces honestly based on whether opaque public `Act` is now implementable.

### Property Requirements (proptest)

```rust
// Prefer focused regression tests unless the export/refinement rule introduces
// a broader invariant worth property coverage.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that prove current Phase-97 Ash still couples public type identity and constructor visibility too tightly for opaque `Act`.

### Step 2: Implement (Green)

Land the smallest honest semantic/parser/engine/typeck slice needed for:
- discoverable type constructor identity
- private constructor/representation visibility by default
- explicit public constructor/representation export through `pub type`

### Step 3: Integration (Green)

Verify the real module/import/type boundary, not just local parser acceptance.

### Step 4: Verification

Re-run focused checks and update plan/task surfaces to match reality.

## Verification Steps

- [x] Focused tests capture the missing library export abstraction boundary.
- [x] `type T = ...` and `pub type T = ...` have explicitly documented and tested distinct export semantics.
- [x] Imported signatures can refer to public type identity without forcing constructor visibility.
- [x] TASK-689D status is updated honestly based on the landed substrate.
- [x] `cargo fmt --check` passes.

## Dependencies for Next Task

This task determines whether TASK-689D can proceed with an honest opaque `Act` implementation path.

## Notes

- Phase 97 is additive.
- Preferred direction: refine existing library/type export semantics instead of adding a new `opaque` keyword.
- The target semantic split is:
  - `type T = ...` => public/discoverable type identity, private constructors/representation
  - `pub type T = ...` => public/discoverable type identity plus public constructors/representation
- Do not overexpand this task into full runtime-environment exposure or a larger visibility-system redesign.
- Landed evidence:
  - `ash-engine` module loading now preserves type identity separately from constructor export, so plain `type` participates in imported signatures without auto-exporting constructors.
  - `ash-typeck::TypeEnv` now separates type-identity registration from representation exposure.
  - Focused `ash-engine` and `ash-typeck` regression tests cover plain-`type` identity imports, hidden constructor behavior, and continued `pub type` representation visibility.
- Runtime implication: none beyond the engine/type boundary. This task did not expose `ActEnv` or add broader runtime visibility machinery.
