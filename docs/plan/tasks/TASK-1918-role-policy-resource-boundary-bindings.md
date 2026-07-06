# TASK-1918: Role Policy Resource Boundary Bindings

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Bind roles, policies, resources, providers, and contracts at application runtime boundaries.

## Requirements

- Integrate application invocation with existing role, policy, resource, provider, row admission,
  and contract/evidence mechanisms.
- Keep boundary bindings explicit and auditable.
- Reject unresolved, unauthorized, stale, or incompatible boundary bindings fail-closed.
- Preserve redaction and evidence identity in diagnostics and reports.

## TDD Steps

1. Add failing tests for accepted and rejected boundary binding combinations.
2. Implement application-boundary binding records over existing runtime state.
3. Verify contracts, evidence rows, and provider frames remain authoritative.

## Completion Checklist

- [x] Boundary binding records cover roles, policies, resources, providers, and contracts.
- [x] Invalid bindings fail closed.
- [x] Reports/traces include redacted boundary evidence.
- [x] Existing authority checks remain the final authority.

## Evidence

- Added `ApplicationBoundaryBindings`, `ApplicationBoundaryBindingManifest`, and structured
  fail-closed diagnostics in `ash-core::runtime_kernel`.
- Threaded boundary binding metadata through RuntimeKernel artifact build requests and invocation
  packets without creating admission grants.
- Added one-shot `ash run` and daemon start/status/list JSON report coverage for provider boundary
  evidence over program arguments.
- Focused verification:
  - `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder boundary_binding`
  - `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode ash_run_reports_provider_boundary_bindings_without_authority_grants`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_start_protocol_round_trips_args_config_and_admission_profile`
