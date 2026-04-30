# TASK-771: Workflow Type, Qualified Builtins, Shared Carriers, and Intrinsic Parameters

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [TASK-769](TASK-769-workflow-form-projection-semantics.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)

## Objective

Register public `Workflow<A>`, add the first-slice compiler-known qualified `workflow::...` builtins, define shared workflow carrier ownership, and implement the non-denotable intrinsic parameter model for workflow contract arguments.

## Dependencies

- 📝 TASK-769: Workflow form, projection, obligation, and adapter semantics.
- 📝 TASK-770: contract surface/classifier decisions that define non-denotable intrinsic arguments.

## Requirements

1. Register `Workflow` as a builtin unary public type constructor in `crates/ash-typeck/src/type_env.rs` without exposing `Workflow<C, A>`. Update `TypeEnv::check_type_constructor_arity` or its current equivalent so `Workflow` receives the same arity protection as `Proc` / `P`.
2. Add shared Rust carriers in `ash-core` for `WorkflowForm`, `WorkflowNodeId`, `ProjectionEvent`, `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, `CoverageEvidence`, `CoverageError`, public workflow summary types, and lower summary carriers needed by coverage, aligned with TASK-769/SPEC-056. Typechecker-private helper artifacts may wrap these, but shared semantic/runtime definitions must not live only in `ash-typeck`.
3. Add or expose compiler-known qualified builtins `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` with signatures from SPEC-056.
4. Register `workflow::...` names in the same qualified builtin namespace style as `proc::...` names. Do not implicitly import unqualified `unit`, `bind`, `then`, `from_proc`, `from_act`, `requires`, or `ensures` when `do:Workflow` is selected.
5. Treat `Requirement` and `OpenPostcondition` as non-denotable intrinsic parameter classes. They must not be ordinary Ash type names, record fields, parameter types, return types, import/export types, pattern types, or constructor payload types.
6. Do not implement `workflow::requires` / `workflow::ensures` as normal `Type::Fn([Requirement], Workflow<Unit>)` calls that require first-class value typing of their arguments. Use an internal signature/intrinsic-call marker instead.
7. Attempts to store, pass, return, partially apply, import/export, or pattern-match `Requirement` / `OpenPostcondition` as values must fail with a dedicated diagnostic.
8. Ensure `workflow::from_act(a)` is equivalent to `workflow::from_proc(proc::from_act(a))` and emits the same delayed lower-contract coverage obligations.
9. Preserve explicit tower boundaries: no implicit Act/Proc-to-Workflow conversion.
10. If a future ordinary stdlib module backs the compiler-known workflow namespace, preserve qualified exports and intrinsic markers through module summaries rather than changing this first-slice resolution model.

## TDD Steps

1. Write failing tests for resolving `Workflow<A>` as a type.
2. Write failing tests for each qualified `workflow::...` operation signature.
3. Write namespace tests proving qualified `workflow::...` names resolve like qualified `proc::...` names and unqualified workflow operation names are not implicitly imported by `do:Workflow`.
4. Write tests proving `Requirement` and `OpenPostcondition` are not denotable source types.
5. Write negative tests for higher-order or stored use of `workflow::requires` / `workflow::ensures` requiring first-class contract values.
6. Write negative tests for implicit `Act<A>`/`Proc<A>` use where `Workflow<A>` is expected.
7. Implement minimal type, `ash-core` shared carriers, intrinsic signature, and qualified operation registration.
8. Run focused `ash-typeck` tests and affected `cargo check`.

## Verification

- [ ] `Workflow<A>` resolves as a unary type constructor and wrong-arity `Workflow` uses are rejected by the type-constructor arity path.
- [ ] All seven workflow operations are registered as qualified compiler-known builtins.
- [ ] Unqualified workflow operation names are not implicitly imported by `do:Workflow`.
- [ ] `Requirement` and `OpenPostcondition` are non-denotable in Ash source.
- [ ] Shared carriers live in `ash-core`, are aligned with `WorkflowForm`, and do not expose `Workflow<C, A>`.
- [ ] Explicit lifts work and emit delayed lower-contract coverage obligations.
- [ ] Implicit lifts fail.
- [ ] Existing `Act`/`Proc` operations still pass regression tests.
- [ ] CHANGELOG.md updated.
