# TASK-771: Workflow Do Target Dictionary

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)

## Objective

Add `Workflow` as a compiler-known typed-do target using the existing SPEC-054 dictionary path.

## Requirements

1. Extend do-target resolution so `Workflow` resolves as kind `* -> *`.
2. Add a workflow `DoDictionary` using `workflow::unit` and `workflow::bind`.
3. Add a workflow tower level or equivalent internal classification.
4. Ensure `do:Workflow` synthesizes `Workflow<A>` from final `return`.
5. Ensure `<-` in `do:Workflow` requires `Workflow<A>` RHS.
6. Add diagnostics suggesting `workflow::from_proc` / `workflow::from_act` for wrong RHS tower.
7. Do not change `do:Act` or `do:Proc` behavior.

## TDD Steps

1. Write failing target-resolution tests for `Workflow`.
2. Write failing `do:Workflow { return 1 }` type/elaboration test.
3. Write failing bind test with two workflow RHS values.
4. Write negative tests for `Proc<A>` and `Act<A>` RHS values.
5. Implement dictionary resolution and typed elaboration support.
6. Run focused typechecker tests and do-notation regression suite.

## Verification

- [ ] `Workflow` resolves as a do target.
- [ ] `do:Workflow` elaborates to nested `workflow::bind`/`workflow::unit`.
- [ ] Wrong RHS constructors are rejected with explicit-lift hints.
- [ ] `do:Act` and `do:Proc` regression tests pass.
- [ ] CHANGELOG.md updated.
