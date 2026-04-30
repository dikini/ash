# TASK-771: Workflow Do Target Dictionary

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)

## Objective

Add `Workflow` as a compiler-known typed-do target using the existing SPEC-054 dictionary path.

## Requirements

1. Depend on [TASK-769](TASK-769-workflow-form-projection-semantics.md) and [TASK-770](TASK-770-workflow-type-and-stdlib-operations.md).
2. Extend do-target resolution so `Workflow` resolves as kind `* -> *`.
3. Add a workflow `DoDictionary` using `workflow::unit` and `workflow::bind`.
4. Add a workflow tower level or equivalent internal classification.
5. Ensure `do:Workflow` synthesizes `Workflow<A>` from final `return`.
6. Ensure `<-` in `do:Workflow` requires `Workflow<A>` RHS.
7. Lower `requires R;` as `_ <- workflow::requires(R);` and `ensures Q;` as `_ <- workflow::ensures(Q);`, preserving the corresponding `WorkflowForm` nodes and projection events.
8. Add diagnostics suggesting `workflow::from_proc` / `workflow::from_act` for wrong RHS tower.
9. Do not change `do:Act` or `do:Proc` behavior.

## TDD Steps

1. Write failing target-resolution tests for `Workflow`.
2. Write failing `do:Workflow { return 1 }` type/elaboration test.
3. Write failing bind test with two workflow RHS values.
4. Write failing workflow-block tests for `requires` and `ensures` statement lowering into preserved `WorkflowForm` nodes.
5. Write negative tests for `Proc<A>` and `Act<A>` RHS values.
6. Implement dictionary resolution and typed elaboration support.
7. Run focused typechecker tests and do-notation regression suite.

## Verification

- [ ] `Workflow` resolves as a do target.
- [ ] `do:Workflow` elaborates to nested `workflow::bind`/`workflow::unit` and preserved `WorkflowForm` nodes.
- [ ] `requires` and `ensures` statements lower through the same bind path as ordinary workflow actions.
- [ ] Wrong RHS constructors are rejected with explicit-lift hints.
- [ ] `do:Act` and `do:Proc` regression tests pass.
- [ ] CHANGELOG.md updated.
