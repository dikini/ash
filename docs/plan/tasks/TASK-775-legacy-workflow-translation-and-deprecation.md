# TASK-775: Legacy Workflow Translation and Deprecation

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)
- [TASK-773](TASK-773-workflow-contract-intrinsic-call-elaboration.md)
- [TASK-774](TASK-774-workflow-lowering-runtime-projection.md)

## Objective

Deprecate the current workflow declaration surface while preserving its semantics by translating it to the same WorkflowForm implementation path as first-class workflow expressions.

## Dependencies

- 📝 TASK-770: source-ordered header events and classifier.
- 📝 TASK-771: Workflow type and operations.
- 📝 TASK-772: WorkflowForm-preserving artifact.
- 📝 TASK-773: intrinsic contract injection equivalence.
- 📝 TASK-774: executable Workflow lowering/runtime projection.

## Requirements

1. Deprecated legacy workflow declarations remain accepted during the migration window.
2. Emit `[NEW] DeprecatedLegacyWorkflowDeclaration` exactly once per legacy workflow declaration, with declaration span and a rewrite hint to a named `Workflow<A>` value or `do:Workflow` expression.
3. Define and implement warning plumbing through a diagnostics-capable parser/typechecker/engine/CLI path. Warnings must not fail `ash check` when no errors exist.
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

1. Write warning tests for a minimal legacy workflow declaration.
2. Write source-order translation tests for mixed header clauses.
3. Write equivalence tests pairing a legacy declaration with a first-class `do:Workflow` expression.
4. Write `ensures` suffix-target tests for translated legacy declarations.
5. Write `any_role` legacy translation tests proving OR semantics.
6. Write adapter rejection tests for unsupported/opaque legacy bodies.
7. Implement warning plumbing, header-event translation, and adapter contract.
8. Run focused parser/typechecker/engine/CLI diagnostics tests.

## Verification

- [ ] Deprecated declarations warn but do not error solely because of deprecation.
- [ ] Legacy declarations lower to the same WorkflowForm path as first-class workflows.
- [ ] Source-ordered header events are preserved in translation.
- [ ] Legacy body summaries enter through `FromProc(...)` with lower coverage obligations.
- [ ] Equivalent legacy and first-class forms produce equivalent public summaries/events.
- [ ] No separate legacy runtime/typechecking semantic path is introduced.
- [ ] CHANGELOG.md updated.
