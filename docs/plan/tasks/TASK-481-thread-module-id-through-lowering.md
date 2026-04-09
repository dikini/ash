# TASK-481: Thread `ModuleId` Through Lowering

## Status: Planned

## Description

Update lowering so symbolic ACT resolution passes the current `ModuleId` into the shared
capability-resolution context.

## Specification Reference

- [TASK-480](TASK-480-module-scoped-resolution-api.md)
- [SPEC-001](../../spec/SPEC-001-IR.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-480](TASK-480-module-scoped-resolution-api.md)

## Requirements

1. Extend `LoweringContext` to carry current `ModuleId`.
2. Resolve unqualified symbolic ACT calls through the module-scoped API.
3. Resolve qualified symbolic ACT calls through the explicit qualified API.
4. Keep explicit `provider:action(...)` lowering direct.

## TDD Steps

### Red

- Add failing lowering tests showing the current module changes symbolic ACT resolution results.

### Green

- Lowering uses explicit `ModuleId` for all symbolic resolution.

## Completion Checklist

- [ ] `LoweringContext` carries `ModuleId`
- [ ] symbolic ACT lowering passes `ModuleId`
- [ ] qualified symbolic ACT lowering uses explicit qualified lookup
- [ ] lowering tests added
