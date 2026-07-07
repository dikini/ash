# TASK-1944: Testing Helper Libraries

**Status:** Planned
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add testing helper libraries over existing Ash test, QuickCheck, law/evidence, coverage, mutation,
and flake orchestration substrates.

## Requirements

- Reuse existing test/evidence/QuickCheck/law substrates.
- Add current-syntax helpers for assertions, property evidence, law evidence, counterexample
  projection, deterministic provider profiles, and common test fixtures.
- Do not create a parallel test runner or hidden evidence mechanism.
- Add examples that parse/check through the real stdlib path.

## TDD Steps

1. Add failing current-syntax testing-helper fixtures.
2. Implement minimal helper modules.
3. Add evidence/counterexample/coverage assertions.
4. Run focused testing-library tests and Rust quality gates.

## Completion Checklist

- [ ] Testing helpers parse/check through stdlib imports.
- [ ] Helpers reuse existing test/evidence substrates.
- [ ] Counterexample and law evidence artifacts remain structured.
- [ ] Deterministic provider profiles compose with test helpers.
