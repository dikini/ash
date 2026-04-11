# TASK-510: Test Execution Isolation and Panic Capture

## Status: 📝 Planned

## Description

Add per-test execution isolation, panic capture, timeout handling, and sealed result classification so one test failure or panic does not abort the whole suite.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-509](TASK-509-ash-test-runner-substrate.md)

## Requirements

1. Execute each discovered test inside a fail-contained boundary.
2. Capture panic as a test outcome, not a suite abort.
3. Enforce per-test timeout handling.
4. Distinguish `pass`, `fail`, `panic`, `error`, and `skip` outcomes.
5. Preserve duration and reproduction metadata in the result record.

## Likely Files

- Modify/Create: runner execution modules under `crates/ash-cli/src/commands/` or shared runner modules
- Add tests under `crates/ash-cli/tests/` and any lower-level runner modules

## Completion Checklist

- [ ] per-test isolation implemented
- [ ] panic capture implemented
- [ ] timeout classification implemented
- [ ] result statuses sealed per test
- [ ] suite continues after panic/failure by default
