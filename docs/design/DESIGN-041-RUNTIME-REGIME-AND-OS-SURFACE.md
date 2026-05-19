# DESIGN-041: Runtime Regime and OS-Facing Execution Surface

**Status:** Draft design note — promoted to normative draft by [SPEC-070](../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md) and [PLAN-118](../plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)
**Date:** 2026-05-19
**Related:** [DESIGN-040](DESIGN-040-ALPHA-ALGEBRAIC-TOWER.md), [DESIGN-030](DESIGN-030-PROC-LIBRARY-AND-MINIMAL-RUNTIME-SUBSTRATE.md), [SPEC-005](../spec/SPEC-005-CLI.md), [SPEC-021](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-047](../spec/SPEC-047-ACT-MONAD.md), [SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md), [MCE-001](../ideas/minimal-core/MCE-001-ENTRY-POINT.md), [WORKFLOW_SPAWNING_AND_SUPERVISION](WORKFLOW_SPAWNING_AND_SUPERVISION.md)

## 1. Summary

Ash alpha needs an explicit runtime regime. DESIGN-040 defines the algebra/type/IR/execution-substrate direction, but it does not define what OS process starts, what runtime state exists, how workflows are discovered, or how long-running workflow/process trees are hosted.

The runtime direction is:

- Ash has one semantic runtime kernel with two OS-facing modes:
  - one-shot `ash run`, and
  - long-lived `ashd` daemon controlled locally.
- Both modes load configured source/library roots, compile/check workflow definitions, admit workflow instances, and execute them through the same runtime semantics.
- `ash run` is an ephemeral host process for tests, scripts, CI, examples, and finite workflow execution.
- `ashd` is a persistent host process that can run many workflow instances from one or more roots, including long-lived service-like workflows/process trees.
- File presence alone does not execute code. The runtime indexes definitions; workflow instances start only through explicit run/start/autostart policy.
- Provider/resource existence is not authority. Workflow admission grants authority to use selected capabilities/resources.

This note records the OS-facing and runtime-hosting counterpart to DESIGN-040. It is intentionally about runtime regime, not syntax, and not yet a CLI/daemon implementation spec.

## 2. Problem statement

Ash already has many pieces of a runtime story:

- workflows as governed computation units;
- `Act`, `Proc`, and `Workflow` tower semantics;
- capability providers and resource configuration;
- process spawning/handles/scheduler direction;
- Tokio as the Rust async substrate;
- `ash run` as an existing CLI path;
- plans for bytecode/VM execution.

The missing design question is:

```text
How does an Ash program execute as an OS-facing system?
```

More concretely:

- Does the host start one workflow and exit?
- Is there a daemon that can host many workflows?
- Can workflows from separate files act as multiple long-running servers?
- Where do roots, libraries, providers, resources, compiled artifacts, logs, and instance state live?
- What is the lifecycle of a workflow definition vs a workflow instance?
- How does OS process lifetime interact with Ash process/workflow lifetime?

Alpha should answer these questions without requiring distributed runtime, remote control, or full production service management.

## 3. Core decision: one runtime kernel, two host modes

The runtime should be modeled around one internal `RuntimeKernel` used by both `ash run` and `ashd`.

```text
ash run FILE[:WORKFLOW]
  -> create RuntimeKernel
  -> load roots/config/providers
  -> compile/check/select workflow definition
  -> admit one root workflow instance
  -> run until terminal outcome or timeout/cancellation
  -> emit report/output
  -> exit with OS status

ashd serve --root DIR --lib DIR --state DIR
  -> create RuntimeKernel
  -> load/watch roots/config/providers
  -> index workflow definitions
  -> expose local control surface
  -> admit/start/stop many workflow instances
  -> keep providers/resources/process supervisors alive
  -> exit only on operator/system shutdown
```

The two modes differ in host lifetime and control plane, not in Ash semantics.

### 3.1 Why both modes are alpha-relevant

One-shot mode is required for:

- tests and CI;
- examples and tutorials;
- script-like usage;
- deterministic exit-code behavior;
- debugging minimal runtime behavior without daemon state.

Daemon mode is required for:

- long-running workflows;
- service-like process trees;
- shared providers/resources under admission control;
- multiple workflow instances from multiple files;
- supervision/restart models;
- realistic Proc/Workflow operational semantics.

Alpha should not pretend Ash is only a CLI script runner, and it should not require a daemon for simple execution.

## 4. RuntimeKernel responsibilities

`RuntimeKernel` is the host container for Ash execution. It is not itself a workflow.

Conceptual responsibilities:

```text
RuntimeKernel
  - runtime roots and package/library search paths
  - module/workflow definition index
  - compiler/cache pipeline
  - loaded TCIR/AMIR/bytecode artifacts
  - provider registry
  - resource registry
  - workflow definition registry
  - workflow instance table
  - process supervisor/scheduler state
  - capability admission state
  - report/audit/trace sinks
  - Tokio runtime handle or integration point
  - control-plane endpoint in daemon mode
```

The kernel owns host-side runtime state. Ash programs observe only the authority and operations admitted through visible language/runtime APIs.

### 4.1 Non-goal: ambient runtime magic

The kernel must not become an untyped side channel around the language.

Acceptable:

```text
A workflow is admitted with capability binding fs.read:/allowed/path.
The runtime has a filesystem provider, but only admitted calls can use it.
```

Not acceptable:

```text
Any workflow loaded by the daemon can use any provider registered globally.
```

Provider/resource registration makes operations possible. Admission makes operations authorized.

## 5. Definitions, instances, and process trees

The runtime should distinguish three levels:

| Level | Meaning | Runtime identity |
| --- | --- | --- |
| Workflow definition | Compiled, named workflow from a module/file/root | `WorkflowDefinitionId` |
| Workflow instance | One admitted execution of a definition with args/config/context | `WorkflowInstanceId` |
| Process tree | Runtime `Proc`/scheduler tree rooted by the workflow instance | root `ProcessId` plus children |

A workflow definition is not a server by itself. A workflow instance may be finite or long-lived. A workflow instance roots a process tree; some child processes may behave like servers.

### 5.1 Workflow instance record

Conceptual fields:

```text
WorkflowInstance {
  id: WorkflowInstanceId,
  definition: WorkflowDefinitionId,
  origin: ModuleOrigin/FileOrigin,
  args: RuntimeValueMap,
  admission_context: AdmissionContextId,
  root_process: ProcessId,
  status: InstanceStatus,
  report: ReportHandle,
  trace: TraceHandle,
}
```

Possible status values:

```text
PendingAdmission
Admitted
Running
Succeeded
Rejected
Failed
Cancelled
Crashed
```

`Rejected` means admission/policy/requirement failure before ordinary workflow execution. `Failed` means workflow-level failure during execution. `Crashed` is a host/runtime bug or panic boundary, not an Ash-level failure.

### 5.2 Long-running workflows

A workflow may be service-like if it does not terminate normally, returns `Never`, or owns long-lived processes.

Alpha does not need new surface syntax such as `workflow service`. The runtime can support service-like behavior by letting workflow instances and process trees remain running. A later spec may add explicit service declarations or manifest-level service metadata.

## 6. OS-facing mode: `ash run`

`ash run` should remain the simple, one-shot execution surface.

Candidate shape:

```bash
ash run path/to/file.ash
ash run path/to/file.ash::workflow_name
ash run path/to/file.ash --workflow workflow_name
ash run --root src --lib std --lib vendor path/to/file.ash::workflow_name
```

Selection rule:

- If a file/module exposes exactly one runnable public workflow, `ash run file.ash` may select it.
- If it exposes a workflow named `main`, `main` is the default entry only for one-shot compatibility.
- If multiple workflows are runnable and no default is unambiguous, require an explicit workflow name.
- Passing a library file that happens to export `main` does not imply daemon autostart or library-wide side effects.

This preserves the existing `ash run` intuition while making explicit selection available.

### 6.1 One-shot lifecycle

```text
Parse CLI/config
  -> construct ephemeral RuntimeKernel
  -> load roots and providers
  -> resolve selected workflow definition
  -> compile/check/evidence/lower/load artifact
  -> validate input args/config
  -> build admission context
  -> admit root workflow instance
  -> run until terminal outcome, timeout, or signal
  -> emit output/report/trace
  -> shutdown runtime resources
  -> exit
```

`ash run` does not leave child workflow/process state behind after the host process exits. If the selected workflow returns while detached descendants remain, alpha should choose a conservative policy rather than leaving implementation-defined behavior.

Preferred alpha policy:

```text
`ash run` owns a root workflow instance and its process tree.
When the root workflow reaches terminal outcome, the one-shot runtime cancels or drains descendants according to an explicit shutdown policy before process exit.
```

This should replace older implementation-defined descendant behavior in a future SPEC-005/SPEC-021 update.

### 6.2 Exit-code classes

Precise codes belong in SPEC-005/SPEC-021, but the runtime regime should preserve distinct outcome classes:

| Class | Meaning |
| --- | --- |
| success | workflow succeeded and required obligations/reporting completed |
| rejected | admission/requirement/policy failure before execution |
| workflow failure | Ash-level failure during execution |
| compile/load/type error | no runnable instance was admitted |
| runtime configuration error | provider/resource/root/config invalid |
| cancelled | signal/operator cancellation |
| host crash | runtime panic/bug boundary |

The exact integer mapping can remain a spec decision. The important alpha requirement is that these classes are not collapsed into one generic failure.

## 7. OS-facing mode: `ashd`

`ashd` is the long-lived runtime daemon.

Candidate shape:

```bash
ashd serve --root src --lib std --state .ash/state --socket $XDG_RUNTIME_DIR/ashd.sock
ashctl list-definitions
ashctl start path/to/file.ash::workflow_name --arg key=value
ashctl list-instances
ashctl status INSTANCE_ID
ashctl cancel INSTANCE_ID
ashctl logs INSTANCE_ID
ashctl report INSTANCE_ID
ashctl reload
ashctl shutdown
```

Command names are illustrative. The design requirement is a local daemon with a control surface, not these exact flags.

### 7.1 Daemon lifecycle

```text
Parse daemon config
  -> construct persistent RuntimeKernel
  -> bind local control endpoint
  -> load source/library/config roots
  -> index workflow definitions
  -> initialize provider/resource registries
  -> optionally start configured autostart workflows
  -> handle start/status/cancel/reload/shutdown requests
  -> gracefully cancel/drain instances on shutdown
```

Daemon mode should be systemd-friendly:

- foreground by default;
- logs to stdout/stderr or journald-compatible output;
- no mandatory PID file;
- Unix signal handling:
  - `SIGTERM`/`SIGINT`: graceful shutdown;
  - `SIGHUP` or control command: reload roots/config where possible.

### 7.2 Control plane

Alpha should prefer a local-only control plane:

```text
Unix domain socket + small JSON or postcard/bincode protocol
```

Reasons:

- same-user local daemon is enough for alpha;
- Unix socket permissions provide a simple OS security boundary;
- no network/TLS/multi-tenant story is required immediately;
- HTTP/gRPC/MCP can be layered later.

Conceptual requests:

```text
ListDefinitions
StartWorkflow { definition_id | root_path + workflow_name, args, admission_profile }
ListInstances
GetStatus { instance_id }
Cancel { instance_id, reason }
GetReport { instance_id }
TailLogs { instance_id }
ReloadRoots
Shutdown
```

Start requests must refer to indexed definitions under allowed roots, or to explicit files under allowed roots. The daemon should not execute arbitrary paths outside its configured roots.

### 7.3 Autostart policy

File presence must not imply execution.

Daemon startup behavior:

```text
load roots
  -> index definitions
  -> compile/check summaries where configured
  -> expose runnable definitions
  -> start only explicitly configured autostart definitions
```

A later manifest can own autostart declarations, for example:

```toml
[[workflow]]
file = "services/chat.ash"
name = "ChatServer"
autostart = true
restart = "on-failure"
```

Alpha may include the manifest shape as design pressure, but the key invariant is that autostart is explicit.

## 8. Runtime roots and directory model

For runtime purposes, assume the host presents a set of directories containing appropriate Ash libraries, source files, configuration, and state.

Conceptual root set:

```text
RuntimeRoots {
  source_roots: Vec<PathBuf>,
  library_roots: Vec<PathBuf>,
  config_roots: Vec<PathBuf>,
  state_dir: PathBuf,
  cache_dir: PathBuf,
  log_dir: Option<PathBuf>,
}
```

Suggested meanings:

| Root | Purpose |
| --- | --- |
| source roots | project/application Ash files containing workflow definitions/modules |
| library roots | stdlib, installed libraries, vendor/project libs |
| config roots | runtime config, provider/resource config, admission profiles |
| state dir | daemon run database, instance records, socket/pid metadata if needed |
| cache dir | compiled TCIR/AMIR/bytecode artifacts and summary cache |
| log dir | logs, traces, reports if not using stdout/journald/external sink |

Default locations should follow platform conventions where possible. Project-local mode may use `.ash/`:

```text
.ash/
  config.toml
  state/
  cache/
  logs/
```

Library/source roots may include:

```text
src/
lib/
std/
vendor/
```

The exact layout belongs in a runtime/CLI spec. DESIGN-041 only requires that roots be explicit runtime inputs and part of module identity/cache invalidation.

### 8.1 Module and definition identity

Definition identity should not be a bare workflow name.

A stable identity should include:

```text
root identity + relative module path + exported workflow name + content/hash/version facts
```

This prevents collisions when two roots contain `main` or `ChatServer`.

The daemon control API should prefer stable definition IDs or fully qualified file/module selectors. Human-friendly names may be aliases, not authority.

## 9. Provider and resource scoping

The daemon introduces provider/resource lifetime questions that one-shot execution can avoid.

Provider/resource scopes should be explicit:

| Scope | Lifetime | Example |
| --- | --- | --- |
| runtime-global | daemon lifetime | shared HTTP client pool, database pool |
| workflow-instance | one workflow instance | per-run temp dir, per-run transaction/session |
| process-local | one process/subtree | mailbox-local state, branch-local resource |
| action-local | one operation | opened file handle for one read/write action |

Rule:

```text
Provider/resource lifetime is not the same as authority.
```

A daemon may keep a runtime-global provider alive, but a workflow instance may use it only if its admission context grants the relevant capability/resource binding.

### 9.1 Admission context

A workflow start request should produce an `AdmissionContext` containing at least:

```text
AdmissionContext {
  workflow_definition: WorkflowDefinitionId,
  caller/operator identity if available,
  selected provider/resource bindings,
  capability grants,
  role grants,
  policy profile,
  input args/config,
  report/audit sink selection,
}
```

Alpha can keep caller identity simple: local same-user operator. The important part is that runtime-global providers are not ambient authority.

## 10. Compilation, cache, and reload

Both host modes should use the same compilation pipeline:

```text
source/library roots
  -> module graph and summaries
  -> typecheck/evidence resolution
  -> TCIR
  -> AMIR
  -> bytecode/loadable artifact
  -> runtime load/admission/execution
```

One-shot mode may use cache opportunistically, but must work without a daemon.

Daemon mode may cache aggressively and reload roots/config.

Reload should be conservative:

- existing running instances continue on the artifact/version they were admitted with unless explicitly migrated/restarted;
- new starts use the new compiled definition after successful reload;
- failed reload preserves the prior runnable set;
- control API reports active definition/artifact versions.

This avoids mutating a running workflow's semantics underneath it.

## 11. Workflow instance lifecycle

A start request follows this conceptual path:

```text
Resolve workflow definition
  -> validate args/config
  -> compile/load artifact if needed
  -> build admission context
  -> evaluate requirements/policy
  -> allocate WorkflowInstanceId
  -> allocate root ProcessId
  -> start root process/workflow boundary
  -> run/suspend/wait/cancel according to host mode
  -> commit report/outcome
```

Terminal outcomes:

| Outcome | Meaning |
| --- | --- |
| `Succeeded(value, report)` | normal workflow completion with required reporting |
| `Rejected(reason)` | admission/requirement/policy failure before ordinary execution |
| `Failed(WorkflowFailure)` | workflow-level failure during execution |
| `Cancelled(reason)` | operator/signal/supervisor cancellation |
| `Crashed(RuntimeFault)` | host/runtime fault outside Ash semantics |

Daemon mode persists enough instance/outcome metadata for status/report queries. One-shot mode emits the outcome and exits.

## 12. Relationship to spawn, Proc, and service-like workflows

`ashd` does not make every workflow a server. It hosts workflow instances whose process trees may include long-lived server-like processes.

Relationship:

```text
RuntimeKernel hosts WorkflowInstance(s)
WorkflowInstance roots Proc process tree
Proc process tree may spawn long-lived service processes
Workflow governance wraps admission, obligations, reports, and failure reinterpretation
```

`spawn` inside Ash remains an Ash operation governed by `Proc`/`Workflow` semantics. Starting a workflow from `ashctl start` is host admission of a new root workflow instance. These are related but not identical:

| Operation | Initiator | Boundary |
| --- | --- | --- |
| `ash run` | OS user/CLI | creates ephemeral kernel and root workflow instance |
| `ashctl start` | daemon control client | asks persistent kernel to admit root workflow instance |
| Ash `spawn`/`par` | running Ash computation | creates child process/workflow/process handle under current runtime context |

This distinction prevents host control operations from being confused with language-level process spawning.

## 13. Security and authority boundary

Alpha should remain local-first:

- same-user daemon;
- Unix socket permissions;
- no remote daemon;
- no TLS/auth/multi-tenant story;
- no distributed cluster.

Still required:

- configured roots restrict what source files can be loaded;
- provider/resource config restricts host access;
- admission restricts capability/resource use by each workflow instance;
- control socket is privileged because it can start/cancel workflows;
- logs/reports should not leak secrets beyond configured sinks.

Remote control, multi-user auth, distributed workflow scheduling, and cluster supervision are future work.

## 14. Interaction with existing specs

### 14.1 SPEC-005 CLI

[SPEC-005](../spec/SPEC-005-CLI.md) currently defines `ash run` as creating an OS process, executing `main`, and exiting when `main` completes. It says descendants after `main` completion are implementation-defined.

DESIGN-041 suggests future SPEC-005/SPEC-021 updates:

- preserve `ash run` as one-shot mode;
- make workflow selection explicit for multi-workflow files;
- replace implementation-defined descendants with explicit shutdown/drain/cancel policy;
- add root/library/config/cache flags or reference a runtime-roots spec;
- preserve distinct outcome classes for exit-code mapping.

### 14.2 MCE-001 entry point

[MCE-001](../ideas/minimal-core/MCE-001-ENTRY-POINT.md) asked how an Ash program starts and explored special entry syntax.

DESIGN-041 chooses an OS/runtime answer first:

```text
The host starts a workflow definition by explicit selection/admission.
No special source-level entry annotation is required for alpha.
```

`main` may remain a one-shot default selection convention, but it is not the general daemon startup model.

### 14.3 Proc/Workflow specs

[SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), and [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md) own in-language process/workflow behavior.

DESIGN-041 owns the host boundary around those semantics:

- kernel lifetime;
- root workflow instance admission;
- daemon control requests;
- provider/resource lifetime scopes;
- OS process shutdown and reload behavior.

It must not redefine `Proc` scheduling or workflow obligations internally, but it should define which runtime object hosts them.

### 14.4 DESIGN-040

[DESIGN-040](DESIGN-040-ALPHA-ALGEBRAIC-TOWER.md) says alpha needs a bounded executable pure/effect/process/workflow execution path. DESIGN-041 supplies the OS-facing regime for that path.

Together:

```text
DESIGN-040: What computations mean and how accepted programs lower/execute.
DESIGN-041: What host runtime starts, hosts, controls, and shuts them down.
```

## 15. Non-goals

Alpha runtime regime does not require:

1. remote daemon or network API;
2. distributed runtime cluster;
3. multi-user authentication/authorization beyond local OS socket permissions;
4. hot migration of already-running instances to newly reloaded code;
5. production-grade service manager replacement;
6. arbitrary plugin loading from untrusted paths;
7. container/sandbox orchestration;
8. source-level `workflow service` syntax;
9. full runtime self-hosting in Ash;
10. long-term stable binary bytecode format before the bytecode spec is ready.

## 16. Spec-update starting points

When promoting this note toward implementation-grade work, split it into specs/plans rather than one mega-spec.

Recommended packets:

1. Runtime roots and module-definition identity:
   - root kinds;
   - module/workflow definition IDs;
   - cache invalidation inputs;
   - duplicate-name behavior.
2. One-shot `ash run` runtime contract:
   - workflow selection;
   - input/admission/config;
   - process-tree shutdown policy;
   - exit-code class mapping;
   - relationship to `ash trace`.
3. RuntimeKernel API and provider/resource scope:
   - kernel responsibilities;
   - provider/resource lifetime vs authority;
   - admission context shape;
   - one-shot vs daemon construction.
4. Daemon/control-plane contract:
   - `ashd` lifecycle;
   - local socket protocol;
   - `ashctl` command surface;
   - instance table/status/report/log operations;
   - reload semantics.
5. Workflow instance lifecycle and persistence:
   - definition vs instance IDs;
   - instance states/outcomes;
   - report/trace persistence;
   - cancellation/shutdown behavior.
6. Runtime observability and reports:
   - logs/traces/reports;
   - error classification;
   - status query payloads;
   - secrets/redaction policy.

## 17. Design position to preserve

```text
Ash has one runtime semantics and one runtime kernel.
`ash run` and `ashd` are two host-lifetime modes over that kernel.
Workflow definitions are indexed, not automatically executed.
Workflow instances are explicitly admitted starts of definitions.
Provider/resource existence is not authority.
Daemon mode hosts many workflow instances and process trees, including long-lived service-like ones.
One-shot mode remains first-class for scripts, tests, examples, and CI.
```
