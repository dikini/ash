# TASK-1920: Supervisor Runtime Profiles

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Add supervisor profiles over process handles with restart, cancellation, and failure policy.

## Requirements

- Model supervisors as runtime profiles over Phase 195 process handles and trace facts.
- Support bounded restart, cancel, child failure, escalation, and terminal reporting.
- Preserve handler/provider, contract, admission, and sendability boundaries for supervised children.
- Reject unsupported supervisor policies fail-closed.

## TDD Steps

1. Add failing supervisor policy and child failure propagation tests.
2. Implement supervisor profile carriers and runtime state integration.
3. Verify process trace and monitor evidence records supervisor decisions.

## Completion Checklist

- [x] Supervisor profiles manage process handles without bypassing process semantics.
- [x] Restart/cancel/failure policy is bounded and diagnostic-rich.
- [x] Unsupported policies fail closed.
- [x] Trace evidence records supervisor decisions.

## Evidence

- Added `SupervisorRuntimeProfile`, `SupervisorPolicy`, `SupervisorDecisionRecord`, and
  `SupervisorDiagnostic` carriers over Phase 195 `ProcessId`/terminal-state semantics.
- Added `RuntimeState` supervisor integration for bounded restart, escalation, explicit child
  cancellation, retained decision reporting, and process trace/monitor evidence.
- Unsupported and authority-widening supervisor policies fail closed at runtime-boundary profile
  construction.
- Focused verification:
  - `cargo test -p ash-core --test alpha_runtime_kernel_carriers supervisor_runtime_profiles_are_authority_neutral_and_fail_closed`
  - `cargo test -p ash-interp --test task_1920_supervisor_runtime_profiles`
