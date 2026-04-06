# TASK-411: Retained Completion Provenance Contents

## Status: ✅ Complete

## Description

Implement the next narrow runtime-side follow-on after TASK-410 by enriching `ash-interp` retained completion records with one honest `CompletionPayload.provenance`-like slice.

This task should stay conservative. It should not attempt to solve full cumulative provenance transport, full trace `T` transport, or the broader cumulative runtime carrier story. Instead, it should add the narrowest retained provenance payload the current runtime can preserve honestly for terminal spawned-child completion observations.

Per `SPEC-004`, retained completion payloads should preserve the child’s terminal provenance state `π'`. After TASK-408 through TASK-410, retained completion records now preserve:

- direct terminal `result` payload,
- conservative retained `effects` summary, and
- honest-but-terminal-visible retained `obligations` summary,

but they still preserve no explicit provenance slice at all. The next contract-first slice is therefore to add one retained provenance summary that reflects runtime-owned child identity and spawn lineage only to the extent the runtime can actually support it.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-408: Richer Retained Completion Payload Contents](TASK-408-richer-retained-completion-payload-contents.md)
- [TASK-409: Retained Completion Effect-Summary Contents](TASK-409-retained-completion-effect-summary-contents.md)
- [TASK-410: Retained Completion Obligations Contents](TASK-410-retained-completion-obligations-contents.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ TASK-407 complete for spawned-child execution substrate and automatic sealing
- ✅ TASK-408 complete for retained `CompletionPayload.result`-like fidelity
- ✅ TASK-409 complete for conservative retained `CompletionPayload.effects`-like fidelity
- ✅ TASK-410 complete for honest retained terminal-visible `CompletionPayload.obligations`-like fidelity

## Requirements

### Functional Requirements

1. Enrich retained completion records with one explicit provenance payload slice that goes beyond coarse runtime classification, retained result payload, retained effect summary, and retained obligations summary alone.
2. Preserve the narrowest honest retained provenance snapshot the current runtime can carry today for terminal spawned-child completion.
3. That retained provenance slice should include at least runtime-owned child identity, and may include immediate parent/lineage information only when the runtime explicitly captures that ancestry at spawn/terminal observation time.
4. If the current runtime can only support child-identity-only provenance summary, or only a partial spawn-lineage summary, expose that honestly rather than claiming exact full `π'` parity.
5. Preserve write-once/stable sealing.
6. Keep control tombstones distinguishable from child-owned retained completion payloads.
7. Expose the retained provenance slice through runtime/control-facing retained completion observation.
8. Add tests demonstrating at least:
   - successful retained completion preserves provenance contents,
   - failing retained completion preserves provenance contents,
   - control tombstones remain distinguishable,
   - write-once sealing still holds with provenance contents present.

### Non-Functional Requirements

1. Do not claim full `SPEC-004` `CompletionPayload` parity unless the runtime genuinely carries the required contents.
2. Do not claim exact cumulative `π'` transport unless the runtime genuinely preserves it.
3. Do not claim full trace `T` transport or trace/provenance closure through the retained completion record.
4. Do not claim broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
5. Preserve current cooperative control semantics and spawned-child execution behavior from TASK-407.
6. Prefer additive, contract-first changes over broad rewrites.
7. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- retained completion records preserve coarse terminal classification, direct terminal result payload, conservative retained effect-summary contents, and honest terminal-visible obligations contents;
- but they still do not preserve a retained provenance slice comparable to `CompletionPayload.provenance`;
- `CompletionPayload.provenance` parity remains explicitly open after TASK-410.

### Green

This task is complete when:

- retained completion records preserve one honest provenance slice for terminal child completion;
- tests prove successful and failing retained completions preserve inspectable provenance contents;
- docs/reporting surfaces reflect the new retained provenance slice conservatively without overclaiming full parity.

## Files

- Modify: `crates/ash-interp` retained-completion/runtime files as needed
- Modify: `docs/plan/tasks/TASK-410-retained-completion-obligations-contents.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require retained provenance contents to remain observable after terminal child completion.

### Step 2: Implement retained provenance slice

Add the narrowest honest retained provenance representation the runtime can currently support.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can observe retained provenance contents alongside the existing retained result payload, conservative retained effect summary, honest retained obligations summary, and coarse terminal classification, while broader parity work remains explicitly open.

## Planned Runtime Slice

`ash-interp` should next retain one honest provenance slice alongside the existing coarse runtime
classification surface, the TASK-408 direct result slice, the TASK-409 conservative effect
summary slice, and the TASK-410 terminal-visible obligations slice:

- `RetainedCompletionRecord.provenance: Option<ConservativeRetainedProvenanceSummary>`
- `RetainedCompletionRecord::conservative_provenance_summary() -> Option<&ConservativeRetainedProvenanceSummary>`
- `ConservativeRetainedProvenanceSummary::workflow_id() -> WorkflowId`
- optional ancestry accessors such as `parent_workflow_id()` / `lineage()` only if the runtime explicitly records that ancestry honestly

Planned semantics should be claimed honestly:

- child-owned retained completion records preserve only the provenance identity/ancestry state the runtime can actually snapshot from the spawned-child lifecycle path;
- if the runtime can only preserve child identity, or only runtime-owned spawn ancestry rather than exact terminal `π'`, the retained slice must say so explicitly in naming and docs;
- control tombstones remain distinct because `RetainedCompletionKind::ControlTerminated` records keep `result: None`, `effects: None`, `obligations: None`, and `provenance: None`;
- write-once sealing still holds because the first retained record continues to win.

Still intentionally unresolved after this task:

- exact full `CompletionPayload.provenance` parity if runtime execution still lacks stronger cumulative provenance carriage;
- exact `CompletionPayload.effects` parity beyond the conservative TASK-409 upper-bound slice;
- exact full `CompletionPayload.obligations` parity beyond the TASK-410 terminal-visible slice;
- broader cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- any dedicated completion-wait carrier beyond the current retained record lookup.

## Completion Checklist

- [x] TASK-411 task file created
- [x] retained provenance fields implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Notes

Important constraints:

- Do not claim exact full `π'` parity unless the runtime genuinely preserves it.
- Do not erase the existing retained `result`, retained conservative effect-summary, or retained terminal-visible obligations slices from TASK-408/TASK-409/TASK-410.
- If the current runtime can only preserve a spawn-lineage summary rather than exact terminal provenance state, make that limitation explicit in code and docs.
- Keep this task scoped to retained completion provenance contents; do not fold in dedicated completion waiting or broader cumulative carrier closure claims.
