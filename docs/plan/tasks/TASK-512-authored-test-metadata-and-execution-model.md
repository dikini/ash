# TASK-512: Authored Test Metadata and Execution Model

## Status: 📝 Planned

## Description

Define and implement how authored Ash tests are structured, discovered, and executed, including the agreed metadata syntax/structure and authored unit/integration/e2e execution wiring.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-510](TASK-510-test-execution-isolation-and-panic-capture.md)
- [TASK-511](TASK-511-ash-test-library-surface.md)

## Requirements

1. Freeze authored test roots in the repository.
2. Implement the file-level metadata syntax/structure used by authored tests.
3. Bind authored test bodies to the Ash test library surface.
4. Support authored unit, integration, and e2e test execution.
5. Keep authored discovery explicit rather than inferred from arbitrary declarations.

## Likely Files

- Modify runner discovery/execution code
- Add test fixtures under `tests/ash/unit/`, `tests/ash/integration/`, and `tests/ash/e2e/`
- Update docs if metadata syntax needs examples/reference

## Completion Checklist

- [ ] authored test roots frozen
- [ ] file-level metadata parsed
- [ ] explicit authored test declaration/discovery path implemented
- [ ] unit/integration/e2e authored execution supported
- [ ] authored tests use Ash test library surface
