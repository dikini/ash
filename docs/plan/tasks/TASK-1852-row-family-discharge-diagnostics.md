# TASK-1852: Separate row-family discharge diagnostics

## Description

Separate admission diagnostics for resource, role, policy, evidence, failure, and unsupported row families.

## Requirements

- Add tests first for each modeled row family.
- Resource rows discharge through selected resource authority.
- Role rows discharge through admitted role authority.
- Policy, evidence, and failure rows fail closed with their own discharge-family diagnostics until full discharge implementations exist.
- Unsupported families remain explicit and fail closed.

## Completion criteria

- [x] Tests fail before implementation and pass after.
- [x] Diagnostics name the row family and discharge path.
- [x] No row family is mislabeled as operation authority.

## Evidence

- Added row-family discharge assertions for resource, role, policy, evidence, and failure rows in `crates/ash-engine/tests/task_1850_1851_1852_operation_authority_model.rs`; existing Phase 179 row-admission regressions verify fail-closed diagnostics for currently unsupported families.

## Depends on

- TASK-1850.
