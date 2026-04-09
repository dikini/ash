# TASK-482: Thread `ModuleId` Through Type Checking

## Status: Planned

## Description

Update capability checking/type checking so symbolic ACT validation uses the shared
`CapabilityResolutionContext` with explicit current `ModuleId`.

## Specification Reference

- [TASK-480](TASK-480-module-scoped-resolution-api.md)
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-480](TASK-480-module-scoped-resolution-api.md)

## Requirements

1. `CapabilityChecker` must accept current `ModuleId` alongside shared resolution context.
2. Symbolic ACT validation must use module-scoped shared-context lookup.
3. Qualified symbolic ACT validation must use explicit qualified lookup.
4. Type-checking behavior must agree with lowering for the same module.

## TDD Steps

### Red

- Add failing tests for module-scoped symbolic ACT validation and module-qualified ACT validation.

### Green

- Type checking uses explicit `ModuleId` plus shared context.

## Completion Checklist

- [ ] `CapabilityChecker` accepts `ModuleId`
- [ ] symbolic validation uses shared context
- [ ] qualified validation uses explicit qualified lookup
- [ ] lowering/typecheck agreement tests added
