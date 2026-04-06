# TASK-407: Spawned-Child Execution Substrate and Completion Sealing

## Status: ✅ Complete

## Description

Implement the missing runtime substrate needed for real spawned-child lifecycle execution in `ash-interp`, so later retained completion observation can be driven by an actual child terminal lifecycle path rather than by manual recording APIs or invented inline semantics.

This task exists because TASK-406 established two important truths:

1. a sealed/write-once retained completion carrier is useful and can exist conservatively; but
2. the current `Workflow::Spawn` runtime surface does not include enough information to execute a real child workflow lifecycle honestly in `ash-interp`.

Today `Workflow::Spawn` carries `workflow_type + init + pattern + continuation`, but the inspected runtime path does not yet have a child-workflow execution substrate or lookup mechanism that would let the runtime execute the spawned child instance itself and then seal its authoritative terminal completion.

Therefore TASK-407 is the next prerequisite follow-on slice: add the missing spawned-child execution substrate first, then use it to drive automatic completion sealing on a real lifecycle path.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ MCE-007 closeout corpus published

## Requirements

### Functional Requirements

1. Introduce an honest runtime substrate for executing a spawned child workflow instance, rather than only allocating an `Instance × ControlLink` handle.
2. The substrate must preserve useful live control authority after spawn; it must not eagerly terminate children inline merely to manufacture completion records.
3. Provide a narrow mechanism for the runtime to associate `workflow_type` with the child workflow execution body or equivalent runtime-owned child entry contract.
4. Add tests demonstrating at least:
   - real spawn creates usable control authority,
   - the child runtime path can actually execute,
   - terminal child completion can then drive automatic retained completion sealing,
   - completion sealing remains write-once/stable.
5. Keep the implementation conservative and do not claim full `CompletionPayload` parity unless the runtime genuinely carries the required contents.

### Non-Functional Requirements

1. Do not regress current pause/resume/check-health/kill control behavior.
2. Do not overclaim cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
3. Prefer additive, contract-first substrate work over broad runtime rewrites.
4. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- `ash-interp` has a sealed/write-once retained completion carrier, but no honest spawned-child execution substrate that can drive it automatically;
- `Workflow::Spawn` currently allocates live control authority only;
- previous attempts to fake child completion inline regressed control semantics and had to be reverted.

### Green

This task is complete when:

- the runtime has a real spawned-child execution substrate;
- automatic retained completion sealing is driven by that real child lifecycle path;
- spawn/control semantics remain live and usable;
- docs/reporting surfaces reflect the new runtime slice conservatively.

## Files

- Modify: `crates/ash-interp` runtime/spawn/control-related files as needed
- Modify: `docs/plan/tasks/TASK-406-retained-completion-payload-observation.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require a real spawned-child runtime path to execute and then seal retained completion automatically without destroying control-link usefulness.

### Step 2: Implement spawned-child execution substrate

Add the narrowest honest runtime mechanism for child execution and automatic completion sealing.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- retained completion is automatically sealed from a real spawned-child terminal lifecycle path, not from manual recording or invented inline completion semantics.

## Completion Checklist

- [x] TASK-407 task file created
- [x] spawned-child execution substrate implemented
- [x] automatic retained completion sealing wired to the real child lifecycle path
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Implemented Runtime Slice

`ash-interp` now has a narrow real spawned-child execution substrate keyed by `workflow_type`.

Implemented substrate/API surface:

- `RuntimeState::register_child_workflow(workflow_type, workflow)`
- `RuntimeState::child_workflow(workflow_type)`
- `RuntimeState::spawned_child_init_bindings(init_value, control_link)`
- `Workflow::Spawn` now looks up an optional runtime-owned child workflow body by `workflow_type`
- when found, the runtime launches a real child execution task through `execute_with_bindings_in_state(...)`
- the child entry contract is intentionally narrow: the evaluated spawn `init` value is bound into the child context as `init`
- TASK-408 now enriches the retained terminal record produced by that child lifecycle with `RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>` and the accessor `terminal_result()`, so successful child values and failing child errors survive sealing without changing the write-once/control-tombstone boundary.

Implemented semantics now claimed honestly:

- `kill` and child-side completion recording now compete through one authoritative terminal transition path in `ControlLinkRegistry`, so the first terminal event to seal the retained record wins and later terminal attempts cannot rewrite it;
- spawn returns live control authority only when the runtime actually has a registered child workflow for that `workflow_type`; if no runtime-owned child entry is registered, the returned `Instance` is honest and carries no usable control link;
- if the runtime has a registered child workflow for that `workflow_type`, it executes on a runtime-owned child path after spawn;
- when that child path reaches a terminal runtime outcome/state, the runtime automatically seals the retained completion record through `RuntimeState::record_control_completion(...)`;
- benign completion-vs-kill seal races remain quiet, but unexpected automatic sealing failures now surface instead of being swallowed broadly;
- retained completion sealing remains write-once/stable because the existing `ControlLinkRegistry` seal is preserved;
- this remains intentionally conservative and does not claim full `SPEC-004` `CompletionPayload` parity.

Evidence now covers:

- live supervisor control authority immediately after spawn,
- real child workflow execution keyed by `workflow_type`,
- automatic retained completion sealing from both successful and failing child terminal paths,
- stable write-once sealing after automatic capture.

## Notes

Important constraints:

- Do not fake child completion by immediately terminating spawned children inline.
- Do not collapse retained completion observation back into generic runtime error/outcome state.
- Control semantics in the current TASK-407 realization are cooperative, not preemptive: pause/resume/kill are enforced at workflow-entry / step boundaries rather than by interrupting an in-flight provider call mid-await.
- The current spawned-child runtime path is detached/cooperative rather than backed by a stronger owned join/shutdown lifecycle; that limitation should remain explicit until a deeper substrate exists.
- If the required child-workflow lookup/body association cannot be implemented honestly with current IR/runtime contracts, document that contract gap explicitly before broadening the runtime design.
