# TASK-938: Daemon start args/config/admission-profile protocol

## Status: 📝 Planned

## Description

Extend `ash daemon start` to accept and persist start arguments, config/profile selection, and admission-profile fields while preserving failed-admission no-body behavior.

## Specification Reference

- SPEC-070 A70-4
- SPEC-070 §5 Local daemon
- SPEC-070 §7 Authority and admission

## Dependencies

- TASK-937 completion

## Requirements

### Functional Requirements

1. Extend daemon start request/CLI protocol with args, config/profile, and admission-profile fields.
2. Persist selected fields in the instance record/report.
3. Reject invalid admission before body execution or instance activation according to the chosen alpha semantics.
4. Preserve existing zero-argument empty-admission start behavior.

Property invariant: different args/config/admission profiles produce distinguishable instance admission records without changing definition identity.

## TDD Steps

1. Write RED tests in `crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs`.
2. Implement protocol and records in `crates/ash-cli/src/commands/daemon.rs` and `crates/ash-core/src/runtime_kernel.rs`.
3. Verify old daemon behavior still passes.

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
  - cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture
  - cargo test -p ash-cli --test alpha_admission_profile -- --nocapture
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
