# TASK-1387: Closeout: final audit, status, changelog, review

## Status: 📝 Planned

## Description

Close Phase 137 by running the final size audit, reconciling plan/task/changelog status, running full workspace gates, and obtaining independent Codex approval.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378 through TASK-1386 completed or explicitly reconciled.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Run the Phase 137 size audit and compare against baseline.
2. Update `docs/audit/RUST-FILE-SIZE-AUDIT.md` with final counts, deltas, and remaining exceptions.
3. Update all TASK-1378 through TASK-1387 statuses and checklists.
4. Update `PLAN-137` and `PLAN-INDEX.md` to complete only after gates pass.
5. Update `CHANGELOG.md` with closeout results.
6. Run full workspace verification.
7. Delegate final Codex phase audit and fix/re-review blockers before completion.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Run final size audit

```bash
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-final-size.md
python3 tools/dev/rust_file_size_report.py --json > /tmp/phase137-final-size.json
```

### Step 2: Reconcile docs/status

Patch task files, `PLAN-137`, `PLAN-INDEX.md`, `docs/audit/RUST-FILE-SIZE-AUDIT.md`, and `CHANGELOG.md` with actual evidence.

### Step 3: Run full gates

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo doc --workspace --no-deps
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```

### Step 4: Codex phase audit

Ask Codex to audit all Phase 137 commits for behavior preservation, public API stability, size-budget honesty, and status-surface consistency.


## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo doc --workspace --no-deps
  - git diff --check
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-final-size.md
  - python3 tools/dev/rust_file_size_report.py --fail-on-regression
checklist:
  - [ ] Final size audit and deltas recorded
  - [ ] All Phase 137 task statuses reconciled
  - [ ] PLAN-137 and PLAN-INDEX statuses reconciled
  - [ ] CHANGELOG.md updated
  - [ ] Full workspace tests, clippy, fmt, docs pass
  - [ ] Size guard passes or documents approved exceptions
  - [ ] Codex phase audit reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- Final size audit delta.
- Complete Phase 137 status reconciliation.
- Full workspace gate evidence and Codex approval.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
