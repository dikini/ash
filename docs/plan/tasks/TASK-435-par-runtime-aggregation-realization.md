# TASK-435: `Par` Runtime Aggregation Realization

## Status: ✅ Complete

## Description

Implement the `Par` runtime aggregation follow-on in `ash-interp` against the frozen branch-state and aggregation contract from TASK-434. This task should make the runtime preserve branch-local semantic carriers and combine them according to the accepted helper-backed concurrent aggregation contract, rather than only returning useful concurrent value collation. The goal is not to redesign the scheduler; it is to make the current runtime honestly realize more of the accepted `Par` semantic story.

This is real Rust/runtime work.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- [TASK-434: `Par` Branch-State and Aggregation Contract](TASK-434-par-branch-state-and-aggregation-contract.md)

## Dependencies

- ✅ [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- ✅ [TASK-434: `Par` Branch-State and Aggregation Contract](TASK-434-par-branch-state-and-aggregation-contract.md)

## Requirements

### Functional Requirements

1. Update `ash-interp` so `Workflow::Par` preserves and aggregates branch-local semantic carriers according to TASK-434 rather than only collating terminal values.
2. Ensure the runtime remains interleaving-compatible and helper-backed in semantics, even if concrete branch execution still uses current async/task mechanisms.
3. Add or update tests covering at minimum:
   - all-success branch aggregation,
   - mixed success/failure behavior,
   - blocked/suspended branch interaction where applicable,
   - aggregation of the semantic carrier slices introduced by TASK-433.
4. Preserve current useful concurrency behavior and avoid regressing existing runtime action/control tests.
5. Keep the implementation honest about any still-conservative slices; do not claim stronger exactness than the runtime really carries.
6. Update docs/planning/reporting surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not redesign the runtime into a new scheduler architecture here.
2. Prefer additive wiring over broad rewrites.
3. Keep public/runtime-facing APIs and tests readable.
4. Mark complete only if `Par` runtime behavior is materially closer to the frozen semantic contract and the remaining gaps are stated honestly.

## TDD Evidence

### Red

Before this task:
- the current runtime provides useful bulk async child execution and terminal value collation for `Par`, but not yet the full frozen branch-local semantic-carrier aggregation contract;
- MCE-007 still classifies full helper-backed concurrent cumulative-state aggregation for `Par` as true residual drift.

### Green

This task is complete when:
- `ash-interp` aggregates `Par` branch-local carrier state more honestly under the TASK-434 contract;
- tests demonstrate the runtime behavior against the relevant branch-combination cases;
- docs/reporting surfaces record the new runtime slice conservatively.

## Files

- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/runtime_state.rs` (if branch-local execution/state support needs wiring)
- Modify: `crates/ash-interp/src/lib.rs`
- Modify: `crates/ash-interp` tests as needed
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require `Par` branch-local carrier aggregation under success/failure/blocking combinations.

### Step 2: Implement runtime aggregation

Update `ash-interp` `Par` execution to preserve and combine branch-local carrier state according to TASK-434.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- runtime `Par` aggregation now materially reflects the frozen contract rather than only coarse successful value collation.

## Completion Checklist

- [x] TASK-435 task file created
- [x] runtime `Par` aggregation updated
- [x] branch-local carrier handling implemented
- [x] tests added or updated
- [x] docs/planning surfaces updated
- [x] `CHANGELOG.md` updated

## Completion Notes

TASK-435 is complete as the first honest runtime-side `Par` aggregation realization in `ash-interp`.

`Workflow::Par` no longer relies on one shared execution recorder across all concurrent branches.
Instead, the interpreter now creates branch-local `ExecutionRecorder` instances, executes each
branch against its own recorder/provenance context, snapshots the branch-local execution records,
and then rebuilds the enclosing parent execution record from those branch-local records using the
aggregation helpers in `crates/ash-interp/src/execution_record.rs`.

This landed two material runtime corrections against the frozen TASK-434 contract:

- spawned child execution no longer overwrites `RuntimeState::last_execution_record()` for the
  enclosing top-level/stream execution path; and
- `Par` now aggregates branch-local trace, effect, obligation, and provenance carriers into the
  parent execution record instead of collapsing everything through one shared recorder.

Focused regression coverage now includes:

- `test_spawned_child_does_not_overwrite_top_level_last_execution_record`
- `test_spawned_child_does_not_overwrite_stream_top_level_last_execution_record`
- `test_par_execution_record_aggregates_branch_local_carriers`

Verification for the landed runtime slice was run with:

- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets -- -D warnings`
- `cargo fmt --check`

This closeout remains intentionally conservative. TASK-435 materially improves `Par` runtime
aggregation and makes the enclosing execution record reflect branch-local carrier state more
honestly, but it does not claim closure of every residual `Par` gap named by MCE-007. In
particular, the runtime still does not claim a fully explicit interleaving machine or full closure
of every helper-backed concurrent aggregation latitude beyond the landed branch-local execution-
record slice.

## Dependencies for Next Task

This task outputs:
- a more faithful runtime `Par` realization aligned with the Phase 67 contract work.

Required by:
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Keep `Par` contract-first and helper-backed.
- Do not overspecify scheduler/fairness behavior.
- Record any remaining conservative slices explicitly.
