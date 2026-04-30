# TASK-774: Workflow Diagnostics and Negative Tests

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)

## Objective

Harden diagnostics and negative coverage for first-class workflow target behavior.

## Requirements

1. Depend on [TASK-769](TASK-769-workflow-form-projection-semantics.md), [TASK-776](TASK-776-workflow-contract-syntax-and-legacy-translation.md), [TASK-770](TASK-770-workflow-type-and-stdlib-operations.md), [TASK-771](TASK-771-workflow-do-target-dictionary.md), and [TASK-772](TASK-772-workflow-comprehension-target.md).
2. Add diagnostics for unknown/wrong-kind/missing Workflow dictionary states.
3. Add wrong RHS diagnostics for `do:Workflow` and workflow comprehensions.
4. Add explicit-lift hints for `workflow::from_proc` and `workflow::from_act`.
5. Add coverage/obligation diagnostics with evidence component labels, including lower Proc/Act coverage obligations emitted by `from_proc` / `from_act`.
6. Add diagnostics for `requires` assumptions that refine checking context but cannot be proven by final coverage/admission.
7. Add diagnostics for unresolved `ensures` result targets or postconditions whose suffix result type is incompatible.
8. Add opaque imported summary diagnostics.
9. Ensure parser-only lowering errors remain clear for workflow do/comprehension nodes.
10. Add shape diagnostics/tests proving neutral Proc-projection nodes are not erased before evidence-preserving optimization.
11. Add deprecation warning tests for legacy workflow declarations and rewrite hints.
12. Add diagnostics for ordinary first-class misuse of opaque `Requirement` / `OpenPostcondition` contract arguments.

## TDD Steps

1. Write focused diagnostic tests for every SPEC-056 diagnostic family.
2. Implement diagnostic wording and spans.
3. Run full affected typechecker/parser diagnostic tests.
4. Run Act/Proc diagnostic regression tests.

## Verification

- [ ] Diagnostics state expected type/constructor and found type.
- [ ] Coverage/obligation errors mention the failed evidence component.
- [ ] Lift hints are present where applicable.
- [ ] `requires` refinement failures distinguish assumed availability from proven admission.
- [ ] `ensures` target/type failures identify the suffix result boundary.
- [ ] Neutral-Proc contract nodes are not silently erased before evidence-preserving optimization.
- [ ] Deprecated legacy workflow declarations emit warnings with rewrite hints.
- [ ] Opaque `Requirement` / `OpenPostcondition` misuse diagnostics are covered.
- [ ] Act/Proc diagnostics do not regress.
- [ ] CHANGELOG.md updated.
