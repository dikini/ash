# TASK-1013: Contract Target and Postcondition Synthesized Execution

> **TASK-2041 status:** This completed task's older execution descriptions are historical. Current
> `ash test` execution uses its local Engine instance and does not use a direct evaluator or daemon
> transport.

## Status: Complete

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

## TDD Evidence

### RED

Command:

```bash
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli run_suite_executes_structured_snapshot_contract_postconditions_against_target_output -- --nocapture
```

Result:

```text
running 1 test

thread 'test_runner::executor::tests::run_suite_executes_structured_snapshot_contract_postconditions_against_target_output' (2999) panicked at crates/ash-cli/src/test_runner/executor.rs:924:9:
assertion `left == right` failed
  left: Null
 right: "ash_interp_core_expr"
test test_runner::executor::tests::run_suite_executes_structured_snapshot_contract_postconditions_against_target_output ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 112 filtered out; finished in 0.00s
```

### GREEN

Command:

```bash
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli postcondition -- --nocapture
```

Result:

```text
running 7 tests
test test_runner::synthesized::tests::contract_postcondition_with_missing_setup_defers ... ok
test test_runner::synthesized::tests::contract_postcondition_with_unsupported_target_kind_defers ... ok
test test_runner::synthesized::tests::contract_postcondition_without_executable_target_metadata_defers ... ok
test test_runner::synthesized::tests::structured_contract_metadata_executes_postcondition_against_target_output ... ok
test test_runner::executor::tests::run_suite_executes_structured_snapshot_contract_postconditions_against_target_output ... ok
test test_runner::synthesized::tests::contract_postcondition_without_structured_oracle_metadata_defers ... ok
test test_runner::synthesized::tests::structured_contract_postcondition_failure_is_fail_not_skip_or_pass ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 107 filtered out; finished in 0.00s

test only_synthesized_contract_postcondition_executes_supported_pure_function_metadata ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.01s
```

The implemented slice is intentionally narrow: checked metadata can execute pure `Int` function targets with exact finite `ContractValid` representatives by lowering the checked function body and `ensures` clauses to `ash_core::Expr` and evaluating them through `ash_interp::eval_expr`. Display strings remain report/repro text only. Unsupported target kinds, missing setup, missing executable target metadata, missing structured postcondition oracle metadata, and unsupported postcondition metadata defer with skip reasons.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli run_suite_executes_structured_snapshot_contract_postconditions_against_target_output -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`
- `git diff --check`

## Completion Checklist

- [x] Supported contract postconditions execute end to end.
- [x] Unsupported cases defer.
- [x] Repro artifacts include target input/output context.
- [x] RED/GREEN evidence recorded.
