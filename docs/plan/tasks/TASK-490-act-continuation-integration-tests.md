# TASK-490: Integration Tests for Act Continuation Forms

## Status: Done

## Description

Write end-to-end integration tests that exercise the full pipeline (parse → lower → execute) for
all three Act continuation surface forms: `then`, `as`, and `let = cap-call`.

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)

## Dependencies

- [TASK-489](TASK-489-interpreter-act-continuation.md) — interpreter must execute continuations

## Requirements

1. Test `act <cap>(args) then <workflow>`:
   - Action executes, result discarded, continuation runs.
   - Continuation result is the final value.
2. Test `act <cap>(args) as <name>`:
   - Action executes, result bound, continuation runs with name in scope.
   - Name is accessible in subsequent steps.
3. Test `let <name> = <cap>(args)` sugar:
   - Identical behavior to `act <cap>(args) as <name>`.
4. Test bare `act <cap>(args)`:
   - Unchanged terminal behavior, returns action result.
5. Test error cases:
   - Action fails: error propagates, continuation does not run.
   - Guard fails: error propagates, continuation does not run.
6. Use `MockProvider` for deterministic testing.
7. Tests in `crates/ash-interp/tests/` or appropriate integration test location.

## TDD Steps

### Red

- Write integration tests first. They fail until the full pipeline is wired.

### Green

- Verify all integration tests pass.

### Refactor

- Factor out common test setup (mock provider registration, context construction).

## Completion Checklist

- [ ] Integration test for `act ... then`
- [ ] Integration test for `act ... as`
- [ ] Integration test for `let = cap-call` sugar
- [ ] Integration test for bare `act` (regression)
- [ ] Integration test for error propagation
- [ ] All tests pass
- [ ] CHANGELOG.md entry added
