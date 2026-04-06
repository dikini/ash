# TASK-408: Richer Retained Completion Payload Contents

## Status: ✅ Complete

## Description

Implement the next narrow runtime-side follow-on after TASK-406 and TASK-407 by enriching the retained completion carrier in `ash-interp` so it preserves more of the accepted `SPEC-004` `CompletionPayload` structure than the current coarse `{ instance_id, outcome_state, kind }` record.

This task should not try to solve all cumulative runtime carrier work. Instead, it should take the next contract-first slice of retained completion fidelity by moving from a coarse terminal classification record toward a richer retained completion payload surface that can preserve at least the terminal result shape and the first directly available completion metadata the runtime can currently carry honestly.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)
- [TASK-407: Spawned-Child Execution Substrate and Completion Sealing](TASK-407-spawned-child-execution-substrate-and-completion-sealing.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ TASK-407 complete for spawned-child execution substrate and automatic sealing

## Requirements

### Functional Requirements

1. Enrich the retained completion carrier so it preserves more than coarse `RuntimeOutcomeState`, moving toward `SPEC-004` `CompletionPayload` structure.
2. Preserve at least one richer retained terminal result field that distinguishes terminal success values from terminal error payloads more directly than the current coarse classification alone.
3. Add the first honest retained metadata fields the runtime can currently preserve without overclaiming broader cumulative carrier closure.
4. Keep the retained payload write-once/stable.
5. Expose the richer retained payload through runtime/control-facing code, not only internal helpers.
6. Add tests demonstrating at least:
   - successful child completion retains terminal result data,
   - failed child completion retains terminal error/result data,
   - the richer retained payload remains stable/write-once,
   - terminal control tombstones remain distinguishable from child-owned retained completion payloads.

### Non-Functional Requirements

1. Do not claim full `SPEC-004` `CompletionPayload` parity unless the runtime genuinely carries the required contents.
2. Do not claim cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
3. Preserve current cooperative control semantics and spawned-child execution behavior from TASK-407.
4. Prefer additive changes over broad rewrites.
5. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- retained completion is real and automatically sealed from a spawned-child lifecycle path;
- but the retained record is still coarse-grained and does not preserve a richer terminal result payload comparable to `SPEC-004` `CompletionPayload.result`;
- obligations, provenance, and effect-summary fidelity remain broader open follow-on work.

### Green

This task is complete when:

- the retained completion carrier preserves a richer terminal payload than coarse `RuntimeOutcomeState` alone;
- tests prove successful and failed child completions retain directly inspectable terminal result data;
- docs/reporting surfaces reflect the richer retained payload slice conservatively without overclaiming full parity.

These conditions are now met by retaining one honest `CompletionPayload.result`-like slice as
`RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>` with
`RetainedCompletionRecord::terminal_result()` for inspection, preserving direct terminal child
values and terminal child errors for completed children while keeping control tombstones as
`result: None`.

## Files

- Modify: `crates/ash-interp` retained-completion/runtime files as needed
- Modify: `docs/plan/tasks/TASK-406-retained-completion-payload-observation.md`
- Modify: `docs/plan/tasks/TASK-407-spawned-child-execution-substrate-and-completion-sealing.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require richer retained terminal payload contents than coarse `RuntimeOutcomeState` alone.

### Step 2: Implement richer retained payload slice

Add the narrowest honest richer retained completion payload shape the runtime can currently support.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can observe a richer retained terminal payload slice than coarse terminal classification alone, while broader parity work stays explicitly open.

## Implemented Runtime Slice

`ash-interp` now retains one richer terminal payload field alongside the existing coarse runtime
classification surface:

- `RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>`
- `RetainedCompletionRecord::terminal_result() -> Option<&ExecResult<Value>>`

This is the narrowest honest slice of `SPEC-004` `CompletionPayload.result` the current runtime can
preserve without inventing unsupported cumulative carrier data.

Implemented semantics now claimed honestly:

- child-owned retained completion records preserve the terminal `Result<Value, ExecError>` that the
  child runtime path actually produced;
- successful child completion retains the direct terminal value;
- failed child completion retains the direct terminal error payload;
- `outcome_state` remains present as the coarse authoritative classification surface from TASK-405;
- write-once sealing still holds because the first retained record continues to win;
- control tombstones remain distinct because `RetainedCompletionKind::ControlTerminated` records keep
  `result: None`.

Still intentionally unresolved:

- `CompletionPayload.obligations` parity;
- `CompletionPayload.provenance` parity;
- full `CompletionPayload.effects` parity beyond the later conservative TASK-409 effect-summary
  slice;
- broader cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- any dedicated completion-wait carrier beyond the current retained record lookup.

## Completion Checklist

- [x] TASK-408 task file created
- [x] richer retained payload fields implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Notes

Important constraints:

- Do not try to solve the full cumulative `Ω` / `π` / `T` / `ε̂` carrier story in this task.
- Do not erase the existing coarse runtime classification surface from TASK-405; extend retained completion fidelity alongside it.
- If obligations/provenance/effect-summary fields cannot be preserved honestly yet, document the exact residual gap rather than faking parity.
