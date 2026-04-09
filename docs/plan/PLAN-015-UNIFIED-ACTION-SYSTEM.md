# PLAN-015: Unified Action System Implementation

## Status: 📝 Planned

## Overview

Implement DESIGN-015 to eliminate the duality between `ash_core::Action` and library provider actions. This plan starts with the two authored task files and treats the AST, parser/lowering, and interpreter execution boundary as one atomic first phase so the workspace remains coherent while `Action` changes shape.

## Design Reference

- [DESIGN-015: Unified Action System](../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)

## Goals

1. Unify `Action` to use `Vec<Value>` instead of `Vec<Expr>`
2. Land parser/lowering and interpreter execution changes in the same phase as the `Action` type change
3. Create unified `CapabilityProvider` trait in `ash_core`
4. Remove adapter layer (`InterpProviderAdapter`)
5. Migrate all providers to unified trait
6. Keep `observe` unchanged (accept phase misalignment)

## Scope

**In Scope**:
- `ash_core::Action` type change
- Parser/lowering updates required to keep core AST construction valid
- Interpreter ACT execution updates required to evaluate expressions before provider calls
- Unified `CapabilityProvider` trait
- `CapabilityContext` updates
- Provider migrations (FsProvider, StdioProvider, McpProvider)
- Error handling unification
- Documentation updates

**Out of Scope**:
- `observe` constraint evaluation (unchanged)
- `Pure` effect addition
- Action registration/discovery features

## Phases

### Phase 1: Action Boundary Realignment (ash-core, ash-parser, ash-interp)

**Goal**: Change `Action` and land the parser/lowering plus interpreter execution updates required for that change to compile and preserve semantics.

**Tasks**:
- [TASK-449](tasks/TASK-449-action-vec-value.md): Update `Action` to use `Vec<Value>` and land parser/lowering plus ACT execution changes in the same phase
- [TASK-450](tasks/TASK-450-unified-provider-trait.md): Add unified `CapabilityProvider` trait

**Deliverable**: `ash-core`, `ash-parser`, and `ash-interp` remain aligned on evaluated-action execution, and `ash-core` exposes the unified provider trait

**Estimated Effort**: 8 hours

---

### Phase 2: Provider Interface Migration (ash-interp)

**Goal**: Update `CapabilityContext` to use the unified trait and remove the adapter.

**Tasks**:
- [TASK-451](tasks/TASK-451-capability-context-unified-trait.md): Update `CapabilityContext` and registry types to use `ash_core::CapabilityProvider`
- [TASK-452](tasks/TASK-452-remove-interp-provider-adapter.md): Remove interpreter-side wrapper and adapter code that only exists for the split trait design

**Deliverable**: `ash-interp` uses the unified provider trait directly, without split-trait adapter scaffolding

**Estimated Effort**: 2 hours

---

### Phase 3: Provider Migrations (ash-engine)

**Goal**: Migrate engine providers to the unified trait.

**Tasks**:
- [TASK-455](tasks/TASK-455-fs-provider-unified-trait.md): Migrate `FsProvider` to `ash_core::CapabilityProvider`
- [TASK-456](tasks/TASK-456-stdio-provider-unified-trait.md): Migrate `StdioProvider` to `ash_core::CapabilityProvider`
- [TASK-457](tasks/TASK-457-mcp-provider-unified-trait.md): Migrate `McpProvider` to `ash_core::CapabilityProvider`
- [TASK-458](tasks/TASK-458-engine-unified-trait.md): Update `EngineBuilder`, custom-provider registration, and runtime-state wiring to the unified trait
- [TASK-459](tasks/TASK-459-remove-old-provider-trait.md): Remove the old engine-side provider trait and finalize the public API migration

**Deliverable**: Engine providers and engine public APIs all use the unified provider trait

**Estimated Effort**: 3 hours

---

### Phase 4: Cleanup and Documentation

**Goal**: Author cleanup, error-handling, documentation, and integration-test follow-on tasks after Phases 1-3 are stable.

**Tasks**:
- [TASK-460](tasks/TASK-460-error-handling-unified.md): Normalize `CapabilityError` to `ExecError`/CLI conversion boundaries after trait migration
- [TASK-461](tasks/TASK-461-documentation-updates.md): Update docs, examples, and API references for the unified action/provider model
- [TASK-462](tasks/TASK-462-final-integration-testing.md): Run final integration, regression, and quality-gate coverage for PLAN-015

**Deliverable**: Error conversion, docs, and verification are aligned with the completed migration

**Estimated Effort**: 2 hours

---

## Critical Path

```
TASK-449 (Action Vec<Value>)
    ↓
TASK-450 (Unified trait)
    ↓
TASK-451 / TASK-452 (interp migration)
    ↓
TASK-455 / TASK-456 / TASK-457 / TASK-458 / TASK-459 (engine/provider migration)
    ↓
TASK-460 / TASK-461 / TASK-462 (cleanup/docs/testing)
```

Parallel paths:
- Parser/lowering and ACT execution changes are part of `TASK-449`; they are not valid follow-up verification work
- Provider migrations (`TASK-455` / `TASK-456` / `TASK-457`) can run in parallel after `TASK-450` and `TASK-451`

---

## Dependencies

**External Dependencies**:
- None

**Internal Dependencies**:
- Phase 2 depends on Phase 1
- Phase 3 depends on Phases 1 and 2
- Phase 4 depends on all previous phases

---

## Risks

### Risk 1: Breaking Change Disrupts Downstream

**Probability**: High  
**Impact**: High  
**Mitigation**: Comprehensive testing, clear documentation

### Risk 2: Phase Misalignment Causes Confusion

**Probability**: Medium  
**Impact**: Medium  
**Mitigation**: Explicit documentation in specs

### Risk 3: Error Handling Edge Cases

**Probability**: Medium  
**Impact**: Medium  
**Mitigation**: Thorough error testing, property tests

---

## Success Criteria

1. **Single trait**: All providers implement `ash_core::CapabilityProvider`
2. **No adapter**: `InterpProviderAdapter` removed
3. **Single provider error surface**: providers converge on `CapabilityError`, with explicit interpreter-boundary conversion retained where needed
4. **Tests pass**: Full test suite green
5. **Docs updated**: All specs and examples updated
6. **Performance**: No regression in action execution

---

## Timeline

| Phase | Duration | Start Date | End Date |
|-------|----------|------------|----------|
| Phase 1 | 1.5 days | TBD | TBD |
| Phase 2 | 0.5 day | TBD | TBD |
| Phase 3 | 0.5 day | TBD | TBD |
| Phase 4 | 0.5 day | TBD | TBD |
| **Total** | **3 days** | TBD | TBD |

---

## Next Steps

1. Review and approve this plan
2. Execute [TASK-449](tasks/TASK-449-action-vec-value.md) and [TASK-450](tasks/TASK-450-unified-provider-trait.md) as the authored Phase 1 foundation
3. Execute later phases through [TASK-462](tasks/TASK-462-final-integration-testing.md) once the Phase 1 boundary is stable
4. Begin Phase 1 with TASK-449

---

*Document Version: 1.0*  
*Status: Planned*  
*Author: hermes*  
*Date: 2026-04-09*
