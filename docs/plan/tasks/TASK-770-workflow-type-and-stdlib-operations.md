# TASK-770: Workflow Type and Stdlib Operations

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)

## Objective

Register public `Workflow<A>`, add internal carrier scaffolding derived from TASK-769's preserved `WorkflowForm`, and add the first-slice `workflow` namespace operations.

## Requirements

1. Depend on [TASK-769](TASK-769-workflow-form-projection-semantics.md); do not implement carriers until the workflow-form/projection/obligation model is hardened.
2. Register `Workflow` as a builtin unary public type constructor.
3. Add internal Rust carriers for `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, `CoverageEvidence`, and `CoverageError`, derived from or aligned with `WorkflowForm` rather than stored as an unrelated metadata wrapper.
4. Add or expose `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` with signatures from SPEC-056.
5. Ensure `workflow::from_act(a)` is equivalent to `workflow::from_proc(proc::from_act(a))` and emits the same delayed lower-contract coverage obligations.
6. Preserve explicit tower boundaries: no implicit Act/Proc-to-Workflow conversion.
7. Add typechecker tests for positive and negative call shapes.
8. Add shape tests proving `requires`/`ensures` remain workflow-form nodes even when their Proc projection is neutral.

## TDD Steps

1. Write failing tests for resolving `Workflow<A>` as a type.
2. Write failing tests for each workflow operation signature, including `workflow::requires` and `workflow::ensures`.
3. Write negative tests for implicit `Act<A>`/`Proc<A>` use where `Workflow<A>` is expected.
4. Write carrier-shape tests proving public `Workflow<A>` hides contract/evidence parameters while preserving workflow-form alignment internally.
5. Implement minimal type, carrier, evidence, and operation registration.
6. Run focused `ash-typeck` tests and affected `cargo check`.

## Verification

- [ ] `Workflow<A>` resolves as a unary type constructor.
- [ ] All seven workflow operations type-check.
- [ ] Internal carriers are aligned with `WorkflowForm` and do not expose `Workflow<C, A>`.
- [ ] Explicit lifts work and emit delayed lower-contract coverage obligations.
- [ ] Implicit lifts fail.
- [ ] `requires` and `ensures` nodes are preserved despite neutral Proc projections.
- [ ] Existing `Act`/`Proc` operations still pass regression tests.
- [ ] CHANGELOG.md updated.
