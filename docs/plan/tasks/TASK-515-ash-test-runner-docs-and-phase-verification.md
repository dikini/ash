# TASK-515: Ash Test Runner Docs and Phase Verification

## Status: 🟡 In Progress

## Description

Finalize planning/bookkeeping/docs for the Ash test runner phase and run the final verification gate, including targeted `ash test` smoke runs for authored and synthesized test paths.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-509](TASK-509-ash-test-runner-substrate.md)
- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-512](TASK-512-authored-test-metadata-and-execution-model.md)
- [TASK-513](TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md)
- [TASK-514](TASK-514-property-and-smallworld-execution.md)

## Requirements

1. Update `PLAN-INDEX.md` and related task bookkeeping.
2. Update `CHANGELOG.md`.
3. Run the final verification gate for the affected workspace.
4. Run targeted `ash test` smoke cases for authored tests.
5. Run targeted synthesized-test smoke cases for contracts/policies/obligations.
6. Report residual limitations explicitly if any remain.

## Likely Files

- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Modify: active test-runner design/plan/task docs touched during implementation

## TDD Steps

### Red

- Identify stale docs/bookkeeping and missing verification evidence after the implementation tasks land.

### Green

- Update docs/bookkeeping and run the verification/smoke gate until the recorded closeout state matches the actual repository state.

## Completion Checklist

- [x] PLAN-INDEX updated
- [x] CHANGELOG updated
- [x] final verification commands run successfully against the current implementation
- [x] authored `ash test` smoke cases run successfully against the current implementation
- [x] synthesized test smoke cases run successfully against the current implementation
- [x] residual limitations recorded honestly if any remain
- [ ] phase-level bookkeeping only marks complete behavior that is truly closed for v1
