# TASK-774: Workflow Lowering and Runtime Projection

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-049](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)
- [TASK-773](TASK-773-workflow-contract-intrinsic-call-elaboration.md)

## Objective

Make first-class Workflow values executable through the existing Proc/workflow boundary path by deriving a runtime/lowering projection from `WorkflowTypedArtifact` without erasing contract/evidence metadata.

## Dependencies

- 📝 TASK-771: Workflow type, operations, carrier substrate, and intrinsic parameter classes.
- 📝 TASK-772: WorkflowForm-preserving `do:Workflow` typed artifact.
- 📝 TASK-773: direct contract intrinsic call elaboration.

## Requirements

1. Define the lowering boundary from `WorkflowTypedArtifact` to the executable representation used by `ash-interp` / `ash-engine` / the existing workflow boundary.
2. `workflow::unit` and `workflow::bind` must produce executable Workflow values whose Proc projection sequences through the existing Proc unit/bind machinery.
3. `workflow::then` must be implemented as non-dependent sequencing equivalent to `workflow::bind(w1, |_| w2)` while preserving contract/projection event order.
4. `workflow::from_proc` and `workflow::from_act` enter Workflow without bypassing admission/coverage; they preserve lower summaries and emit delayed lower-carrier coverage obligations.
5. `workflow::requires` and `workflow::ensures` must survive lowering as contract/projection metadata and obligations. They must not become dead placeholders or be erased because their Proc projection is neutral.
6. Simple first-class workflow values created by `do:Workflow` must either run through the existing workflow boundary or fail at a named, tested execution boundary with a diagnostic that states Phase 108 is still check/lowering-only for the unsupported case.
7. Prefer executable behavior whenever the legacy workflow semantics can already execute the same shape. Do not defer execution of shapes that the existing legacy workflow path can run.
8. Add affected `ash-interp` / `ash-engine` / typechecker tests or explicit non-execution diagnostic tests.

## TDD Steps

1. Write lowering/runtime tests for `workflow::unit` and `workflow::bind` sequencing the Proc projection.
2. Write lowering/runtime tests for `workflow::then` preserving sequence order.
3. Write explicit-lift tests for `workflow::from_proc` and `workflow::from_act` preserving lower summaries and delayed obligations.
4. Write tests proving `requires` / `ensures` nodes remain present in runtime/projection metadata after lowering.
5. Write a simple `do:Workflow { return x }` run/lowering test through the existing workflow boundary, or a named-boundary diagnostic test if execution is still impossible for a documented reason.
6. Write regression tests proving legacy executable workflow shapes are not made less executable by the new path.
7. Implement lowering/runtime projection and diagnostics.
8. Run focused affected `ash-typeck`, `ash-interp`, and/or `ash-engine` tests.

## Verification

- [ ] `workflow::unit`, `workflow::bind`, and `workflow::then` have executable Proc projections.
- [ ] Explicit lower-carrier lifts preserve summaries and coverage obligations.
- [ ] Contract-injection nodes survive lowering into metadata/obligations.
- [ ] First-class Workflow execution matches existing legacy semantics where legacy execution already exists.
- [ ] Unsupported execution cases fail at a named, tested boundary rather than silently producing dead values.
- [ ] Existing Proc/workflow boundary semantics are not redefined.
- [ ] CHANGELOG.md updated.
