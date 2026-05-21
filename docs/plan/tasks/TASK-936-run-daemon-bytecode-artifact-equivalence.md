# TASK-936: `ash run` / daemon bytecode artifact equivalence

## Status: ✅ Complete (review-remediated)

## Description

Prove `ash run` and `ash daemon` host modes expose equivalent verified alpha checked workflow-boundary artifacts for the same source/config/profile while preserving distinct host identity.

This task does not claim full workflow-body TCIR equivalence. The current production run/daemon pipeline does not expose a complete checked workflow-body TCIR carrier to RuntimeKernel artifact construction. TASK-936 therefore compares the explicitly labeled `alpha_checked_workflow_boundary` carrier after parse/check succeeds, and keeps full body-semantic artifact equivalence out of scope until a later lowering/admission task exposes that carrier.

## Specification Reference

- SPEC-069 A69-12
- SPEC-070 A70-8

## Dependencies

- TASK-935 completion

## Requirements

### Functional Requirements

1. Add an integration test that builds/runs the same workflow through one-shot and daemon paths.
2. Compare verifier-normalized bytecode/provenance summaries for the explicit alpha checked workflow-boundary carrier, not only string IDs.
3. Verify host-mode identity differs while language artifact summary matches.
4. Ensure failed reload does not mutate an already-admitted artifact summary.

## TDD Steps

1. Write RED tests in `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs`.
2. Use TASK-935 builder from both `run.rs` and `daemon.rs` after parse/check succeeds.
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
  - [x] Focused RED test was observed failing for the intended reason, unless this is a docs/planning task.
  - [x] Focused GREEN test passes and runs non-zero tests, unless this is a docs/planning task.
  - [x] cargo fmt --check passes when Rust code changed.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed (`RUSTC_WRAPPER=` used after the sandbox rejected the configured `sccache` wrapper).
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed (`RUSTC_WRAPPER=` used after the sandbox rejected the configured `sccache` wrapper).
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers.
```

## Review Remediation Evidence

Independent review found the first TASK-936 closeout overclaimed full language artifact equivalence because run/daemon summaries were both derived from synthetic TCIR before the one-shot dry-run parse/check boundary.

Remediation:
- `ash run` no longer emits a RuntimeKernel artifact report on parse/check failure. The RED test failed with a JSON `artifact_summary.verifier = "verified"` emitted before the parse error, then passed after moving report construction behind successful validation.
- Daemon indexing now parse/checks each workflow through `ash-engine` before building the RuntimeKernel artifact summary.
- Serialized `artifact_summary.tcir.carrier_scope` is now `alpha_checked_workflow_boundary`, making the MVP boundary explicit instead of claiming full workflow-body TCIR semantics.
- Bytecode verification still consumes the carried TCIR and reports `requires_source_reparse = false`.

Fresh verification for this remediation:
- `cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture`: 2 passed, 0 failed.
- `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture`: 3 passed, 0 failed.
- `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane -- --nocapture`: 4 passed, 0 failed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `RUSTC_WRAPPER= cargo check --workspace`: passed.

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
