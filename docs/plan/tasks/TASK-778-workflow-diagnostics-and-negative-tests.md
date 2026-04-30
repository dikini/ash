# TASK-778: Workflow Diagnostics and Negative Tests

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)

## Objective

Harden diagnostics and negative coverage for first-class workflow target behavior after all semantic paths exist.

## Dependencies

- 📝 TASK-770: Workflow contract surface, classifier, and header events.
- 📝 TASK-771: Workflow type, qualified builtins, shared carriers, and intrinsic parameters.
- 📝 TASK-772: WorkflowForm-preserving Workflow do target.
- 📝 TASK-773: Workflow algebra and contract intrinsic call elaboration.
- 📝 TASK-774: Workflow lowering and runtime projection.
- 📝 TASK-775: Legacy workflow translation and deprecation.
- 📝 TASK-776: Workflow comprehension target.
- 📝 TASK-777: Workflow contract summary import/export.

## Requirements

1. Add diagnostics for unknown/wrong-kind/missing Workflow dictionary states.
2. Add wrong RHS diagnostics for `do:Workflow` and workflow comprehensions.
3. Add explicit-lift hints for `workflow::from_proc` and `workflow::from_act`.
4. Add diagnostics for contract statements outside `do:Workflow`.
5. Add diagnostics for ordinary first-class misuse of non-denotable `Requirement` / `OpenPostcondition` classes.
6. Add diagnostics for contract-expression classification failures, including unresolved role-policy or empty `any_role([])` failures.
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

- [ ] Diagnostics state expected type/constructor and found type.
- [ ] Coverage/obligation errors mention the failed evidence component.
- [ ] Lift hints are present where applicable.
- [ ] Contract statement misuse and intrinsic parameter misuse diagnostics are covered.
- [ ] `requires` refinement failures distinguish assumed availability from proven admission.
- [ ] `ensures` target/type failures identify the suffix result boundary.
- [ ] Deprecated legacy workflow declarations emit warnings with rewrite hints.
- [ ] `DeprecatedLegacyWorkflowDeclaration` reaches `ash check` as a non-fatal warning when no errors exist.
- [ ] Neutral-Proc contract nodes are not silently erased before evidence-preserving optimization.
- [ ] Act/Proc diagnostics do not regress.
- [ ] CHANGELOG.md updated.
