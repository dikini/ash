# TASK-776: Workflow Comprehension Target

## Status: ✅ Complete

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
- 📝 TASK-773: Workflow algebra and contract intrinsic call elaboration, if direct workflow algebra calls are used in comprehension tests.
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

- [x] `[...]: Workflow` type-checks for workflow qualifiers.
- [x] Elaboration shape matches `do:Workflow` and produces the same `WorkflowForm` / projection-event alignment.
- [x] Negative tower mismatch diagnostics work.
- [x] Existing Act/Proc comprehensions still pass.
- [x] CHANGELOG.md updated.

## Completion Notes

Completed in Phase 108 by adding explicit TASK-776 regression coverage around the existing SPEC-055 comprehension-to-do path. `[...]: Workflow` now has parser coverage, typechecker coverage proving `Workflow<Int>` synthesis, elaboration equivalence with explicit `do:Workflow`, artifact/projection/obligation/source-origin alignment, raw `Proc` / `Act` RHS lift diagnostics, and explicit `workflow::from_proc` / `workflow::from_act` acceptance coverage. No new comprehension semantics were added: target inference, guards, pattern binders, and applicative semantics remain out of scope.
