# TASK-1375: Stage 3 — proof totality checking

## Status: 📝 Planned

## Description

Implement actual totality checking for proof bodies.

## Requirements

This task is split into sub-tasks:
- [TASK-1375a](TASK-1375a-fuel-termination.md): Fuel-based termination analysis
- [TASK-1375b](TASK-1375b-partial-match-detection.md): Partial match detection
- [TASK-1375c](TASK-1375c-circular-proof-detection.md): Circular proof detection

## Acceptance Criteria

- [ ] All sub-tasks complete

## Acceptance Criteria

- [ ] Non-total proofs rejected
- [ ] Total proofs accepted
- [ ] Fuel limit configurable
- [ ] Typechecker test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1367](TASK-1367-typeck-proof-totality-stub.md)
