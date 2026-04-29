# TASK-770: Workflow Type and Stdlib Operations

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)

## Objective

Register public `Workflow<A>` and add the first-slice `workflow` namespace operations.

## Requirements

1. Register `Workflow` as a builtin unary public type constructor.
2. Add or expose `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, and `workflow::from_act` with signatures from SPEC-056.
3. Ensure `workflow::from_act(a)` is equivalent to `workflow::from_proc(proc::from_act(a))`.
4. Preserve explicit tower boundaries: no implicit Act/Proc-to-Workflow conversion.
5. Add typechecker tests for positive and negative call shapes.

## TDD Steps

1. Write failing tests for resolving `Workflow<A>` as a type.
2. Write failing tests for each workflow operation signature.
3. Write negative tests for implicit `Act<A>`/`Proc<A>` use where `Workflow<A>` is expected.
4. Implement minimal type and operation registration.
5. Run focused `ash-typeck` tests and affected `cargo check`.

## Verification

- [ ] `Workflow<A>` resolves as a unary type constructor.
- [ ] All five workflow operations type-check.
- [ ] Explicit lifts work.
- [ ] Implicit lifts fail.
- [ ] Existing `Act`/`Proc` operations still pass regression tests.
- [ ] CHANGELOG.md updated.
