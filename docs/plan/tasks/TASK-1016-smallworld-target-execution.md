# TASK-1016: Small-World Target Execution

## Status: Complete

## Description

Execute Ash targets against deterministic small-world states rather than only evaluating metadata oracles over world snapshots.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Materialize deterministic finite worlds before execution.
2. Execute supported Ash targets with world bindings, roles, capabilities, policies, obligations, mailbox, and resource state as applicable.
3. Apply world-specific oracles after target execution.
4. Ensure `--max-worlds` bounds world materialization and execution.
5. Emit failing world snapshots and replay commands.

## TDD Steps

- RED: Add failing tests showing explicit worlds do not yet execute against Ash targets.
- GREEN: Execute supported target/world combinations and evaluate oracles.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused small-world execution tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [x] Worlds execute against supported Ash targets.
- [x] `--max-worlds` bounds execution count.
- [x] Failing cases report concrete world snapshots.
- [x] RED/GREEN evidence recorded.

## Evidence

- RED: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli smallworld -- --nocapture` failed before implementation because small-world rows evaluated world metadata directly and lacked executable target metadata.
- GREEN: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli smallworld -- --nocapture` passed with 5 focused small-world target execution tests after adding explicit target metadata, target-output oracle execution, and a metadata-only oracle fail-closed regression.
- Implementation: `crates/ash-cli/src/test_runner/synthesized.rs` now requires `SmallWorldExecutableTarget` metadata and target-output oracle metadata for supported small-world pass rows, executes a narrow pure-expression/literal target over materialized world bindings, defers legacy metadata-only world oracles until real post-execution state semantics exist, defers missing/unsupported target metadata, and records target output/error details in repro artifacts.
- Verification: `cargo fmt --check`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli smallworld -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`; `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`; and `git diff --check` all exited 0.
