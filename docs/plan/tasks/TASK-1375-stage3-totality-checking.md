# TASK-1375: Stage 3 — proof totality checking

## Status: 🔄 In Progress

## Description

Implement actual totality checking for proof bodies.

## Requirements

This task is split into sub-tasks:
- [x] [TASK-1375a](TASK-1375a-fuel-termination.md): Fuel-based termination analysis
- [ ] [TASK-1375b](TASK-1375b-partial-match-detection.md): Partial match detection
- [ ] [TASK-1375c](TASK-1375c-circular-proof-detection.md): Circular proof detection

## Acceptance Criteria

- [ ] All sub-tasks complete
- [ ] Non-total proofs rejected
- [ ] Total proofs accepted
- [x] Fuel limit configurable
- [ ] Typechecker test passes for all Stage 3 totality checks
- [ ] No regressions

## Progress Notes

- TASK-1375a completed the fuel substrate: proof expression traversal consumes configurable fuel, default proof fuel is 1000, the direct proof checker returns an untested proof-totality result rather than a type error on fuel exhaustion, and `ash check --proof-fuel <N>` accepts explicit CLI configuration. Program registration treats that untested result as non-error and does not persist/report it through CLI output yet.
- TASK-1375b and TASK-1375c still own rejecting non-exhaustive matches and circular proof dependencies.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1367](TASK-1367-typeck-proof-totality-stub.md)
