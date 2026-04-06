# TASK-405: Authoritative Runtime Outcome/State Classification

## Status: ✅ Complete

## Description

Start the first runtime-side follow-on step after MCE-007 closeout by introducing one authoritative runtime outcome/state classification for interpreter execution. The immediate goal is to stop scattering blocked/suspended/terminated/general-failure distinctions across ad hoc `ExecError` variants and control-link state alone. This task should define and implement a conservative runtime classification surface that makes the blocked / terminal / invalid distinction explicit without overclaiming broader closure of cumulative carriers, retained completion payloads, or `Par` aggregation.

This is real Rust/runtime work. It must preserve the accepted MCE-005 / MCE-006 / MCE-007 corpus boundaries while improving the concrete runtime classification story that those docs identify as the first true residual drift item.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [MCE-008: Runtime Cleanup](../../ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md)
- [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)
- [TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)

## Dependencies

- ✅ MCE-007 closeout published via TASK-397 through TASK-400
- ✅ MCE-006 frozen runtime correspondence packet via TASK-401 through TASK-404
- ✅ Current interpreter/runtime implementation in `crates/ash-interp`

## Requirements

### Functional Requirements

1. Introduce one explicit runtime classification type for the interpreter that can distinguish at minimum:
   - terminal success,
   - blocked / suspended,
   - invalid / terminally unusable,
   - execution failure.
2. The new classification must be wired to current interpreter/runtime surfaces in a way that is actually usable by code and tests, not left as a dead type.
3. Preserve the existing external behavior unless a narrower, clearly justified API improvement is needed.
4. Use the new classification to reduce ambiguity around currently mixed cases such as:
   - `YieldSuspended`,
   - control-link termination / invalidation,
   - generic execution failures,
   - blocked receive / wait-style behavior where observable runtime state currently lacks one authoritative class.
5. Add or update tests demonstrating that the runtime classification distinguishes blocked/suspended, terminally invalid, and generic execution-failure cases.
6. Update relevant docs/planning/reporting surfaces so the corpus reflects the first concrete runtime-side follow-on step after MCE-007 closeout.

### Non-Functional Requirements

1. Be conservative: do not claim this task resolves cumulative carrier packaging, retained completion payload observation, or helper-backed `Par` aggregation.
2. Prefer additive, contract-first changes over broad rewrites.
3. Follow project Rust standards and keep public API/docs clear.
4. Update `CHANGELOG.md`.

## TDD Evidence

### Red

Before this task:

- the interpreter exposes multiple runtime conditions through a mix of `ExecError` variants and `ControlLinkRegistry::LinkState`;
- blocked / suspended / invalid distinctions are real in the docs but not unified under one authoritative runtime classification surface;
- MCE-007 explicitly leaves “one authoritative blocked / terminal / invalid runtime class” as true residual drift.

### Green

This task is complete when:

- one authoritative runtime classification type exists and is used by the interpreter/runtime code;
- tests cover at least blocked/suspended, invalid/terminated, and generic execution-failure cases;
- the docs/planning corpus reflects that this first runtime follow-on task is in place;
- no broader residual-drift items are falsely marked resolved.

Status now: satisfied. `ash-interp` exposes one public `RuntimeOutcomeState` classification, wires current `ExecError`, `ControlLinkError`, `LinkState`, and `RuntimeState` control-link visibility into it, and keeps all broader cumulative-carrier / completion-retention / `Par` aggregation residuals explicitly open.

## Files

- Modify: `crates/ash-interp/src/error.rs`
- Modify: `crates/ash-interp/src/lib.rs`
- Modify: `crates/ash-interp/src/control_link.rs`
- Modify: affected runtime/interpreter files as needed
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require one runtime classification surface to distinguish:
- suspended/blocked states,
- terminal invalidation,
- generic execution failure.

### Step 2: Implement the classification type and wiring

Add the new runtime classification and connect it to the current runtime/interpreter surfaces.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- a reader and caller can observe one authoritative runtime classification surface for the first blocked/terminal/invalid follow-on slice.

## Completion Checklist

- [x] TASK-405 task file created
- [x] runtime classification type implemented
- [x] interpreter/runtime wiring updated
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] CHANGELOG updated

## Notes

Important constraints:

- Do not try to solve cumulative `Ω` / `π` / `T` / `ε̂` packaging in this task.
- Do not fold completion-payload retention into this task unless only minimal classification hooks are needed.
- Do not overstate blocked receive as already fully closed if the runtime still lacks deeper carrier work.
- Prefer introducing an authoritative classification surface first; later tasks can extend what evidence feeds it.
