# TASK-784: Surface-to-Core Type Metadata Lowering and Source Anchors

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

TASK-782 and TASK-783.

## Objective

Lower surface ordinary type declarations into core summaries with stable source anchors.

## Requirements

1. Convert parser surface type definitions to core TypeDef/summary entries. `ash-core` owns carriers; parser/engine-side lowering code performs conversion because `ash-core` must not depend on `ash-parser`.
2. Preserve visibility, generic params, alias/struct/enum body, builtin marker, variant payloads, and spans.
3. Attach module/declaration source anchors for diagnostics.
4. Avoid string-only identity when module identity is available.
5. Add lowering tests for aliases, structs, enums, generic types, and builtin opaque types.

## Verification

- [ ] Surface declarations lower to core ordinary type declarations.
- [ ] Source spans survive into summary diagnostics.
- [ ] Existing SPEC-020 ADT behavior is preserved.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
