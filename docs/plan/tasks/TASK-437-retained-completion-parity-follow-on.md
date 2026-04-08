# TASK-437: Retained-Completion Parity Follow-On

## Status: ✅ Complete

## Description

Implement the next honest retained-completion parity slice after TASK-412 under the frozen parity contract from TASK-436. This task should add one bounded, contract-justified fidelity improvement to retained completion observation in `ash-interp` without pretending to solve the whole execution-record or full `CompletionPayload` story.

This is real Rust/runtime work.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)
- [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- [TASK-436: Completion-Payload Parity Contract](TASK-436-completion-payload-parity-contract.md)

## Dependencies

- ✅ [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- ✅ [TASK-436: Completion-Payload Parity Contract](TASK-436-completion-payload-parity-contract.md)

## Requirements

### Functional Requirements

1. Implement one contract-justified retained-completion fidelity improvement after TASK-436 freezes the target boundary.
2. The selected improvement must be stated explicitly in the task execution notes and must align with the parity categories from TASK-436 (for example: exact transport for one remaining dimension, stronger retained terminal trace slice, fuller obligations fidelity, or another frozen next slice).
3. Preserve compatibility with the existing retained-completion carrier and wait API surfaces.
4. Add or update tests demonstrating the new retained-completion fidelity slice and showing that current conservative distinctions remain honest.
5. Update docs/planning/reporting surfaces so the corpus reflects the added retained-completion parity slice without overclaiming full parity.
6. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not broaden this task into full execution-record closure.
2. Prefer one bounded fidelity slice over multiple loosely justified additions.
3. Preserve write-once retained record semantics.
4. Mark complete only if the new slice is both implemented and described honestly against TASK-436.

## TDD Evidence

### Red

Before this task:
- retained completion remains intentionally partial even after TASK-412;
- the next parity slice is not yet implemented under one frozen contract;
- later consumers still lack at least one fidelity improvement identified by Phase 67.

### Green

This task is complete when:
- one explicit retained-completion parity slice has been added under TASK-436;
- tests demonstrate the new fidelity improvement;
- docs/reporting surfaces record the improvement and the remaining open parity gaps honestly.

## Files

- Modify: `crates/ash-interp` retained-completion/control/runtime files as needed
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests for the next retained-completion fidelity slice selected under TASK-436.

### Step 2: Implement the bounded parity improvement

Add the narrowest honest retained-completion fidelity slice justified by the frozen contract.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- retained completion now preserves one additional contract-justified fidelity slice without pretending full parity.

## Completion Checklist

- [x] TASK-437 task file created
- [x] one retained-completion parity slice implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] `CHANGELOG.md` updated

## Completion Notes

TASK-437 is complete as one bounded retained-completion parity follow-on slice.

The selected slice is exact `CompletionPayload.effects` parity for child-owned retained
completions. `ash-interp` now derives retained effect contents from the authoritative sealed child
execution record rather than from workflow-form conservative upper bounds. This means retained child
completions now preserve exact terminal and reached effect summaries for the `effects` dimension,
while control tombstones remain effect-payload-free (`effects: None`).

This task remains intentionally narrow:

- retained `result` was already exact before this task and remains so;
- retained `obligations` remain terminal-visible subset only;
- retained `provenance` remains conservative;
- retained completion still does not transport trace `T` and does not claim full execution-record
  closure.

Focused and full-crate verification for the landed slice was run with:

- `cargo test -p ash-interp conservative_effect_summary_can_overapproximate_untaken_higher_effect_paths --test runtime_boundary_visibility -- --exact`
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets -- -D warnings`
- `cargo fmt --check`

## Dependencies for Next Task

This task outputs:
- one stronger retained-completion fidelity slice under the Phase 67 parity contract.

Required by:
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Keep this task narrow even if multiple parity gaps remain attractive.
- Record exactly which retained dimension was improved.
- Preserve the distinction between retained observation and semantic full-state carriage.
