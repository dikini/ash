# AUDIT-197: Host Boundary Seams

**Task:** [TASK-1925](../tasks/TASK-1925-host-boundary-seam-audit.md)
**Phase:** [PLAN-197](../PLAN-197-HOST-FFI-BUILTINS.md)
**Date:** 2026-07-06

## Purpose

Map the live builtin, provider, runtime-adapter, sandbox, and provenance seams before Phase 197
changes host behavior. Host access is an authority boundary. Existing host-facing code is either
current-state compatibility, target substrate, implementation gap, or legacy reference; none of it
should be treated as a separate semantic foundation or an authority shortcut around row admission.

## Current Host Boundary Seams

| Seam | Current State | Classification | Phase 197 Owner |
|------|---------------|----------------|-----------------|
| Builtin dispatch table | `crates/ash-interp/src/eval/builtins.rs` defines `BuiltinEntry { arity, variadic, implemented }` and `builtin_dispatch_table()`. Provider-backed stdlib surfaces such as HTTP, time, fs, stdio, and quickcheck are forward-declared as `implemented: false`; `process::run`, `process::which`, `act::*`, `proc::*`, and `workflow::*` remain implemented bridge builtins. | Implementation gap for host metadata; legacy reference for `act::*`, `proc::*`, and `workflow::*` bridge names. | [TASK-1926](../tasks/TASK-1926-builtin-host-hook-metadata.md), [TASK-1931](../tasks/TASK-1931-extern-decision-gate.md) |
| Builtin execution path | `crates/ash-interp/src/eval.rs` rejects unimplemented entries, enforces arity, and dispatches to `eval_function_call`. `process::run` directly constructs `std::process::Command`; `process::which` uses host process lookup. | Authority-bypass risk: process host effects can execute from builtin dispatch without explicit hook metadata, row/admission evidence, sandbox policy, or provenance policy. | [TASK-1926](../tasks/TASK-1926-builtin-host-hook-metadata.md), [TASK-1929](../tasks/TASK-1929-host-sandbox-policy-enforcement.md), [TASK-1930](../tasks/TASK-1930-host-provenance-and-redaction.md) |
| Provider trait | `crates/ash-core/src/capability.rs` exposes `CapabilityProvider::{name, effect, observe, execute}` and `CapabilityError`. It has no operation-surface declaration, row identity, resource-use declaration, sandbox policy, provenance policy, adapter identity, or version metadata. | Target substrate with API gaps. | [TASK-1927](../tasks/TASK-1927-provider-authoring-api.md) |
| Standard providers | `crates/ash-engine/src/providers/` implements stdio, fs, HTTP, time, process, MCP, and LLM providers. Fs has allowed paths/read-only/base-dir checks; HTTP has allowed hosts and timeout; process has timeout/allowlist/working-dir; time has mockable clock and sleep. | Target substrate with provider-local policy, not unified host-boundary policy. | [TASK-1927](../tasks/TASK-1927-provider-authoring-api.md), [TASK-1929](../tasks/TASK-1929-host-sandbox-policy-enforcement.md) |
| Provider registration and resource authority | `crates/ash-interp/src/runtime_state.rs` registers capability interface operation surfaces, implementation operation bodies, standard internal pilots, resource instances, and validates host-provider vs implementation authority provenance. | Target substrate for admission/resource authority; missing host adapter and sandbox metadata. | [TASK-1927](../tasks/TASK-1927-provider-authoring-api.md), [TASK-1928](../tasks/TASK-1928-trusted-runtime-adapter-registry.md) |
| Runtime artifacts and adapters | `crates/ash-engine/src/runtime_artifact.rs` builds `RuntimeKernelVerifiedArtifact` from entrypoint metadata, admission profile, boundary bindings, runtime profile/config identity, and optional `runtime_support_identity`. CLI `run` and daemon paths attach selected runtime support identity and application boundary facts. | Target substrate for trusted runtime adapters; missing explicit adapter registry and trust/admission records. | [TASK-1928](../tasks/TASK-1928-trusted-runtime-adapter-registry.md) |
| CLI and daemon host surfaces | `crates/ash-cli/src/commands/run.rs`, `trace.rs`, and `daemon.rs` read source files, write trace/output artifacts, construct runtime boundary bindings, and start long-running service control paths. These are host operations of the toolchain/runtime shell, not Ash program authority. | Current-state runtime host seam; must be reported separately from language-level authority. | [TASK-1928](../tasks/TASK-1928-trusted-runtime-adapter-registry.md), [TASK-1930](../tasks/TASK-1930-host-provenance-and-redaction.md) |
| Constraint enforcement | `crates/ash-interp/src/constraint_enforcement.rs` checks paths, hosts, ports, and permissions from `ConstraintBlock`; unknown constraints are allowed. Provider configs also perform local checks. | Implementation gap: useful checks exist, but unknown sandbox fields fail open and provider configs are not one policy object. | [TASK-1929](../tasks/TASK-1929-host-sandbox-policy-enforcement.md) |
| Provenance and reports | `crates/ash-core/src/provenance.rs` tracks workflow lineage and trace events; `crates/ash-interp/src/capability_provenance.rs` tracks capability event type, direction, value, constraints, effect, and policy decisions; `crates/ash-engine/src/lib.rs` projects execution provenance into reports. | Target substrate with redaction and host-boundary gaps. | [TASK-1930](../tasks/TASK-1930-host-provenance-and-redaction.md) |
| `extern` and old host forms | `NOTE-024` records `extern` as reserved but inactive, `builtin(...)` as the current host-reaching mechanism, and `builtin fn` as removed from target surface. Legacy `Act`, `Proc`, and `Workflow` bridge builtins remain compatibility/reference vocabulary only. | Legacy reference; no new surface/core/IR presence should be added. | [TASK-1931](../tasks/TASK-1931-extern-decision-gate.md) |

## Authority-Bypass Risks

- `process::run` in the interpreter builtin path can spawn a host process without the provider
  allowlist/timeout substrate and without explicit row, sandbox, or provenance metadata. TASK-1926
  must add fail-closed builtin hook metadata, and TASK-1929 must route process execution through a
  sandbox policy before command launch.
- Provider-local config defaults are permissive in several places: filesystem allows all paths when
  `allowed_paths` is empty, HTTP allows all hosts when `allowed_hosts` is `None`, process allows all
  commands when `allowed_commands` is `None`, and MCP defaults to a localhost base URL. TASK-1927
  and TASK-1929 must define whether permissive defaults are allowed only for trusted runtime
  adapters or must be rejected without an admission profile.
- The central provider trait exposes a whole-provider `Effect`, but individual operations may differ
  (`time::sleep` is operational while `time::now` is observational; process `which` differs from
  `run`). TASK-1927 must make per-operation effects and row identities explicit.
- Constraint enforcement currently ignores unknown constraint fields. TASK-1929 must make host
  sandbox policies fail closed for unknown, stale, or malformed policy fields.
- Runtime artifacts carry `runtime_support_identity`, admission profile, and boundary bindings, but
  no registry proves which host adapters are trusted, versioned, sandboxed, or admitted. TASK-1928
  owns that registry.
- Provenance currently records execution lineage and capability events, but host boundary attempts,
  denials, sandbox decisions, adapter identity, and redacted values are not one mandatory event
  family. TASK-1930 owns the redacted host-boundary evidence model.

## Compatibility And Legacy Boundaries

- The old forms `Act`, `Proc`, and `Workflow` are deprecated development forms. Their bridge
  builtins and historical docs may remain as legacy references only and must not gain new surface,
  Core, or IR presence in Phase 197.
- `extern` remains reserved and inactive. Any future revival must lower through the same trusted
  adapter/provider substrate, not a separate host-call path.
- Toolchain host operations in CLI, daemon, module loading, and MCP server code are runtime shell
  behavior. They should be visible in runtime reports and adapter identity, but they do not grant
  Ash program authority by being present in the toolchain.

## Required Follow-Up Ownership

1. [TASK-1926](../tasks/TASK-1926-builtin-host-hook-metadata.md): require builtin host hook metadata
   for implemented host-reaching builtins and fail closed when metadata is missing.
2. [TASK-1927](../tasks/TASK-1927-provider-authoring-api.md): extend provider authoring with
   operation surfaces, row identities, per-operation effects, resources, constraints, and
   provenance policy.
3. [TASK-1928](../tasks/TASK-1928-trusted-runtime-adapter-registry.md): add an admitted trusted
   runtime adapter registry with identity, version, admission source, sandbox policy, and report
   identity.
4. [TASK-1929](../tasks/TASK-1929-host-sandbox-policy-enforcement.md): unify sandbox checks across
   filesystem, process, network, environment, time, LLM, MCP, and denial evidence.
5. [TASK-1930](../tasks/TASK-1930-host-provenance-and-redaction.md): emit redacted provenance and
   report/trace facts for every attempted host boundary crossing, including denials.
6. [TASK-1931](../tasks/TASK-1931-extern-decision-gate.md): decide whether `extern` is still needed;
   keep it inactive unless it lowers through the trusted adapter/provider substrate.
7. [TASK-1932](../tasks/TASK-1932-host-boundary-cross-boundary-fixtures.md): add fixtures proving
   builtin, provider, adapter, sandbox, and provenance alignment.

## Audit Decision

Phase 197 can proceed as a hardening phase over existing builtin, provider, runtime artifact,
constraint, and provenance seams. The current implementation has enough substrate to avoid a new
host semantic island, but implemented process builtins and permissive provider-local defaults must
be fenced before additional host functionality is exposed.
