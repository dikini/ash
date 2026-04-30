# TASK-777: Workflow Contract Summary Import/Export

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [TASK-775](TASK-775-legacy-workflow-translation-and-deprecation.md)
- [TASK-776](TASK-776-workflow-comprehension-target.md)

## Objective

Preserve enough workflow type and contract summary metadata across modules for first-class and deprecated legacy workflows to compose through the same WorkflowForm path.

## Dependencies

- 📝 TASK-771: Workflow type, stdlib operations, and intrinsic parameters.
- 📝 TASK-772: WorkflowForm-preserving Workflow do target.
- 📝 TASK-774: Workflow lowering and runtime projection.
- 📝 TASK-775: Legacy workflow translation and deprecation.
- 📝 TASK-776: Workflow comprehension target.

## Requirements

1. Audit current module export/import carriers for workflow signatures and metadata loss.
2. Extend carriers to preserve workflow parameter types, return type, public `Workflow<A>` type, staged `WorkflowContractSummary<A>`, admission envelope summary, failure/report/provenance summary, and public alignment/source-origin anchors.
3. Preserve public alignment boundaries without exposing private body internals or private `WorkflowNodeId`s.
4. Reject imported workflow/proc/act values used in first-class workflow composition if required summaries are absent.
5. Add tests covering imported workflow values in `do:Workflow` and `[...]: Workflow`.
6. Ensure summaries for deprecated legacy workflow declarations and equivalent first-class workflow expressions expose equivalent public contract events.
7. Do not expose private body internals unnecessarily.

## TDD Steps

1. Write failing module import test where imported workflow is used in `do:Workflow`.
2. Write failing negative test for opaque/missing summary.
3. Write legacy-vs-first-class export equivalence tests.
4. Implement summary propagation through parser/engine/typechecker carriers.
5. Run module import/export tests.

## Verification

- [ ] Imported workflows preserve `Workflow<A>` type.
- [ ] Imported workflow summaries support coverage checking and staged contract composition.
- [ ] Public alignment/source-origin summaries survive without exposing private node ids.
- [ ] Deprecated legacy and equivalent first-class workflows export equivalent public contract summaries.
- [ ] Missing summaries produce diagnostics.
- [ ] Private body details are not required in public summaries.
- [ ] CHANGELOG.md updated.
