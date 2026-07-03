# TASK-1850: Add admission discharge model

## Description

Add an explicit admission-side discharge model for row requirements.

## Requirements

- Add tests first for operation authority discharge classification.
- Represent operation discharge separately from resource, role, policy, evidence, failure, and unsupported row families.
- Keep the carrier metadata-only: deriving requirements must not mutate engine state.

## Completion criteria

- [x] Tests fail before implementation and pass after.
- [x] Admission code exposes a separate discharge family for each modeled row requirement.
- [x] Operation authority terminology is used for target operation rows.

## Evidence

- Added `RowAdmissionDischarge` and `RowAdmissionRequirement::discharge()` in `crates/ash-engine/src/row_admission.rs`. RED compile failure occurred before the API existed; GREEN verified by `cargo test -p ash-engine --test task_1850_1851_1852_operation_authority_model`.

## Depends on

- TASK-1848.
