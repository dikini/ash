# TASK-433: `ash-interp` Execution-Record Substrate

## Status: 📝 Planned

## Description

Implement the first authoritative execution-record substrate in `ash-interp` so cumulative semantic carriers are no longer only partially reconstructed from scattered runtime state. This task should introduce the narrowest honest runtime structure and wiring needed to carry the semantic dimensions frozen by TASK-432 — obligations `Ω`, provenance `π`, cumulative trace `T`, cumulative effect summary `ε̂`, and terminal outcome classification/projection — through real interpreter execution paths.

This is real Rust/runtime work. It must remain conservative: the goal is to establish the first authoritative execution-record substrate, not to solve every remaining runtime-alignment problem at once.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-405: Authoritative Runtime Outcome/State Classification](TASK-405-authoritative-runtime-outcome-state-classification.md)
- [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Dependencies

- ✅ [TASK-412: Dedicated Completion-Wait Carrier](TASK-412-dedicated-completion-wait-carrier.md)
- 📝 [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Requirements

### Functional Requirements

1. Introduce the first explicit execution-record substrate in `ash-interp` that can carry, at minimum, the semantic dimensions frozen by TASK-432.
2. Thread that substrate through real interpreter execution paths rather than leaving it as a dead or documentation-only type.
3. Preserve compatibility with the existing runtime outcome/state classification and retained-completion work from TASK-405 through TASK-412.
4. Make terminal projection from the runtime substrate to the semantic outcome contract direct and testable.
5. Add or update tests demonstrating at minimum:
   - terminal success projects from the execution record correctly,
   - terminal failure/rejection projects correctly,
   - the substrate carries cumulative state rather than only terminal payload,
   - existing runtime behavior is not silently regressed.
6. Update relevant docs/planning/reporting surfaces so Phase 67 reflects the first runtime carrier-alignment implementation slice.
7. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Be conservative: do not claim this task resolves full `Par` aggregation alignment or full retained-completion parity.
2. Prefer additive, contract-first changes over broad interpreter rewrites.
3. Keep public/runtime-facing APIs clear and documented.
4. Preserve current tests and observable behavior unless a narrower contract improvement is required.

## TDD Evidence

### Red

Before this task:
- the runtime correspondence corpus still classifies authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging as true residual drift;
- `ash-interp` exposes useful slices of runtime state, but not one authoritative execution-record substrate carrying the semantic dimensions the specs talk about;
- terminal observation remains broader and weaker than the contract Phase 67 intends to freeze.

### Green

This task is complete when:
- `ash-interp` exposes and uses one authoritative execution-record substrate;
- terminal projection is explicit and testable;
- cumulative semantic dimensions are carried more honestly than before;
- docs/reporting surfaces record the new runtime slice without overclaiming later follow-on work.

## Files

- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/runtime_state.rs`
- Modify: `crates/ash-interp/src/lib.rs`
- Modify: `crates/ash-interp/src/error.rs`
- Modify: `crates/ash-interp` tests as needed
- Modify: `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing tests

Add tests that require execution-time carrier packaging and direct terminal projection from one runtime-owned substrate.

### Step 2: Implement execution-record substrate

Add the narrowest honest runtime substrate and thread it through interpreter execution.

### Step 3: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- `ash-interp` now has one authoritative execution-record substrate that later runtime/conformance work can build on directly.

## Completion Checklist

- [ ] TASK-433 task file created
- [ ] execution-record substrate implemented
- [ ] interpreter wiring updated
- [ ] terminal projection made explicit and testable
- [ ] tests added or updated
- [ ] docs/planning surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- the first authoritative `ash-interp` execution-record substrate for cumulative semantic-carrier alignment.

Required by:
- TASK-435: `Par` Runtime Aggregation Realization
- TASK-437: Retained-Completion Parity Follow-On
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Do not confuse "first authoritative substrate" with "full runtime closure."
- Keep exact-vs-conservative carrier claims aligned with TASK-432.
- Preserve the distinction between terminal execution record and retained completion record.
