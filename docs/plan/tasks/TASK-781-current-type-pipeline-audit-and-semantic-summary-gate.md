# TASK-781: Current Type Pipeline Audit and Semantic-Summary Gate

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

TASK-780.

## Objective

Audit the current fragmented type declaration pipeline and freeze the semantic-summary implementation gate before code changes.

## Requirements

1. Audit `ash-parser` ModuleFile/type-definition parsing paths.
2. Audit `ash-core` TypeDef/ModuleItem/ModuleGraph carriers.
3. Audit `ash-engine` ModuleExports and source-snippet type collection paths.
4. Audit `ash-typeck` TypeEnv declaration/registration/constructor exposure paths.
5. Document which snippet paths must be replaced or fenced, including all normal-path and compatibility call sites.
6. Produce `docs/plan/audits/TASK-781-type-pipeline-audit.md` with: exact files/functions inspected; a live call graph for `parse_type_def`, `module_file`, `check_module_file`, `collect_module_exports`, `load_ordinary_file`, `runtime_stdlib_type_defs`, imported type registration, and TypeEnv registration; snippet scanner replacement/fencing decisions; and a SPEC-057 requirement-to-task traceability matrix.

## Verification

- [ ] `docs/plan/audits/TASK-781-type-pipeline-audit.md` names exact files/functions affected.
- [ ] Semantic-summary gate is stated before parser/core/engine/typeck edits begin.
- [ ] No code behavior changes are made in this task except optional docs/test scaffolding.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
