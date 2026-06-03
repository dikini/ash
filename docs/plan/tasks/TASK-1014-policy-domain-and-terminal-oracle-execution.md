# TASK-1014: Policy Domain and Terminal Oracle Execution

## Status: Complete

## Description

Execute synthesized policy cases over checked bounded domains and stable terminal oracles beyond the Phase 76B exact metadata equality slice.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)

## Requirements

1. Materialize finite policy input domains from checked policy metadata.
2. Execute policy targets to produce terminal outcomes.
3. Support allow/deny first, then approval/transform when metadata exposes stable oracles.
4. Preserve required authority metadata and defer missing authority setup.
5. Emit repro artifacts for each executed policy case.

## TDD Steps

- RED: Add failing policy execution tests that currently rely on terminal metadata instead of evaluated policy targets.
- GREEN: Execute supported policy domains and terminal oracles.

## TDD Evidence

### RED

Command:

```bash
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli structured_policy -- --nocapture
```

Result:

```text
error[E0422]: cannot find struct, variant or union type `PolicyExecutableTarget` in this scope
error[E0560]: struct `synthesized::RunnerPolicyMetadata` has no field named `executable_target`
error[E0559]: variant `synthesized::SynthesizedOracle::PolicyTerminalEquals` has no field named `policy_ref`
error[E0559]: variant `synthesized::SynthesizedOracle::PolicyTerminalEquals` has no field named `terminal_oracle`
error: could not compile `ash-cli` (lib test) due to 17 previous errors
```

The failing tests required typed policy executable target/oracle metadata and proved the old terminal-field equality path could not support TASK-1014.

### GREEN

Command:

```bash
CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli policy -- --nocapture
```

Result:

```text
running 11 tests
test test_runner::synthesized::tests::policy_approval_and_transform_terminals_defer_without_stable_exact_oracle_slice ... ok
test test_runner::synthesized::tests::policy_terminal_expected_mismatch_fails_even_if_input_terminal_matches_expected ... ok
test test_runner::synthesized::tests::policy_with_empty_executable_target_ref_defers ... ok
test test_runner::synthesized::tests::policy_with_mismatched_executable_target_ref_defers ... ok
test test_runner::synthesized::tests::policy_with_required_authority_and_matching_explicit_setup_executes ... ok
test test_runner::synthesized::tests::policy_with_required_authority_without_explicit_setup_defers ... ok
test test_runner::synthesized::tests::structured_policy_terminal_oracle_evaluates_input_fields_instead_of_terminal_metadata ... ok

test result: ok. 11 passed; 0 failed; 0 ignored
```

The implemented slice is intentionally narrow: structured policy metadata may execute allow/deny terminal cases only when it supplies an explicit finite input domain, lowered policy ref, terminal-equals oracle shape, executable terminal-oracle target metadata whose non-empty `target_ref` matches the lowered policy ref, and supported authority setup. The terminal oracle is an exact-match table over finite input fields. Approval/transform, missing targets, target-ref mismatch, unsupported oracles, unsupported domains, missing lowered policy refs, and missing required-authority setup defer as skips.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused policy synthesized runner tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [x] Policy domains materialize from checked metadata.
- [x] Policy terminal outcomes are evaluated.
- [x] Authority gaps defer.
- [x] RED/GREEN evidence recorded.
