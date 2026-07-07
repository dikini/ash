# TASK-1944: Testing Helper Libraries

**Status:** Complete
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

- [x] Testing helpers parse/check through stdlib imports.
- [x] Helpers reuse existing test/evidence substrates.
- [x] Counterexample and law evidence artifacts remain structured.
- [x] Deterministic provider profiles compose with test helpers.

## Evidence

- Added `std/src/test/artifact.ash` with pure testing artifact constructors for assertions,
  property evidence, law evidence, counterexamples, coverage, mutation, flake quarantine, and
  provider evidence summaries. Provider summaries reuse existing `std::evidence` helpers rather
  than adding a parallel evidence mechanism.
- Added `std/src/test/fixtures.ash` with deterministic provider-profile and common test-case
  fixture records. These are pure data helpers and do not grant provider authority.
- Added `examples/10-testing-helpers/testing_helpers.ash`, a current-syntax fixture that imports
  the helpers through `std::test`.
- Focused verification:
  `cargo test -p ash-cli --test phase199_testing_helpers -- --nocapture`,
  `cargo test -p ash-cli --test phase199_current_syntax_audit -- --nocapture`, and
  `cargo test -p ash-cli --test example_corpus_check --test stdlib_corpus_check -- --nocapture`.
