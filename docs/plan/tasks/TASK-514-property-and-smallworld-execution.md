# TASK-514: Property and Small-World Execution

## Status: Complete (Phase 76B)

## Description

Add bounded seeded property-test execution and bounded small-world execution to the Ash test runner, with reproducible failure reporting and explicit runner controls.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)
- [DESIGN-023: Small-World Exploration Substrate](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)
- [TASK-1010: Phase 76B Rescope and Spec-Hardening Packet](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Dependencies

- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-512](TASK-512-authored-test-metadata-and-execution-model.md)
- [TASK-1010](TASK-1010-phase-76b-rescope-spec-hardening-packet.md)

## Requirements

1. Add seeded property-test execution with bounded case counts over TASK-1010 `TypeGeneratorDescriptor` inputs where descriptors are finite and supported.
2. Add bounded small-world execution over TASK-1010 `SmallWorldState` / `SmallWorldDomain` models with deterministic enumeration.
3. Expose runner controls for seed/case/world limits.
4. Emit reproducible failure metadata (seed, case index, world index, generated input or world snapshot, source/check summary identity, replay command).
5. Keep panic/failure containment consistent with the rest of the runner.
6. Preserve honest reporting: bounded reruns of one authored body are not true generated property inputs or true small-world exploration.

## Likely Files

- Modify: runner generative execution modules
- Modify: CLI option parsing/output for seed/case/world controls
- Add authored property/small-world fixtures and runner tests

## TDD Steps

### Red

- Add failing tests showing missing seeded property execution, missing bounded small-world execution, or missing reproducibility metadata.
- RED evidence collected: `cargo test -p ash-cli test_runner -- --nocapture` failed before implementation because `SmallWorldDomain`, `SmallWorldState`, `SmallWorldOracle`, `small_world_domains`, and `synthesize_from_snapshot_with_limits` did not exist.

### Green

- Implement bounded seeded property and small-world execution with stable reproduction metadata and runner controls.
- GREEN evidence collected: `cargo test -p ash-cli test_runner -- --nocapture` passed with 46 runner tests after implementation.

## Implementation Reality Check

The runner now has two distinct execution paths:

- authored `property` / `smallworld` tests retain the Phase 76A compatibility behavior and do not claim generated input or world snapshots unless explicit metadata exists
- structured Phase 76B snapshots execute exact finite `TypeGeneratorDescriptor` values as generated property cases and explicit finite `SmallWorldDomain` / `SmallWorldState` models as deterministic small-world cases

The generated property slice supports exact finite descriptor values with a narrow metadata oracle
(`property_holds`) and skips unsupported/empty descriptors instead of reporting success. The small-world
slice supports explicit states plus bool, bounded-int, and explicit-value worlds with narrow
control-state/binding oracles. `--max-worlds` now truncates enumerated metadata worlds in the structured
snapshot path rather than rerunning one authored body, and repro artifacts include source/check summary
identity, seed, case/world indexes, generated input or world snapshots, oracle snapshots, and replay
commands.

## Explicit Deferred Follow-Up Items

Deferred after this completion slice:
- live checked/lowered snapshot production from ordinary CLI source files; tests inject `SuiteConfig::synthesized_snapshots`
- richer property oracles beyond the narrow metadata-backed `property_holds` shape
- richer small-world domain families such as product, list, and state-machine descriptors
- broader synthesized contract/policy/obligation execution beyond the TASK-513 narrow metadata cases

## Baseline Already Satisfied by Phase 76A

- [x] seeded property execution implemented and verified at the bounded runner level
- [x] bounded small-world execution implemented and verified at the bounded runner level
- [x] `--seed` / `--max-cases` / `--max-worlds` controls implemented
- [x] reproducible failure output implemented and verified at the bounded runner-counter level

## Phase 76B Completion Checklist

- [x] type/contract-derived generated property inputs implemented through TASK-1010 descriptors
- [x] `SmallWorldState` / `SmallWorldDomain` deterministic enumeration implemented
- [x] `--max-worlds` bounds actual explored worlds, not bounded reruns
- [x] repro artifacts include generated input or world snapshots, source/check summary identity, and replay commands
