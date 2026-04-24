# NOTE-007: Runtime Environment Identity and Components

**Date:** 2026-04-24
**Status:** Draft
**Priority:** High — records the current environment model for `Act`, `Proc`, and workflow semantics
**Related:** DESIGN-030, SPEC-048, SPEC-047, NOTE-006

## 1. Purpose

This note captures the current working model for environment lookup in the Ash semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

The central idea is that runtime context is not one ambient map. Environment lookup is indexed by:

```text
(TowerLevel, EntityId, ComponentType, Key)
```

This supports direct access to the correct contextual data without ambiguous ambient lookup.

## 2. Identity Discipline

Every live context component belongs to an identity-bearing entity.
At minimum, a run starts from a workflow/run identity created by the outside runtime or by another workflow.

Current working identity tower:

```text
WorkflowId / RunId
  owns/adopts ProcessId(s)
ProcessId
  owns/adopts child ProcessId(s), BranchId(s), and effect scopes
BranchId / ProcessId
  owns EffectScopeId(s)
EffectScopeId
  owns sequential effect trace entries
LexicalFrameId
  owns ordinary lexical bindings
```

The exact split between child `ProcessId` and `BranchId` remains open for some operations, but the current async `par` direction strongly favors returning running process handles over anonymous branch values.

## 3. Component Store Model

A practical runtime model may be a typed ECS-style component store partitioned by tower level and entity identity.

Conceptual lookup:

```text
lookup(tower, entity_id, component_type, key, access_mode)
```

Examples:

```text
(Pure, LexicalFrameId, LexicalBindings, name)
(Effectful, EffectScopeId, CapabilitySurface, capability)
(Effectful, EffectScopeId, ProviderRegistry, provider)
(Effectful, EffectScopeId, PolicyContext, current)
(Effectful, EffectScopeId or BranchId, ProvenanceLog, current)
(Proc, ProcessId, MailboxSet, self)
(Proc, ProcessId, CancellationScope, current)
(Workflow, WorkflowId, AdmittedRoles, role)
(Workflow, WorkflowId, WorkflowContract, current)
```

## 4. Access Modes

Minimum access modes for the first model:

```text
Read
Write
Append
Consume
```

Likely follow-on access/splitting modes:

```text
Split
Join
BorrowShared
BorrowExclusive
Refine
```

These should not be added until specific operation semantics require them.

## 5. EffEnv vs ProcEnv

### 5.1 EffEnv

`EffEnv` is the environment needed to perform one sequential effectful computation correctly.

Conceptual Act model:

```text
Act<A> ~= EffEnv -> (EffEnv, A)
```

Operationally:

```text
EffEnv -> Result<(EffEnv, A), EffectFailure>
```

`EffEnv` contains or indexes components for:

- provider registry / provider handles
- capability availability/admissibility surface
- policy context for effect execution
- sequential effect state
- effect facts/refinements
- provenance/audit trace for effect execution
- effect-level resources
- effect failure channel

Effect-level failures include:

- provider unavailable
- policy denied
- invalid action
- invalid arguments
- timeout at provider invocation level
- capability violation

These remain effect-level failures even if workflow later reinterprets them as workflow failures.

### 5.2 ProcEnv

`ProcEnv` is the environment needed to place effectful computations into process structure.

`ProcEnv` contains or indexes components for:

- process identity and parentage
- running process handles (`P<A>` / possible future `Process<A>` spelling)
- branch/process-local context
- mailbox/channel handles
- process-local resources
- cancellation scope
- process failure scope
- scheduler/runtime placement metadata
- child process registry
- split/join policy

Boundary rule:

```text
EffEnv answers: may this effect happen, how is it performed, and what effect trace does it produce?
ProcEnv answers: where is this computation running as a process, how does it compose with sibling processes, and how are process-local states isolated/observed/joined?
```

## 6. Top-Down Admission, Lower-Level Lookup

Operational availability restricts from the top of the tower downward:

```text
outside runtime / `ash run` / another workflow
  starts Workflow
Workflow
  starts/adopts Proc
Proc
  invokes Effectful / Act computation
Effectful / Act
  calls Pure functions
```

Higher strata create, admit, project, or restrict lower-stratum components.
Lower strata can use projected components, but must not manufacture higher-stratum authority.

## 7. `par` and Identity Creation

Current design direction:

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

`par` starts/adopts running processes and returns process handles.

Current answer to the identity split question:

- `par` creates new child processes or workflows, depending on the level where it is interpreted.
- the returned handles are identity-bearing process handles, not a special join token.
- `join`, `gather`, `send`, cancellation, and future mailbox/channel operations target those handles.

This means `par` does not clone one ambiguous context blob. It creates identity-indexed running contexts with explicit parentage.

## 8. Open Questions

1. Exact public spelling of process handle. Current preference: `Process<A>` eventually, with `P<A>` retained as draft shorthand in DESIGN-030/SPEC-048 until naming is finalized.
2. Whether every `par` branch is a child `ProcessId` or whether some lower-level proc operations use `BranchId` under an existing `ProcessId`. Current direction: `par` creates child processes or workflows depending on the level where it is interpreted.
3. Split/join classification for each component type.
4. Static vs runtime component stores and how closely their shapes should mirror each other.
5. Whether access modes beyond `Read`, `Write`, `Append`, and `Consume` are needed in the first implementation slice.
