# TASK-771: Workflow Type, Stdlib Operations, and Intrinsic Parameters

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [TASK-769](TASK-769-workflow-form-projection-semantics.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)

## Objective

Register public `Workflow<A>`, add the first-slice `workflow` namespace operations, and implement the non-denotable intrinsic parameter model for workflow contract arguments.

## Dependencies

- 📝 TASK-769: Workflow form, projection, obligation, and adapter semantics.
- 📝 TASK-770: contract surface/classifier decisions that define non-denotable intrinsic arguments.

## Requirements

1. Register `Workflow` as a builtin unary public type constructor in `crates/ash-typeck/src/type_env.rs` without exposing `Workflow<C, A>`.
2. Add internal Rust carriers for `WorkflowForm`, `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, `CoverageEvidence`, and `CoverageError`, aligned with TASK-769.
3. Add or expose `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` with signatures from SPEC-056.
4. Treat `Requirement` and `OpenPostcondition` as non-denotable intrinsic parameter classes. They must not be ordinary Ash type names, record fields, parameter types, return types, import/export types, pattern types, or constructor payload types.
5. Do not implement `workflow::requires` / `workflow::ensures` as normal `Type::Fn([Requirement], Workflow<Unit>)` calls that require first-class value typing of their arguments. Use an internal signature/intrinsic-call marker instead.
6. Attempts to store, pass, return, partially apply, import/export, or pattern-match `Requirement` / `OpenPostcondition` as values must fail with a dedicated diagnostic.
7. Ensure `workflow::from_act(a)` is equivalent to `workflow::from_proc(proc::from_act(a))` and emits the same delayed lower-contract coverage obligations.
8. Preserve explicit tower boundaries: no implicit Act/Proc-to-Workflow conversion.

## TDD Steps

1. Write failing tests for resolving `Workflow<A>` as a type.
2. Write failing tests for each workflow operation signature.
3. Write tests proving `Requirement` and `OpenPostcondition` are not denotable source types.
4. Write negative tests for higher-order or stored use of `workflow::requires` / `workflow::ensures` requiring first-class contract values.
5. Write negative tests for implicit `Act<A>`/`Proc<A>` use where `Workflow<A>` is expected.
6. Implement minimal type, carrier, intrinsic signature, and operation registration.
7. Run focused `ash-typeck` tests and affected `cargo check`.

## Verification

- [ ] `Workflow<A>` resolves as a unary type constructor.
- [ ] All seven workflow operations are registered.
- [ ] `Requirement` and `OpenPostcondition` are non-denotable in Ash source.
- [ ] Internal carriers are aligned with `WorkflowForm` and do not expose `Workflow<C, A>`.
- [ ] Explicit lifts work and emit delayed lower-contract coverage obligations.
- [ ] Implicit lifts fail.
- [ ] Existing `Act`/`Proc` operations still pass regression tests.
- [ ] CHANGELOG.md updated.
