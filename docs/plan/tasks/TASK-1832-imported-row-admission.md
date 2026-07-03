# TASK-1832: Apply row admission checks across imported callables

## Description

Ensure that row admission checks behave identically for local and imported callables. Imported row requirements must be transported through the module boundary and checked at the same admission point as local row requirements.

## Owner decision gate

D6: How should imported row-bearing callable requirements be checked?

## Requirements

- Imported `callable_row_requirements` and `core_callable_types` must be available to the admission helper in the same shape as local ones.
- Admission checks must aggregate row requirements across all callables reachable from the workflow body, not only local ones.
- Add tests importing row-bearing callables and verifying admission/rejection parity with local equivalents.

## Completion criteria

- [x] Imported row-bearing callables participate in admission checks.
- [x] Local and imported row-bearing callables produce the same admission outcomes.
- [x] Tests cover operation, resource, role, and policy rows imported from a module.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1829, TASK-1830, TASK-1831.
