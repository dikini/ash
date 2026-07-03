# TASK-1828: Add explicit row admission requirement carriers

## Description

Define a derived, metadata-only admission-facing requirement carrier that can be computed from `CallableRowRequirementSummary` and `CoreRow` without registering authority. This carrier will be consumed by TASK-1829 through TASK-1831.

## Owner decision gate

D2: What minimal admission carrier should represent explicit row requirements?

## Requirements

- Add a new type in `crates/ash-engine` (suggested module `src/row_admission.rs`) representing an admission-side view of explicit row requirements.
- Carrier must be constructible from a `CallableRowRequirementSummary` and/or `CoreType::Function` row.
- Carrier must distinguish row families: operation, resource, role, policy, process, failure, evidence, effect group.
- Carrier must be pure metadata: no provider registration, no resource initializer selection, no role/policy admission side effects.
- Include structured diagnostic information for unsupported requirement families.
- Add unit tests proving the carrier is metadata-only and round-trips from Phase 178 examples.

## Completion criteria

- [x] `RowAdmissionRequirement` (or equivalent) type defined with clear variants per family.
- [x] Conversion from `CallableRowRequirementSummary` and `CoreRow` implemented and tested.
- [x] Unsupported families produce precise diagnostic placeholders.
- [x] Tests prove no authority registration happens during carrier construction.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1827 audit report.
- Phase 178 row metadata (`CallableRowRequirementSummary`, `CoreRow`).
