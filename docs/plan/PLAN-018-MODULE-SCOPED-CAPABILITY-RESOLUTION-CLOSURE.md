# PLAN-018: Module-Scoped Capability Resolution Closure

## Status: 📝 Planned

## Overview

Finish the remaining architectural work needed to close Phase 71. This phase makes shared symbolic
capability resolution explicitly module-scoped and removes the last fallback resolver path from type
checking.

## Design Reference

- [DESIGN-018: Module-Scoped Capability Resolution Closure](../design/DESIGN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md)

## Goals

1. Require explicit `ModuleId` for unqualified symbolic capability resolution.
2. Thread `ModuleId` through lowering and type-checking resolution calls.
3. Remove type-checker-local symbolic resolver fallback.
4. Verify the remaining gap is closed independently of unrelated engine execution failures.
5. Keep `docs/spec/` as the canonical semantic authority.

## Scope

**In Scope**:
- module-scoped resolution API cleanup
- lowering integration
- type-checking integration
- removal of local fallback resolver path
- documentation/status closeout for Phase 71

**Out of Scope**:
- unrelated `ash-engine` conditional execution failures
- redesign of capability declarations/imports beyond the Phase 71 model
- runtime/provider dispatch changes

## Implementation Guardrails

1. Do not restore any bridge-style built-in mapping helpers.
2. Do not allow unqualified symbolic lookup to search across all modules.
3. Do not keep a “temporary” fallback local resolver in type checking after this phase.
4. Do not mark Phase 71 complete until the explicit module-scoped contract is implemented.

## Tasks

- [TASK-480](tasks/TASK-480-module-scoped-resolution-api.md)
- [TASK-481](tasks/TASK-481-thread-module-id-through-lowering.md)
- [TASK-482](tasks/TASK-482-thread-module-id-through-typeck.md)
- [TASK-483](tasks/TASK-483-remove-typeck-fallback-resolver.md)
- [TASK-484](tasks/TASK-484-phase-71-closeout-docs-and-verification.md)

## Deliverable

One fully module-scoped symbolic capability resolution contract where lowering and type checking
share the same `CapabilityResolutionContext` plus explicit `ModuleId`, with no global-search helper
and no local fallback symbolic resolver path.
