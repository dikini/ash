# TASK-934: `do:Result` operational-bottom execution evidence

## Status: ✅ Complete

## Description

Add execution-grade evidence that `fail` inside a concrete `do:Result<_, E>` remains operational bottom and is not converted into a domain `Err`.

## Specification Reference

- SPEC-069 A69-8
- SPEC-050 operational bottom
- SPEC-054 generalized typed do notation

## Dependencies

- TASK-933 completion

## Requirements

### Functional Requirements

1. Add a focused RED test for a concrete `do:Result<Int, E>` body containing `fail`.
2. Verify typed lowering still records selected `Monad<Result<_, E>>` evidence.
3. Verify execution returns the operational failure/bottom path, not `Err`/domain failure.
4. Preserve existing `Result` bind/return success behavior.

Property invariant: operational `fail` must not be observationally equal to a returned `Err` value.

## TDD Steps

1. Write RED tests in `crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs` and a new or existing `crates/ash-interp/tests/alpha_do_result_fail_execution.rs`.
2. Run focused tests and confirm non-zero failures for the intended missing behavior.
3. Implement minimal runtime/typeck fix in `crates/ash-interp/src/eval.rs`, `crates/ash-interp/src/execute.rs`, or `crates/ash-typeck/src/do_target.rs` as needed.
4. Verify GREEN plus HKT/generalized-do regressions.

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
  - cargo test -p ash-typeck --test alpha_generalized_do_full_bind_lowering -- --nocapture
  - cargo test -p ash-interp --test alpha_do_result_fail_execution -- --nocapture
  - cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix -- --nocapture
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason: `cargo test -p ash-interp --test alpha_do_result_fail_execution -- --nocapture` initially failed because `result::and_then` runtime execution was not callable.
  - [x] Focused GREEN test passes and runs non-zero tests: `cargo test -p ash-interp --test alpha_do_result_fail_execution -- --nocapture` ran 2 tests; `cargo test -p ash-typeck --test alpha_generalized_do_full_bind_lowering -- --nocapture` ran 5 tests.
  - [x] cargo fmt --check passes when Rust code changed.
  - [x] git diff --check passes.
  - [x] cargo check --workspace passes if shared carriers or public APIs changed.
  - [x] cargo clippy --workspace --all-targets --all-features -- -D warnings passes before task closeout if code changed.
  - [x] CHANGELOG.md updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Codex verification reports no blockers: TASK-934 review returned APPROVE after focused tests, fmt, diff check, and `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`.
```

## Dependencies for Next Task

Produces Phase 123 evidence for downstream closeout and status reconciliation.

## Notes

Do not mark this task complete until its own focused evidence, status surfaces, and Codex verification are reconciled.
