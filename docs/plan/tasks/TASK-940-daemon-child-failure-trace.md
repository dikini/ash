# TASK-940: Daemon child Proc failure trace semantics

## Status: 📝 Planned

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
  - [ ] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task.
  - [ ] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task.
  - [ ] cargo fmt --check passes when Rust code changed.
  - [ ] git diff --check passes.
  - [ ] cargo check --workspace passes if shared carriers or public APIs changed.
  - [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [ ] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [ ] Codex verification reports no blockers.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
