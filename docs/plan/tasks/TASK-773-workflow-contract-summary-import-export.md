# TASK-773: Workflow Contract Summary Import/Export

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)

## Objective

Preserve enough workflow type and contract summary metadata across modules for first-class workflow composition.

## Requirements

1. Depend on [TASK-769](TASK-769-workflow-form-projection-semantics.md) and [TASK-776](TASK-776-workflow-contract-syntax-and-legacy-translation.md).
2. Audit current module export/import carriers for workflow signatures and metadata loss.
3. Extend carriers to preserve workflow parameter types, return type, public `Workflow<A>` type, public staged `WorkflowContractSummary<A>`, admission envelope summary, and failure/report/provenance summary.
4. Preserve public alignment boundaries and source-origin diagnostics without exposing private body internals or private `WorkflowNodeId`s.
5. Reject imported workflow/proc/act values used in first-class workflow composition if required summaries are absent.
6. Add tests covering imported workflow values in `do:Workflow` and `[...]: Workflow`.
7. Ensure summaries for deprecated legacy workflow declarations and equivalent first-class workflow expressions expose equivalent public contract events.
8. Do not expose private body internals unnecessarily.

## TDD Steps

1. Write failing module import test where imported workflow is used in `do:Workflow`.
2. Write failing negative test for opaque/missing summary.
3. Implement summary propagation through parser/engine/typechecker carriers.
4. Run module import/export tests.

## Verification

- [ ] Imported workflows preserve `Workflow<A>` type.
- [ ] Imported workflow summaries support coverage checking and staged contract composition.
- [ ] Public alignment/source-origin summaries survive without exposing private node ids.
- [ ] Deprecated legacy and equivalent first-class workflows export equivalent public contract summaries.
- [ ] Missing summaries produce diagnostics.
- [ ] Private body details are not required in public summaries.
- [ ] CHANGELOG.md updated.
