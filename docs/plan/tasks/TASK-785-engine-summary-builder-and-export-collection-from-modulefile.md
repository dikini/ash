# TASK-785: Engine Summary Builder and Export Collection from ModuleFile

## Status: ✅ Complete

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
6. Preserve Phase 108 workflow-summary export data while refactoring type export collection: `ModuleExports.callables`, `InlineCallable.workflow_summary`, `PublicWorkflowSummary`, imported-summary origin stamping, and TASK-777 workflow-returning callable export behavior must survive the new ordinary-type summary builder.
7. Add or retain tests proving the new ordinary-type summary path does not clear workflow summaries on imported workflow-returning callables.

## Verification

- [x] Normal module loading and checking exports type metadata from ModuleFile/core summaries across the relevant engine entry points.
- [x] Existing import/export tests continue to pass.
- [x] Snippet scanner is not required for normal public type exports.

## Completion Notes

- Added the engine ModuleFile-backed ordinary-type metadata helper that parses a module with path metadata and lowers through `ash_parser::lower::lower_module_type_metadata` using a deterministic path-derived `ModuleIdentity`.
- Routed `collect_module_exports` and `Engine::check_module_file` through the ModuleFile/core summary path for ordinary type definitions while leaving legacy scanner helpers fenced for compatibility/TASK-789.
- Preserved Phase 108 workflow summary transport (`InlineCallable.workflow_summary`, import-origin stamping, and workflow-returning callable summaries) with focused TASK-785 regression coverage.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
