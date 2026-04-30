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

Public workflow summaries are shared semantic/module carriers owned by `ash-core`. `ash-engine` serializes/imports those public summaries; downstream checking must not require parser ASTs, raw `WorkflowHeaderEvent`s, or typeck-private `WorkflowTypedArtifact` structs.

## Dependencies

- 📝 TASK-771: Workflow type, qualified builtins, shared carriers, and intrinsic parameters.
- 📝 TASK-772: WorkflowForm-preserving Workflow do target.
- 📝 TASK-774: Workflow lowering and runtime projection.
- 📝 TASK-775: Legacy workflow translation and deprecation.
- 📝 TASK-776: Workflow comprehension target.

## Requirements

1. Audit current module export/import carriers for workflow signatures and metadata loss.
2. Extend `ash-core` public summary carriers to preserve workflow parameter types, return type, public `Workflow<A>` type, staged `WorkflowContractSummary<A>`, admission envelope summary, failure/report/provenance summary, and public alignment/source-origin anchors.
3. Ensure `ash-engine` serializes/imports public summaries using those `ash-core` types and does not expose parser AST or typeck-private artifacts as the module summary format.
4. Preserve public alignment boundaries without exposing private body internals or private `WorkflowNodeId`s.
5. Reject imported workflow/proc/act values used in first-class workflow composition if required summaries are absent.
6. Add tests covering imported workflow values in `do:Workflow` and `[...]: Workflow`.
7. Ensure summaries for deprecated legacy workflow declarations and equivalent first-class workflow expressions expose equivalent public contract events.
8. If a future stdlib module backs compiler-known qualified builtins for `workflow::...` operations, preserve qualified workflow exports and intrinsic markers through module summaries without making unqualified `unit` / `bind` / `requires` / `ensures` implicit imports.
9. Do not expose private body internals unnecessarily.

## TDD Steps

1. Write failing module import test where imported workflow is used in `do:Workflow`.
2. Write failing negative test for opaque/missing summary.
3. Write legacy-vs-first-class export equivalence tests.
4. Write export tests proving future/backing `workflow::...` qualified exports, if present, retain intrinsic markers and do not imply unqualified imports.
5. Implement summary propagation through `ash-core`/engine/typechecker carriers.
6. Run module import/export tests.

## Verification

- [ ] Imported workflows preserve `Workflow<A>` type.
- [ ] Imported workflow summaries support coverage checking and staged contract composition.
- [ ] Public summaries are represented with `ash-core` types and serialized/imported by `ash-engine` without parser AST or typeck-private struct dependencies.
- [ ] Public alignment/source-origin summaries survive without exposing private node ids.
- [ ] Qualified workflow exports remain preservable if future stdlib backing is added, without implicit unqualified operation imports.
- [ ] Deprecated legacy and equivalent first-class workflows export equivalent public contract summaries.
- [ ] Missing summaries produce diagnostics.
- [ ] Private body details are not required in public summaries.
- [ ] CHANGELOG.md updated.
