# TASK-2062: Module-Aware Core/CPS Lowering

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 8; SPEC-098c; SPEC-099b
**Owned rule:** MOD-REAL-005
**Run-route impact:** prerequisite

## Semantic accounting

**Implementation:** not_implemented. **Evidence:** none. **Parity:** below_spec.
**Missing target-spec clauses:** identity-preserving module Core/CPS artifacts and imported-reference lowering.
**Layers:** type `partial`; Core `not_implemented`; CPS `not_implemented`; admission-runtime `not_applicable`; verification `not_implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-005-CORE-CPS-MODULE`; negative `TEST-MOD-REAL-005-UNRESOLVED-IMPORT-LOWER`; mutation `TEST-MOD-REAL-005-IDENTITY-FORGERY`; parity `TEST-MOD-REAL-005-FILE-INLINE-LOWERING`.
**Next obligation:** hand checked linked artifacts to TASK-2063.

## Description

Lower resolved checked module definitions to Core and CPS artifacts without source rediscovery. Preserve module/declaration identities, import bindings, public interfaces, and source origins across both file-backed and inline source forms.

## Dependencies

- 📝 TASK-2060 — checked interfaces and export closure.
- 📝 TASK-2061 — resolved imports and visibility.

## Current → target

**Current files:** lowering and summary seams in `crates/ash-engine/src/module_loader.rs`, Core/CPS carriers in `crates/ash-core`, and existing source-to-Core bridges.

**Current state:** selected callable/type metadata travels through bounded module-loader paths. There is no general module-level Core/CPS artifact tied to a resolved interface graph.

**Target state:** each checked reachable module produces a Core module and CPS module artifact keyed by canonical module identity. Imported references lower from resolved declaration identities, never from names or source snippets. File/inline equivalent units produce alpha-equivalent normalized artifacts.

## Requirements

1. Define Core/CPS module artifact carriers or extend existing carriers in `ash-core`.
2. Lower resolved module declarations in dependency order while preserving source anchors and declaration identities.
3. Thread checked import/reference facts into Core/CPS references; reject unresolved or stale facts before lowering.
4. Preserve row, contract, macro-origin, and syntax-phase metadata according to their existing owners without granting authority.
5. Prove no file/inline source-kind branch remains after common module-unit construction.

## TDD Steps and evidence

1. Add paired file/inline fixtures and assert normalized Core/CPS artifact equality.
2. Test imported function/type/constructor/interface resolution through lowered identities.
3. Add negative tests for forged, stale, private, cyclic, or incomplete interfaces before lowering.
4. Add mutation tests that change declaration order and aliases while preserving defining identity and normalized artifact shape.

## Completion checklist

- [ ] Resolved imports lower through declaration identities, not names or source snippets.
- [ ] File/inline equivalent modules yield equal normalized Core/CPS artifacts.
- [ ] Invalid/stale/inaccessible interfaces reject before lowering.
- [ ] Focused Core/CPS tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** checked module and resolved-binding facts from TASK-2060/2061.
- **Produces:** checked Core/CPS module artifacts for TASK-2063.
- **Downstream owner:** TASK-2063 owns Engine link/admission; TASK-2064 owns parity evidence.
- **Non-goals:** provider-frame installation, direct source execution, dynamic module loading, or client protocol changes.

## Files and verification

**Files:** relevant `crates/ash-core/src/{core,cps,semantic_summary}*`, source-to-Core lowering modules, `crates/ash-engine/src/module_loader.rs`, Core/CPS/module integration tests.

```text
cargo test -p ash-core
cargo test -p ash-engine module
cargo clippy -p ash-core -p ash-engine --all-targets -- -D warnings
cargo fmt --check
```
