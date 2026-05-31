# TASK-990: Ashgrove source payload local-state closeout

## Status: ✅ Complete

## Description

Close Phase 129 after TASK-989 implementation by running composed acceptance, independent review, status reconciliation, and broad ashgrove/Rust gates. Promote SPEC-074 only if implementation evidence satisfies A74-1 through A74-8.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §10
- [PLAN-124](../PLAN-124-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §7-§9

## Dependencies

- ✅ TASK-989: Source payload ignore implementation and focused regressions complete.

## Requirements

### Functional Requirements

1. Rerun every focused TASK-989 test, the source-archive non-regression command, and the update-parity regression, then record evidence in `docs/plan/audits/TASK-990-ashgrove-source-payload-local-state-closeout.md`.
2. Run an independent code/spec review and address blocking findings.
3. Prove the reported failure mode is fixed by either a deterministic equivalent regression or the actual local checkout install command under isolated XDG roots.
4. Reconcile SPEC-074, PLAN-124, PLAN-INDEX, task statuses, and CHANGELOG.
5. Keep SPEC-073's historical Implemented MVP status intact while pointing to SPEC-074 as the amendment owner.
6. Run broad formatting, clippy, and test gates appropriate for ashgrove.

### Property Requirements

No proptest is required. Closeout invariant:

```text
All A74 acceptance rows have concrete evidence and SPEC-074 is promoted only after those rows are satisfied.
```

## TDD Steps

### Step 1: Collect focused evidence

Run the focused TASK-989 commands and record command, exit status, and result summary in the TASK-990 audit artifact.

### Step 2: Run composed local-checkout acceptance

Use isolated XDG roots and a live or deterministic source root with ignored `.agents/` state. Prove source install no longer fails due to local-state churn.

### Step 3: Run independent review

Dispatch an independent reviewer with SPEC-074, PLAN-124, TASK-988 audit, and TASK-989 diff. Require review of:

- source-root/source-archive policy separation, including source-shaped archives;
- digest/copy membership sharing;
- fail-closed nonignored mutation behavior;
- test quality, including fake-cargo observation plumbing and update parity;
- docs/status consistency.

### Step 4: Patch findings

Address all blocking findings. Rerun focused tests after patches.

### Step 5: Run broad gates

Run broad gates from the Verification block or repo-standard equivalent scripts.

### Step 6: Reconcile status surfaces

Patch SPEC-074 status only if acceptance rows are satisfied. Patch PLAN-INDEX task statuses, PLAN-124 task table, task files, and CHANGELOG.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --all-targets -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - cargo fmt --all --check
  - python3 -c "from pathlib import Path; audit=Path('docs/plan/audits/TASK-990-ashgrove-source-payload-local-state-closeout.md'); assert audit.exists(), audit; text=audit.read_text(); required=['A74-1','A74-8','independent review','cargo']; missing=[s for s in required if s not in text]; assert not missing, missing; print('TASK-990 closeout artifact verified')"
checklist:
  - [x] A74-1 through A74-8 evidence recorded, including A74-6 implementation evidence and A74-7 update parity.
  - [x] Independent review completed and blockers resolved.
  - [x] SPEC/PLAN/TASK/CHANGELOG statuses reconciled.
  - [x] Broad gates pass; no blockers remain, and SPEC-074 is promoted by the closeout evidence.
```

## Closeout Evidence

- Audit artifact: `docs/plan/audits/TASK-990-ashgrove-source-payload-local-state-closeout.md`.
- Focused regression evidence: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture` exited 0 with 10 tests passed.
- Source-archive non-regression evidence: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove source_archive -- --nocapture` exited 0.
- Broad ashgrove evidence: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove`, `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --all-targets -- --nocapture`, `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` exited 0.
- Status reconciliation: SPEC-074 is Accepted/Implemented, PLAN-124 and PLAN-INDEX Phase 129 are complete, and SPEC-073 remains historical Implemented MVP with SPEC-074 as the source-payload/local-state amendment owner.

## Dependencies for Next Task

None. This is the Phase 129 closeout task.

## Notes

If broad offline cargo gates fail because the local cache is incomplete, do not fabricate success. Record the blocker and run the non-offline equivalent if network access is acceptable.
