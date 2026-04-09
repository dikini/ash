# TASK-473: Resolve Imported Capability Symbol Bindings

## Status: Planned

## Description

Extend import resolution so imported and re-exported capability symbols become visible symbolic
operational bindings in the importing module, including aliases and module-qualified references.

## Specification Reference

- [TASK-471](TASK-471-spec-module-owned-capability-resolution.md)
- [TASK-472](TASK-472-capability-symbol-export-metadata.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-471](TASK-471-spec-module-owned-capability-resolution.md)
- ✅ [TASK-472](TASK-472-capability-symbol-export-metadata.md)

## Requirements

1. Imported capability symbols must resolve to the same canonical `(provider, action)` target pair
   as the original declaration.
2. `use ... as ...` aliases must work for symbolic operational calls.
3. Re-export chains must preserve the same capability target metadata.
4. Module-qualified symbolic calls must resolve through the same import/module path rules as other
   qualified references.

## Implementation Notes

- This task owns import-visible symbolic capability bindings, not lowering/runtime dispatch.
- Keep explicit `provider:action(...)` outside this path.
- Avoid leaking runtime provider registry assumptions into import resolution.

## TDD Steps

### Red

- Add failing tests for:
  - importing a capability symbol
  - aliasing a capability symbol
  - re-exporting a capability symbol
  - module-qualified symbolic operational calls

### Green

- Import resolution produces visible capability symbol bindings with canonical target pairs.

## Completion Checklist

- [x] imported capability symbols resolve correctly - `Binding::capability_target` preserves (provider, action)
- [x] aliased imports covered - `test_capability_import_with_alias` test passes
- [x] re-export chains covered - `test_capability_reexport_chain` test passes  
- [x] qualified symbolic calls covered - crate::io::fs_read paths work
- [x] import resolver tests added - 4 new tests in import_resolver.rs
