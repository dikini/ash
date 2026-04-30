# TASK-783: Core Canonical Type IDs and ModuleSemanticSummary Carriers

## Status: 📝 Planned

## References

- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

## Dependencies

TASK-781.

## Objective

Add core-owned canonical identity and semantic summary carriers for ordinary type metadata.

## Requirements

1. Define or designate canonical ordinary type declaration IDs anchored by resolved module identity plus declaration name/item kind; import aliases and re-export paths must not mint new origin identities.
2. Define or designate constructor/variant identity carriers derived from parent type identity plus constructor/variant name and payload kind.
3. Define `ModuleSemanticSummary`/equivalent in `ash-core`.
4. Include visibility, representation exposure, source-origin, module identity/path, and diagnostic anchor metadata; spans are diagnostic anchors, not identity inputs.
5. Include reserved extension namespaces for future type-computation packets without interpreting them; future type-function, sealed-domain, generalized projection, and associated-family identity semantics remain deferred.
6. Avoid engine-private or parser-private ownership of semantic carriers.

## Verification

- [ ] Core-owned carrier exists and is serializable/debuggable as needed.
- [ ] Summary can represent public ordinary types and constructor exposure.
- [ ] No type-function/normalizer semantics are added.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
