# NOTE-016: Runtime Organization, Behaviours, and Reactive Modes

**Date:** 2026-06-24
**Status:** Living document — exploration in progress
**Purpose:** Separate Ash's runtime organization story from overloaded `workflow`
syntax. Defines the missing meta-layer for multi-app runtime hosting, explicit
bootstrapping, supervision, behaviour patterns, and push/pull/declarative reactive modes.
Companion to NOTE-015 (current-to-target language forms), SPEC-070 (runtime kernel), and
the OTP/process/stream/comonad design notes.

## 0. Motivation

Ash currently looks like it wants to implement agent-like workflows, long-running services,
process trees, streams, and reactive graphs. The problem is that too much of that intent is
hidden behind the word `workflow`.

The overloaded uses are different things:

1. **Computation definition:** a function or workflow body.
2. **Governance boundary:** admission, roles, policies, contracts, reports.
3. **Runtime process:** something scheduled, supervised, messaged, cancelled, or restarted.
4. **Service behaviour:** a reusable server/worker/stage pattern.
5. **Agent loop:** a particular long-running service protocol.
6. **Stream/dataflow:** pull, push, or graph-based propagation.
7. **Application runtime:** an OS-facing unit started by a kernel/daemon.

When these are collapsed into one construct, workflow start, supervision, message routing,
agent lifecycle, and reactive behavior feel magical. The target language direction should
make each layer explicit and composable.

## 1. The Missing Meta-Story: Runtime, Apps, and Instances

SPEC-070 defines a `RuntimeKernel` that can run in one-shot or daemon mode. The missing
layer is the **app**.

An Ash runtime may host more than one app at the same time.

```text
RuntimeKernel
  AppInstance "billing"
    root Supervisor
      child Process/Workflow/Service instances
  AppInstance "agents"
    root Supervisor
      child AgentLoop/ToolServer/Router instances
  AppInstance "monitoring"
    root Supervisor
      child Stream/FRP graph interpreters
```

This is the meta-level distinction:

| Level | Meaning | Example identity |
|---|---|---|
| Runtime kernel | Host container and control plane | `RuntimeKernelId` |
| App definition | Loadable application blueprint | `AppDefinitionId` |
| App instance | One admitted running app | `AppInstanceId` |
| Supervisor tree | Runtime organization inside an app | `SupervisorId`, `ChildId` |
| Process/service instance | Scheduled unit of execution | `ProcessId`, `ServiceId` |
| Workflow instance | Governed computation instance | `WorkflowInstanceId` |
| Graph instance | Running interpreted reactive blueprint | `GraphInstanceId` |

A workflow definition is not an app. A process is not an app. A graph is not an app. An app
is the unit that says which roots, providers, handlers, supervisors, child specs, admission
profiles, and graph interpreters should be started together.

## 2. Bootstrapping Without Magic

The target runtime story should be:

```text
ash run APP_OR_ENTRY
  -> create RuntimeKernel
  -> load modules/artifacts/config
  -> resolve app definition or one-shot entry
  -> admit one AppInstance or one root entry instance
  -> install admitted providers/handlers/resources
  -> start root supervisor or root computation
  -> run until completion/shutdown

ash daemon serve ...
  -> create long-lived RuntimeKernel
  -> index app/workflow/module definitions
  -> accept local start/stop/reload commands
  -> run multiple AppInstances concurrently
```

File presence never starts code. Loading definitions never starts code. Starting is an
explicit admission event.

```text
definition loaded  != instance admitted
provider exists    != authority granted
workflow exported  != service started
graph declared     != interpreter running
```

## 3. Multi-App Runtime Semantics

The runtime kernel can host multiple app instances, but they must not share authority
implicitly.

### 3.1 App isolation

Each `AppInstance` has:

- app identity and artifact identity;
- root supervisor identity;
- app-local provider/resource admission;
- app-local process namespace;
- app-local graph/interpreter namespace;
- app-local report and trace sinks;
- explicit inter-app communication grants, if any.

Default rule:

```text
two app instances in the same RuntimeKernel are isolated unless a capability,
channel, resource, or router explicitly connects them.
```

This lets a local daemon run `billing` and `agents` together without either app gaining the
other app's providers, mailboxes, graph nodes, or report sinks by accident.

### 3.2 App-to-app communication

Inter-app communication should be explicit and capability-like:

```text
App A owns endpoint Orders.Out
App B is admitted to subscribe to Orders.Out
Runtime installs a channel/router/provider grant
```

Possible mechanisms:

1. typed channels between app instances;
2. published service handles with admitted call/cast permissions;
3. event bus providers with topic-level policy;
4. graph edge adapters from one app's output to another app's input;
5. host-level routing capability controlled by the daemon/operator.

No app should discover or call another app merely because both are loaded.

### 3.3 Scheduling and failure domains

One runtime kernel may schedule multiple apps. Failure boundaries should be layered:

```text
process failure       -> handled by local supervisor or observed by parent
supervisor failure    -> app root policy decides restart/escalate
app failure           -> app instance stops or restarts under host policy
runtime kernel failure -> host/OS failure
```

An app crash should not crash unrelated app instances unless the host policy says the
runtime is unhealthy or a shared provider failure forces global shutdown.

## 4. App Definitions

Ash needs an explicit application definition concept. The syntax is open; the semantics are
not.

An app definition is a blueprint:

```text
AppDefinition {
  name
  version/artifact identity
  root supervisor spec
  provider/resource requirements
  admission profile
  child specs
  graph specs
  exported service endpoints
  reports/traces policy
}
```

Illustrative Ash-shaped surface:

```ash
app agents {
    providers {
        http: HttpProvider;
        model: LlmProvider;
    }

    supervisor root: one_for_one {
        child router = service agent_router(config.router);
        child tools = service tool_server(config.tools);
        child monitor = graph agent_metrics_graph(config.metrics);
    }
}
```

This is not proposed final syntax. The important point is the separation:

```text
app definition = runtime blueprint
fn/proc/workflow = computation/service definitions
supervisor = runtime organization
provider admission = authority boundary
graph = reactive blueprint
```

## 5. Supervision Hierarchy

Supervision should be a library/runtime pattern over process effects, not hidden workflow
behavior.

Core pieces:

```text
SupervisorSpec
ChildSpec
RestartPolicy
ShutdownPolicy
ChildEvent
SupervisorStrategy
```

Typical strategies:

```text
one_for_one
one_for_all
rest_for_one
simple_one_for_one / dynamic pool
```

The supervisor owns child lifecycle:

```text
start child
monitor child
observe exit/failure
apply restart policy
escalate if restart intensity exceeded
shutdown children in policy order
```

The runtime owns primitive process identity, scheduling, monitoring, and cancellation. The
supervisor library owns restart strategy.

## 6. Behaviours as Interfaces plus Runners

Erlang behaviours map naturally to Ash interfaces and implementations.

```text
interface + impl + library runner
```

A behaviour should not be a special runtime primitive. It is a reusable protocol over
process, channel, state, and effect operations.

Illustrative shape:

```ash
interface GenServer<S, Req, Reply> {
    init(args: InitArgs) -> {proc yield | r} S;
    handle_call(req: Req, from: From<Reply>, state: S)
        -> {channel send Reply | r} ServerStep<S>;
    handle_cast(msg: Req, state: S) -> {r} ServerStep<S>;
    terminate(reason: StopReason, state: S) -> {r} Unit;
}
```

The runner is ordinary library/runtime code:

```ash
gen_server::start<Impl>(args, options) -> Proc<ServerHandle<Req, Reply>>
```

The runner:

1. starts a process;
2. installs mailbox/endpoint handlers;
3. calls `Impl.init`;
4. loops over messages;
5. calls `handle_call` / `handle_cast`;
6. emits lifecycle events;
7. terminates according to supervisor policy.

Other behaviours can follow the same pattern:

| Behaviour | Runner |
|---|---|
| `GenServer` | mailbox request/reply state machine |
| `Supervisor` | child lifecycle manager |
| `Stage` | stream/dataflow processing stage |
| `Source` | pull or push producer |
| `Sink` | effectful consumer |
| `Router` | message/event routing service |
| `AgentLoop` | agent state machine over tools/model/context |

The behaviour interface gives static shape. The runner gives runtime meaning. The
supervisor tree decides lifecycle.

## 7. Agent-Like Workflows Without Magic

An agent is not a primitive language form.

An agent is usually:

```text
GenServer-like behaviour
  + model/tool capabilities
  + memory/resource access
  + policy/contracts
  + supervision child spec
  + optional stream/event inputs
```

For example:

```text
AgentLoop =
  state: AgentState
  input: UserMessage | ToolResult | Timer | Shutdown
  effects: {cap llm.complete, cap tool.call, resource memory write, policy agent_policy}
  runner: gen_server-like loop
  supervisor: restart transient on recoverable failure
```

This makes agent startup explicit:

```text
app starts root supervisor
root supervisor starts agent child spec
agent runner starts process
process loop receives messages
model/tool calls require admitted effects
```

There is no hidden "workflow engine" deciding when an agent exists.

## 8. Reactive Modes: Pull, Push, and Graph

Ash should not use one stream/workflow construct for every reactive style.

### 8.1 Pull mode: codata and machines

Pull mode means downstream demand asks upstream for the next value.

Useful models:

- Haskell `pipes`;
- Haskell `conduit`;
- Haskell `machines`;
- iterators/generators;
- productive corecursion.

Ash target:

```text
Producer<A>
Consumer<A>
Pipe<A, B>
Machine<I, O>
```

This mode is codata/corecursion-oriented. A pull stream is not a workflow. It is a value or
process-backed object with a `next` protocol.

```text
next : Producer<A> -> {effects} Step<A, Producer<A>>
```

Backpressure is natural: values are produced only when demanded.

### 8.2 Push mode: events and channels

Push mode means upstream emits and downstream reacts.

Useful models:

- channels;
- event buses;
- callbacks;
- effectful `emit`;
- mailbox receive loops.

Ash target:

```text
effect Emit<T>
effect Subscribe<T>
channel send/receive
```

This mode is operational. Buffering, dropping, replay, ordering, fairness, and backpressure
are runtime policies, not pure stream facts.

### 8.3 Declarative FRP graph mode

FRP graph mode should be a blueprint plus interpreter:

```text
GraphDefinition
  nodes
  edges
  input ports
  output ports
  state cells
  clocks/schedulers
  policies/contracts

GraphInterpreter
  starts graph instance
  schedules propagation
  installs providers/handlers
  records traces
```

A graph declaration is data. It does not run until an app/supervisor starts an interpreter
for it.

This avoids making FRP semantics implicit in ordinary `fn` or `workflow` evaluation.

## 9. Data, Codata, Algebra, Coalgebra

The organizing dualities are useful, but they should primarily guide library and type
design.

| Side | Concept | Ash home |
|---|---|---|
| data | finite values, ADTs, records | core type/data language |
| codata | streams, signals, processes observed over time | libraries plus process/runtime support |
| algebra | fold/consume structure | ordinary functions, handlers, consumers |
| coalgebra | unfold/produce observations | producers, machines, process loops |
| recursion | finite or well-founded self-reference | functions, `let-rec`, CPS recursion |
| corecursion | productive generation | streams, machines, process loops |
| monad | effect sequencing | ambient monad plus handlers |
| comonad | contextual observation | focused streams, zippers, lawful contexts |

This does not imply all terms need syntax. Most should be library-level interfaces and
lawful instances.

## 10. Expression Modes vs Organization Modes

Ash needs to keep these separate:

| Category | Examples | Runs by itself? |
|---|---|---|
| Expression mode | `fn`, `do`, `match`, `handle` | No, only when called/admitted. |
| Declaration mode | `type`, `interface`, `impl`, `effect`, `app`, graph blueprint | No, defines artifacts. |
| Organization mode | app spec, supervisor spec, child spec | Starts only by runtime admission. |
| Runtime mode | process instance, workflow instance, graph instance | Yes, after start/admission. |
| Reactive mode | pull stream, push event, interpreted graph | Only through explicit consumer/interpreter. |

This distinction is the antidote to workflow overload.

## 11. Target Layering

Recommended mental model:

```text
Language Core
  functions, data, types, rows, contracts, handlers, Core/CPS

Standard Algebra
  Functor/Applicative/Monad/Comonad/Kleisli/Cokleisli where lawful

Effect Libraries
  capabilities, resources, channels, failure, process operations, extern providers

Runtime Libraries
  Proc, Workflow, supervisors, behaviours, service runners, process registry

Reactive Libraries
  Producer/Consumer/Pipe/Machine, Event/Signal, graph definitions/interpreters

Application Layer
  app definitions, root supervisors, provider admission, startup/shutdown policy

Runtime Kernel
  loads definitions, admits apps/instances, schedules processes, hosts many apps
```

## 12. Resolved Direction

1. `workflow` should not be the universal organizing construct.
2. The runtime can host multiple apps concurrently.
3. Apps are explicit admitted runtime blueprints, not files or workflow definitions.
4. Behaviours are interfaces plus runners.
5. Supervision is a runtime/library pattern over process effects.
6. Agent loops are behaviours plus effects plus supervision, not primitives.
7. Pull streams, push events, and FRP graphs are distinct reactive modes.
8. Graphs are declarations/data interpreted by a runner.
9. Inter-app communication is explicit authority, not ambient discovery.

## 13. To Be Resolved

### 13.1 App surface syntax

Open choices:

1. source-level `app` declaration;
2. external manifest format;
3. ordinary Ash value exported under a known name;
4. generated app specs from package metadata.

The semantic requirement is explicit app definitions and app instances.

### 13.2 Multi-app daemon policy

Questions:

- Can one daemon start apps from multiple roots?
- Are app namespaces globally unique or root-qualified?
- How are shared providers allocated and metered?
- Can one app depend on another app as a service?
- What happens when a shared provider fails?

### 13.3 Supervisor child typing

Heterogeneous child specs require:

- existential packaging;
- erased handles;
- typed service registries;
- or a restricted homogeneous first slice.

### 13.4 Behaviour evidence and specialization

Questions:

- Are behaviours ordinary interfaces?
- Do runners receive explicit dictionaries?
- Can implementations be specialized/monomorphized?
- How are callback contracts inherited and checked?

### 13.5 Pull/push bridge

Need explicit adapters:

```text
Producer -> channel publisher
channel subscriber -> Producer
push graph input -> pull source
pull source -> graph input port
```

Backpressure and buffering semantics must be explicit at each bridge.

### 13.6 Graph interpreter semantics

Open questions:

- push-only, pull-only, or hybrid propagation?
- discrete time, logical clocks, or host time?
- glitch freedom?
- incremental recomputation?
- state cell semantics?
- failure and restart behavior?
- graph hot-reload and migration?

## 14. Migration Implications

Current workflow forms should be reclassified:

| Current workflow-ish form | Better target home |
|---|---|
| `workflow` declaration | compatibility for governed `fn` or app child entry |
| `spawn` | process/supervisor child operation |
| `receive` | channel/mailbox effect or push event loop |
| `observe` | comonadic observation or stream/graph operation |
| `orient`/`propose`/`decide` | domain libraries, policy effects, or agent-loop helpers |
| `yield`/`resume` | process scheduling or protocol continuation operation |
| `maybe`/`must` | failure/contract combinators |

Some of these may remain as sugar, but none should define a separate runtime path.

## 15. Working Principle

The runtime organization rule:

```text
Definitions do not run. Apps start. Supervisors organize. Processes execute.
Workflows govern. Behaviours structure loops. Streams and graphs move data.
The RuntimeKernel hosts all of them under explicit admission.
```

This gives Ash a non-magical answer to:

- how systems start;
- whether multiple apps can run at once;
- how supervision is organized;
- where agent loops live;
- how push and pull dataflow differ;
- why `workflow` no longer has to mean everything.

## 16. References

Internal references:

- [NOTE-015: Current-to-Target Language Forms](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [SPEC-070: Alpha Runtime Kernel and OS-Facing Execution Surface](../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [DESIGN-041: Runtime Regime and OS-Facing Execution Surface](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)
- [SPEC-049: Process Runtime Semantics](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-051: Workflow Semantics](../spec/SPEC-051-WORKFLOW-SEMANTICS.md)
- [WORKFLOW_SPAWNING_AND_SUPERVISION](../design/WORKFLOW_SPAWNING_AND_SUPERVISION.md)
- [OTP-002: OTP-like Functionality in Ash](../ideas/otp/OTP-002-ash-otp-design.md)
- [OTP-003: GenServer-like Design Patterns for Ash](../ideas/otp/OTP-003-genserver-design-patterns.md)
- [SPEC-013: Streams and Event Processing](../spec/SPEC-013-STREAMS.md)
- [DESIGN-NOTE-COMONADIC-COMPUTATION](../design/DESIGN-NOTE-COMONADIC-COMPUTATION.md)
- [effectful-stream-sinks](../design/effectful-stream-sinks.md)
- [SPEC-079: Standard Algebra Comonad and Kleisli Helper Surfaces](../spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)

## 17. Changelog

- 2026-06-24: Initial synthesis note. Introduces app/app-instance as the missing
  meta-layer above workflows/processes, separates runtime organization from expression
  semantics, and classifies behaviours plus pull/push/graph reactive modes.
