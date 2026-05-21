# TASK-937: Admission-profile rejection before user body execution

## Status: ✅ Complete

## Description

Implement one-shot RuntimeKernel admission-profile evaluation that can reject before user workflow body execution and reports admission failure distinctly from body failure.

## Specification Reference

- SPEC-070 A70-2
- SPEC-070 §7 Authority and admission
- NI-4 visible authority/admission boundary

## Dependencies

- TASK-933 completion
- TASK-935 shared artifact substrate

## Requirements

### Functional Requirements

1. Define minimal alpha admission-profile input for `ash run`.
2. Evaluate admission before body execution.
3. Add a side-effect sentinel test proving rejected admission does not run user code.
4. Emit admission-specific status/report distinct from parse/check/body failure.
5. Preserve default empty-admission behavior for existing tests.

Property invariant: rejected admission has zero user-code observable effects.

## TDD Steps

1. Write RED tests in `crates/ash-cli/tests/alpha_admission_profile.rs` and/or `crates/ash-interp/tests/act_env_runtime_boundary.rs`.
2. Implement minimal profile model in `crates/ash-core/src/runtime_kernel.rs`, `crates/ash-cli/src/commands/run.rs`, and `crates/ash-interp/src/runtime_state.rs` as needed.
3. Verify `invoke_runtime_dispatch` fail-closed tests still pass.

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
  - cargo test -p ash-cli --test alpha_admission_profile -- --nocapture
  - cargo test -p ash-interp --test invoke_runtime_dispatch -- --nocapture
  - cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason: `cargo test -p ash-cli --test alpha_admission_profile -- --nocapture` ran 2 tests and failed because `--admission-profile` was not yet accepted.
  - [x] Focused GREEN test passes and runs non-zero tests: `cargo test -p ash-cli --test alpha_admission_profile -- --nocapture` ran 2 tests; `cargo test -p ash-interp --test invoke_runtime_dispatch -- --nocapture` ran 7 tests; `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture` ran 3 tests.
  - [x] cargo fmt --check passes when Rust code changed.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed (`RUSTC_WRAPPER=` used to avoid the sandboxed sccache wrapper).
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed (`RUSTC_WRAPPER=` used to avoid the sandboxed sccache wrapper).
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers after status-surface reconciliation; independent review found only docs/changelog/status drift, which this closeout update resolves.
```

Implemented evidence:
- `ash run --admission-profile reject` evaluates admission before workflow body execution and reports `admission.status = rejected` without producing the body output file.
- Default `--admission-profile empty` remains admitted and preserves existing `ash run` behavior.
- Rejection reporting is distinct from parse/check/body failures and does not emit a verified artifact summary for a rejected body execution path.

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
