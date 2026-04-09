# PLAN-017: Module-Owned Capability Resolution

## Status: 📝 Planned

## Overview

Implement DESIGN-017 by replacing the Phase 70 bridge resolver with module-system-owned capability
resolution. Symbolic operational calls should resolve through capability declarations, imports, and
re-exports that flow through the module/import pipeline, while explicit `provider:action(...)`
continues to bypass symbolic lookup.

## Design Reference

- [DESIGN-017: Module-Owned Capability Resolution](../design/DESIGN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)

## Goals

1. Make capability declarations and imports the source of truth for symbolic operational
   resolution.
2. Build one capability-resolution context from the module/import pipeline.
3. Pass that context through lowering, type checking, and capability checking.
4. Remove parser/typechecker-local built-in resolver construction for symbolic ACT calls.
5. Keep the final semantic contract canonical in `docs/spec/`.

## Scope

**In Scope**:
- spec updates for module-owned symbolic capability resolution
- module/import metadata required to expose capability symbol targets
- resolver-context pipeline plumbing from module resolution to lowering/type checking
- lowering and capability-check migration away from bridge resolver construction
- standard-library capability metadata bootstrap through the same authoritative path
- docs/examples/status cleanup and final verification

**Out of Scope**:
- redesigning explicit `provider:action(...)`
- changing runtime provider dispatch semantics beyond the existing split model
- dynamic provider discovery or plugin loading
- redesigning non-operational capability forms (`observe`, `set`, `send`) in this phase

## Canonical Document Rule

This phase is not complete until the final normative contract lives in `docs/spec/`.

- `docs/design/` records rationale and migration intent.
- `docs/plan/` and `docs/plans/` record decomposition and implementation sequencing.
- `docs/spec/` must be the lasting source of truth once the phase is complete.

## Implementation Guardrails

1. Do not reintroduce a flat one-name ACT contract.
2. Do not preserve parser-local or typechecker-local built-in resolver construction after the phase
   closes.
3. Do not treat module-qualified symbolic calls such as `io::fs_read(...)` as direct provider
   encodings.
4. Do not make explicit `provider:action(...)` depend on symbolic name lookup.
5. Do not claim the bridge is removed until lowering and compile-time checking both consume the
   same externally supplied resolver context.

## Phases

### Phase 1: Spec Contract Freeze

**Goal**: Freeze the normative contract for module-owned symbolic capability resolution.

**Tasks**:
- [TASK-471](tasks/TASK-471-spec-module-owned-capability-resolution.md)

**Deliverable**: `docs/spec/` states that capability declarations/imports own symbolic operational
resolution and that explicit `provider:action(...)` remains a direct form.

### Phase 2: Module and Import Metadata

**Goal**: Make capability declarations, exports, and imports expose the metadata needed for
symbolic resolution.

**Tasks**:
- [TASK-472](tasks/TASK-472-capability-symbol-export-metadata.md)
- [TASK-473](tasks/TASK-473-imported-capability-symbol-bindings.md)

**Deliverable**: The module/import pipeline can answer what symbolic capability names are visible in
each module and what `(provider, action)` pair they denote.

### Phase 3: Resolver Context Plumbing

**Goal**: Build and pass one capability-resolution context through the compile-time pipeline.

**Tasks**:
- [TASK-474](tasks/TASK-474-capability-resolution-context-pipeline.md)

**Deliverable**: Lowering and type checking can consume one externally supplied capability
resolution context.

### Phase 4: Lowering and Type System Integration

**Goal**: Replace bridge lookup with module-owned resolution in lowering and compile-time checks.

**Tasks**:
- [TASK-475](tasks/TASK-475-lowering-module-owned-capability-resolution.md)
- [TASK-476](tasks/TASK-476-typecheck-module-owned-capability-resolution.md)

**Deliverable**: Symbolic ACT calls resolve through module-owned metadata end-to-end before runtime.

### Phase 5: Standard Library Bootstrap and Bridge Removal

**Goal**: Bring std capability symbols through the same authoritative path and remove ad hoc
bridge construction.

**Tasks**:
- [TASK-477](tasks/TASK-477-stdlib-capability-bootstrap-and-bridge-removal.md)

**Deliverable**: Built-in symbolic capability names no longer require hard-coded parser/typechecker
resolver tables.

### Phase 6: Docs and Verification

**Goal**: Align active docs/examples and verify the full pipeline.

**Tasks**:
- [TASK-478](tasks/TASK-478-module-owned-capability-resolution-docs.md)
- [TASK-479](tasks/TASK-479-module-owned-capability-resolution-verification.md)

**Deliverable**: The bridge-status note can be removed and the active docs/specs reflect the final
module-owned resolver contract.

## Success Criteria

PLAN-017 is complete when:

1. capability declarations/imports drive symbolic operational resolution
2. module-qualified symbolic calls resolve through module/import metadata
3. lowering and capability/type checking share one passed-in resolution context
4. parser/typechecker-local built-in resolver construction is removed
5. standard-library capability symbols enter through the same resolver pipeline
6. `docs/spec/` accurately describes the final state
