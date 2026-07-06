# TASK-1919: Application Reports Traces Artifacts

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Emit application reports, trace bundles, runtime artifacts, and monitor evidence for application
runtime invocations.

## Requirements

- Record source identity, check identity, entrypoint identity, admission profile, boundary bindings,
  process facts, contract evidence, and terminal outcome.
- Keep reports and traces authority-neutral.
- Support human and JSON CLI output where relevant.
- Preserve stable artifact identity for one-shot and daemon/service callers.

## TDD Steps

1. Add failing report/trace schema tests.
2. Implement report and trace bundle carriers.
3. Wire CLI/engine/runtime emission for success, failure, cancellation, and admission rejection.

## Completion Checklist

- [x] Reports include application invocation identity and terminal outcome.
- [x] Trace bundles include process, contract, admission, and boundary facts.
- [x] Reports/traces do not grant or mutate authority.
- [x] CLI JSON fixtures cover stable schema fields.

## Evidence

- Added `ApplicationRuntimeReport`, `ApplicationTraceBundle`, and `ApplicationTerminalOutcome`
  carriers in `ash-core::runtime_kernel`.
- Runtime reports project source identity, check identity, entrypoint identity, admission profile,
  boundary bindings, process facts, contract evidence, monitor evidence, and terminal outcome
  without authority grants or authority mutation.
- `ash run` JSON kernel reports now include `application_report` for success, execution failure, and
  admission rejection.
- Daemon start/status/list records now retain `application_report`; start-execute and cancel update
  terminal outcome records.
- Focused verification:
  - `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder application_trace_bundle_and_report_project_invocation_identity_without_authority`
  - `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode ash_run_reports_checked_callable_entrypoint_metadata_for_fn_main_source`
  - `cargo test -p ash-cli --test alpha_admission_profile`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_start_protocol_round_trips_args_config_and_admission_profile`
