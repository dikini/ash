# TASK-1013: Contract Target and Postcondition Synthesized Execution

## Status: Planned

## Description

Execute synthesized contract cases against supported checked targets, including end-to-end postcondition checks over actual target results.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)

## Requirements

1. Execute supported pure function, act function, or workflow callable targets only when setup is explicit and finite.
2. Preserve precondition boundary acceptance/rejection behavior.
3. Add postcondition `ensures` checks over actual target outputs.
4. Defer unsupported target kinds, missing setup, open domains, and unrenderable values.
5. Emit repro artifacts with generated input and oracle snapshots.

## TDD Steps

- RED: Add failing tests for a supported contract postcondition that currently defers or lacks target execution.
- GREEN: Execute the target and evaluate the postcondition oracle.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused contract synthesized runner tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [ ] Supported contract postconditions execute end to end.
- [ ] Unsupported cases defer.
- [ ] Repro artifacts include target input/output context.
- [ ] RED/GREEN evidence recorded.
