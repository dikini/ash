# TASK-480: Make Capability Resolution API Explicitly Module-Scoped

## Status: Planned

## Description

Refactor `CapabilityResolutionContext` so unqualified symbolic resolution requires an explicit
current `ModuleId`, and qualified symbolic resolution uses a distinct explicit API.

## Specification Reference

- [PLAN-018](../PLAN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md)
- [DESIGN-018](../../design/DESIGN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- Existing Phase 71 infrastructure

## Requirements

1. Replace module-agnostic lowering lookup with `ModuleId`-scoped lookup.
2. Keep unqualified and qualified symbolic resolution as distinct operations.
3. Remove any helper that searches across all modules for unqualified names.
4. Preserve explicit unresolved-name failures.

## TDD Steps

### Red

- Add failing tests for:
  - two modules resolving the same unqualified symbol differently
  - unqualified lookup not finding another module's symbol
  - qualified lookup resolving through explicit target module

### Green

- `CapabilityResolutionContext` exposes only module-scoped symbolic lookup APIs.

## Completion Checklist

- [ ] module-scoped unqualified API added
- [ ] qualified API explicit
- [ ] global-search helper removed
- [ ] tests added
