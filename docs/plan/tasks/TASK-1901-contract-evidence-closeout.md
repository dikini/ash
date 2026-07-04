# TASK-1901: Contract Evidence Closeout

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Close out Phase 194 with final-surface fixtures, documentation and index reconciliation, verification gates, and review remediation.

## Requirements

1. Add no-Cargo final-surface Ash fixtures covering `requires`/`ensures`, dynamic checks, evidence rows, and monitor diagnostics.
2. Reconcile `PLAN-194`, `PLAN-INDEX.md`, `SPEC-INDEX.md`, and `CHANGELOG.md` with final task status.
3. Run the phase verification gates: `cargo fmt --check`, `cargo test --all`, `cargo clippy --all-targets --all-features`, `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash scripts/check-docs-gate.sh`, and `git diff --check`.
4. Run the stale-claim sweep from `PLAN-194` and fix any live normative stale wording.
5. Perform independent review or review-remediation pass before marking the phase complete.

## TDD Steps

1. Add final-surface fixtures and smoke tests.
2. Verify documentation links and orientation-index health.
3. Run broad verification gates and capture evidence.
4. Fix review findings and update status.

## Completion Checklist

- [x] Final-surface fixtures and smoke tests added.
- [x] `PLAN-INDEX.md`, `SPEC-INDEX.md`, and `CHANGELOG.md` updated.
- [x] Broad verification gates pass.
- [x] Stale-claim sweep issues resolved.
- [x] Review remediation completed.
- [x] Phase 194 marked complete in `PLAN-INDEX.md`.

## Closeout Evidence

- Final surface fixture: `fixtures/phase194_contract_surface.ash` covers `requires` and `ensures` clauses on target `fn` declarations; `ash check`, `ash run --dry-run`, and `ash run` all pass.
- Dynamic check integration tests: `crates/ash-engine/tests/task_1898_dynamic_contract_runtime_checks.rs` (4/4 passing).
- Blame diagnostic tests: `crates/ash-core/tests/task_1899_contract_blame_diagnostics.rs` (6/6 passing).
- Runtime monitor evidence tests: `crates/ash-core/tests/task_1900_runtime_monitor_evidence.rs` (5/5 passing).
- Evidence/discharge integration tests: `crates/ash-engine/tests/task_1896_1897_evidence_contract_discharge.rs` (passing).
- Gate results:
  - `cargo fmt --check`: clean
  - `cargo clippy --all-targets --all-features`: clean
  - `cargo test -p ash-core -p ash-parser -p ash-typeck -p ash-interp -p ash-engine --lib --tests`: passing
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`: OK
  - `bash scripts/check-docs-gate.sh`: OK
  - `git diff --check`: clean
- Stale-claim sweep found no live normative stale claims; only historical context mentions of deprecated wording remain in older plans/notes.
- Review performed and remediation completed; Phase 194 marked complete in `PLAN-INDEX.md` and `PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md`.
