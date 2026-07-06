# AUDIT-196: Legacy Workflow Boundary Reconciliation

**Task:** [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md)
**Phase:** [PLAN-196](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)
**Date:** 2026-07-06

## Purpose

Fence legacy `workflow` form language before implementing the Phase 196 application/runtime layer.
The target direction is application entrypoint metadata over ordinary checked computations, explicit
admission profiles, role/policy/resource/provider boundaries, reports/traces, supervisors,
services, and typed external actor adapters. The old `workflow` syntax and `Workflow` carriers may
remain as compatibility, current-state, or historical references only.

## Pre-Patch Stale-Claim Evidence

The TASK-1915 stale-claim sweep found target-facing text that could still be read as routing new
runtime work through workflow boundaries:

| Finding | Why It Was Risky | Resolution |
|---------|------------------|------------|
| `SPEC-INDEX.md` read paths stopped at PLAN-195 and did not route application/runtime work through PLAN-196. | Agents could treat process/runtime or workflow references as the latest active routing guidance. | Added a dedicated application/runtime read path and linked PLAN-196 from target effect/type/IR planning. |
| `NOTE-INDEX.md` target-convergence and runtime topics stopped at PLAN-195. | Notes still oriented readers to workflow/process interpretation without the Phase 196 application layer. | Added PLAN-196 to target convergence and runtime organization routing. |
| `SPEC-096b` examples named `workflow/process start`, `workflow boundary`, `workflow_summary`, and `workflow admission`. | These phrases were compatible with the old form but too target-looking for application runtime work. | Reworded target guidance to application/process/runtime admission boundaries and application reports. |
| `SPEC-097b` described profile compatibility as `Proc` or `Workflow` and evidence checks against `workflow/reporting boundaries`. | This preserved deprecated tower names as active type-checking profile names. | Reworded to process-capable/governance-capable runtime profiles and application/reporting boundaries. |
| `NOTE-016` introduced an app layer but still showed `WorkflowInstanceId` beside process/service instances. | Without a Phase 196 pointer, this could be mistaken as a target revival of workflow instances. | Added explicit Phase 196 routing and labeled workflow instances as compatibility-governed computation instances. |

## Post-Patch Routing

- Application/runtime work starts with [PLAN-196](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md) and
  [AUDIT-196: Application Runtime Seams](AUDIT-196-application-runtime-seams.md).
- Target effect/type/IR planning reads PLAN-196 after PLAN-195 when runtime entrypoints,
  admission, reports, services, supervisors, or external actors are in scope.
- Legacy `workflow` syntax is not a target surface, Core term, IR node, public stdlib type, or
  runtime entry path.
- Compatibility diagnostics for old workflow declarations remain valid, but new development must
  route through application metadata over checked computations.

## Remaining Allowed References

The sweep intentionally leaves historical/current-state references in older specs, plans, tasks,
and notes when they are clearly labeled as legacy, compatibility, current-state, historical, or
superseded. Those references are evidence of prior design and migration behavior, not target
implementation instructions.
