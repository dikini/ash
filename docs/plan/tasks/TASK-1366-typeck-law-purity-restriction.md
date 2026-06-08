# TASK-1366: Typechecker — restrict law propositions to Pure functions

## Status: ✅ Complete

## Description

Law propositions must reference only `Pure` functions. `Act`/`Proc`/`Workflow` in law body = compile-time error.

## Requirements

1. Track effect level of each function referenced in law proposition
2. Reject if any referenced function has effect > Pure
3. Error message indicates which function violates purity

## Acceptance Criteria

- [x] Law referencing `Act` function produces error
- [x] Law referencing only `Pure` functions passes
- [x] Typechecker test passes
- [x] No regressions

## Verification

- `cargo test -p ash-typeck --test task_1366_law_purity -- --nocapture` — 5 passed
- `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Law proposition validation now runs the existing pure-context checker before expression typechecking.
- Calls to functions returning `Act`, `Proc`, or `Workflow` are rejected in pure contexts, including law propositions.
- Law diagnostics include the law name and offending callee/purity violation.
- This task does not add proof body checking, totality, runner integration, synthetic tests, or `Prop` kind semantics.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
