# TASK-1364: Typechecker — verify law proposition names exist

## Status: ✅ Complete

## Description

Typechecker verifies that all names referenced in a law proposition exist and are well-typed.

## Requirements

1. Add `register_interface_laws` to `TypeEnv`
2. Add `register_module_laws` to `TypeEnv`
3. Check that all identifiers in proposition expression resolve
4. Typecheck the proposition expression

## Acceptance Criteria

- [x] Unknown names in law proposition produce error
- [x] Well-formed law propositions pass
- [x] Typechecker test passes
- [x] No regressions

## Verification

- `cargo test -p ash-typeck --test task_1364_law_name_checking -- --nocapture` — 3 passed
- `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `TypeEnv::register_interface_laws` and `TypeEnv::register_module_laws`.
- Law proposition checking now runs through the program typecheck pipeline after interface/function registration.
- Law scopes bind declared law parameters; interface law scopes also bind the interface's own methods for unqualified law propositions.
- This task only checks expression name/type resolution; proof matching, Pure-only restrictions, totality, runner integration, and synthetic tests remain later Phase 136 tasks.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
