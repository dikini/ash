# TASK-1016: Small-World Target Execution

## Status: Planned

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

- [ ] Worlds execute against supported Ash targets.
- [ ] `--max-worlds` bounds execution count.
- [ ] Failing cases report concrete world snapshots.
- [ ] RED/GREEN evidence recorded.
