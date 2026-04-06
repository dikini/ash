# TASK-410: Retained Completion Obligations Contents

## Status: ✅ Complete

## Description

Implement the next narrow runtime-side follow-on after TASK-409 by enriching `ash-interp` retained completion records with one honest `CompletionPayload.obligations`-like slice.

This task should stay conservative. It should not attempt to solve the full cumulative runtime carrier story. Instead, it should add the narrowest retained obligations payload the current runtime can preserve honestly for terminal spawned-child completion observations.

Per `SPEC-004`, retained completion payloads should preserve the child’s terminal obligation state `Ω'`. The current runtime already has two obligation-related carriers:

- workflow-local obligations in `Context`
- role obligations in `RoleContext`

But retained completion records do not yet preserve any obligation-state snapshot. TASK-410 should add the first honest retained obligations summary based on the currently available runtime state.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-019: Role Runtime Semantics](../../spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md)
- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-408: Richer Retained Completion Payload Contents](TASK-408-richer-retained-completion-payload-contents.md)
- [TASK-409: Retained Completion Effect-Summary Contents](TASK-409-retained-completion-effect-summary-contents.md)

## Dependencies

- ✅ TASK-405 complete
- ✅ TASK-406 complete for scoped retained-completion carrier goal
- ✅ TASK-407 complete for spawned-child execution substrate and automatic sealing
- ✅ TASK-408 complete for retained `CompletionPayload.result`-like fidelity
- ✅ TASK-409 complete for conservative retained `CompletionPayload.effects`-like fidelity

## Requirements

### Functional Requirements

1. Enrich retained completion records with one explicit obligations payload slice that goes beyond coarse runtime classification, retained result payload, and retained effect summary alone.
2. Preserve the narrowest honest retained obligation-state snapshot the current runtime can carry today.
3. If the current runtime only supports a split or partial obligations summary (for example local-vs-role or pending-vs-discharged subsets), expose that honestly rather than claiming exact full `Ω'` parity.
4. Preserve write-once/stable sealing.
5. Keep control tombstones distinguishable from child-owned retained completion payloads.
6. Expose the retained obligations slice through runtime/control-facing retained completion observation.
7. Add tests demonstrating at least:
   - successful retained completion preserves obligations contents,
   - failing retained completion preserves obligations contents,
   - control tombstones remain distinguishable,
   - write-once sealing still holds with obligations contents present.

### Non-Functional Requirements

1. Do not claim full `SPEC-004` `CompletionPayload` parity unless the runtime genuinely carries the required contents.
2. Do not claim broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.
3. Preserve current cooperative control semantics and spawned-child execution behavior from TASK-407.
4. Prefer additive, contract-first changes over broad rewrites.
5. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- retained completion records preserve coarse terminal classification, retained result payload, and conservative retained effect-summary contents;
- but they still do not preserve a retained obligations slice comparable to `CompletionPayload.obligations`;
- `CompletionPayload.obligations` parity remains explicitly open after TASK-409.

### Green

This task is complete when:

- retained completion records preserve one honest obligations slice for terminal child completion;
- tests prove successful and failing retained completions preserve inspectable obligations contents;
- docs/reporting surfaces reflect the new retained obligations slice conservatively without overclaiming full parity.

## Files

- Modify: `crates/ash-interp` retained-completion/runtime files as needed
- Modify: `docs/plan/tasks/TASK-409-retained-completion-effect-summary-contents.md`
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require retained obligations contents to remain observable after terminal child completion.

### Step 2: Implement retained obligations slice

Add the narrowest honest retained obligations representation the runtime can currently support.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime/control-facing code can observe retained obligations contents alongside the existing retained result payload, conservative retained effect summary, and coarse terminal classification, while broader parity work remains explicitly open.

## Completion Checklist

- [x] TASK-410 task file created
- [x] retained obligations fields implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Implemented Runtime Slice

`ash-interp` now retains one honest obligations slice alongside the existing coarse runtime
classification surface, the TASK-408 direct result slice, and the TASK-409 conservative effect
summary slice:

- `RetainedCompletionRecord.obligations: Option<ConservativeRetainedObligationsSummary>`
- `RetainedCompletionRecord::conservative_obligations_summary() -> Option<&ConservativeRetainedObligationsSummary>`
- `ConservativeRetainedObligationsSummary::local_pending_visible_at_terminal() -> &BTreeSet<Name>`
- `ConservativeRetainedObligationsSummary::active_role_visible_at_terminal() -> Option<&str>`
- `ConservativeRetainedObligationsSummary::role_pending_visible_at_terminal() -> &BTreeSet<Name>`
- `ConservativeRetainedObligationsSummary::role_discharged_visible_at_terminal() -> &BTreeSet<Name>`

Implemented semantics now claimed honestly:

- child-owned retained completion records preserve only the terminal-visible obligation state the current runtime can actually snapshot from the observed child execution context;
- the retained local-obligation slice reflects local pending obligations visible in that terminal context frame, not a claim of exact cumulative `Ω'` transport across hidden/earlier frames;
- the retained role-obligation slice reflects terminal-visible `RoleContext` state (active role name, pending role obligations, discharged role obligations) when such a role context is present;
- control tombstones remain distinct because `RetainedCompletionKind::ControlTerminated` records keep `result: None`, `effects: None`, and `obligations: None`;
- write-once sealing still holds because the first retained record continues to win.

Still intentionally unresolved:

- exact full `CompletionPayload.obligations` parity;
- `CompletionPayload.provenance` parity;
- exact `CompletionPayload.effects` parity beyond the conservative TASK-409 upper-bound slice;
- broader cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- any dedicated completion-wait carrier beyond the current retained record lookup.

## Notes

Important constraints:

- Do not claim exact full `Ω'` parity unless the runtime genuinely preserves it.
- Do not erase the existing retained `result` or retained conservative effect-summary slices from TASK-408/TASK-409.
- If the current runtime can only preserve a split or conservative obligations snapshot, make that limitation explicit in code and docs.
