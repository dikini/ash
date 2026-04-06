# TASK-406: Retained Completion-Payload Observation

## Status: ✅ Complete for scoped goal

## Description

Implement the next runtime-side follow-on step after TASK-405 by introducing a minimal retained completion-payload observation surface for spawned child workflows. The goal is not to solve all cumulative carrier packaging, but to stop treating completion observation as a mostly semantic/documentary contract without a correspondingly explicit retained runtime carrier.

This task should add a conservative runtime-visible completion record/path that makes child terminal completion observable after spawn/control authority handoff, while preserving the existing MCE-004 / MCE-005 / MCE-006 / MCE-007 corpus boundaries.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-405: Authoritative Runtime Outcome/State Classification](TASK-405-authoritative-runtime-outcome-state-classification.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ MCE-007 closeout corpus published
- ✅ Current interpreter/runtime implementation in `crates/ash-interp`

## Requirements

### Functional Requirements

1. Introduce one explicit retained completion-observation carrier/path in the runtime for spawned child workflows.
2. The new observation surface must be queryable/observable by runtime/control-facing code, not just described in docs.
3. Preserve existing behavior unless a narrow contract-first API addition is needed.
4. Add tests demonstrating that:
   - a spawned child can reach terminal completion,
   - a retained completion record remains observable after completion,
   - completion observation remains distinct from generic execution failure.
5. Update docs/planning/reporting surfaces so the corpus reflects that retained completion observation now has a first concrete runtime implementation slice.

### Non-Functional Requirements

1. Be conservative: do not claim this task resolves cumulative `Ω` / `π` / `T` / `ε̂` packaging.
2. Do not claim full MCE-004 `CompletionPayload` parity unless the runtime actually carries the required information.
3. Prefer additive, contract-first changes over broad rewrites.
4. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- the docs explicitly say there is no dedicated retained `CompletionPayload`-style carrier or equally explicit completion-observation path in the runtime;
- spawn/control authority exists, but retained completion observation remains weak/missing on the inspected main runtime path;
- MCE-007 keeps retained completion-payload-style observation as true residual drift.

### Green

This task is complete when:

- one explicit retained completion-observation carrier/path exists in the runtime;
- tests show that completed child state can be observed after completion;
- docs/planning surfaces reflect the new runtime slice without overclaiming broader cumulative-carrier closure.

## Files

- Modify: `crates/ash-interp` runtime/control-related files as needed
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require retained child completion to remain observable after completion.

### Step 2: Implement the minimal retained completion carrier/path

Add the narrowest runtime-visible completion record/path that satisfies the observation contract.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can observe retained child completion through an explicit path, not only by relying on semantic intent.

## Completion Checklist

- [x] TASK-406 task file created
- [x] retained completion carrier/path implemented as an explicit runtime-visible record surface
- [x] exploratory tests/prototype wiring added
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated
- [x] actual spawned-child completion lifecycle hook moved into TASK-407 and now has a concrete runtime-owned spawned-child substrate implementation there

## Implemented Runtime Slice

`ash-interp` now provides a conservative retained completion-observation surface. TASK-406 supplies the explicit retained completion carrier/path itself, and TASK-407 now supplies the real spawned-child completion lifecycle hook that automatically seals that carrier from an honest child runtime path.

Implemented carrier/API surface:

- `RetainedCompletionRecord { instance_id, outcome_state, kind }`
- `RetainedCompletionKind::{Completed, ControlTerminated}`
- `ControlLinkRegistry::record_completion(...)`
- `ControlLinkRegistry::retained_completion(...)`
- `RuntimeState::register_spawned_control_link(...)`
- `RuntimeState::record_control_completion(...)`
- `RuntimeState::retained_completion(...)`

Implemented semantics now claimed honestly:

- retained terminal observations are explicit runtime-visible records keyed by control target;
- retained observations are now sealed/write-once once first recorded;
- `kill` preserves the first retained terminal tombstone rather than rewriting it later;
- `Workflow::Spawn` registers a live control link without eagerly driving the child to a terminal state inline, preserving pause/resume/check-health/kill usefulness after spawn.

Integration evidence now covers:

- real spawn,
- preserved live control authority immediately after spawn,
- explicit retained completion observation through `RuntimeState`,
- distinction between retained completion `ExecutionFailure` and generic runtime/control failure classification.

Still intentionally unresolved in TASK-406 itself:

- full `SPEC-004` `CompletionPayload` parity;
- `Ω` / `π` / `T` / `ε̂` packaging.

Follow-on note:

- [TASK-407](TASK-407-spawned-child-execution-substrate-and-completion-sealing.md) now supplies the missing runtime-owned child execution substrate keyed by `workflow_type` and uses that real child lifecycle path to drive automatic retained completion sealing.
- [TASK-408](TASK-408-richer-retained-completion-payload-contents.md) now enriches that retained record with one honest `CompletionPayload.result`-like slice: `RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>` plus the accessor `RetainedCompletionRecord::terminal_result() -> Option<&ExecResult<Value>>`. Successful child terminal values and failing child terminal errors are therefore directly inspectable, while obligations, provenance, and effects remain explicitly open.

## Notes

Important constraints:

- Do not try to solve full cumulative semantic-carrier packaging in this task.
- Do not collapse completion observation into generic runtime error/outcome state.
- Keep this focused on retained completion observation as a separate runtime follow-on slice.
