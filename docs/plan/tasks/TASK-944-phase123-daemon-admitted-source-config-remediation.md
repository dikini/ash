# TASK-944: Phase 123 Daemon Admitted-Source and Config Remediation

Status: ✅ Complete
Phase: Phase 123 / PLAN-119 post-merge remediation
Specs: SPEC-070

## Description

Post-merge review found two remaining daemon honesty issues after TASK-943:

1. `DaemonState::execute_instance` checks the live workflow source hash, then later
   executes through `Engine::parse_file(path)`, allowing a second filesystem read
   after the admitted-source drift check.
2. Daemon `start` accepts and records non-default `config_id` values even though
   daemon definition, artifact, and cache identity are still built under the
   daemon default config profile.

This task remediates those issues without adding profile-specific daemon artifact
support. Until that support exists, the daemon must reject non-default start
config IDs honestly.

## Requirements

1. Add focused daemon regressions before implementation where practical.
2. Execute daemon `start-execute` from the exact source bytes validated by the
   admitted-source drift check or a stored admitted source snapshot; do not
   re-read the workflow source file for execution after the drift check.
3. Preserve failed-reload and admitted-artifact identity behavior.
4. Reject non-default daemon start `config_id` values before recording an
   instance, with a clear diagnostic.
5. Preserve default `config_id = "default"` behavior and daemon args/profile
   recording.
6. Reconcile PLAN-119, PLAN-INDEX, TASK-941 audit evidence, SPEC-070 caveats,
   and CHANGELOG.

## TDD Steps

1. RED: Add daemon control-plane tests proving non-default `config_id` requests
   fail without recording an instance, while default config starts still record
   args/profile.
2. RED: Add a daemon start-execute regression proving source mutation after the
   admitted-source validation window cannot change the bytes executed.
3. GREEN: Add an engine path-context parser for already-read ordinary workflow
   source and route daemon execution through the source bytes already hashed.
4. GREEN: Reject non-default daemon start config IDs before admission/recording.
5. REFACTOR: Keep daemon record serialization unchanged for default config IDs.

## Evidence

- RED config-id evidence: `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_start_rejects_non_default_config_id_without_recording_instance -- --nocapture; echo EXIT:$?` failed before implementation with `left: Bool(true) right: false` and `EXIT:101`.
- RED source TOCTOU evidence: attempted targeted regression; the original timing-based mutation test was not deterministic enough to fail before implementation, so final evidence relies on code review plus the GREEN regression below proving execution succeeds from the already-read snapshot after a post-validation live-source mutation.
- GREEN focused daemon control-plane evidence: `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture` passed: 9 passed, 0 failed.
- GREEN run/daemon artifact-equivalence evidence: `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture` passed: 4 passed, 0 failed.
- Focused authority regression evidence: `RUSTC_WRAPPER= cargo test -p ash-interp --test invoke_runtime_dispatch --test runtime_action_control --test task_736_capability_binding_admission --test act_env_runtime_boundary -- --nocapture` passed: 27 + 9 + 18 + 15 tests passed.
- Check/clippy evidence: `RUSTC_WRAPPER= cargo check -p ash-engine -p ash-cli` passed; `RUSTC_WRAPPER= cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` passed after fixing a needless borrow.
- Broad workspace test evidence: `RUSTC_WRAPPER= scripts/check-rust-tests.sh --workspace` passed to final exit 0 after running workspace tests and doctests serially.
- Rustdoc evidence: `RUSTC_WRAPPER= cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-task944-doc.log && ! grep -i '^warning:' /tmp/ash-task944-doc.log` passed.
- Formatting/review evidence: `git diff --check`, `cargo fmt --check`, and independent Codex read-only review `git diff --check` passed.

## Completion Checklist

- [x] Regression tests written before implementation.
- [x] Daemon execution avoids a second workflow source read after drift check.
- [x] Non-default daemon config IDs are rejected before instance recording.
- [x] Default daemon args/config/admission-profile behavior remains green.
- [x] SPEC-070 documents the current `FILE[:WORKFLOW]` and daemon args/config
      caveats honestly.
- [x] PLAN-119, PLAN-INDEX, TASK-941 audit, and CHANGELOG cite TASK-944.
- [x] Focused requested tests and `cargo fmt --check` run or gaps recorded.
