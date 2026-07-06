# AUDIT-196: Application Runtime Seams

**Task:** [TASK-1914](../tasks/TASK-1914-application-runtime-seam-audit.md)
**Phase:** [PLAN-196](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)
**Date:** 2026-07-06

## Purpose

Map the current CLI, engine, runtime kernel, daemon, admission, report, trace, process, and external
integration seams before Phase 196 adds an application/runtime layer. This audit treats the legacy
`workflow` form as compatibility syntax only. Any live path that still names workflow is either a
compatibility path to fence in [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md)
or a current implementation label to rename, adapt, or document behind application runtime metadata
in later tasks.

## Current Application-Like Seams

| Seam | Current State | Phase 196 Owner |
|------|---------------|-----------------|
| CLI run entry selection | `crates/ash-cli/src/commands/run.rs` parses file selection, profile flags, trace/dry-run/timeout options, source classification, `fn main` bootstrap, and compatibility `workflow` selection. The default compatibility entry name is still `main`. | [TASK-1916](../tasks/TASK-1916-application-entrypoint-metadata.md), [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) |
| Runtime kernel invocation and artifacts | `OneShotRuntimeKernel`, `RuntimeArtifactBuildRequest`, `build_runtime_kernel_artifact`, runtime identity fields, provider registry identity, and language summaries already form a bounded invocation/report seam. Some artifact evidence still uses `RuntimeKernel<Workflow>` and `Workflow` labels. | [TASK-1916](../tasks/TASK-1916-application-entrypoint-metadata.md), [TASK-1919](../tasks/TASK-1919-application-reports-traces-artifacts.md) |
| Admission profiles | `RunAdmissionProfile` maps CLI names to `AlphaAdmissionProfile`; `AdmissionReport` records selected profile, grants, and whether admission authority was present. This is a useful input seam, but it is still alpha-shaped and not a complete application-boundary model. | [TASK-1917](../tasks/TASK-1917-admission-profile-runtime-boundary.md) |
| Capability, resource, and provider bindings | CLI flags such as `--capability-impl` and `--resource-init`, engine provider registration, provider registry reports, and provider identity rows provide existing boundary hooks. They are not yet a unified role/policy/resource/provider/contract boundary packet. | [TASK-1918](../tasks/TASK-1918-role-policy-resource-boundary-bindings.md) |
| Trace and provenance entrypoints | `ash run --trace`, `execute_with_trace`, `crates/ash-cli/src/commands/trace.rs`, and `ash_provenance` trace sessions already collect execution evidence. Names such as `WorkflowTraceSession` remain legacy vocabulary. | [TASK-1919](../tasks/TASK-1919-application-reports-traces-artifacts.md), [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) |
| Daemon and runtime service shell | `crates/ash-cli/src/commands/daemon.rs` has start, start-execute, reload, control socket, runtime artifact drift checks, provider registry JSON, and process-root registration. It is the closest existing long-running runtime seam, but not yet a service lifecycle model. | [TASK-1921](../tasks/TASK-1921-long-running-service-lifecycle.md), [TASK-1919](../tasks/TASK-1919-application-reports-traces-artifacts.md) |
| Engine parse/check/entry bootstrap | `Engine::parse`, `Engine::check`, module loading, `verify_entry_workflow`, and entry bootstrap diagnostics already split source checking from runtime invocation. Compatibility workflow names still leak through the entry API. | [TASK-1916](../tasks/TASK-1916-application-entrypoint-metadata.md), [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) |
| Legacy workflow adapter | `crates/ash-engine/src/legacy_workflow_adapter.rs`, module loader compatibility extractors, CLI check warnings, and legacy workflow warning tests intentionally preserve old syntax as compatibility input. | [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md) |
| Process and concurrency runtime state | `crates/ash-interp/src/process_registry.rs`, `process_env.rs`, `runtime_state.rs`, channel registry state, process trace facts, and monitor evidence provide the Phase 195 substrate needed by supervisors and services. | [TASK-1920](../tasks/TASK-1920-supervisor-runtime-profiles.md), [TASK-1921](../tasks/TASK-1921-long-running-service-lifecycle.md) |
| External providers and actors | Process, HTTP, LLM, MCP, filesystem, stdio, and custom providers already cross host/runtime boundaries through capability/provider code. They are provider integrations today, not yet typed external actor adapters with sendability and lifecycle policy. | [TASK-1922](../tasks/TASK-1922-external-actor-integration.md), [TASK-1918](../tasks/TASK-1918-role-policy-resource-boundary-bindings.md) |

## Legacy Workflow Compatibility Seams

- Parser, engine, module loader, CLI, LSP, MCP, and test fixtures still contain `workflow main`,
  `WorkflowDef`, and `WorkflowForm` references. These are compatibility or historical carriers, not
  a target semantic foundation for Phase 196.
- `legacy_workflow_adapter.rs` is the correct explicit compatibility boundary pattern: it adapts old
  surface declarations into shared carriers while preserving diagnostics.
- Runtime artifact and trace labels that still spell `Workflow` are target-naming risks. Later tasks
  should either rename them to application/runtime terms or clearly fence them as compatibility
  evidence labels.
- Historical plan/task/spec documents may keep `workflow` vocabulary only when marked as legacy,
  compatibility, or historical context.

## Entrypoint And Admission Risks

- CLI selection still has a compatibility default around a `main` workflow name. Application
  entrypoints need explicit invocation metadata over checked computations.
- Source classification currently distinguishes entry files, modules, and legacy workflow-shaped
  sources across CLI and engine code. Phase 196 should keep this useful split but route target
  entry files through application metadata rather than workflow syntax.
- Admission profile names are explicit CLI inputs, but they do not yet express all boundary
  constraints expected by application entrypoints.
- Admission reports must remain evidence. They must not become a hidden authority grant.

## Boundary And Authority Risks

- Capability implementations, resource initialization, provider registration, role/policy rows, and
  contract evidence are currently adjacent seams rather than one application-boundary packet.
- Provider registry reports are useful for observability, but boundary validation still needs
  fail-closed checks tying selected providers and resources to admission, role, policy, and contract
  obligations.
- Existing provider integrations should be reused, but Phase 196 must not let application entrypoint
  selection bypass handler/provider discharge or row admission checks.

## Report, Trace, And Artifact Risks

- One-shot runtime reports and daemon/runtime artifact reports already carry identity and provider
  registry evidence, but their schema is not yet an application report contract.
- Trace sessions still use workflow vocabulary in several names. This is acceptable only as legacy
  compatibility until [TASK-1919](../tasks/TASK-1919-application-reports-traces-artifacts.md)
  introduces application report and trace bundles.
- Reports, traces, monitor evidence, and runtime artifacts must remain authority-neutral and
  redaction-aware.

## Supervisor And Service Risks

- Phase 195 process registry and runtime trace facts provide the right substrate for supervisors,
  but supervisor profiles do not yet exist as application runtime policy.
- The daemon supports long-running process shape and reload/control behavior, but service lifecycle
  states, health checks, graceful shutdown, forced shutdown, and retention policy need explicit
  application runtime semantics.
- Cancellation and failure propagation must stay grounded in process facts rather than daemon-only
  control flow.

## External Actor Risks

- Existing process, HTTP, LLM, MCP, filesystem, stdio, and custom providers cross runtime
  boundaries, but they do not yet share a typed external actor adapter model.
- External integration must validate inbound and outbound payload types, ownership movement,
  sendability, capability policy, cancellation, retry, and structured failure diagnostics.
- Provider convenience APIs must not imply ambient authority for external actors.

## Required Follow-Up Ownership

1. [TASK-1915](../tasks/TASK-1915-legacy-workflow-form-boundary-reconciliation.md): fence legacy
   `workflow` references and reconcile stale target claims.
2. [TASK-1916](../tasks/TASK-1916-application-entrypoint-metadata.md): introduce application
   entrypoint metadata and invocation packets over checked computations.
3. [TASK-1917](../tasks/TASK-1917-admission-profile-runtime-boundary.md): make admission profiles
   explicit runtime-boundary inputs that fail closed.
4. [TASK-1918](../tasks/TASK-1918-role-policy-resource-boundary-bindings.md): bind roles, policies,
   resources, providers, and contracts at the application boundary.
5. [TASK-1919](../tasks/TASK-1919-application-reports-traces-artifacts.md): stabilize application
   reports, trace bundles, runtime artifacts, monitor evidence, and redaction behavior.
6. [TASK-1920](../tasks/TASK-1920-supervisor-runtime-profiles.md): add supervisor profiles over
   Phase 195 process handles.
7. [TASK-1921](../tasks/TASK-1921-long-running-service-lifecycle.md): define service lifecycle,
   health, reload, shutdown, and retention semantics.
8. [TASK-1922](../tasks/TASK-1922-external-actor-integration.md): add typed external actor adapters
   with sendability and boundary checks.
9. [TASK-1923](../tasks/TASK-1923-application-runtime-cross-boundary-fixtures-and-closeout.md): close
   the phase with cross-boundary fixtures and broad verification.

## Audit Decision

Phase 196 can proceed as a composition and hardening phase over existing seams. The runtime already
has enough entry, admission, provider, report, trace, daemon, process, and external integration
hooks to build an application layer without reviving the legacy `workflow` form. Target work must
route through application metadata over ordinary checked computations, explicit boundary inputs,
authority-neutral evidence artifacts, Phase 195 process facts, and typed external actor adapters.
