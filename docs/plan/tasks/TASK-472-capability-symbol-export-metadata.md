# TASK-472: Add Capability Symbol Export Metadata

## Status: Planned

## Description

Teach module parsing/export collection to represent capability declarations as exported symbolic
operational targets with canonical `(provider, action)` metadata and visibility information.

## Specification Reference

- [TASK-471](TASK-471-spec-module-owned-capability-resolution.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-471](TASK-471-spec-module-owned-capability-resolution.md)

## Requirements

1. Capability declarations must contribute explicit symbolic-resolution metadata.
2. Export metadata must preserve visibility information for `pub` and non-`pub` capability items.
3. The representation must distinguish symbolic exported names from explicit provider/action pairs.
4. The design must be usable by import resolution without reconstructing ad hoc mappings later.

## Implementation Notes

- Likely touch points include `parse_module`, module item/export collection, and the structures
  passed into import resolution.
- The metadata should be explicit enough to support local names, qualified names, aliasing, and
  re-exports in later tasks.
- Do not hard-code std capability names into this export collection path.

## TDD Steps

### Red

- Add failing tests showing that a module-local capability declaration does not yet expose enough
  metadata for symbolic ACT resolution.

### Green

- Module parsing/export collection records capability symbols with canonical `(provider, action)`
  metadata and visibility.

## Completion Checklist

- [ ] capability declarations produce export metadata
- [ ] visibility preserved
- [ ] metadata includes canonical target pair
- [ ] parser/module tests added
