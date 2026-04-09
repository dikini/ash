# TASK-475: Make Lowering Use Module-Owned Capability Resolution

## Status: Planned

## Description

Replace lowering-local bridge resolution with the pipeline-owned capability-resolution context for
symbolic and module-qualified operational calls.

## Specification Reference

- [TASK-474](TASK-474-capability-resolution-context-pipeline.md)
- [SPEC-001](../../spec/SPEC-001-IR.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-474](TASK-474-capability-resolution-context-pipeline.md)

## Requirements

1. Lowering must stop constructing built-in capability resolver tables.
2. Symbolic and module-qualified operational calls must resolve via the passed-in context.
3. Explicit `provider:action(...)` must still lower directly.
4. Unresolved symbolic calls must fail explicitly and deterministically.

## Implementation Notes

- This task closes the parser-local bridge for symbolic ACT lowering.
- Keep the canonical lowered target as `Act { provider_name, action_name, ... }`.
- Do not collapse explicit and symbolic surface forms into one overloaded pre-resolution string.

## TDD Steps

### Red

- Add failing tests proving lowering depends on built-in resolver construction rather than a passed
  context.

### Green

- Lowering consumes the module-owned capability-resolution context and no longer builds its own
  bridge resolver.

## Completion Checklist

- [x] lowering-local built-in resolver removed - `CapabilityResolver::with_builtin_mappings()` no longer called in lowering
- [x] symbolic lowering uses shared context - `ctx.resolve_capability()` used for symbolic targets
- [x] qualified symbolic lowering uses shared context - `ctx.resolve_capability()` used for qualified targets
- [x] explicit form still lowers directly - explicit `provider:action` bypasses resolution
- [x] lowering tests updated - 3 new tests for module-owned capability resolution
