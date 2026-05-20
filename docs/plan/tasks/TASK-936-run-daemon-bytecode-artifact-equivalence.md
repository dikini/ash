# TASK-936: `ash run` / daemon bytecode artifact equivalence

## Status: 📝 Planned

## Description

Prove `ash run` and `ash daemon` host modes use equivalent verified bytecode artifacts and preserve language-level semantics for the same source/config/profile.

## Specification Reference

- SPEC-069 A69-12
- SPEC-070 A70-8

## Dependencies

- TASK-935 completion

## Requirements

### Functional Requirements

1. Add an integration test that builds/runs the same workflow through one-shot and daemon paths.
2. Compare verifier-normalized bytecode/provenance summaries, not only string IDs.
3. Verify host-mode identity differs while language artifact summary matches.
4. Ensure failed reload does not mutate an already-admitted artifact summary.

## TDD Steps

1. Write RED tests in `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs`.
2. Use TASK-935 builder from both `run.rs` and `daemon.rs`.
3. Verify existing one-shot and daemon control-plane tests still pass.

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
  - cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture
  - cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture
  - cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture
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
