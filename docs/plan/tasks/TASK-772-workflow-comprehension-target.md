# TASK-772: Workflow Comprehension Target

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)

## Objective

Enable explicit-target workflow comprehensions `[result | qualifiers]: Workflow` through the existing comprehension-to-do elaboration path.

## Requirements

1. Depend on [TASK-769](TASK-769-workflow-form-projection-semantics.md), [TASK-776](TASK-776-workflow-contract-syntax-and-legacy-translation.md), [TASK-770](TASK-770-workflow-type-and-stdlib-operations.md), and [TASK-771](TASK-771-workflow-do-target-dictionary.md).
2. Accept `Workflow` as a comprehension target after TASK-771.
3. Normalize workflow comprehensions to equivalent `do:Workflow` blocks before workflow-form construction.
4. Preserve source spans, origin metadata, and `WorkflowNodeId` alignment through the comprehension-to-do normalization.
5. Reject `Act<A>`/`Proc<A>` qualifier RHS values without explicit lifts.
6. Do not add guards, pattern binders, target inference, or applicative semantics.

## TDD Steps

1. Write failing parser/typechecker test for `[x | x <- wf]: Workflow`.
2. Write equivalence test comparing comprehension elaboration to explicit `do:Workflow`.
3. Write negative tests for `Proc`/`Act` RHS without explicit lifts.
4. Implement minimal target acceptance/reuse of typed-do path.
5. Run comprehension regression tests.

## Verification

- [ ] `[...]: Workflow` type-checks for workflow qualifiers.
- [ ] Elaboration shape matches `do:Workflow` and produces the same `WorkflowForm` / projection-event alignment.
- [ ] Negative tower mismatch diagnostics work.
- [ ] Existing Act/Proc comprehensions still pass.
- [ ] CHANGELOG.md updated.
