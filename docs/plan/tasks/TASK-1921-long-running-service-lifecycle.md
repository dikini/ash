# TASK-1921: Long-Running Service Lifecycle

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Add long-running service lifecycle, health, reload, shutdown, and retention semantics.

## Requirements

- Represent services as managed runtime instances with explicit lifecycle state.
- Support start, health, reload, graceful shutdown, forced shutdown, terminal retention, and report
  retrieval.
- Keep service state explicit in runtime state and trace artifacts.
- Preserve admission, authority, contract, process, and supervisor boundaries across lifecycle
  transitions.

## TDD Steps

1. Add failing lifecycle tests for start/health/reload/shutdown/retention.
2. Implement service lifecycle state and daemon/runtime integration.
3. Verify traces and reports distinguish service, process, and application outcomes.

## Completion Checklist

- [x] Service lifecycle state is explicit and inspectable.
- [x] Reload and shutdown semantics are bounded and fail closed.
- [x] Retained reports/traces remain stable after terminal state.
- [x] Service state does not bypass admission or authority checks.

## Evidence

- Added `ServiceId`, `ServiceRuntimeRecord`, service lifecycle/health/shutdown enums, and
  fail-closed lifecycle diagnostics.
- Added `RuntimeState` service lifecycle registry with start, health, reload, graceful shutdown,
  forced shutdown, terminal retention, and service trace facts.
- Added distinct `Service` trace facts plus health/reload/shutdown trace events so service outcomes
  remain distinguishable from process and application outcomes.
- Daemon start/status/list/cancel/reload responses now expose retained `service_lifecycle` records.
- Focused verification:
  - `cargo test -p ash-interp --test task_1921_service_lifecycle`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_start_protocol_round_trips_args_config_and_admission_profile`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_serve_indexes_definitions_without_running_workflows`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane ashd_reload_updates_definition_table_and_preserves_kernel_mode`
