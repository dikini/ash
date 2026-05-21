# TASK-940: Daemon child Proc failure trace semantics

## Status: ✅ Complete

## Description

Add daemon-hosted execution evidence that child Proc failure is observed through Proc/Workflow semantics and does not become daemon host failure.

## Specification Reference

- SPEC-070 A70-7
- SPEC-049 Process runtime semantics
- SPEC-051 Workflow semantics

## Dependencies

- TASK-938 completion
- TASK-939 completion

## Requirements

### Functional Requirements

1. Add a daemon-hosted workflow/process execution test where a child Proc fails.
2. Verify daemon host remains healthy and can answer status/list after child failure.
3. Verify instance status/report classifies the failure as Proc/Workflow child failure, not daemon host failure.
4. Preserve existing cancel/status semantics.

Property invariant: child process failure may fail the workflow instance but must not terminate the daemon control plane.

## TDD Steps

1. Write RED test `daemon_child_proc_failure_is_instance_failure_not_host_failure` in `crates/ash-cli/tests/alpha_ashd_child_failure_trace.rs`.
2. Implement execution/status/report plumbing in daemon/runtime files.
3. Verify daemon remains usable after failure by issuing a follow-up status/list request.

## Dispatch

```yaml
agent: codex
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

Codex instructions:
- Work in a dedicated worktree.
- Do not spawn nested agents.
- Follow RED-GREEN-REFACTOR for code tasks.
- Keep the task scope narrow; do not implement later tasks early.
- Return exact files changed, focused commands run, and any remaining blockers.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-cli --test alpha_ashd_child_failure_trace -- --nocapture
  - cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture
  - cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task.
  - [x] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task.
  - [x] cargo fmt --check passes when Rust code changed.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed.
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Completed on 2026-05-21.

Implemented opt-in daemon execution over the existing JSON-lines protocol through `{"command":"start","execute":true,...}` for TASK-940 evidence while preserving ordinary `ash daemon start` CLI record-only behavior from TASK-938. The daemon now records terminal instance execution reports for opted-in execution and keeps child Proc failures on the workflow instance surface:

- `status: failed`
- `class: workflow_child_failure`
- `report.failure.tower: Proc`
- `report.failure.kind: child_proc_failure`
- `report.failure.host_failure: false`

The focused daemon test starts a daemon, triggers a child Proc failure through real Proc `par`/`join` semantics, then issues follow-up `status` and `list` requests through the same control plane to prove the daemon host remains alive after the instance failure.

TDD RED evidence:

- `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_child_failure_trace daemon_child_proc_failure_is_instance_failure_not_host_failure -- --nocapture` initially failed before implementation because the daemon either ignored requested execution and left instances admitted or, during implementation, crashed the host by starting a nested Tokio runtime in the serving runtime. The latter produced EOF on the control socket and proved the intended host-vs-instance failure boundary was not yet satisfied.

Focused GREEN evidence:

- `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_child_failure_trace -- --nocapture` passed: 1 passed, 0 failed.
- `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture` passed: 7 passed, 0 failed.
- `RUSTC_WRAPPER= cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture` passed: 6 passed, 0 failed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
- `RUSTC_WRAPPER= cargo check --workspace` passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.

Codex implementation attempt produced the initial test and most daemon plumbing but stalled; the final diff was salvaged, fixed, independently verified, review-remediated, and clippy-cleaned in this worktree.

Independent review remediation:

- Split daemon host/infrastructure failures from workflow instance failures in daemon execution report classification so runtime/engine build and execution-worker panic paths are reported as `daemon_execution_host_failure` with `host_failure: true` instead of being masked as workflow child failures.
- Made child Proc classification inspect the typed `OperationalFailure` cause chain so a workflow-level wrapper around a Proc failure still reports `child_proc_failure`/`workflow_child_failure` with Proc attribution.
- The `execute=true` protocol path remains documented as a narrow opt-in TASK-940 execution evidence hook; ordinary public `ash daemon start` CLI remains record-only.

Post-review GREEN evidence:

- `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_child_failure_trace -- --nocapture` passed: 1 passed, 0 failed.
- `RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture` passed: 7 passed, 0 failed.
- `RUSTC_WRAPPER= cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture` passed: 6 passed, 0 failed.
- `RUSTC_WRAPPER= cargo check --workspace` passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
