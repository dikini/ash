# TASK-774: Workflow Diagnostics and Negative Tests

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)

## Objective

Harden diagnostics and negative coverage for first-class workflow target behavior.

## Requirements

1. Add diagnostics for unknown/wrong-kind/missing Workflow dictionary states.
2. Add wrong RHS diagnostics for `do:Workflow` and workflow comprehensions.
3. Add explicit-lift hints for `workflow::from_proc` and `workflow::from_act`.
4. Add coverage-failure diagnostics with evidence component labels.
5. Add opaque imported summary diagnostics.
6. Ensure parser-only lowering errors remain clear for workflow do/comprehension nodes.

## TDD Steps

1. Write focused diagnostic tests for every SPEC-056 diagnostic family.
2. Implement diagnostic wording and spans.
3. Run full affected typechecker/parser diagnostic tests.
4. Run Act/Proc diagnostic regression tests.

## Verification

- [ ] Diagnostics state expected type/constructor and found type.
- [ ] Coverage errors mention the failed evidence component.
- [ ] Lift hints are present where applicable.
- [ ] Act/Proc diagnostics do not regress.
- [ ] CHANGELOG.md updated.
