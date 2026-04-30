# TASK-778: Workflow Diagnostics and Negative Tests

## Status: 🚧 Partial

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)

## Objective

Harden diagnostics and negative coverage for first-class workflow target behavior after all semantic paths exist.

## Dependencies

- ✅ TASK-770: Workflow contract surface, classifier, and header events.
- ✅ TASK-771: Workflow type, qualified builtins, shared carriers, and intrinsic parameters.
- ✅ TASK-772: WorkflowForm-preserving Workflow do target.
- ✅ TASK-773: Workflow algebra and contract intrinsic call elaboration.
- ✅ TASK-774: Workflow lowering and runtime projection.
- ✅ TASK-775: Legacy workflow translation and deprecation.
- ✅ TASK-776: Workflow comprehension target.
- 🚧 TASK-777: Workflow contract summary import/export has a partial substrate slice; public `requires:` / `ensures:` summary export is covered, while first-class export equivalence and richer summaries remain follow-up work.

## Requirements

1. Add diagnostics for unknown/wrong-kind/missing Workflow dictionary states. First slice covers generalized `do` target diagnostics for unknown and wrong-kind targets and now names the accepted `Act`, `Proc`, and `Workflow` compiler-known constructors.
2. Add wrong RHS diagnostics for `do:Workflow` and workflow comprehensions.
3. Add explicit-lift hints for `workflow::from_proc` and `workflow::from_act`.
4. Add diagnostics for contract statements outside `do:Workflow`.
5. Add diagnostics for ordinary first-class misuse of non-denotable `Requirement` / `OpenPostcondition` classes.
6. Add diagnostics for contract-expression classification failures, including unresolved role-policy or empty `any_role([])` failures. Current slice covers stable `workflow requires` / `workflow ensures` classifier wording for empty `any_role`, invalid role-policy entries, and non-`result` postcondition targets.
7. Add coverage/obligation diagnostics with evidence component labels, including lower Proc/Act coverage obligations emitted by `from_proc` / `from_act`.
8. Add diagnostics for `requires` assumptions that refine checking context but cannot be proven by final coverage/admission.
9. Add diagnostics for unresolved `ensures` result targets or postconditions whose suffix result type is incompatible.
10. Add opaque imported summary diagnostics.
11. Ensure parser-only lowering errors remain clear for workflow do/comprehension nodes.
12. Add shape diagnostics/tests proving neutral Proc-projection nodes are not erased before evidence-preserving optimization.
13. Audit and, if necessary, extend the non-fatal warning carrier/API before asserting deprecation-warning behavior. `DeprecatedLegacyWorkflowDeclaration` must flow to `ash check` output without making the command fail when no errors exist.
14. Add deprecation warning tests for legacy workflow declarations and rewrite hints.

## TDD Steps

1. Audit current parser/typechecker/engine/CLI warning carriers and add a warning-pipeline smoke test if coverage is missing.
2. Write focused diagnostic tests for every SPEC-056 diagnostic family.
3. Implement diagnostic wording, warning severity, and spans.
4. Run full affected typechecker/parser/engine/CLI diagnostic tests.
5. Run Act/Proc diagnostic regression tests.

## Verification

- [x] Diagnostics state expected type/constructor and found type for generalized `do` target unknown/wrong-kind cases.
- [x] Coverage/obligation errors mention the failed evidence component.
- [ ] Lift hints are present where applicable.
- [ ] Contract statement misuse and intrinsic parameter misuse diagnostics are covered.
- [x] Contract-expression classification failures cover empty `any_role`, invalid role-policy entries, and non-`result` `workflow ensures` targets with stable Requirement/OpenPostcondition wording.
- [x] `requires` refinement failures distinguish assumed availability from proven admission at the carrier diagnostic boundary.
- [x] `ensures` target/type failures identify the successful-result suffix boundary at the carrier diagnostic boundary.
- [x] Deprecated legacy workflow declarations emit warnings with rewrite hints in `ash check` human and JSON output.
- [x] `DeprecatedLegacyWorkflowDeclaration` reaches `ash check` as a non-fatal warning when no errors exist.
- [ ] Neutral-Proc contract nodes are not silently erased before evidence-preserving optimization.
- [ ] Act/Proc diagnostics do not regress.
- [x] CHANGELOG.md updated.

## Partial Slice Notes

Implemented the current TASK-778 diagnostic slices:

- Stable workflow contract classifier diagnostics now identify Requirement/OpenPostcondition classification failures for empty `any_role`, invalid role-policy entries, and non-`result` `workflow ensures` targets.
- Generalized `do` target diagnostics now name stale/missing dictionary states against the current accepted constructors: `Act`, `Proc`, and `Workflow`.
- `ash check` now has CLI-level regression coverage proving deprecated legacy workflow declarations emit non-fatal warnings in both human and JSON output.
- Legacy workflow deprecation diagnostics now use the stable `DeprecatedLegacyWorkflowDeclaration` code instead of the provisional `[NEW] ...` spelling.
- Headerless legacy workflow declarations and declarations with multiple legacy header events both produce one declaration-level warning.
- JSON warning output now uses the workflow declaration span as its diagnostic anchor.
- Coverage and obligation carriers now expose stable evidence-component labels/messages for lower Proc/Act obligations, missing projection events, and opaque imported summary rejections.
- Carrier diagnostics for `RequirementMustHold`, `RequirementRefinementCovered`, and `OpenPostconditionTarget` now distinguish final admission proof, requires-assumption/refinement coverage, and successful-result postcondition target boundaries.

Remaining TASK-778 work covers the broader SPEC-056 diagnostic matrix: wrong-kind Workflow dictionary states, contract intrinsic misuse, source/typechecker-level admission proof validation beyond this carrier-label slice, neutral projection preservation, and Act/Proc diagnostic regression coverage.
