# TASK-483: Remove Type-Checker Fallback Symbolic Resolver

## Status: Planned

## Description

Remove the remaining local fallback symbolic resolver path from type checking once shared-context
resolution is fully module-scoped.

## Specification Reference

- [TASK-482](TASK-482-thread-module-id-through-typeck.md)
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-482](TASK-482-thread-module-id-through-typeck.md)

## Requirements

1. Remove local fallback symbolic ACT resolution from `CapabilityChecker`.
2. Remove now-unused resolver helper code from `ash-typeck` if no longer referenced.
3. Keep explicit unresolved-name failures.

## TDD Steps

### Red

- Add a failing test or audit proving symbolic ACT validation still succeeds through fallback-only
  registration.

### Green

- Type checking has no fallback resolver path for symbolic ACT resolution.

## Completion Checklist

- [ ] fallback resolver removed
- [ ] dead resolver code removed or reduced
- [ ] tests updated
