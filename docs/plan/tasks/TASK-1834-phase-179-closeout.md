# TASK-1834: Close out Phase 179 with gates and review

## Description

Run the full Phase 179 verification baseline, reconcile PLAN-INDEX/task statuses, update reference docs and CHANGELOG, and obtain an independent review before marking the phase complete.

## Owner decision gate

D8: What broad verification and review are required before closeout?

## Requirements

- Run the verification baseline from PLAN-179:
  - `cargo fmt --check`
  - `cargo test -p ash-engine`
  - `cargo test -p ash-typeck`
  - `cargo test -p ash-core`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `git diff --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
- Update `docs/plan/PLAN-INDEX.md` Phase 179 status and task statuses.
- Add a `CHANGELOG.md` entry under `[Unreleased]` for Phase 179.
- Add or update reference docs explaining explicit row admission semantics.
- Fix any findings and re-run gates.

## Completion criteria

- [x] All baseline gates pass.
- [x] PLAN-INDEX and task files reflect completed status.
- [x] CHANGELOG entry present in Common Changelog format.
- [x] Reference docs updated.
- [x] Independent review completed or explicitly waived.
- [x] Phase 179 status set to ✅ Complete.

## Depends on

- All other Phase 179 tasks.
