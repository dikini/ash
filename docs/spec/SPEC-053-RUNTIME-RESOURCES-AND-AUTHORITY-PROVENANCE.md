# SPEC-053: Runtime Resources and Authority Provenance

**Status:** Historical/current-state resource substrate; target authority vocabulary superseded by NOTE-022/023/025
**Date:** 2026-04-27
**Promotes:** [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) resource-type, resource-instance, resource-binding, internal-authority, derived-authority, and authority-provenance design direction *(historical vocabulary; see target reconciliation below)*
**Related:** SPEC-003, SPEC-004, SPEC-017, SPEC-019, SPEC-022, SPEC-047, SPEC-048, SPEC-049, SPEC-050, SPEC-051, [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)

## Summary

> **Target reconciliation.** This spec records the Phase 103-era runtime resource and
> authority-provenance substrate. Its resource identity, ownership, split/join, lifecycle,
> and provenance requirements remain useful. Its `capability binding` vocabulary is
> historical/current-state context, not the target authority model. Target-Ash work should
> describe operation requirements as interface/impl-qualified operation identities discharged
> by provider/handler admission, with resources participating as explicit authority and
> provenance inputs.

This specification defines Ash-owned runtime resources and authority provenance. It supplies the resource substrate required by capability interfaces and implementations in [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md).

The central distinction is:

```text
ResourceType      = static declaration of a runtime resource kind
ResourceInstance  = concrete identity-bearing runtime component
ResourceBinding   = scoped reference/admission slot for an instance
CapabilityBinding = effect authority surface backed by host authority, internal resources, or derived dependencies
```

Ash may not manufacture external authority. Ash may create internal authority over Ash-owned resources when the resource has explicit type, identity, owner, lifecycle, access discipline, split/join policy, and provenance behavior.

## 1. Scope and Authority

### 1.1 In scope

This spec defines:

1. resource types as static declarations of Ash-owned runtime resource kinds;
2. resource requirements for capability implementations;
3. resource allocation/admission sites;
4. resource instances and resource bindings;
5. authority provenance categories: host/external, internal, and derived;
6. access, lifecycle, and split/join policy requirements;
7. runtime environment placement across `Act`, `Proc`, and `Workflow`;
8. failure and provenance obligations for resource-backed operations.

### 1.2 Out of scope

This spec does not define:

1. every built-in resource kind;
2. persistence/checkpoint storage schemas;
3. distributed resource ownership;
4. garbage collection or retention policy beyond minimal lifecycle contracts;
5. host-specific permission syntax;
6. final first-class `ResourceRef<T>` value semantics;
7. implementation bodies for capability interfaces; see [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md).

### 1.3 Normative vs informative

Unless marked informative, sections are normative. Conceptual runtime shapes are semantic contracts, not required Rust layouts.

## 2. Resource Type

A resource type is a static declaration of an Ash-owned runtime resource kind.

It may define:

1. representation shape;
2. allowed owner scopes;
3. access modes;
4. lifecycle expectations;
5. split/join/share/move behavior;
6. provenance/failure behavior for operations.

Required conceptual form:

```ash
pub resource type WorkflowKV {
    map: Map<String, String>
}
```

A resource type is not a resource instance and does not allocate state.

## 3. Resource Requirement

A resource requirement is a definition-time dependency on a resource of a given type.

Capability implementations may declare resource requirements:

```ash
requires resource kv: WorkflowKV
```

Conformance rules:

1. A resource requirement names a required binding slot, not a concrete instance.
2. Requirements are resolved only at binding/admission time.
3. An implementation body may access only resource requirements declared in its header.
4. Resource requirement names are scoped to the implementation recipe.

## 4. Resource Allocation Site

A resource allocation site creates or admits a concrete resource instance for a workflow, process, run, test, or effect scope.

Conceptual workflow form:

```ash
workflow test_job
    owns kv: WorkflowKV
{
    ...
}
```

Conceptual host/run form:

```text
run test_job with resource kv = WorkflowKV { map: empty_map() }
```

Conformance rules:

1. Allocation creates a `ResourceInstance` with a stable identity for its lifetime.
2. Admission may accept a host-created or parent-created resource instance if its type and policy match the requirement.
3. A resource allocation must record owner scope and provenance.
4. Resource names introduced by `owns` or equivalent clauses are environment bindings, not ordinary pure local variables.

## 5. Resource Instance

A resource instance is a concrete runtime entity with identity, state, lifetime, and access rules.

Conceptual shape:

```text
ResourceInstance {
  id: ResourceId,
  type_id: ResourceTypeId,
  owner: WorkflowId | ProcessId | EffectScopeId | RunId | TestId,
  state: RuntimeResourceState,
  lifecycle: ResourceLifecycle,
  access_policy: AccessPolicy,
  split_join_policy: SplitJoinPolicy,
  provenance: ResourceProvenance,
}
```

A conforming implementation must preserve enough instance metadata for access checks, provenance, failure reporting, and process/workflow boundary reporting.

## 6. Resource Binding / Handle

A resource binding is a scoped environment entry that names a resource instance.

Examples:

1. `kv` in `owns kv: WorkflowKV`;
2. a dependency slot in `MemoryKV(kv)`;
3. a future first-class `ResourceRef<WorkflowKV>` value, if introduced.

Initial conformance profile:

1. Resource bindings are environment entries, not ordinary pure values.
2. Resource handles are not first-class unless a later spec explicitly introduces them.
3. Resource bindings may be passed to capability implementation recipes at binding/admission time.
4. Runtime lookup is identity-indexed, not ambient untyped context lookup.

## 7. Authority Provenance

Every capability binding and resource-backed operation must have explicit authority provenance.

Minimum provenance categories:

```text
HostAuthority      -- authority over external host resources
InternalAuthority  -- authority over Ash-created resources
DerivedAuthority   -- authority derived from declared dependencies
```

### 7.1 Host / external authority

External authority covers resources outside Ash, including filesystem, network, process execution, system clock, secrets, database sockets, GPUs, and OS APIs.

Conformance rules:

1. External authority must come from host/runtime admission.
2. Ash-defined code may restrict, adapt, delegate, log, or compose external authority.
3. Ash-defined code must not manufacture external authority.

### 7.2 Internal authority

Internal authority covers Ash-owned runtime resources such as in-memory stores, test clocks, replay logs, mailboxes, internal queues, simulation worlds, workflow-local registries, and deterministic random sources.

Conformance rules:

1. Internal authority may be created by Ash allocation/admission sites.
2. Internal authority requires explicit resource type, identity, owner scope, lifecycle, access policy, split/join policy, and provenance behavior.
3. Internal authority does not grant host/external authority.

### 7.3 Derived authority

Derived authority is produced by applying an implementation recipe to existing authority sources.

Examples:

```text
SandboxFs(inner: Fs, root: PathPolicy) => Fs
CachingKV(inner: KVStore, cache: WorkflowKV) => KVStore
RecordingHttp(inner: Http, log: ReplayLog) => Http
```

Conformance rules:

1. Derived authority must declare dependencies explicitly.
2. Derived authority must not widen beyond dependencies and internal resources.
3. Derived authority must preserve provenance links to its inputs.

## 8. Runtime Environment Placement

Resource and capability lookup follows the identity-indexed component model:

```text
(TowerLevel, EntityId, ComponentType, Key) -> Component
```

Examples:

```text
(Workflow, WorkflowId, ResourceInstance, kv)
(Workflow, WorkflowId, CapabilityBinding, store)
(Proc, ProcessId, ResourceInstance, mailbox)
(Effectful, EffectScopeId, CapabilityBinding, current_call)
```

Conformance rules:

1. Workflow admits and owns workflow-scoped resources.
2. Proc projects, splits, moves, shares, or joins resources according to resource policy.
3. Act sequentially invokes capability bindings and resource-backed operations.
4. Pure evaluation cannot access resource instances except via ordinary values explicitly produced by allowed effectful operations.

## 9. Split / Join / Share / Move Policy

A resource used across process operations such as `par`, `scatter`, or `gather` must declare or inherit honest process-boundary behavior.

Minimum policy categories:

```text
ReadOnlyShare     -- branches may share immutable/read-only access
BranchLocalClone  -- each branch receives isolated cloned state
LinearMove        -- one branch receives ownership; others do not
Mergeable         -- branch states can be joined by a specified merge operation
NonShareable      -- resource cannot cross the process split
CommunicationOnly -- resource is accessed only through message/handle protocols
```

Conformance rules:

1. `Proc` environment projection must reject resources without valid split policy for the requested operation.
2. `join`/`gather` must apply merge policies before reporting success where resources are mergeable.
3. Resource conflicts must surface as operational failures with provenance evidence.
4. Implementations must not silently clone linear or non-shareable resources.

## 10. Lifecycle

A resource lifecycle includes at least:

```text
Allocated
Admitted
Active
Splitting
Joined
Released
Failed
```

Conformance rules:

1. Resource instances have a clear owner scope.
2. Releasing an owner scope releases or transfers owned resources according to policy.
3. Failed resource operations must preserve resource identity and operation provenance.
4. Workflow reports must be able to include resource lifecycle/evidence summaries when resources participate in workflow execution.

## 11. Provenance and Failure

Resource-backed operations must record:

1. resource instance identity;
2. resource type;
3. operation name;
4. capability binding, if any;
5. authority provenance category;
6. owner scope;
7. success/failure result;
8. lower cause for wrapped failures.

Failures from resource-backed operations use SPEC-050 operational-failure semantics and may be reinterpreted at workflow boundaries by SPEC-051.

## 12. Implementation Tasks

- [TASK-721](../plan/tasks/TASK-721-write-spec-053-runtime-resources-authority-provenance.md): Write [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) runtime resource and authority contract.
- [TASK-722](../plan/tasks/TASK-722-reconcile-capability-resource-spec-ownership.md): Reconcile existing capability/runtime specs with [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) ownership.
- TASK-726: Add parser and AST substrate for resource types and resource binding clauses.
- TASK-731: Add resource type and binding type checking.
- TASK-732: Add authority provenance static validation.
- TASK-735: Add runtime resource instance carriers.
- TASK-737: Add internal authority allocation and resource admission.
- TASK-738: Add derived authority non-widening runtime checks.
- TASK-739: Add Proc environment split/join resource policy enforcement.
- TASK-740: Add runtime resource/capability integration tests.
- TASK-744: Add standard internal WorkflowKV and FrozenClock/TestClock runtime API pilots.

### 12.1 Implementation Status Note

As of Phase 104, runtime resources remain environment-owned metadata and authority carriers. The standard WorkflowKV and FrozenClock pilots admit constructor-only internal resources and derived implementation-backed bindings through the runtime API; they do not introduce first-class `ResourceRef<T>` values, persistence/checkpointing, or source-level `ash run` lowering for resource declarations.

## 13. Deferred Questions

1. Exact concrete syntax for resource initialization expressions.
2. Whether `ResourceRef<T>` becomes a first-class value.
3. Exact resource-state serialization schema.
4. Exact persistence/checkpoint integration.
5. Default split/join policy for common resource kinds.
6. How resource policies compose across nested workflow/process boundaries.
7. Exact standard library internal resources for tests, replay, clocks, and queues.

## Changelog

### 2026-07-03

- Added target reconciliation notice: SPEC-053's resource substrate remains useful, but NOTE-009-era `capability binding` vocabulary is historical/current-state context for target planning.

### 2026-04-27

- Initial draft promoted from [NOTE-009](../notes/NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), defining resource types, resource requirements, resource allocation, resource instances, resource bindings, authority provenance, runtime placement, split/join policy, lifecycle, provenance, and failure semantics.
