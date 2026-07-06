# TASK-1922: External Actor Integration

**Status:** Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Integrate external actors through explicit typed adapters and capability boundaries.

## Requirements

- Register external actor adapters explicitly with type, capability, policy, and ownership metadata.
- Validate inbound and outbound payloads with sendability and ownership checks.
- Surface actor failure, retry, cancellation, timeout, and unsupported protocol diagnostics.
- Preserve reports/traces for external actor calls without leaking secrets or granting authority.

## TDD Steps

1. Add failing adapter registration and payload validation tests.
2. Implement adapter carrier records and runtime boundary validation.
3. Add CLI/engine/runtime fixtures for actor success, failure, timeout, and cancellation.

## Completion Checklist

- [x] External actors cross typed adapter boundaries only.
- [x] Payloads enforce type, sendability, ownership, and capability policy.
- [x] Failures and retries are structured and bounded.
- [x] Reports/traces include redacted actor evidence.

## Evidence

- Added `ExternalActorAdapter`, `ActorProtocol`, `ActorCallPolicy`, `ExternalActorCallRecord`,
  `ActorCallOutcome`, and `ExternalActorDiagnostic` runtime carriers in `ash-core`.
- Added `TraceFactKind::ExternalActor` and `RuntimeTraceEvent::Register` so adapter registration
  and actor calls produce authority-free operational trace evidence.
- Added `RuntimeState` adapter registration, lookup, call recording, failure, timeout, retry, and
  cancellation methods with rendered-schema validation, sendability checks, bounded retry policy,
  retained call records, and redacted trace subjects.
- Added focused tests:
  - `cargo test -p ash-core --test alpha_runtime_kernel_carriers`
  - `cargo test -p ash-interp --test task_1922_external_actor_integration`
