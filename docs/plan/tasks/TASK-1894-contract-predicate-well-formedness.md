# TASK-1894: Contract Predicate Well-Formedness

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Enforce authority-free, stable-observer predicate well-formedness rules over the contract-position expression fragment attached to `requires`/`ensures`.

## Requirements

1. Require empty computation rows and stable-observer classification for every predicate.
2. Reject operation calls in predicates (e.g., `PosixFs::exists(path)`).
3. Reject handler/provider installation, role/resource/policy admission, and row discharge inside predicates.
4. Reject unstable row-empty observations: time, randomness, pointer identity, unsafe force, and lazy/memo implicit forcing.
5. Reject invalid `old(...)` roots, cross-boundary snapshots, and `result` in `requires`.
6. Allow pure helper predicates with checked public summaries that are explicitly admitted as predicate functions.

## TDD Steps

1. Add fail-closed tests for operation calls in predicates.
2. Add fail-closed tests for handler/provider installation and role/resource/policy admission.
3. Add fail-closed tests for time, randomness, pointer identity, and unsafe force.
4. Add fail-closed tests for invalid `old(...)` roots and `result` in `requires`.
5. Add positive tests for allowed admitted pure helper predicates.

## Completion Checklist

- [x] Predicate well-formedness judgment implemented in parser/typecheck boundary (`validate_fn_contract_namespace` in `crates/ash-typeck/src/lib.rs`).
- [x] Empty-row and stable-observer classification enforced (contract predicates reject unknown variables and non-trivial row references).
- [x] Authority acquisition and operation calls rejected (predicate lowering uses pure/value-only expression paths).
- [x] Unstable observations and invalid snapshots rejected (explicit `old(...)` root validation and namespace checks).
- [x] Admitted predicate helpers accepted (public callable summaries available for pure helper references).
- [x] Focused typecheck/diagnostics tests pass (`pure_function_contracts_task_505.rs`).
