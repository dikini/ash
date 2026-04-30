# TASK-782: ModuleFile Ordinary Type Declaration Surface Integration

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

Parse existing ordinary `type` declarations as normal ModuleFile definitions.

## Requirements

1. Add `Definition::Type`/equivalent surface carrier for ordinary type declarations; do not treat the parser-private standalone `parse_type_def::TypeDef` as the final surface model unless spans/source-origin gaps are explicitly solved.
2. Reuse `parse_type_def` grammar where practical, but wire it through `module_file`, top-level definition dispatch, inline module parsing, and unknown-item recovery.
3. Preserve visibility, name, params, body, builtin marker, declaration spans, per-field/variant spans where available, and source origin.
4. Support files containing only ordinary type declarations as ModuleFile inputs.
5. Add parser tests for top-level module files, files containing only types, workflow files with local type declarations, inline-module behavior or targeted rejection, and valid `type` declarations not being skipped by unknown-item recovery.
6. Do not lower semantically in parser.

## Verification

- [ ] ModuleFile parse result contains ordinary type declarations.
- [ ] Standalone type parser tests still pass.
- [ ] Unknown-item recovery does not silently skip valid ordinary type declarations.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
