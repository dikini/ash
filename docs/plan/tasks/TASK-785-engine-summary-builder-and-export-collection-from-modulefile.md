# TASK-785: Engine Summary Builder and Export Collection from ModuleFile

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

TASK-784.

## Objective

Build engine export summaries from parsed ModuleFile/core summaries instead of raw source snippets.

## Requirements

1. Add summary builder path that consumes parsed ModuleFile/core items for `check_module_file`, `collect_module_exports`, `load_ordinary_file`, `parse_file`, `parse_workflow_source_with_imports`, and runtime stdlib type discovery as applicable.
2. Preserve public callable and module import behavior.
3. Ensure public function/workflow signatures can pull public type identities from summaries.
4. Route normal module checks away from snippet type extraction and add instrumentation/tests proving public type export works with snippet scanning disabled or fenced.
5. Keep any compatibility fallback explicitly fenced.

## Verification

- [ ] Normal module loading and checking exports type metadata from ModuleFile/core summaries across the relevant engine entry points.
- [ ] Existing import/export tests continue to pass.
- [ ] Snippet scanner is not required for normal public type exports.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
