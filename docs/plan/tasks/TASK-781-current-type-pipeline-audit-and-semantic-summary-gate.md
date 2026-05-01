# TASK-781: Current Type Pipeline Audit and Semantic-Summary Gate

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

TASK-780.

## Objective

Audit the current fragmented type declaration pipeline and freeze the semantic-summary implementation gate before code changes.

## Requirements

1. Audit `ash-parser` ModuleFile/type-definition parsing paths.
2. Audit `ash-core` TypeDef/ModuleItem/ModuleGraph carriers.
3. Audit `ash-engine` ModuleExports and source-snippet type collection paths.
4. Audit `ash-typeck` TypeEnv declaration/registration/constructor exposure paths.
5. Document which snippet paths must be replaced or fenced, including all normal-path and compatibility call sites.
6. Produce `docs/plan/audits/TASK-781-type-pipeline-audit.md` with: exact files/functions inspected; a live call graph for `parse_type_def`, `ash_parser::parse_module::module_file`, `ash_parser::parse_surface_file`, `ash_parser::parse_surface_file_with_path`, `check_module_file`, `check_importable_module_file`, `collect_module_exports`, `load_ordinary_file`, `Engine::check`, `runtime_stdlib_type_defs`, `register_imported_type_defs`, imported type registration, and TypeEnv registration; snippet scanner replacement/fencing decisions; and a SPEC-057 requirement-to-task traceability matrix.
7. The audit MUST record current ModuleFile drift precisely: ordinary `type` declarations are parsed by standalone `parse_type_def`, but the live `surface::Definition` / `ModuleFile` path lacks an ordinary type item and module-file unknown-item recovery can skip type declarations.
8. The audit MUST record current parser-private type-definition carrier limitations, including `parse_type_def::TypeDef` being separate from `surface::Definition` and lacking the source-origin/span metadata required by SPEC-057 summaries.
9. The audit MUST record current private type export/import compatibility behavior, including any opaque empty-struct/builtin `CoreTypeDef` identity placeholders and whether TASK-786/TASK-787 should preserve, tag, or reject them.
10. The audit MUST include Phase 108 workflow-summary transport in the non-interference call graph: `ModuleExports.callables`, `InlineCallable.workflow_summary`, `ash_core::workflow_carrier::PublicWorkflowSummary`, `stamp_workflow_summary_import_origin`, workflow-returning pub-fn summary builders, `build_imported_closures`, `Workflow.imported_workflow_summaries`, `bind_imported_callable_types`, `TypeEnv::bind_public_workflow_summary`, `TypeEnv::lookup_public_workflow_summary`, and `check_expr` imported Workflow summary consumers. SPEC-A ordinary-type summaries must preserve this path.

## Verification

- [x] `docs/plan/audits/TASK-781-type-pipeline-audit.md` names exact files/functions affected.
- [x] Semantic-summary gate is stated before parser/core/engine/typeck edits begin.
- [x] No code behavior changes are made in this task except optional docs/test scaffolding.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
