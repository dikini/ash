# TASK-775: Legacy Workflow Translation and Deprecation

## Status: 🛠️ In Progress (first warning-plumbing slice landed)

> Slice note: this task is not complete. The current slice only audits/extends the non-fatal warning path enough for accepted legacy workflow header declarations to carry and surface `[NEW] DeprecatedLegacyWorkflowDeclaration` through `ash check` without failing otherwise-successful checks. Full WorkflowForm translation, source-origin spans/rewrite hints, header-event semantic lowering, and legacy-body adapter work remain deferred.

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)
- [TASK-773](TASK-773-workflow-algebra-and-contract-intrinsic-call-elaboration.md)
- [TASK-774](TASK-774-workflow-lowering-runtime-projection.md)

## Objective

Deprecate the current workflow declaration surface while preserving its semantics by translating it to the same WorkflowForm implementation path as first-class workflow expressions.

## Dependencies

- 📝 TASK-770: source-ordered header events and classifier.
- 📝 TASK-771: Workflow type and operations.
- 📝 TASK-772: WorkflowForm-preserving artifact.
- 📝 TASK-773: Workflow algebra and contract intrinsic call equivalence.
- 📝 TASK-774: executable Workflow lowering/runtime projection.

## Requirements

1. Deprecated legacy workflow declarations remain accepted during the migration window.
2. Emit `[NEW] DeprecatedLegacyWorkflowDeclaration` exactly once per legacy workflow declaration, with declaration span and a rewrite hint to a named `Workflow<A>` value or `do:Workflow` expression.
3. Audit the existing non-fatal warning carrier/API before emitting `[NEW] DeprecatedLegacyWorkflowDeclaration`. Extend parser/typechecker/engine/CLI warning plumbing as needed so warnings are collected, surfaced by `ash check`, and do not fail `ash check` when no errors exist.
4. Translate `WorkflowDef.header_events` in source order into leading `Requires` / `Ensures` / admission/resource WorkflowForm events.
5. `ensures:` header events attach to the successful result boundary of the translated body suffix.
6. `any_role([...])` remains a single OR-role requirement event, never multiple AND requirements.
7. Define and implement `legacy_body_as_proc_summary` or equivalent adapter:
   - input: legacy body, params, source origin, checking environment;
   - output: Proc projection/artifact, Proc contract summary, failure summary, authority/resource summary, provenance summary, and source-origin map;
   - rejection: opaque/missing summaries, dynamic-admission-only bodies, or unsupported legacy constructs reject conservatively with diagnostics.
8. The adapter is compatibility-only and must not create a separate legacy runtime/typechecking semantic path. The wrapped body enters WorkflowForm as `FromProc(legacy_body_as_proc_summary)`.
9. Equivalent deprecated legacy declarations and first-class workflow expressions must produce equivalent public contract-event sequences and summaries, modulo source-origin metadata.

## TDD Steps

1. Audit current warning/diagnostic carriers and write or update a warning-pipeline smoke test proving non-fatal warnings flow to `ash check` without failure.
2. Write warning tests for a minimal legacy workflow declaration.
3. Write source-order translation tests for mixed header clauses.
4. Write equivalence tests pairing a legacy declaration with a first-class `do:Workflow` expression.
5. Write `ensures` suffix-target tests for translated legacy declarations.
6. Write `any_role` legacy translation tests proving OR semantics.
7. Write adapter rejection tests for unsupported/opaque legacy bodies.
8. Implement warning plumbing, header-event translation, and adapter contract.
9. Run focused parser/typechecker/engine/CLI diagnostics tests.

## Verification

- [x] Deprecated declarations warn but do not error solely because of deprecation for the accepted legacy header events covered by this slice.
- [x] Warning carrier/API support has been audited or extended, and `ash check` surfaces deprecation warnings without failing when no errors exist.
- [ ] Legacy declarations lower to the same WorkflowForm path as first-class workflows.
- [ ] Source-ordered header events are preserved in translation.
- [ ] Legacy body summaries enter through `FromProc(...)` with lower coverage obligations.
- [ ] Equivalent legacy and first-class forms produce equivalent public summaries/events.
- [ ] No separate legacy runtime/typechecking semantic path is introduced.
- [ ] CHANGELOG.md updated.
