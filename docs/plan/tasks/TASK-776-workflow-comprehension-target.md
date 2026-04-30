# TASK-776: Workflow Comprehension Target

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)
- [TASK-774](TASK-774-workflow-lowering-runtime-projection.md)

## Objective

Enable explicit-target workflow comprehensions `[result | qualifiers]: Workflow` through the existing comprehension-to-do elaboration path while preserving WorkflowForm alignment.

## Dependencies

- 📝 TASK-772: WorkflowForm-preserving Workflow do target.
- 📝 TASK-773: Workflow contract intrinsic call elaboration, if direct intrinsic calls are used in comprehension tests.
- 📝 TASK-774: executable Workflow lowering/runtime projection for execution tests.

## Requirements

1. Accept `Workflow` as a comprehension target after TASK-772.
2. Normalize workflow comprehensions to equivalent `do:Workflow` blocks before WorkflowForm construction, reusing SPEC-055 infrastructure.
3. Preserve source spans, origin metadata, and `WorkflowNodeId` alignment through comprehension-to-do normalization.
4. Reject `Act<A>`/`Proc<A>` qualifier RHS values without explicit `workflow::from_act` / `workflow::from_proc` lifts.
5. Do not add guards, pattern binders, target inference, or applicative semantics.

## TDD Steps

1. Write failing parser/typechecker test for `[x | x <- wf]: Workflow`.
2. Write equivalence test comparing comprehension elaboration to explicit `do:Workflow` and WorkflowForm events.
3. Write negative tests for `Proc`/`Act` RHS without explicit lifts.
4. Implement minimal target acceptance/reuse of typed-do path.
5. Run comprehension regression tests.

## Verification

- [ ] `[...]: Workflow` type-checks for workflow qualifiers.
- [ ] Elaboration shape matches `do:Workflow` and produces the same `WorkflowForm` / projection-event alignment.
- [ ] Negative tower mismatch diagnostics work.
- [ ] Existing Act/Proc comprehensions still pass.
- [ ] CHANGELOG.md updated.
