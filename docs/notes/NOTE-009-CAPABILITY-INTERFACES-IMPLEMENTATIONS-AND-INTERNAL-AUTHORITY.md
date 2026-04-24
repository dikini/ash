# NOTE-009: Capability Interfaces, Implementations, Resources, and Internal Authority

**Date:** 2026-04-24
**Status:** Exploratory design note
**Priority:** High — records the current DX direction before syntax and normative semantics are hardened
**Related:** SPEC-047, SPEC-048, SPEC-049, SPEC-051, NOTE-004, NOTE-007

## 1. Purpose

This note captures the current working model for future Ash capability DX:

- capability interfaces as stateless effectful operation shapes;
- capability implementations as behavior/dispatch recipes satisfying those shapes;
- workflow/process/effect environments as the carriers of state and resources;
- resource types, resource instances, and resource bindings as distinct concepts;
- explicit authority provenance for external, internal, and derived authority;
- late binding between capability interfaces, implementations, and concrete resources.

The goal is to preserve Ash's three-pillar separation:

```text
Pure functions transform values.
Capabilities expose controlled authority surfaces.
Act / Proc / Workflow carry, thread, split, join, admit, and govern effects/resources.
```

This note is intentionally not a syntax proposal. The examples use sketch syntax only to make phase boundaries visible. A later design/spec pass should choose concrete syntax and exact lowering rules.

## 2. Motivation

Ash should support workflows parameterized over capability shapes rather than hardcoded providers. This enables:

1. production and mock implementations of the same effectful surface;
2. deterministic testing through injected internal/simulated capabilities;
3. replay/record capability implementations;
4. adapters such as logging, caching, retry, policy wrapping, and sandboxing;
5. Erlang-behaviour-like workflow skeletons where a generic workflow receives concrete callback capability implementations;
6. workflow/process-local internal worlds that do not correspond to external host resources.

A generic workflow should be able to depend on a capability interface such as `KVStore` while tests bind it to an in-memory implementation and production binds it to a host-backed or remote implementation.

## 3. Definitions

### 3.1 Capability Interface

A capability interface is a stateless declaration of an effectful authority surface.

It defines:

- operation names;
- operation modes such as `observe` and `execute`;
- parameter and return types;
- later, optional laws/contracts/policy metadata.

It does not contain state. It does not allocate resources. It does not choose an implementation.

Sketch:

```ash
capability interface KVStore:
    observe get(key: String) returns Option<String>
  | execute put(key: String, value: String) returns Unit
  | execute delete(key: String) returns Unit;
```

### 3.2 Capability Implementation

A capability implementation is a definition-time recipe that satisfies a capability interface.

It defines operation bodies and declares dependencies it needs to do so. Dependencies may include:

- other capability interfaces;
- resource requirements;
- ordinary configuration values;
- host/runtime provider bindings for primitive external authority.

A capability implementation should not itself be a state container. State belongs to Act/Proc/Workflow environments or is passed explicitly through parameters/handles.

Sketch:

```ash
capability impl MemoryKV for KVStore
    requires resource kv: WorkflowKV
{
    observe get(key: String) returns Option<String> { ... }
    execute put(key: String, value: String) returns Unit { ... }
}
```

This should be read as a recipe:

```text
MemoryKV(kv: WorkflowKV) => KVStore
```

not as an object with hidden fields.

### 3.3 Capability Binding

A capability binding is the admission/binding-time association of:

- a capability interface requirement;
- a chosen implementation recipe;
- concrete dependency bindings, such as resource instances or other capabilities.

Sketch:

```ash
workflow example
    owns kv: WorkflowKV
    uses store: KVStore = MemoryKV(kv)
{
    act execute store.put("a", "b");
    let x = act observe store.get("a");
}
```

Here `store` is a capability binding exposed to the workflow body. It is not merely an ordinary data value; it is an admitted effect-environment binding with authority provenance and access rules.

### 3.4 Resource Type

A resource type is a static declaration of an Ash-owned runtime resource kind.

It may define:

- representation shape;
- access modes;
- lifecycle expectations;
- split/join behavior;
- allowed tower level or scope constraints.

It is not a concrete resource instance.

Sketch:

```ash
resource type WorkflowKV {
    map: Map<String, String>
}
```

The bare phrase `resource WorkflowKV { ... }` should be avoided in future docs unless the type-vs-instance distinction is already clear.

### 3.5 Resource Requirement

A resource requirement is a definition-time dependency on a resource of a given type.

Capability implementations may declare resource requirements without selecting a concrete resource instance.

Sketch:

```ash
requires resource kv: WorkflowKV
```

This means: given a binding to a resource instance of type `WorkflowKV`, this implementation can use it.

### 3.6 Resource Allocation Site

A resource allocation site is a program/admission point that says an Act/Proc/Workflow/run/test scope owns or receives a resource of a given type.

Sketch:

```ash
workflow test_job
    owns kv: WorkflowKV
{
    ...
}
```

or:

```ash
run test_job
    with resource kv = WorkflowKV { map: empty_map() }
```

Allocation/admission creates a concrete runtime resource instance.

### 3.7 Resource Instance

A resource instance is a concrete runtime entity with identity, lifetime, state, and access rules.

Conceptual runtime shape:

```text
ResourceInstance {
  id: ResourceId,
  type: ResourceTypeId,
  owner: WorkflowId | ProcessId | EffectScopeId | RunId | TestId,
  state: RuntimeResourceState,
  lifecycle: ResourceLifecycle,
  access_policy: AccessPolicy,
  split_join_policy: SplitJoinPolicy,
}
```

A resource instance is part of the runtime environment model, not the type layer.

### 3.8 Resource Binding / Handle

A resource binding or handle is how code refers to a particular resource instance.

Examples:

- `kv` in `owns kv: WorkflowKV` as a scoped environment binding;
- a future explicit `ResourceRef<WorkflowKV>` value if handles become first-class;
- an implicit dependency slot inside a capability binding such as `MemoryKV(kv)`.

Whether resource handles become ordinary Ash values is deferred. The initial model should prefer environment bindings unless concrete implementation pressure requires value-level handles.

## 4. Resource vs Ordinary State

Not every piece of state should become a resource.

Ordinary lexical/process state is enough when identity, sharing, lifecycle, provenance, or access discipline is not independently important.

Example: a gen-server-like process can carry callback state as an ordinary loop value:

```ash
let state = act execute callbacks.init(config);
loop receive {
    Call(msg, reply_to) => {
        let (reply, next) = act execute callbacks.handle_call(state, msg);
        send reply_to reply;
        state = next;
    }
}
```

A resource is warranted when the state is an identity-bearing runtime component, such as:

- a process mailbox;
- a workflow-local in-memory store shared through capability bindings;
- a replay log;
- an internal event bus;
- a simulation world;
- a queue used across child processes;
- any component needing explicit split/join/access/lifecycle rules.

Working definition:

```text
Resource = identity-bearing runtime component owned by an Act/Proc/Workflow/run scope whose access or lifecycle must be governed separately from ordinary lexical values.
```

## 5. Authority Provenance

The previous shorthand "authority conservation" is too narrow. The better rule is authority provenance:

```text
Every capability binding must have an explicit source of authority.
```

Ash should distinguish at least three authority sources.

### 5.1 External / Host Authority

External authority is authority over things outside Ash:

- filesystem;
- network;
- process execution;
- system clock;
- secrets;
- database sockets;
- GPU/accelerators;
- OS APIs.

External authority must come from host/runtime admission. Ash-defined code may compose, restrict, delegate, log, or adapt it, but may not manufacture it.

### 5.2 Internal Authority

Internal authority is authority over Ash-owned runtime resources that do not exist outside the Ash run/test/workflow/process world.

Examples:

- in-memory KV store;
- test clock;
- fake HTTP replay log;
- process-local mailbox;
- internal queue;
- simulation world;
- workflow-local registry;
- deterministic random source.

Ash may safely create internal authority if the resource has explicit:

1. type/kind;
2. runtime identity;
3. owning scope;
4. lifecycle;
5. access discipline;
6. split/join/concurrency behavior where needed;
7. provenance/failure behavior for effectful operations.

Internal authority does not violate external authority conservation because it only governs Ash-created resources.

### 5.3 Derived Authority

Derived authority is produced by applying an implementation recipe to existing authority sources.

Examples:

- sandboxed file access derived from a broader file capability;
- caching `KVStore` derived from an inner `KVStore` plus cache resource;
- logging wrapper derived from an inner capability plus logger capability;
- retrying HTTP derived from an HTTP capability plus policy/config;
- mock/replay HTTP derived from an internal replay-log resource.

Derived authority may narrow, adapt, or compose authority; it must not widen authority beyond its dependencies.

## 6. Binding-Time Rule

The resource-capability link should be embodied at capability binding/admission time.

Not at capability interface definition time:

```text
KVStore says what operations exist, not where state lives.
```

Not at resource type definition time:

```text
WorkflowKV says what kind of resource can exist, not which capability exposes it.
```

Not as hidden state inside the implementation:

```text
MemoryKV is a recipe requiring kv: WorkflowKV, not an object containing kv.
```

Binding/admission applies implementation recipes to concrete dependencies:

```text
Resource type:          WorkflowKV
Resource binding:       kv: WorkflowKV
Resource instance:      kv#123 at runtime
Capability interface:   KVStore
Capability impl recipe: MemoryKV(kv: WorkflowKV) => KVStore
Capability binding:     store = MemoryKV(kv#123)
Invocation:             store.get("a"), store.put("a", "b")
```

This late binding enables:

- same interface, multiple implementations;
- same implementation, multiple resource instances;
- test/prod substitution;
- internal simulations;
- adapter/decorator capability implementations;
- clear provenance of which concrete resource/provider backed each effect.

## 7. Semantic Placement in the Tower

The current semantic tower remains:

```text
Pure < Effectful / Act < Proc < Workflow
```

Capability and resource semantics should align with the tower:

| Stratum | Role in this model |
| --- | --- |
| Pure | Defines ordinary values/types/functions. Cannot invoke capabilities. May mention resource/capability types only where allowed by the type/elaboration system. |
| Effectful / Act | Sequentially invokes admitted capability bindings and threads effect/resource environment components. |
| Proc | Creates process identities, derives/splits/joins environments, governs process-local resources such as mailboxes and child handles. |
| Workflow | Admits roles/capabilities/resources, binds implementations to concrete dependencies, owns/governs workflow-scoped resources, and maps lower failures to workflow outcomes. |

Conceptual environment entries fit the existing identity-indexed model:

```text
(Workflow, WorkflowId, ResourceInstance, kv)
(Workflow, WorkflowId, CapabilityBinding, store)
(Proc, ProcessId, ResourceInstance, mailbox)
(Effectful, EffectScopeId, CapabilityBinding, current_call)
(Pure, LexicalFrameId, LexicalBinding, key)
```

## 8. Rules and Invariants

### 8.1 Interface Purity

Capability interfaces are stateless operation shapes. They must not own resource state.

### 8.2 Implementation Statelessness

Capability implementations should be recipes/functions over dependencies, not objects with hidden mutable state. Their dependencies must be explicit as requirements.

### 8.3 State Carrier Rule

State belongs in:

- ordinary lexical values when identity/governance is unnecessary;
- Act/Proc/Workflow/run/test environment resources when identity/governance is necessary;
- explicit parameters/handles when value-level state threading is intended.

### 8.4 Explicit Authority Source

Every capability binding must have a provenance source: host, internal, or derived from declared dependencies.

### 8.5 No External Authority Manufacture

Ash-defined implementations may not create authority over external resources unless that authority is granted by host/runtime admission.

### 8.6 Internal Authority Creation

Ash-defined code may create authority over internal resources allocated by Ash, provided their identity, scope, lifecycle, and access rules are explicit.

### 8.7 Derived Authority Non-Widening

Derived capability bindings may restrict, adapt, log, cache, retry, simulate, or compose dependencies. They must not grant operations/effects beyond what their dependencies and internal resources justify.

### 8.8 Late Capability-Resource Link

The concrete link between a resource instance and a capability implementation is established when a capability binding is admitted/constructed, not when the interface or resource type is declared.

### 8.9 Environment Discipline

Capability/resource lookup should remain identity-indexed and component-based, not an ambient untyped context map.

### 8.10 Concurrency Discipline

A resource used across `Proc` operations such as `par`, `scatter`, or `gather` must declare or inherit honest split/join/share/move behavior before the operation is allowed.

## 9. DX Patterns Enabled

### 9.1 Test Substitution

A workflow depending on `Clock` can bind:

- `SystemClock` in production, sourced from host authority;
- `FrozenClock` in tests, sourced from an internal workflow/test resource.

### 9.2 Replay / Record

A workflow depending on `Http` can bind:

- host-backed HTTP in production;
- replay-backed HTTP in tests using an internal replay-log resource;
- recording HTTP using host HTTP plus an append-only internal log resource.

### 9.3 Capability Adapters

A capability implementation may depend on another capability and produce the same interface:

```text
LoggingHttp(inner: Http, log: Logger) => Http
CachingKV(inner: KVStore, cache: WorkflowKV) => KVStore
SandboxFs(inner: FileSystem, root: PathPolicy) => FileSystem
```

### 9.4 Behaviour-Style Workflows

Generic workflow/process skeletons can depend on callback capability interfaces while carrying state explicitly in the workflow/process.

The callback implementation supplies behavior. The process/workflow owns lifecycle, message loop, state threading, supervision shape, and resource admission.

This mirrors the useful part of Erlang behaviours without turning capability implementations into stateful objects.

## 10. Deferred Questions

1. Concrete syntax for `capability interface`, `capability impl`, resource type declarations, and binding clauses.
2. Whether resource handles become first-class values or remain environment-only in the first design slice.
3. How capability implementation bodies are type-checked against operation modes and effects.
4. How generic capability interfaces interact with current Ash generic/type syntax limitations.
5. Whether capability bindings live in a distinct namespace from values, functions, roles, and resources.
6. Exact representation of host-backed primitive implementations versus Ash-defined implementations.
7. How resource initialization expressions are evaluated and at which tower level.
8. How resource split/join policies are declared, inferred, or inherited.
9. Provenance event shape for internal-resource operations.
10. The relationship between this model and existing `pub capability` stdlib declarations.

## 11. Next Steps

1. Keep this note as the current exploratory marker for the design direction.
2. Later, draft a focused design doc for capability interfaces/implementations/resource bindings once syntax candidates are ready.
3. Then write a narrow normative spec covering definitions, binding-time semantics, authority provenance, and minimal runtime obligations.
4. Avoid implementation tasks until the binding model and namespace/resource distinctions are settled.
