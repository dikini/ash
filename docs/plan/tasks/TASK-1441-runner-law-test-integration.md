# TASK-1441: Runner Integration — Discover, Generate, Execute Algebra Law Tests

## Status: ✅ Complete

## Description

Wire the law profile data structures from TASK-1440 into the Ash test runner so that `ash test` discovers `law` declarations, generates property tests from them, and reports results.

## Owner

Phase 144 — Stream A (Law Tests)

## Specification References

- `docs/spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md`
- `docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md`
- `docs/plan/tasks/TASK-1029-generated-algebra-law-tests.md`
- `docs/plan/tasks/TASK-1440-law-profile-structures.md`

## Requirements

1. **Law discovery** in test runner:
   - Parse `law` declarations from AST (reuse TASK-1368 extraction)
   - Match law names to `LawProfile` registry entries
   - Skip unknown laws with diagnostic (not silent)

2. **Test generation**:
   - For each matched law + pure carrier instance, generate `LawTestCase`
   - Use seeded RNG for reproducibility
   - Emit test count in runner output

3. **Test execution**:
   - Execute property tests with configurable case count (`--max-cases`)
   - On failure: emit interface, law name, carrier, seed, minimized counterexample
   - On success: report pass count per law family

4. **Tower carrier handling**:
   - Detect tower carriers (`Act`, `Proc`, `Workflow`)
   - Emit explicit `deferred` diagnostic with reason: "bounded equivalence metadata required"
   - Do not generate or execute tower law tests

5. **Runner output**:
   - Human: grouped by interface, with pass/fail/deferred counts
   - JSON: structured array of `{interface, law, carrier, status, seed, counterexample?}`

6. **CLI integration**:
   - `ash test --include-law-tests` (opt-in, default off)
   - Respect `--max-cases`, `--seed` from existing runner flags
   - Respect `--skip-law-tests` / `--skip-law-test=<name>` from TASK-1371

## Acceptance Criteria

- [x] `ash test --include-law-tests` discovers and executes pure carrier algebra law tests
- [x] Non-zero generated test counts reported for String, List, Option, Result carriers
- [x] Tower carriers emit explicit `deferred` diagnostics (not silent skip)
- [x] Failed laws report interface, law name, carrier, seed, counterexample
- [x] `--skip-law-tests` and `--skip-law-test=<name>` still work
- [x] Runner output includes law test summary in both human and JSON modes
- [x] `cargo test -p ash-cli generated_algebra_laws -- --nocapture` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo fmt --check` passes

## Verification

```bash
# Unit tests
cargo test -p ash-cli generated_algebra_laws -- --nocapture

# Integration: run law tests on a sample .ash file with algebra laws
# (requires a test .ash file with law declarations)
cargo run -p ash-cli -- test --include-law-tests tests/fixtures/law_test_sample.ash

# Clippy + fmt
cargo clippy -p ash-cli --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Out of Scope

- Comonad/Kleisli/Cokleisli law tests (deferred; TASK-1036 handoff preserved)
- Tower carrier law execution (deferred)
- Proof body verification/totality checking
- Law test caching or incremental execution

## Notes

- Build on existing `RunnerIntrospectionSnapshot` from TASK-1368
- Reuse existing test runner reporting substrate
- Keep generated tests separate from authored tests in output
- Seed must be reproducible: include seed in failure output for re-running

## Dependencies

- TASK-1440 (law profile structures and generators)
- TASK-1368 (law extraction from AST — already complete)
- TASK-1371 (CLI opt-out flags — already complete)
