# TASK-412: Dedicated Completion-Wait Carrier

## Status: ✅ Complete

## Description

Implement the next narrow runtime-side follow-on after TASK-411 by adding one dedicated completion-wait carrier/API for retained completion observation in `ash-interp`.

This task should stay conservative. It should not attempt to solve full cumulative carrier closure, exact `CompletionPayload` parity, or broader runtime scheduler redesign. Instead, it should add the narrowest honest runtime surface that lets callers await one control target’s first sealed retained terminal observation without busy-polling `RuntimeState::retained_completion()`.

After TASK-406 through TASK-411, retained completion observation is materially stronger: the runtime now preserves coarse terminal classification, direct result payload, conservative effect summary, terminal-visible obligations summary, and conservative provenance summary. But callers still have no dedicated completion-wait surface; they must either poll retained completion lookup or infer terminality indirectly from coarse control-liveness state. The next contract-first slice is therefore to add one authoritative wait API for the existing retained completion carrier.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)
- [TASK-407: Spawned-Child Execution Substrate and Completion Sealing](TASK-407-spawned-child-execution-substrate-and-completion-sealing.md)
- [TASK-408: Richer Retained Completion Payload Contents](TASK-408-richer-retained-completion-payload-contents.md)
- [TASK-409: Retained Completion Effect-Summary Contents](TASK-409-retained-completion-effect-summary-contents.md)
- [TASK-410: Retained Completion Obligations Contents](TASK-410-retained-completion-obligations-contents.md)
- [TASK-411: Retained Completion Provenance Contents](TASK-411-retained-completion-provenance-contents.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ TASK-407 complete for spawned-child execution substrate and automatic sealing
- ✅ TASK-408 complete for retained `CompletionPayload.result`-like fidelity
- ✅ TASK-409 complete for conservative retained `CompletionPayload.effects`-like fidelity
- ✅ TASK-410 complete for honest retained terminal-visible `CompletionPayload.obligations`-like fidelity
- ✅ TASK-411 complete for conservative retained `CompletionPayload.provenance`-like fidelity

## Requirements

### Functional Requirements

1. Add one dedicated runtime/control-facing API that waits for the first sealed retained completion record for a control target.
2. The new wait surface must return the same retained completion carrier already exposed through `RuntimeState::retained_completion()` rather than inventing a second competing payload type.
3. The wait surface must resolve for both child-owned terminal completions and control tombstones.
4. The wait surface must preserve write-once semantics: the first sealed retained record remains authoritative and is what waiters observe.
5. The wait surface must behave honestly for already-sealed targets: if a retained record already exists when the wait begins, it should return immediately with that record.
6. The wait surface must distinguish invalid/unregistered targets from real retained completion observation in a conservative, documented way.
7. Add tests demonstrating at least:
   - waiting returns the child-owned retained record after real spawned-child completion,
   - waiting returns the control tombstone after kill wins,
   - waiting returns immediately for already-sealed records,
   - invalid/unregistered targets do not hang or falsely synthesize completion.

### Non-Functional Requirements

1. Do not claim that this task solves full `CompletionPayload` parity.
2. Do not claim broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
3. Do not replace or weaken the existing `RuntimeState::retained_completion()` lookup API; extend it with a wait surface.
4. Preserve current cooperative control semantics and spawned-child execution behavior from TASK-407 onward.
5. Prefer additive, contract-first changes over broad runtime rewrites.
6. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- retained completion records can be read after sealing via `RuntimeState::retained_completion()`;
- but callers still need polling or indirect control-state observation to wait for completion-style data to appear;
- a dedicated completion-wait carrier/API remains explicitly open after TASK-411.

### Green

This task is complete when:

- runtime/control-facing code can await one retained completion record directly;
- tests prove the wait surface returns the same sealed retained record for both child completions and control tombstones;
- docs/reporting surfaces reflect that dedicated completion waiting has landed while broader parity and cumulative-carrier work remain open.

## Files

- Modify: `crates/ash-interp` retained-completion/runtime files as needed
- Modify: `docs/plan/tasks/TASK-411-retained-completion-provenance-contents.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require one dedicated completion-wait surface to return the sealed retained completion record without polling loops in test code.

### Step 2: Implement completion-wait carrier/API

Add the narrowest honest wait surface the runtime can currently support, reusing the existing retained completion carrier.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can await retained completion observation directly alongside the existing lookup API, while broader parity work remains explicitly open.

## Planned Runtime Slice

`ash-interp` should next expose one dedicated completion wait surface alongside the existing retained completion lookup API:

- `RuntimeState::wait_for_retained_completion(&ControlLink) -> ...`
- or an equivalent conservative control-runtime wait API built on the current `ControlLinkRegistry`/runtime ownership model

Planned semantics should be claimed honestly:

- waiting observes the first sealed retained record already stored by the runtime rather than inventing a new completion payload;
- waiting may resolve to either a child-owned `RetainedCompletionKind::Completed` record or a `RetainedCompletionKind::ControlTerminated` tombstone;
- already-sealed records should return immediately;
- invalid or unregistered targets must remain distinguishable from real retained completion records;
- this task adds observation/waiting ergonomics, not new semantic payload parity.

Still intentionally unresolved after this task:

- exact full `CompletionPayload.provenance` parity;
- exact `CompletionPayload.effects` parity beyond the conservative TASK-409 upper-bound slice;
- exact full `CompletionPayload.obligations` parity beyond the TASK-410 terminal-visible slice;
- broader cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- any stronger scheduler/interleaving/runtime ownership closure beyond the current cooperative substrate.

## Completion Checklist

- [x] TASK-412 task file created
- [x] dedicated completion-wait API implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Notes

Important constraints:

- Do not replace the retained record with a new parallel completion payload abstraction.
- Do not claim that waiting implies full retained-payload parity.
- Prefer a wait surface built on existing runtime-owned control/completion machinery rather than introducing a broad new synchronization subsystem.
- Keep this task scoped to completion waiting; do not fold in exact payload parity or broader cumulative carrier closure claims.
