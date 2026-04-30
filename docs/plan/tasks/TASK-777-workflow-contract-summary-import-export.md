# TASK-777: Workflow Contract Summary Import/Export

## Status: ✅ Complete

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

- ✅ TASK-771: Workflow type, qualified builtins, shared carriers, and intrinsic parameters.
- ✅ TASK-772: WorkflowForm-preserving Workflow do target.
- ✅ TASK-774: Workflow lowering and runtime projection.
- ✅ TASK-775: Legacy workflow translation and deprecation.
- ✅ TASK-776: Workflow comprehension target.

## Requirements

1. Audit current module export/import carriers for workflow signatures and metadata loss.
2. Extend `ash-core` public summary carriers to preserve workflow parameter types, return type, public `Workflow<A>` type, staged `WorkflowContractSummary<A>`, admission envelope summary, failure/report/provenance summary, and public alignment/source-origin anchors.
3. Ensure `ash-engine` serializes/imports public summaries using those `ash-core` types and does not expose parser AST or typeck-private artifacts as the module summary format.
4. Preserve public alignment boundaries without exposing private body internals or private `WorkflowNodeId`s.
5. Reject imported workflow/proc/act values used in first-class workflow composition if required summaries are absent.
6. Add tests covering imported workflow values in `do:Workflow` and `[...]: Workflow`.
7. Ensure summaries for deprecated legacy workflow declarations and equivalent first-class workflow expressions expose equivalent public contract events.
8. If a future stdlib module backs compiler-known qualified builtins for `workflow::...` operations, preserve qualified workflow exports and intrinsic markers through module summaries without making unqualified `unit`, `bind`, `then`, `from_proc`, `from_act`, `requires`, or `ensures` implicit imports.
9. Audit Cargo dependency boundaries for module-summary propagation. Record whether this slice only enforces public API boundaries through `ash-core` summary types or also removes any direct parser/typeck-private dependency from engine/import/export paths.
10. Do not expose private body internals unnecessarily.

## TDD Steps

1. Write failing module import test where imported workflow is used in `do:Workflow`.
2. Write failing negative test for opaque/missing summary.
3. Write legacy-vs-first-class export equivalence tests.
4. Write export tests proving future/backing `workflow::...` qualified exports, if present, retain intrinsic markers and do not imply unqualified imports.
5. Audit Cargo dependency boundaries for the module-summary path and decide whether this task performs API-boundary cleanup only or actual dependency removal.
6. Implement summary propagation through `ash-core`/engine/typechecker carriers.
7. Run module import/export tests.

## Verification

- [x] Imported workflows preserve `Workflow<A>` type in the typechecker when paired with public summary metadata.
- [x] Imported workflow summaries support staged composition in `do:Workflow` and `[...]: Workflow` by recovering `WorkflowForm::ImportedSummary` from `TypeEnv`.
- [x] Public summaries are represented with `ash-core` types and imported by `ash-engine` without exposing typeck-private `WorkflowTypedArtifact` structs.
- [x] Public alignment/source-origin summaries survive as `SourceOrigin::ImportedSummary` without requiring private body internals.
- [x] Public summaries preserve exported workflow header `requires:` / `ensures:` contract events and coverage obligations via the shared `WorkflowForm` lowering path.
- [x] Qualified workflow exports are recorded as a future stdlib-backing follow-up; current Phase 108 keeps `workflow::...` operations compiler-known and verifies they do not imply unqualified operation imports.
- [x] Cargo dependency boundaries for summary propagation are audited; this slice enforces public API boundaries through `ash-core` summary types and does not remove existing engine parser/typechecker dependencies.
- [x] Deprecated legacy and equivalent first-class workflows export equivalent public contract summaries for supported first-class `do:Workflow` contract-statement bodies.
- [x] Missing summaries produce diagnostics.
- [x] Private body details are not required in public summaries.
- [x] CHANGELOG.md updated.

## Completion Notes

TASK-777 is complete for the Phase 108 public-summary contract:

- `ash-core` now has `WorkflowForm::ImportedSummary`, and lowering preserves imported public projection events / coverage obligations while projecting the imported opaque body as neutral rather than fabricating private internals.
- `ash-typeck::TypeEnv` stores `PublicWorkflowSummary` values and `do:Workflow` / `[...]: Workflow` artifact construction can recover imported summary-backed `WorkflowForm` nodes for imported variables and calls.
- `ash-engine` carries public workflow summaries from `module_loader::InlineCallable` into `Workflow.imported_workflow_summaries` and binds them into the type environment during checking.
- Tests cover typechecker composition from imported summaries, missing-summary rejection, engine-level module import/export of public workflow summary origins, and supported legacy-vs-first-class public contract-summary equivalence.
- `pub fn ... -> Workflow<A>` module exports now derive public summaries for the supported first-class `do:Workflow` subset containing public `requires:` / `ensures:` contract statements and a final `return`, while unsupported Workflow-returning function bodies remain opaque rather than fabricating summaries.

Follow-up work explicitly deferred out of TASK-777:

- Derive full public summaries from first-class workflow expression exports beyond the currently supported `do:Workflow` contract-statement subset, and from legacy adapters beyond header contract events.
- Add engine end-to-end `do:Workflow` / comprehension import tests after typed workflow expressions can be elaborated before core lowering in `parse_file`.
- Extend first-class-vs-legacy public contract-summary equivalence beyond the supported module-export subset as the summary adapter grows.
- Add future `workflow::...` backing-module preservation tests if/when such a stdlib module exists.
