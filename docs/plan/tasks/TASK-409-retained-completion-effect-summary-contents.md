# TASK-409: Retained Completion Effect-Summary Contents

## Status: ✅ Complete

## Description

Implement the next narrow runtime-side follow-on after TASK-408 by enriching `ash-interp` retained completion records with one honest `CompletionPayload.effects`-like summary slice.

This task should stay conservative. It should not attempt to transport the full execution trace `T`, nor should it claim broader cumulative carrier closure. Instead, it should add the narrowest retained effect-summary contents that the current runtime can preserve honestly for terminal spawned-child completion observations.

Per `SPEC-004`, the retained completion payload’s `effects` field is a summary value, not the full execution trace. The next valid slice is therefore to preserve a terminal effect summary comparable to:

- `effects.terminal_upper_bound`
- `effects.reached_upper_bound`

without pretending that full trace transport or all cumulative runtime carriers are now solved.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-406: Retained Completion-Payload Observation](TASK-406-retained-completion-payload-observation.md)
- [TASK-407: Spawned-Child Execution Substrate and Completion Sealing](TASK-407-spawned-child-execution-substrate-and-completion-sealing.md)
- [TASK-408: Richer Retained Completion Payload Contents](TASK-408-richer-retained-completion-payload-contents.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ TASK-407 complete for spawned-child execution substrate and automatic sealing
- ✅ TASK-408 complete for retained `CompletionPayload.result`-like fidelity

## Requirements

### Functional Requirements

1. Enrich retained completion records with one explicit effect-summary payload slice that goes beyond coarse `RuntimeOutcomeState` and beyond retained result payload alone.
2. Preserve at least:
   - a terminal effect summary field comparable to `effects.terminal_upper_bound`
   - a conservative reached-effect summary comparable to `effects.reached_upper_bound`
3. Keep the summary honest: if the current runtime can only support a coarse or conservative reached-effect summary, document that explicitly rather than faking stronger precision.
4. Preserve write-once/stable sealing.
5. Keep control tombstones distinguishable from child-owned retained completion payloads.
6. Expose the effect summary through runtime/control-facing retained completion observation.
7. Add tests demonstrating at least:
   - successful retained completion preserves effect-summary contents,
   - failing retained completion preserves effect-summary contents,
   - control tombstones remain distinguishable,
   - write-once sealing still holds with effect-summary contents present.

### Non-Functional Requirements

1. Do not claim full `SPEC-004` `CompletionPayload` parity unless the runtime genuinely carries the required contents.
2. Do not claim full trace `T` transport over control links.
3. Do not claim broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
4. Preserve current cooperative control semantics and spawned-child execution behavior from TASK-407.
5. Prefer additive, contract-first changes over broad rewrites.
6. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- retained completion records preserve coarse terminal classification and direct terminal result payload;
- but they still do not preserve a retained effect-summary slice comparable to `CompletionPayload.effects`;
- `CompletionPayload.effects` parity remains explicitly open after TASK-408.

### Green

This task is complete when:

- retained completion records preserve one honest effect-summary slice for terminal child completion;
- tests prove successful and failing retained completions preserve inspectable effect-summary contents;
- docs/reporting surfaces reflect the new retained effect-summary slice conservatively without overclaiming full parity.

These conditions are now met by retaining one conservative `CompletionPayload.effects`-like slice as
`RetainedCompletionRecord.effects: Option<ConservativeRetainedEffectSummary>` with:

- `ConservativeRetainedEffectSummary.terminal: Effect`
- `ConservativeRetainedEffectSummary.reached: BTreeSet<Effect>`

exposed through `RetainedCompletionRecord::conservative_effect_summary()`.

## Files

- Modify: `crates/ash-interp` retained-completion/runtime files as needed
- Modify: `docs/plan/tasks/TASK-408-richer-retained-completion-payload-contents.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require retained effect-summary contents to remain observable after terminal child completion.

### Step 2: Implement retained effect-summary slice

Add the narrowest honest retained effect-summary representation the runtime can currently support.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can observe retained effect-summary contents alongside the existing retained result payload and coarse terminal classification, while broader parity work remains explicitly open.

## Completion Checklist

- [x] TASK-409 task file created
- [x] retained effect-summary fields implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Implemented Runtime Slice

`ash-interp` now retains one conservative effect-summary slice alongside the existing coarse runtime
classification surface and the TASK-408 direct result slice:

- `RetainedCompletionRecord.effects: Option<ConservativeRetainedEffectSummary>`
- `RetainedCompletionRecord::conservative_effect_summary() -> Option<&ConservativeRetainedEffectSummary>`
- `ConservativeRetainedEffectSummary::terminal_upper_bound() -> Effect`
- `ConservativeRetainedEffectSummary::reached_upper_bound() -> &BTreeSet<Effect>`

Implemented semantics now claimed honestly:

- child-owned retained completion records preserve conservative effect upper bounds rather than exact retained `EffectTrace` contents;
- `effects.terminal_upper_bound` is a conservative upper bound derived from workflow-visible effect layers, not a claim of exact big-step terminal `eff` transport;
- `effects.reached_upper_bound` is also conservative: it over-approximates workflow-visible effect layers and may include untaken higher-effect paths rather than pretending exact trace transport;
- control tombstones remain distinct because `RetainedCompletionKind::ControlTerminated` records keep
  both `result: None` and `effects: None`;
- write-once sealing still holds because the first retained record continues to win.

Still intentionally unresolved:

- exact `CompletionPayload.effects` parity;
- exact `CompletionPayload.obligations` parity beyond the later conservative TASK-410 retained obligations slice;
- `CompletionPayload.provenance` parity;
- broader cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- any dedicated completion-wait carrier beyond the current retained record lookup.

## Notes

Important constraints:

- Do not transport or claim the full execution trace `T` through the retained completion record.
- Do not erase the existing retained `result` slice from TASK-408; extend retained completion fidelity alongside it.
- If the current runtime can only preserve a conservative effect-summary approximation, make that limitation explicit in code and docs.
