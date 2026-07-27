# TASK-2031A: Daemon Startup Gate Remediation

**Status:** Complete
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Test-gate remediation
**Depends on:** TASK-929's local daemon control plane; the reproducible workspace-gate failure
**Blocks:** TASK-2031 closeout only; it does not alter TASK-2031's semantic ownership or TASK-2032's
shared-Engine/client-parity ownership.

## Context

The mandatory workspace test gate currently fails in the existing
`alpha_ashd_local_daemon_control_plane` integration test before the daemon creates its Unix-domain
socket. The existing test retains the child process but discards its captured stdout/stderr when
socket readiness times out, so the startup boundary that failed is not observable in the failure
report.

This is a narrowly scoped gate repair authorized to unblock TASK-2031's required verification. It
does not add a new execution route, admission authority, provider behavior, source lowering, or
CLI/daemon parity claim. The fixed architecture remains:

```text
Surface Ash → checked Core → checked CPS → shared Engine → terminal envelope
```

## Requirements

1. Add a focused RED regression that reports the daemon child's captured stdout/stderr and exit
   status if socket readiness fails, without leaking unrelated environment data.
2. Use that evidence to identify the earliest daemon startup boundary that fails.
3. Repair only an identified repository daemon startup/indexing defect. If the failure is instead
   an unavailable test-host capability, make that prerequisite explicit without changing public
   daemon protocol, frame/admission authority, the Engine execution route, or the TASK-2031
   calculus.
4. Preserve same-user local-control checks, rejection of unsafe paths, and stale-socket handling.
5. Prove the focused daemon control-plane test passes, then run the workspace gate that blocked
   TASK-2031.

## Explicit non-goals

- A generic Engine executor, CLI/daemon parity implementation, or a new daemon transport.
- Direct evaluation, source-shape dispatch, compatibility fallback, or client-local execution.
- Changing the semantic task record, coverage domain, or run-route impact of TASK-2031.
- Broad daemon feature work beyond the diagnosed startup failure.

## TDD steps

1. **RED — diagnostic regression.** Extend the readiness helper's failing path to include only the
   child exit status and captured stdout/stderr; run the exact failing daemon test and verify the
   underlying pre-bind startup error is exposed.
2. **Root-cause review.** Trace the reported failure through `DaemonState::new`, control-path
   validation, and definition indexing. Record the precise failing boundary in this task.
3. **RED — behavior regression.** Add the smallest deterministic test that describes the diagnosed
   bug, then confirm it fails before production changes.
4. **GREEN.** Make the smallest daemon/indexing change that satisfies that test, retaining existing
   local-control security checks and shared RuntimeKernel/Engine ownership.
5. **QA and review.** Run the focused daemon suite, formatter, Clippy, workspace test gate, and
   documentation checks; obtain independent specification and quality reviews before completion.

## Initial evidence

```text
RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane \
  ashd_serve_indexes_definitions_without_running_applications -- --exact --nocapture
```

This initially failed after eight seconds with `daemon socket did not become ready`. The diagnostic
RED now reports that `UnixListener::bind` fails with `Operation not permitted (os error 1)`. An
independent bare AF_UNIX bind in the test sandbox produces the same `EPERM`, while a direct daemon
serve outside that restricted test context binds successfully. The cause is therefore an
unavailable sandbox capability, not a DaemonState/indexing or Engine defect.

## Recorded remediation evidence

- The two daemon-dependent integration fixtures preflight a bare AF_UNIX bind in their own
  mode-`0700` socket parent. Only `ErrorKind::PermissionDenied` reports an explicit environment
  skip; every other preflight failure panics.
- After a successful preflight, readiness failures still retain the child and report its exit
  status plus safely decoded stdout/stderr. The preflight cannot mask an application startup
  failure.
- Fresh focused evidence: `alpha_ashd_local_daemon_control_plane` passed 13/13 and
  `alpha_run_daemon_artifact_equivalence` passed 4/4. `cargo fmt --check`, Clippy for all
  targets/features, and `cargo test --workspace --all-targets --all-features` passed. Independent
  code review approved the scope and confirmed that non-socket security/rejection tests continue
  to run.
- Historical change `de4043d8` is limited to test diagnostics and AF_UNIX preflight handling.
  This task adds no daemon protocol, Engine route, admission authority, or client-parity behavior.

## Completion checklist

- [x] The failing readiness path reveals the child startup failure deterministically and safely.
- [x] The host-capability prerequisite has a focused RED/GREEN regression without masking a
      post-preflight daemon failure.
- [x] The daemon control-plane and artifact-equivalence integration suites pass.
- [x] The workspace Rust test gate passes without bypassing hooks.
- [x] TASK-2031's existing checks, semantic ownership, and no-second-route constraints remain
      unchanged.
- [x] CHANGELOG and PLAN-INDEX are updated; QA and independent reviews are recorded.
