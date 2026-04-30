# TASK-773: Workflow Contract Intrinsic Call Elaboration

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [TASK-770](TASK-770-workflow-contract-surface-classifier-and-header-events.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)

## Objective

Implement direct intrinsic-call spelling for workflow contract injection so `workflow::requires(expr)` and `workflow::ensures(expr)` produce the same WorkflowForm events as `requires:` / `ensures:` statements without exposing first-class contract values.

## Dependencies

- 📝 TASK-770: classifier and contract surface.
- 📝 TASK-771: workflow operations and non-denotable intrinsic parameter classes.
- 📝 TASK-772: WorkflowForm-preserving typed-do artifact.

## Requirements

1. Special-case only calls whose callee resolves exactly to the compiler-known `workflow::requires` or `workflow::ensures` intrinsic.
2. Capture the raw argument expression before ordinary argument typechecking/name resolution of a `Requirement` / `OpenPostcondition` parameter.
3. Classify the argument with the same classifier used by statement forms and legacy header events.
4. Produce the same `Requires` / `Ensures` WorkflowForm nodes and projection events as statement forms, modulo source-origin metadata.
5. Allow direct intrinsic calls only in Workflow construction contexts where the result is used as `Workflow<Unit>`: `<-` / `_ <-` in `do:Workflow`, `[...]: Workflow` qualifier RHS after normalization, or compiler-known `workflow::bind`/`workflow::then` composition.
6. Reject higher-order use, partial application, storing the intrinsic name as a value, passing prebuilt `Requirement` variables, or exporting/importing contract argument values.
7. Standalone open `workflow::ensures(Q)` without a suffix workflow result target must reject at WorkflowForm finalization unless `Q` is explicitly closed and SPEC-056 allows that narrow case.

## TDD Steps

1. Write equivalence tests for `requires: role(admin);` and `_ <- workflow::requires(role(admin));`.
2. Write equivalence tests for `ensures: result > 0;` and `_ <- workflow::ensures(result > 0);`.
3. Write `any_role([...])` intrinsic-call tests proving OR-role semantics match statement form semantics.
4. Write negative tests for higher-order/stored/partial intrinsic use.
5. Write negative tests for standalone unresolved open `ensures`.
6. Implement intrinsic call recognition and WorkflowForm event construction.
7. Run focused typechecker/elaboration tests.

## Verification

- [ ] Direct intrinsic calls elaborate to the same WorkflowForm events as statement forms.
- [ ] Contract arguments are classified before ordinary value typing as contract values.
- [ ] `Requirement` / `OpenPostcondition` remain non-denotable.
- [ ] Standalone unresolved `ensures` rejects with a targeted diagnostic.
- [ ] Existing ordinary function-call behavior does not regress.
- [ ] CHANGELOG.md updated.
