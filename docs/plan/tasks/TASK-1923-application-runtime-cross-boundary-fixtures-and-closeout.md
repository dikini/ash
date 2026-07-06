# TASK-1923: Application Runtime Cross-Boundary Fixtures And Closeout

**Status:** Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Add cross-boundary fixtures and close out Phase 196 with docs, changelog, gates, and review
remediation.

## Requirements

- Cover CLI, engine, runtime, daemon/service, report, supervisor, and external actor behavior.
- Include successful application runtime paths and fail-closed invalid boundary crossings.
- Reconcile PLAN-196, PLAN-INDEX, task statuses, specs, notes, and changelog.
- Run full verification gates and document review findings.

## TDD Steps

1. Add failing cross-boundary fixtures for the completed Phase 196 feature set.
2. Wire fixture execution into focused tests.
3. Run broad verification and stale-claim sweep.
4. Update status surfaces and changelog after evidence is collected.

## Completion Checklist

- [x] Cross-boundary fixtures cover Phase 196 runtime behavior.
- [x] All Phase 196 tasks are complete or explicitly deferred.
- [x] CHANGELOG.md records the completed phase.
- [x] Docs gates, Rust gates, and diff checks pass.
- [x] Review findings are addressed or documented with follow-up ownership.

## Evidence

- Added `crates/ash-interp/tests/task_1923_application_runtime_cross_boundary_closeout.rs`, which
  composes process supervision, retained service lifecycle, external actor adapter calls, trace
  facts, and monitor evidence in one `RuntimeState` without granting provider authority or leaking
  payload contents into actor trace subjects.
- Closeout fixture coverage is paired with the existing Phase 196 focused suites:
  - CLI one-shot reports and daemon/service lifecycle:
    `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode`
    `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane`
  - Engine/runtime artifact reports:
    `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder`
  - Runtime kernel carriers:
    `cargo test -p ash-core --test alpha_runtime_kernel_carriers`
  - Admission/boundary/runtime state:
    `cargo test -p ash-cli --test alpha_admission_profile`
    `cargo test -p ash-interp --test task_1920_supervisor_runtime_profiles`
    `cargo test -p ash-interp --test task_1921_service_lifecycle`
    `cargo test -p ash-interp --test task_1922_external_actor_integration`
    `cargo test -p ash-interp --test task_1923_application_runtime_cross_boundary_closeout`
- Stale-claim sweep ran the PLAN-196 patterns. Findings were limited to historical/compatibility
  docs, the PLAN-196 pattern list itself, and authority-negative comments/status text; no live target
  guidance was found that revives legacy `workflow` as a target primitive, grants authority through
  reports/traces, or treats external actors as untyped/no-sendability boundaries.
