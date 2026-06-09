# TASK-1375: Stage 3 — proof totality checking

## Status: 🔄 In Progress

## Description

Implement actual totality checking for proof bodies.

## Requirements

This task is split into sub-tasks:
- [x] [TASK-1375a](TASK-1375a-fuel-termination.md): Fuel-based termination analysis
- [x] [TASK-1375b](TASK-1375b-partial-match-detection.md): Partial match detection
- [ ] [TASK-1375c](TASK-1375c-circular-proof-detection.md): Circular proof detection

## Acceptance Criteria

- [ ] All sub-tasks complete
- [x] Non-exhaustive proof matches rejected
- [x] Proof matches with wildcard or complete constructor coverage accepted
- [x] Fuel limit configurable
- [ ] Typechecker test passes for all Stage 3 totality checks
- [ ] No regressions

## Progress Notes

- TASK-1375a completed the fuel substrate: proof expression traversal consumes configurable fuel, default proof fuel is 1000, the direct proof checker returns an untested proof-totality result rather than a type error on fuel exhaustion, and `ash check --proof-fuel <N>` accepts explicit CLI configuration. Program registration treats that untested result as non-error and does not persist/report it through CLI output yet.
- TASK-1375b completed AST-level proof-body `Expr::Match` exhaustiveness checking using the existing conservative match coverage engine. Source `match` syntax is not introduced in this slice.
- TASK-1375c still owns circular proof dependency detection.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1367](TASK-1367-typeck-proof-totality-stub.md)
