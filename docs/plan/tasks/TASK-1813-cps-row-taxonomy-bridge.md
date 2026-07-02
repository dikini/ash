# TASK-1813: Align CPS row/effect carriers and Core-to-CPS row lowering

## Status: ✅ Complete

## Description

Align CPS `EffectRow`/`EffectItem`/`EffectOp` carriers and Core-to-CPS lowering so Phase 177 supported Core row families are preserved or rejected explicitly. This task owns the main row-loss boundary in `core_ash_lower`.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099b: Target Operational Semantics](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [NOTE-020: Computation Row Taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)

## Dependencies

- TASK-1812 Core row taxonomy alignment complete.
- TASK-1807 seam audit identifies existing CPS consumers and compatibility risks.

## Requirements

### Functional Requirements

1. Review `crates/ash-core/src/cps.rs` row/effect variants against Core row families.
2. Extend CPS carriers or add explicit bridge metadata for supported Phase 177 row families.
3. Patch `crates/ash-core/src/core_ash_lower.rs` so `lower_row` and `lower_row_item` do not silently drop supported Core row facts.
4. Add a precise lowering error for unsupported row families if any remain outside the CPS slice.
5. Preserve existing interpreter behavior for older capability-style tests through compatibility conversions where necessary.
6. Add Core-to-CPS lowering tests for operation, resource, role, policy, channel, process, failure, evidence, and group rows as supported by TASK-1812.
7. Add negative tests proving unsupported rows fail closed instead of becoming empty rows.

### Property Requirements

- Core-to-CPS lowering must be conservative: preserve or reject, never silently erase.
- CPS operation identities must distinguish impl-qualified operation identity from generic string namespaces where supported.
- Runtime handler dispatch compatibility must remain covered by existing tests.

## TDD Steps

### Step 1: Write failing CPS bridge tests

Add tests under `crates/ash-core/tests/` for lowering representative Core rows to CPS rows and rejecting unsupported rows.

### Step 2: Verify RED

Run focused tests and confirm missing row preservation or silent erasure is exposed.

### Step 3: Implement CPS row bridge

Patch CPS carriers and Core-to-CPS row lowering minimally, with compatibility helpers for existing tests.

### Step 4: Verify GREEN

Run focused tests, `cargo test -p ash-core`, and interpreter tests identified by TASK-1807 if affected.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core
  - cargo test -p ash-interp
  - git diff --check
checklist:
  - [x] CPS row carriers preserve supported Phase 177 row families.
  - [x] Core-to-CPS lowering has no silent supported-row drops.
  - [x] Unsupported rows fail closed with precise errors.
  - [x] Existing handler/interpreter tests remain compatible.
```

## Completion

- Added explicit `EffectItemKind` variants for phase-177 families in `cps.rs` (`Resource`, `Process`, `Evidence`, `Failure`).
- Updated Core-to-CPS row lowering to map supported families to explicit kinds and reject unsupported row tails via `UnsupportedCoreRow`.
- Updated `CoreEffectOp` lowering so `Process` and `Failure` operations carry explicit family-specific `EffectItemKind` values.
- Added unit tests for open-row rejection and family-kind preservation.
- Updated integration coverage to assert explicit failure item kind in lowered failure raises.

## Dependencies for Next Task

This task feeds TASK-1814.
