# TASK-514: Property and Small-World Execution

## Status: 🟡 In Progress

## Description

Add bounded seeded property-test execution and bounded small-world execution to the Ash test runner, with reproducible failure reporting and explicit runner controls.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)
- [TASK-512](TASK-512-authored-test-metadata-and-execution-model.md)

## Requirements

1. Add seeded property-test execution with bounded case counts.
2. Add bounded small-world execution with bounded world counts/depth.
3. Expose runner controls for seed/case/world limits.
4. Emit reproducible failure metadata (seed, case index, world index, repro info).
5. Keep panic/failure containment consistent with the rest of the runner.

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
- true generated property inputs derived from stable type/contract metadata
- true small-world state/world exploration rather than bounded reruns
- richer repro artifacts tied to generated inputs/world states rather than only runner-level counters

## Completion Checklist

- [x] seeded property execution implemented and verified at the bounded runner level
- [x] bounded small-world execution implemented and verified at the bounded runner level
- [x] `--seed` / `--max-cases` / `--max-worlds` controls implemented
- [x] reproducible failure output implemented and verified at the bounded runner level
