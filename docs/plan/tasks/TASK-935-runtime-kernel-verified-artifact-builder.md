# TASK-935: Shared RuntimeKernel verified artifact builder

## Status: ✅ Complete

## Description

Introduce a shared RuntimeKernel artifact-building path that produces verifier-normalized TCIR/AMIR/bytecode summaries from source/check/profile inputs for both one-shot and daemon hosts.

## Specification Reference

- SPEC-069 A69-10, A69-12
- SPEC-070 §6, A70-8

## Dependencies

- TASK-933 completion

## Requirements

### Functional Requirements

1. Add a shared artifact builder used outside tests by both `ash run` and daemon code paths.
2. Builder output must include stable source hash, check summary hash, artifact version, TCIR/AMIR/bytecode provenance summary, and verifier result.
3. Builder must not reparse source during bytecode verification.
4. Existing one-shot/daemon identity outputs must remain stable except where intentionally switched to builder-derived fields.

Property invariant: identical root/profile/config/source inputs produce deterministic equal builder output.

## TDD Steps

1. Write RED builder tests in `crates/ash-core/tests/alpha_runtime_kernel_artifact_builder.rs` and/or `crates/ash-engine/tests/alpha_runtime_kernel_artifact_builder.rs`.
2. Implement shared builder in `crates/ash-core/src/runtime_kernel.rs`, `crates/ash-engine/src/lib.rs`, or a new `crates/ash-engine/src/runtime_artifact.rs`.
3. Wire non-test callsites in `crates/ash-cli/src/commands/run.rs` and `crates/ash-cli/src/commands/daemon.rs`.
4. Verify real callsites with search and focused tests.

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
  - cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder -- --nocapture
  - cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder -- --nocapture
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason: a detached HEAD worktree with only the new TASK-935 tests failed with unresolved imports for `RuntimeArtifactBuildInput`, `RuntimeArtifactVerifierResult`, and `RuntimeKernelArtifactBuilder` before implementation.
  - [x] Focused GREEN test passes and runs non-zero tests: `cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder -- --nocapture` ran 2 tests; `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder -- --nocapture` ran 2 tests.
  - [x] cargo fmt --check passes when Rust code changed.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed.
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers: TASK-935 review returned APPROVE after focused tests, fmt, diff check, workspace check, and clippy.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
