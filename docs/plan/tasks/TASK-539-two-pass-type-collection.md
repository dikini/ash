# TASK-539: Two-pass type collection in module loader

## Status: Draft

## Description

Refactor the module loader's type collection to register all type names in a first pass, then validate type expressions in a second pass. This allows `pub type` definitions within a single file to reference each other regardless of declaration order.

## Spec Reference

- [SPEC-030: Module Type Resolution](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D1

## Dependencies

None (this is the root task).

## Requirements

1. `collect_public_type_defs_from_source` performs two passes: register names, then validate.
2. Type definitions can reference sibling types regardless of declaration order.
3. All 11 SPEC-029 types in `std/src/llm/types.ash` collect without error.
4. Unbound type references produce clear error messages.

## TDD Steps

### Red
1. Test: forward reference (`pub type A = A { x: B }; pub type B = B { y: Int };`) parses.
2. Test: `std/src/llm/types.ash` collects 11 types.

### Green
3. Add name-registration pass.
4. Add validation pass with accumulated name set.
5. Wire into `collect_module_exports`.

## Completion Checklist

- [ ] Two-pass collection implemented
- [ ] Forward type references work
- [ ] `types.ash` collects all 11 types
- [ ] Existing tests pass

