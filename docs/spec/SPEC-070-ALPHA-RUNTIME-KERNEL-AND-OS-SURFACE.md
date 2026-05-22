# SPEC-070: Alpha Runtime Kernel and OS-Facing Execution Surface

**Status:** Implemented MVP (Phase 123 closeout; see TASK-941 successor evidence plus TASK-942/TASK-943/TASK-944 post-merge remediation)
**Date:** 2026-05-19
**Promotes:** [DESIGN-041](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)
**Builds on:** [SPEC-005](SPEC-005-CLI.md), [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md)
**Related:** [MCE-001](../ideas/minimal-core/MCE-001-ENTRY-POINT.md), [WORKFLOW_SPAWNING_AND_SUPERVISION](../design/WORKFLOW_SPAWNING_AND_SUPERVISION.md)
**Plan:** [PLAN-118](../plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md), [PLAN-119](../plan/PLAN-119-SPEC-069-070-IMPLEMENTED-MVP-CLOSURE.md)
**Implementation Tasks:** [TASK-919](../plan/tasks/TASK-919-design040041-current-state-and-scope-reconciliation.md) through [TASK-944](../plan/tasks/TASK-944-phase123-daemon-admitted-source-config-remediation.md)

## 1. Summary

SPEC-070 defines the alpha OS-facing runtime regime for Ash. Ash has one semantic `RuntimeKernel` with two host modes:

```text
ash run FILE[:WORKFLOW] = one-shot host process
ash daemon serve ...    = long-lived local daemon
```

Both modes execute the same compiled semantics from [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md). They differ in host lifetime and control plane, not language meaning.

## 2. Core concepts

| Concept | Definition |
| --- | --- |
| RuntimeKernel | Host container for roots, compiler/cache pipeline, provider/resource registries, workflow definitions, workflow instances, scheduler/supervisor state, admission state, reports/traces, and optional daemon control endpoint. |
| Workflow definition | A compiled/named workflow exported from a source root/module/artifact. Definitions are indexed; file presence does not execute them. |
| Workflow instance | One admitted execution of a workflow definition with arguments, config, authority, artifact version, and lifecycle state. |
| Process tree | Runtime `Proc`/scheduler tree rooted by a workflow instance. |
| Runtime roots | Explicit source, library, config, state, cache, and log roots used for identity, loading, cache invalidation, and authority boundaries. |

## 3. RuntimeKernel responsibilities

A `RuntimeKernel` owns root configuration, definition indexing, artifact cache selection, TCIR/AMIR/bytecode loading and verification, provider/resource registries, workflow definition registry, workflow instance table, process scheduler/supervisor state, capability/resource admission state, report/audit/trace sinks, Tokio runtime integration, and a daemon control endpoint only when running in local daemon host mode.

The initial implementation may split these responsibilities across existing crates. The public architecture must still converge toward one kernel abstraction instead of separate semantic implementations for CLI and daemon modes.

## 4. One-shot `ash run`

Lifecycle:

```text
create RuntimeKernel
load roots/config/providers
compile/check/select workflow definition
admit one root workflow instance
run until terminal outcome, timeout, cancellation, or host failure
emit report/output
exit with OS status
```

Requirements:

1. `ash run` must not require a daemon.
2. It uses the same definition identity and artifact verification rules as the local daemon.
3. It grants authority only through admission, not by provider/resource existence.
4. Terminal workflow outcomes map to deterministic OS status classes.
5. Reports/traces are emitted even on workflow failure when local report construction remains possible.

Alpha caveat: `FILE[:WORKFLOW]` currently records the workflow suffix in RuntimeKernel identity/report selection surfaces. Semantic execution of arbitrary non-`main` exported workflows remains outside the Implemented MVP until the full workflow-selection path is wired through parser, typecheck, and execution.

## 5. Local daemon

The local daemon is a local-first alpha service using the same `RuntimeKernel`. TASK-929 selected the final command spelling under the existing CLI as `ash daemon ...`.

Alpha daemon scope:

- Unix-domain-socket or equivalent same-user local control surface, with alpha validation of root/socket/state/cache/log path ownership before binding and rejection of pre-existing non-socket control paths;
- list definitions and instances;
- start workflow instance records pinned to the active artifact/source identity, including alpha start args/config/admission-profile request fields; non-default daemon `config_id` values are rejected until config-specific daemon artifacts exist;
- observe instance status and pinned artifact identity; report/log-path projection remains beyond the TASK-929 MVP;
- request cancellation/stop;
- reload roots/config for future starts.

Out of scope: remote/multi-user daemon API, distributed scheduling, cluster service discovery, production init-system integration, and hot-swapping artifacts of already-running instances.

## 6. Runtime roots and definition identity

Definition identity includes root identity, relative module path, exported workflow name, source/artifact version or content hash, and selected library/config profile where relevant.

Roots include source roots, library roots, config roots, state dir, cache dir, and log dir. Roots participate in module identity and cache invalidation.

## 7. Authority and admission

Provider/resource existence is not authority. Admission grants a workflow instance access to selected capabilities/resources under policy.

Requirements:

1. provider registry entries are host resources, not user authority;
2. workflow admission creates explicit grants;
3. `Act` capability invocation checks admitted grant state, and fallback host-provider dispatch must fail closed when no admitted grant or binding authorizes the invocation;
4. `Proc` child processes inherit or derive authority only according to split/join policy;
5. workflow reports record admission and authority-relevant facts needed for audit.

## 8. Reload and artifact lifetime

Successful reload indexes new definitions/artifacts for future starts. Existing running instances keep the artifact/version they were admitted with. Failed reload leaves the previous valid index active. Control responses distinguish compile/check failure from runtime/admission failure. Cache invalidation is keyed by roots, profiles, source hashes, and summary/artifact versions.

## 9. Host-level start vs in-language process start

Host-level start creates a root workflow instance. In-language `proc::par`, `spawn`, `await`, `join`, and workflow operations operate inside an admitted instance's process tree. A workflow definition is not itself a server merely because it is indexed by the daemon.

## 10. Acceptance matrix

| ID | Case | Expected result |
| --- | --- | --- |
| A70-1 | `ash run file.ash:main` finite success | one RuntimeKernel, one instance, emitted report, success OS status |
| A70-2 | `ash run` admission failure | no user code executes; diagnostic distinguishes admission from body failure |
| A70-3 | `ash daemon serve --root DIR --socket PATH ...` | daemon indexes definitions without file-presence execution |
| A70-4 | daemon start command | creates instance record pinned to artifact/source identity with args/config/admission-profile fields, preserving empty-admission defaults and rejecting invalid admission before activation |
| A70-5 | daemon reload while instance runs | running instance keeps old artifact identity; future starts use new artifact identity after successful reload |
| A70-6 | provider exists but not admitted | capability invocation fails at authority boundary |
| A70-7 | child process failure | observed through Proc/Workflow semantics, not daemon host failure unless host breaks |
| A70-8 | same artifact under `ash run` and `ash daemon` | language-level semantics match; host lifetime/control plane differs |

## 11. Implementation tasks

See [PLAN-118](../plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md).

## 12. Changelog

### 2026-05-19

- Initial draft promoted from DESIGN-041 and paired with SPEC-069/PLAN-118.

### 2026-05-21

- Promoted to Implemented MVP after PLAN-119/TASK-941 reconciled Phase 123 successor evidence and TASK-942/TASK-943 completed post-merge remediation for admission-profile rejection, daemon start records, policy-profile grant enforcement, daemon child-failure trace semantics, and run/daemon artifact equivalence while preserving local-only daemon scope and resource-operation enforcement limitations.

### 2026-05-22

- TASK-944 closed the remaining daemon admitted-source/config identity gaps: daemon start-execute now executes from the source bytes already read and hash-checked for admitted-artifact drift, and non-default daemon start `config_id` values fail before instance recording until profile-specific daemon artifacts exist. The public caveats for `FILE[:WORKFLOW]` semantic selection and daemon args/config execution semantics remain explicit alpha boundaries.
