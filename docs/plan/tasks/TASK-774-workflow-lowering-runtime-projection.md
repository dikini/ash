# TASK-774: Workflow Lowering and Runtime Projection

## Status: 🚧 In Progress

First slice implemented: `ash-core` now owns a public `WorkflowForm` lowering/projection carrier (`LoweredWorkflowProjection` / `WorkflowProcProjection`) and `lower_workflow_form` API. This slice proves `unit`, `bind`, `then`-shaped ignored binds, `requires`, `ensures`, `from_proc`, and `from_act` lower from shared carriers while preserving projection events, contract metadata, and delayed coverage obligations. Second slice implemented: `ash-interp` now exposes a runtime-facing projection boundary that consumes `ash-core` `WorkflowProcProjection<Value>` directly, executes sound `unit`/transparent `scope` cases, and names unsupported first-class Workflow projection execution with `FirstClassWorkflowProjectionExecutionUnsupported` rather than silently producing dead values. It does not claim full `bind` / `from_proc` / `from_act` execution yet.

Second-slice dependency audit: `ash-interp` already depends on `ash-core`, `ash-parser`, and `ash-typeck`, but the new runtime-facing projection boundary intentionally imports only `ash_core::{Value, workflow_carrier::{WorkflowNodeId, WorkflowProcProjection}}` and local `ExecError`/`ExecResult`. This slice performs API-boundary cleanup/enforcement only; it does not remove the existing broad `ash-interp` parser/typeck dependencies.

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-049](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-051](../../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [TASK-771](TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md)
- [TASK-772](TASK-772-workflow-form-preserving-do-target.md)
- [TASK-773](TASK-773-workflow-algebra-and-contract-intrinsic-call-elaboration.md)

## Objective

Make first-class Workflow values executable through the existing Proc/workflow boundary path by deriving a runtime/lowering projection from `WorkflowTypedArtifact` without erasing contract/evidence metadata.

## Dependencies

- 📝 TASK-771: Workflow type, qualified builtins, `ash-core` carrier substrate, and intrinsic parameter classes.
- 📝 TASK-772: WorkflowForm-preserving `do:Workflow` typed artifact.
- 📝 TASK-773: Workflow algebra and contract intrinsic call elaboration.

## Requirements

1. Define the lowering boundary from `WorkflowTypedArtifact` to the executable representation used by `ash-interp` / `ash-engine` / the existing workflow boundary.
2. The lowering boundary must consume shared `ash-core` workflow carriers or public summaries, not parser ASTs or typeck-private structs. `ash-interp` consumes executable projection/runtime metadata derived from those carriers.
3. `workflow::unit` and `workflow::bind` must produce executable Workflow values whose Proc projection sequences through the existing Proc unit/bind machinery.
4. `workflow::then` must be implemented as non-dependent sequencing equivalent to `workflow::bind(w1, |_| w2)` while preserving contract/projection event order.
5. `workflow::from_proc` and `workflow::from_act` enter Workflow without bypassing admission/coverage; they preserve lower summaries and emit delayed lower-carrier coverage obligations.
6. `workflow::requires` and `workflow::ensures` must survive lowering as contract/projection metadata and obligations. They must not become dead placeholders or be erased because their Proc projection is neutral.
7. Simple first-class workflow values created by `do:Workflow` must either run through the existing workflow boundary or fail at a named, tested execution boundary with a diagnostic that states Phase 108 is still check/lowering-only for the unsupported case.
8. Prefer executable behavior whenever the legacy workflow semantics can already execute the same shape. Do not defer execution of shapes that the existing legacy workflow path can run.
9. Add affected `ash-interp` / `ash-engine` / typechecker tests or explicit non-execution diagnostic tests.
10. Audit Cargo dependency boundaries for the chosen implementation shape. If `ash-engine` / `ash-interp` already depend broadly enough for implementation, document that this task enforces API-boundary cleanup rather than immediate dependency removal; if new dependencies would be needed, route shared carriers through `ash-core` instead of adding parser/typeck-private runtime dependencies.

## TDD Steps

1. Write lowering/runtime tests for `workflow::unit` and `workflow::bind` sequencing the Proc projection.
2. Write lowering/runtime tests for `workflow::then` preserving sequence order.
3. Write explicit-lift tests for `workflow::from_proc` and `workflow::from_act` preserving lower summaries and delayed obligations.
4. Write boundary tests proving `ash-interp` / `ash-engine` consume `ash-core` carriers or public summaries rather than parser AST or typeck-private `WorkflowTypedArtifact` internals.
5. Write tests proving `requires` / `ensures` nodes remain present in runtime/projection metadata after lowering.
6. Write a simple `do:Workflow { return x }` run/lowering test through the existing workflow boundary, or a named-boundary diagnostic test if execution is still impossible for a documented reason.
7. Write regression tests proving legacy executable workflow shapes are not made less executable by the new path.
8. Audit Cargo dependency boundaries and record whether this slice performs API-boundary cleanup only or actual dependency removal between `ash-engine`, `ash-interp`, `ash-parser`, and `ash-typeck`.
9. Implement lowering/runtime projection and diagnostics.
10. Run focused affected `ash-typeck`, `ash-interp`, and/or `ash-engine` tests.

## Verification

- [x] `workflow::unit` and transparent projection scopes have an `ash-interp` runtime-facing consumer through `WorkflowProcProjection<Value>`; `bind` / `then` remain explicitly unsupported at execution.
- [x] Runtime/lowering boundaries use `ash-core` carriers/public summaries and do not require parser ASTs or typeck-private structs for the first shared lowering slice (`lower_workflow_form`).
- [x] Cargo dependency boundaries are audited, and this slice records API-boundary cleanup/enforcement only rather than dependency removal.
- [x] Explicit lower-carrier lifts preserve summaries and coverage obligations in `ash-core` shared lowering tests.
- [x] Contract-injection nodes survive shared lowering into metadata/obligations (`requires` admission + obligations, `ensures` delayed result obligations).
- [ ] First-class Workflow execution matches existing legacy semantics where legacy execution already exists.
- [x] Unsupported execution cases fail at a named, tested boundary rather than silently producing dead values (`FirstClassWorkflowProjectionExecutionUnsupported` in `ash-interp`).
- [ ] Existing Proc/workflow boundary semantics are not redefined.
- [x] CHANGELOG.md updated for the first shared `ash-core` lowering/projection slice and the `ash-interp` named-boundary slice.
