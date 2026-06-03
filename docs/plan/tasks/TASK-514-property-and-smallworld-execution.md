# TASK-514: Property and Small-World Execution

## Status: Planned (Phase 76B)

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

### Green

- Implement bounded seeded property and small-world execution with stable reproduction metadata and runner controls.

## Implementation Reality Check

The runner now routes property and small-world kinds through dedicated bounded execution paths, uses
seed/case/world controls honestly in runner output, and avoids leaking generative metadata onto ordinary
unit/integration/e2e tests. However, the current v1 implementation still reruns the same authored test
body in bounded loops rather than exploring true generated inputs or true small-world state spaces.

## Explicit Deferred Follow-Up Items

Deferred until after spec work improvement:
- implemented type/contract-derived generated property inputs following TASK-1010
- implemented small-world state/world exploration following TASK-1010 rather than bounded reruns
- richer repro artifacts tied to generated inputs/world states rather than only runner-level counters

## Baseline Already Satisfied by Phase 76A

- [x] seeded property execution implemented and verified at the bounded runner level
- [x] bounded small-world execution implemented and verified at the bounded runner level
- [x] `--seed` / `--max-cases` / `--max-worlds` controls implemented
- [x] reproducible failure output implemented and verified at the bounded runner-counter level

## Phase 76B Completion Checklist

- [ ] type/contract-derived generated property inputs implemented through TASK-1010 descriptors
- [ ] `SmallWorldState` / `SmallWorldDomain` deterministic enumeration implemented
- [ ] `--max-worlds` bounds actual explored worlds, not bounded reruns
- [ ] repro artifacts include generated input or world snapshots, source/check summary identity, and replay commands
