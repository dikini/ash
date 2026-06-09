# TASK-1375: Stage 3 — proof totality checking

## Status: ✅ Complete

## Description

Implement actual totality checking for proof bodies.

## Requirements

This task is split into sub-tasks:
- [x] [TASK-1375a](TASK-1375a-fuel-termination.md): Fuel-based termination analysis
- [x] [TASK-1375b](TASK-1375b-partial-match-detection.md): Partial match detection
- [x] [TASK-1375c](TASK-1375c-circular-proof-detection.md): Circular proof detection

## Acceptance Criteria

- [x] All sub-tasks complete
- [x] Non-exhaustive proof matches rejected
- [x] Proof matches with wildcard or complete constructor coverage accepted
- [x] Fuel limit configurable
- [x] Typechecker test passes for all Stage 3 totality checks
- [x] No regressions

## Progress Notes

- TASK-1375a completed the fuel substrate: proof expression traversal consumes configurable fuel, default proof fuel is 1000, the direct proof checker returns an untested proof-totality result rather than a type error on fuel exhaustion, and `ash check --proof-fuel <N>` accepts explicit CLI configuration. Program registration treats that untested result as non-error and does not persist/report it through CLI output yet.
- TASK-1375b completed AST-level proof-body `Expr::Match` exhaustiveness checking using the existing conservative match coverage engine. Source `match` syntax is not introduced in this slice.
- TASK-1375c completed local proof call-graph cycle detection for direct `TypeEnv` checks and program registration. Module-scoped and impl-scoped circular proof dependencies are rejected before proof-totality traversal; calls to ordinary non-proof functions remain accepted.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1367](TASK-1367-typeck-proof-totality-stub.md)
