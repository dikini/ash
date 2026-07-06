# PLAN-196: Application / Workflow Runtime

**Status:** ✅ Complete (11/11 tasks complete)
**Depends on:** Phase 182 Core Computation Model Conformance, Phase 183 Operation And Authority
Model, Phase 184 Handler / Provider Semantics, Phase 194 Contract And Evidence System, and Phase
195 Process And Concurrency Model.
**Specs/notes:** `SPEC-096b`, `SPEC-097b`, `SPEC-098b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`,
`PLAN-182`, `PLAN-183`, `PLAN-184`, `PLAN-194`, `PLAN-195`, `NOTE-020`, `NOTE-021`, and
`NOTE-035`.
**Audit:** [AUDIT-196: Application Runtime Seams](audits/AUDIT-196-application-runtime-seams.md)

## Goal

Build workflow as an application/runtime layer over ordinary Ash computations, admission profiles,
authority boundaries, contracts, process supervision, reports, traces, and external actor adapters.

## Architecture

The legacy `workflow` form remains a historical compatibility form, not a primitive language island
and not the foundation for new development. Phase 196 uses "workflow runtime" to mean an
application orchestration layer: named application entrypoints select ordinary checked computations,
bind explicit admission profiles, install role/policy/resource/provider boundaries, supervise
processes, and emit reports/traces. The runtime composes earlier layers rather than bypassing them:
Core computation remains the semantic substrate, handlers/providers govern authority, contracts and
evidence govern behavioral obligations, and Phase 195 process facts govern concurrency.

## Scope

- Define application entrypoint metadata and runtime invocation packets.
- Model admission profiles as explicit application-boundary inputs, not ambient grants.
- Bind roles, policies, resources, providers, and contracts at application boundaries.
- Emit application reports, trace bundles, runtime artifacts, and monitor evidence.
- Add supervisor profiles over process handles with restart/cancellation/failure policy.
- Support long-running services with lifecycle, health, reload, and shutdown semantics.
- Add external actor integration through explicit adapters, capability boundaries, and sendability.
- Preserve compatibility diagnostics for the legacy `workflow` form without using it as the target
  runtime foundation.

## Non-Goals

- No revival of the legacy `workflow` form as a target surface, Core, IR, or semantic primitive.
- No hidden authority from application entrypoint selection or admission profile names.
- No distributed actor runtime without explicit external adapter boundaries.
- No scheduler fairness proof or generalized service mesh.
- No implicit long-running service state outside explicit runtime state and trace artifacts.
- No bypass of handler/provider, contract/evidence, process, or sendability checks.

## Design Locks

1. `workflow` as a form is legacy compatibility syntax only; new application runtime work must route
   through ordinary computations and explicit runtime metadata.
2. Application entrypoints are named invocation profiles over checked computations, not a separate
   language stratum.
3. Admission profiles select allowed boundary capabilities and policies but do not grant authority
   unless discharged by existing handler/provider and role/resource mechanisms.
4. Reports and traces are evidence artifacts; they must not mutate admission or authority state.
5. Supervisors observe and control process handles through Phase 195 process semantics.
6. Long-running services are managed runtime instances with explicit lifecycle transitions,
   cancellation, health, reload, and shutdown evidence.
7. External actors cross the runtime boundary through typed adapters with sendability, ownership,
   capability, and policy validation.
8. Legacy `workflow` diagnostics remain honest: compatibility paths may warn or adapt, but target
   development must not depend on legacy form semantics.

## Task Overview

| Task | Description | Estimate | Depends on | Status |
|------|-------------|----------|------------|--------|
| [TASK-1913](tasks/TASK-1913-application-workflow-runtime-plan-packet.md) | Create the Phase 196 plan and task packet | 2h | Phase 195 | ✅ Complete |
| [TASK-1914](tasks/TASK-1914-application-runtime-seam-audit.md) | Audit existing CLI, engine, runtime kernel, daemon, admission, report, trace, and process seams | 4h | TASK-1913 | ✅ Complete |
| [TASK-1915](tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) | Reconcile docs/specs so legacy `workflow` form is compatibility-only for target planning | 6h | TASK-1914 | ✅ Complete |
| [TASK-1916](tasks/TASK-1916-application-entrypoint-metadata.md) | Add application entrypoint metadata and invocation packet carriers over checked computations | 10h | TASK-1915 | ✅ Complete |
| [TASK-1917](tasks/TASK-1917-admission-profile-runtime-boundary.md) | Wire admission profiles to runtime entry boundaries without granting ambient authority | 12h | TASK-1916 | ✅ Complete |
| [TASK-1918](tasks/TASK-1918-role-policy-resource-boundary-bindings.md) | Bind roles, policies, resources, providers, and contracts at application boundaries | 12h | TASK-1917 | ✅ Complete |
| [TASK-1919](tasks/TASK-1919-application-reports-traces-artifacts.md) | Emit application reports, trace bundles, runtime artifacts, and monitor evidence | 10h | TASK-1918 | ✅ Complete |
| [TASK-1920](tasks/TASK-1920-supervisor-runtime-profiles.md) | Add supervisor profiles over process handles with restart/cancel/failure policy | 12h | TASK-1919, TASK-1909 | ✅ Complete |
| [TASK-1921](tasks/TASK-1921-long-running-service-lifecycle.md) | Add long-running service lifecycle, health, reload, shutdown, and retention semantics | 14h | TASK-1920 | ✅ Complete |
| [TASK-1922](tasks/TASK-1922-external-actor-integration.md) | Integrate external actors through explicit typed adapters and capability boundaries | 14h | TASK-1921 | ✅ Complete |
| [TASK-1923](tasks/TASK-1923-application-runtime-cross-boundary-fixtures-and-closeout.md) | Add cross-boundary fixtures, docs, gates, and closeout | 8h | TASK-1916 through TASK-1922 | ✅ Complete |

Estimated implementation effort after the plan packet: 102 hours.

## Required Test Families

### Entrypoint Tests

Prove application entrypoint metadata selects checked computations without introducing a new
`workflow` semantic primitive. Cover CLI and engine invocation paths, missing entrypoints,
ambiguous entrypoints, module imports, and structured diagnostics.

### Admission Boundary Tests

Verify admission profiles are explicit boundary inputs. Profiles must fail closed when missing,
malformed, stale, incompatible with resource/provider bindings, or attempting to widen authority.

### Authority And Contract Tests

Cover role, policy, resource, provider, and contract binding at entry boundaries. Application
runtime must preserve handler/provider discharge, row admission, contract evidence, and process
sendability checks.

### Report And Trace Tests

Assert stable application reports, trace bundles, runtime artifacts, monitor evidence, redaction,
and source/check/runtime identity fields. Reports must not grant authority.

### Supervisor And Service Tests

Cover supervisor restart/cancel/failure policy, child process observation, retained terminal state,
long-running health checks, reload, graceful shutdown, forced shutdown, and runtime retention.

### External Actor Tests

Cover adapter registration, typed inbound/outbound payloads, sendability, ownership transfer,
capability policy, actor failure, cancellation, retry, and structured diagnostics.

## Verification Gates

Each implementation task must run focused tests for touched crates plus:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Phase closeout must run:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

## Stale-Claim Sweep

Before closeout, search live normative docs and code comments for stale claims:

```text
workflow.*primitive language
workflow.*semantic island
workflow form.*target
workflow.*grants authority
application entrypoint.*bypasses admission
admission profile.*ambient authority
supervisor.*bypasses process
service.*implicit state
external actor.*untyped
external actor.*no sendability
report.*grants authority
trace.*mutates admission
```

Historical docs may mention the legacy `workflow` form when clearly marked as compatibility or
historical context. Live guidance must route through application/runtime metadata over ordinary
computations, explicit admission, authority-neutral reports/traces, Phase 195 process facts, and
typed external adapter boundaries.

## Acceptance Criteria

- [x] Phase 196 plan and task files exist and are indexed.
- [x] Legacy `workflow` form is documented as compatibility-only for target planning.
- [x] Application entrypoints select checked computations without becoming a primitive language
      form.
- [x] Admission profiles are explicit runtime-boundary inputs and fail closed.
- [x] Roles, policies, resources, providers, and contracts are bound at application boundaries.
- [x] Reports, traces, runtime artifacts, and monitor evidence are stable and authority-neutral.
- [x] Supervisors use Phase 195 process semantics for restart, cancellation, and failure policy.
- [x] Long-running services have explicit lifecycle, health, reload, shutdown, and retention
      semantics.
- [x] External actor integration uses typed adapters with sendability and capability validation.
- [x] Cross-boundary fixtures cover CLI, engine, runtime, daemon/service, reports, supervisors, and
      external actors.
