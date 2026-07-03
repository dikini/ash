# TASK-1831: Check role and policy rows against admission authority

## Description

Wire role and policy row requirements to existing role and policy admission checks. Rows such as `role tenant.admin` or `policy pii.redact` require admitted role or policy evidence; rows alone do not grant roles or policies.

## Owner decision gate

D5: How should role/policy rows map to existing role and policy admission?

## Requirements

- In the row-admission helper, derive role and policy requirements from explicit row metadata.
- For each role row item, check against `WorkflowAdmissionRequest.admitted_role` / `active_role` and existing role admission paths.
- For each policy row item, check against existing policy/admission paths where implemented; otherwise fail closed with an unsupported policy diagnostic.
- Missing role or policy authority rejects with a precise `WorkflowFailureKind` diagnostic.
- Satisfied role/policy authority admits through existing paths.
- Add tests for missing, satisfied, and unsupported role/policy authority.

## Completion criteria

- [x] Role row items are checked during admission.
- [x] Policy row items are checked during admission or fail closed as unsupported.
- [x] Missing role/policy authority rejects with a structured diagnostic.
- [x] Satisfied role/policy authority admits through existing paths.
- [x] Tests cover local and imported row-bearing callables.
- [x] `cargo fmt --check`, `cargo clippy`, and `cargo test -p ash-engine` pass.

## Depends on

- TASK-1828 admission carrier.
