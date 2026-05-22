# TASK-945: Phase 123 Final Remediation

Status: Complete (focused security, authority, verifier/report, and daemon
artifact-equivalence regressions pass)
Phase: Phase 123 / PLAN-119 final remediation
Specs: SPEC-069, SPEC-070

## Description

Phase 123 final remediation hardens the alpha daemon local-control surface
required by SPEC-070. The daemon must treat its Unix socket and runtime roots as
same-user local authority boundaries, and must reject unsafe writable
directories before removing stale socket paths or binding the control socket.

The same final review also found capability binding alias authority,
verifier/report, and status gaps after TASK-944. This task records those final
Phase 123 fixes without reopening Phase 122 history: Phase 122 remains the
historical Partial MVP closeout, while Phase 123 owns the Implemented MVP
promotion and the TASK-942 through TASK-945 remediation record.

## Requirements

1. Validate daemon root, socket parent, state dir, cache dir, and log dir
   against the current effective user before binding.
2. Reject daemon root, socket parent, state dir, cache dir, and log dir when any
   directory is group-writable or world-writable.
3. Reject same-user local-control paths not owned by the current effective user
   where that condition is testable without elevated privileges.
4. Remove stale Unix socket files only after the socket parent has passed
   ownership and mode validation.
5. Preserve rejection of a pre-existing regular file at the socket path without
   deleting it.
6. Set the bound daemon Unix socket to same-user-only permissions when the
   platform exposes Unix mode bits.
7. Reconcile SPEC-070, PLAN-119, PLAN-INDEX, and CHANGELOG evidence for this
   remediation slice.
8. Preserve capability binding alias authority projection so admitted
   provider/action grants are scoped to admitted binding IDs and binding names
   rather than unioned by backing provider name.
9. Strengthen AMIR and bytecode verifier negatives for TCIR statement coverage:
   duplicate TCIR statement IDs, missing statement coverage, duplicate statement
   references, and empty instruction streams for non-empty TCIR must reject.
10. Strengthen bytecode verifier negatives for malformed logical offsets:
   duplicate offsets and skipped offsets must reject.
11. Add RuntimeKernel one-shot report-surface coverage for admitted grant
    details. The current alpha one-shot path still admits no concrete grants,
    so the report must expose empty detail arrays honestly rather than implying
    the details are unavailable.
12. Record an explicit imported-module daemon drift caveat: TASK-944's
    start-execute source snapshot closes the admitted workflow file's
    second-read gap, but it does not yet add a separate post-admission watcher
    or digest closure over imported module files.
13. Clarify the current `ash run` admission lifecycle instead of claiming a
    broader refactor: admission-profile rejection happens before user code and
    before verified artifact reporting; verified artifact reports are emitted
    only after parse/check/artifact construction succeeds.
14. Reconcile SPEC-069, SPEC-070, TASK-943 exact evidence, TASK-941 closeout
    evidence, and CHANGELOG for the final verifier/report slice.

## TDD Steps

1. RED: Add focused `ash-cli` Unix daemon control-plane regressions for
   group/world-writable root/socket parent/state/cache/log paths.
2. RED: Add focused coverage for a non-current-user-owned daemon path where the
   host environment exposes one without requiring root.
3. RED: Add a stale-socket regression proving unsafe socket parents are rejected
   before stale socket removal.
4. RED: Preserve regular-file socket-path rejection coverage.
5. GREEN: Validate root/socket parent/state/cache/log ownership against the
   current effective user and reject unsafe writable mode bits.
6. GREEN: Keep stale socket removal after successful local-control path
   validation, reject symlinked local-control directories, and chmod the bound
   socket to `0600`.
7. RED/GREEN: Preserve focused capability binding alias authority regressions
   for binding-name projection, alias-only dispatch rejection, direct `invoke`,
   `Workflow::Act`, implementation-binding metadata, spawned-child empty
   admission, and custom-provider last-wins behavior.
8. RED: Add AMIR/bytecode malformed-artifact tests and verify the current
   verifier accepts missing coverage and offset gaps.
9. GREEN: Add verifier bijection checks between TCIR statements and
   AMIR/bytecode instructions, reject duplicate TCIR statement IDs, plus
   contiguous bytecode offset validation.
10. RED: Add one-shot RuntimeKernel JSON report assertions for admitted grant
   detail fields and verify the current report omits them.
11. GREEN: Serialize admitted grant counts and detail arrays in the one-shot
    RuntimeKernel report.

## Evidence

- RED full target evidence before implementation:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture`
  failed with the new local-control security regressions before `daemon.rs`
  changes. Relevant failures included missing `group/world-writable` rejection
  and missing `current effective user` ownership diagnostics. The same run also
  exposed that this sandbox returns `EPERM` for Unix socket bind attempts, which
  makes existing daemon happy-path tests unable to observe a ready socket here.
- GREEN focused unsafe-directory evidence:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_serve_rejects -- --nocapture`
  passed: 3 passed, 0 failed after adding group/world-writable,
  non-current-user where available, and symlinked socket-parent rejection
  coverage.
- GREEN stale-socket ordering evidence:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_serve_validates_socket_parent_before_removing_stale_socket -- --nocapture`
  passed in this sandbox by explicitly skipping the stale socket fixture after
  `UnixListener::bind` returned `EPERM`; on hosts permitting local Unix sockets,
  the test verifies unsafe parent validation happens before stale socket
  removal.
- GREEN regular-file socket-path preservation evidence:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_rejects_preexisting_non_socket_control_path_without_deleting_it -- --nocapture`
  passed: 1 passed, 0 failed.
- Requested full focused target evidence after implementation:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture`
  did not pass in this sandbox because Unix socket binding is denied with
  `EPERM`; the security rejection tests passed, while seven existing
  socket-serving tests failed waiting for daemon sockets that the host would not
  allow the daemon to bind.
- Capability alias authority evidence is represented by the staged focused
  regressions in `crates/ash-interp/tests/invoke_runtime_dispatch.rs`,
  `crates/ash-interp/tests/runtime_action_control.rs`, and
  `crates/ash-interp/tests/task_736_capability_binding_admission.rs`; those
  files cover alias-only dispatch rejection, direct `invoke` and
  `Workflow::Act` projection through admitted binding names, implementation
  binding metadata boundaries, spawned-child empty admission, and custom
  provider last-wins behavior.
- RED AMIR/bytecode evidence:
  `cargo test -p ash-core --test alpha_amir_bytecode_schema -- --nocapture`
  failed before implementation with
  `verification should reject malformed artifact: ()` in
  `amir_verifier_rejects_non_bijective_statement_coverage` and
  `bytecode_verifier_rejects_non_bijective_statement_coverage_and_offsets`.
- GREEN AMIR/bytecode evidence:
  `cargo test -p ash-core --test alpha_amir_bytecode_schema -- --nocapture`
  passed with 6 passed, 0 failed after the duplicate-TCIR-ID verifier
  remediation.
- RED one-shot report evidence:
  `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode ash_run_reports_kernel_instance_and_artifact_identity -- --nocapture`
  failed before implementation because `report["admission"]["action_grants"]`
  was `Null` instead of `0`.
- GREEN one-shot report evidence:
  `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode ash_run_reports_kernel_instance_and_artifact_identity -- --nocapture`
  passed with 1 passed, 0 failed after adding grant detail fields.
- Requested final AMIR/bytecode focused evidence:
  `cargo test -p ash-core --test alpha_amir_bytecode_schema -- --nocapture`
  passed with 6 passed, 0 failed.
- Requested final CLI focused evidence:
  `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence --test alpha_ash_run_runtime_kernel_mode -- --nocapture`
  passed after tightening daemon test fixture directory modes for the new
  same-user/non-writable daemon-control rule: `alpha_ash_run_runtime_kernel_mode`
  3 passed, 0 failed; `alpha_run_daemon_artifact_equivalence` 4 passed,
  0 failed.
- Additional focused checks:
  `RUSTC_WRAPPER= cargo fmt --check`, `git diff --check`,
  `RUSTC_WRAPPER= cargo check --workspace`,
  `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `RUSTC_WRAPPER= scripts/check-rust-tests.sh --workspace`,
  `RUSTC_WRAPPER= cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase123-final-doc.log && ! grep -i '^warning:' /tmp/ash-phase123-final-doc.log`,
  and the PLAN-119 embedded Markdown link-check passed on the final diff.

## Completion Checklist

- [x] Regression tests written before implementation.
- [x] RED focused test run captured.
- [x] Daemon local-control paths reject non-current-user ownership where
      testable.
- [x] Daemon local-control paths reject group/world-writable directories.
- [x] Stale socket removal happens only after safe parent validation.
- [x] Existing regular-file socket path rejection remains covered.
- [x] Capability binding alias authority projection remains covered by focused
      alias/binding/admission regressions.
- [x] AMIR verifier rejects duplicate TCIR statement IDs, missing/duplicate
      TCIR statement coverage, and empty instruction streams for non-empty TCIR.
- [x] Bytecode verifier rejects duplicate TCIR statement IDs,
      missing/duplicate TCIR statement coverage, empty instruction streams for
      non-empty TCIR, duplicate offsets, and skipped offsets.
- [x] One-shot RuntimeKernel JSON report exposes admitted grant counts and
      detail arrays.
- [x] Imported-module daemon drift caveat is recorded honestly.
- [x] `ash run` admission lifecycle caveat is recorded honestly.
- [x] PLAN-119 embedded link-check includes TASK-942 through TASK-945.
- [x] SPEC-070, PLAN-119, PLAN-INDEX, and CHANGELOG updated.
- [x] Focused requested test run captured after final review remediation.
