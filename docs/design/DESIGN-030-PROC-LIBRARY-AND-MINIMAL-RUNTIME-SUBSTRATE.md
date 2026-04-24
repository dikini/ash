# DESIGN-030: Proc Library and Minimal Runtime Substrate

**Status:** Draft
**Date:** 2026-04-23
**Related:** NOTE-006, NOTE-007, NOTE-008, SPEC-047, SPEC-022, SPEC-004, THREADING_MODEL.md, WORKFLOW_SPAWNING_AND_SUPERVISION.md

## 1. Problem Statement

The current Ash design work is converging on a process-oriented layer distinct from both pure functions and `Act<A>`.

We now have a useful separation:

- `Act<A>` models sequential effectful computation
- `workflow` is increasingly being understood as an isolated process-like unit with its own `ActEnv`

What is still missing is a tighter middle layer:

- a minimal `proc` library and `Proc<A>` type focused on runnable/composable process structure
- specified independently of full workflow admission/contract complexity
- designed so that workflow can later elaborate into or reuse the same machinery

This document deliberately avoids committing to the full workflow migration. It focuses on a small substrate that can support library design, typing work, and future runtime reuse with minimal immediate runtime interference.

## 2. Goals

1. Introduce `Proc<A>` as a distinct public type constructor and algebra.
2. Keep `Proc<A>` clearly distinct from `Act<A>`.
3. Define a small `proc` library surface that is useful on its own.
4. Keep workflow-specific role/capability/contract machinery out of the initial `proc` layer.
5. Preserve compatibility with a later direction where workflow syntax elaborates into `Proc` construction/enrichment.
6. Minimize immediate runtime disruption: no forced supervision model, no scheduler redesign, no workflow IR rewrite in this slice.

## 3. Non-Goals

This slice does not attempt to settle:

- supervisor trees or restart hierarchies
- workflow lowering details beyond compatibility intent
- full mailbox/address/channel calculus
- exact `run` semantics for every useful `Proc<A>` inhabitant
- `Proc || Proc` environment-distribution law
- role/capability algebra beyond later workflow compatibility
- exposing raw `ActEnv` structure

## 4. Core Position

### 4.0 Semantic strata and environment lattice

The current proc/act design is easiest to reason about as a monotone semantic lattice:

```text
Pure < Effectful < Proc < Workflow
```

Each step adds both:

1. more admissible expression/process power, and
2. a richer execution environment that the computation may understand and rely on.

| Stratum | Extra power beyond lower strata | Environment understood by the computation | Representative operations / forms |
| --- | --- | --- | --- |
| `Pure` | lexical computation only | lexical bindings only | pure `let`, closure/application, pattern/match, pure data construction |
| `Effectful` / `Act` | sequential effects over pure computation | lexical + effect environment | `unit`, `bind`, `then`, `guard`, `invoke`; capability/provider access; policy/capability admissibility; provenance begins when effect execution records it |
| `Proc` | process composition over effectful computation | lexical + effect + proc environment | all sequential `Act` operations plus `par`, `scatter`, `gather`; later mailbox/channel, spawn, process identity, cancellation/failure scope |
| `Workflow` | governed/admitted proc execution | lexical + effect + proc + workflow environment | proc computation plus admitted roles, `requires`, `ensures`, workflow failure boundary, reporting/obligation/supervision semantics |

Important consequences:

- `Act` is not just "less syntax" than `Proc`; it is the effectful/sequential stratum with a smaller environment model.
- `Proc` is not where effects start; it is where process-local composition and split/join semantics start.
- `Workflow` should be understood as proc computation plus extra governance metadata and governance-sensitive operators, not as the first place where capability semantics appear.

Operationally, availability is restricted from the top of the tower downward:

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

Short-circuits may exist for runtime efficiency, ergonomics, or migration compatibility, but the reference semantics should preserve this tower. A lower stratum should not directly assume the environment of a higher stratum; higher strata admit, start, or enrich lower-stratum computations.

Environment components should also be identity-indexed, not just tower-indexed. At minimum, every live execution context is rooted in a workflow/run identity created by the outside runtime or by another workflow. Lower strata then receive projected or derived identities:

```text
WorkflowId
  owns/adopts ProcessId(s)
ProcessId
  owns/adopts BranchId(s) and effect invocation scopes
EffectInvocationId / EffectScopeId
  owns sequential effect trace entries
LexicalFrameId
  owns ordinary lexical bindings
```

So an environment lookup is not merely `(TowerLevel, ComponentType)`, but approximately:

```text
(TowerLevel, EntityId, ComponentType, Key)
```

For example:

- workflow role admission: `(Workflow, WorkflowId, AdmittedRoles, role)`
- process mailbox lookup: `(Proc, ProcessId, MailboxSet, self)`
- branch provenance segment: `(Effectful, BranchId or EffectScopeId, ProvenanceLog, current)`
- lexical variable lookup: `(Pure, LexicalFrameId, LexicalBindings, name)`

The open semantic question is how identities split and derive downward. `par`, `scatter`, and `gather` should not clone one ambiguous process/effect context; they should create child or branch identities with explicit parentage and join semantics. The workflow identity remains the governance root, while process/branch/effect identities determine isolation, attribution, and merge behavior below it. See NOTE-007 for the current runtime environment/component model and NOTE-008 for the corresponding operational bottom/failure model.

This lattice is currently the best working model for the relation between pure expressions, `Act`, `Proc`, and workflow execution.

### 4.1 `Proc<A>` is distinct from `Act<A>`

Current position:

- `Act` and `Proc` are different monads/algebras
- `Act` is the monad of sequential effectful computation
- `Proc` is the algebra of runnable/composable processes

This distinction should be strengthened rather than weakened.
`Proc` is not merely "Act with mailbox" and should not be defined away as reducible to `Act`.

At the same time, the lattice above suggests a compatible semantic reading:

- `Act<A>` is the `Effectful`/sequential stratum
- `Proc<A>` is the general process stratum above it

So every useful `Act` computation should remain embeddable into `Proc`, but the language may still preserve the public distinction because the two strata differ in both admissible power and environment model.

### 4.2 `Proc<Act<A>>` remains especially valuable

Even though `Proc` must remain distinct, one relationship is especially important:

- `Proc<Act<A>>` is a particularly valuable way to think about a process carrying an effectful sequential payload

This is a semantic/implementation convenience, not a reason to introduce a public `Proc<F, A>` form.
The public surface should remain simply:

```text
Proc<A>
```

unless later implementation pressure forces a richer form.

### 4.3 Process composition is not just monadic sequencing

For `Act`, the characteristic composition operator is `bind`.
For `Proc`, the characteristic composition direction appears to be parallel/process composition.

Working intuition:

```text
Act . Act . Act      -- sequential effect composition via bind
Proc || Proc || Proc -- parallel process composition
```

This suggests:

- `Proc` should still support `unit` and `bind`
- but applicative/monoidal structure is likely more semantically central for concurrency than monadic sequencing alone
- `||` should be treated as a process-composition operator, not as a mere alias for `then`

### 4.4 Async `par` returns running process handles

The preferred process-algebra reading of `par` is asynchronous process start, not synchronous pair production.

Therefore the process-level shape should be closer to:

```text
par : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
```

where `P<A>` stands for a running process handle for a process that may eventually produce `A`.
`P<A>` should be understood as an opaque handle around a process identity, not as a join-specific wrapper type.

Consequences:

- `par(p1, p2)` starts/adopts two running processes and returns handles to those processes.
- `join` should consume or observe running process handles, for example `join(pa, pb)`, rather than consume a special `Join<A, B>` object returned by `par`.
- `send`/mailbox/channel operations can target the same process handles.
- `scatter` and `gather` can be arranged around collections of `P<A>` handles using the same identity discipline.
- A `with_error { par(p1, p2) } handle { ... }` block handles failure of the `par` start/admission operation itself, not later failures inside `p1` or `p2`.
- Failures in running process handles propagate along their process identity toward a future `join`/`gather`/observation point, not back to the lexical `par` call after it has returned.

## 5. Library-First Direction

The preferred near-term direction is to define a `proc` library independently of workflow syntax, while keeping it compatible with later workflow goals.

This gives three immediate advantages:

1. a focused target for type and library design
2. a cleaner test surface for process algebra and runtime hooks
3. a migration path where workflow can later reuse proc machinery instead of staying the lowest-level runnable abstraction

### 5.1 Proposed initial proc-library surface

The proc library should expose ordinary unsuffixed names within its own namespace:

- `unit`
- `bind`
- `then`
- `par`
- `scatter`
- `gather`
- later: `send`, `receive`, `spawn`, `run`

The `act` library may simultaneously expose its own unsuffixed `unit` / `bind` / `then` within `act::...`.
The distinction is by algebra/module, not by globally bloated names.

### 5.2 Why not couple proc to workflow immediately

Workflow currently carries extra concerns that would obscure the proc substrate if introduced together:

- admitted role context
- admitted capability surface
- `requires` / `ensures`
- workflow-specific completion/failure interpretation

Proc should begin without those enrichments.
Workflow compatibility matters, but workflow complexity should not define proc's first slice.

## 6. Minimal Runtime Update Strategy

The runtime update should stay deliberately small.

### 6.0 Coordination with active Act-semantics work

Act semantics are active concurrent work. This proc slice must therefore remain explicitly non-invasive toward that effort.

Coordination rule:

- Phase 67 / concurrent Act-semantics work owns Act runtime semantics
- this proc packet owns only the proc library/type surface and the minimal abstraction boundary needed to keep later proc/runtime integration possible
- this packet must not redefine Act carrier semantics, hidden `ActEnv` threading, or the exact runtime meaning of `run`

Practical consequence:

- proc is specified as Act-adjacent, not Act-owning
- references to `Act` in this document are compatibility notes, not normative ownership of Act behavior
- any later proc/runtime integration should build on the landed Act semantics rather than preempt them here

### 6.1 What should not change in the initial proc slice

Do not require, in the first proc slice:

- scheduler redesign
- thread-pinning policy changes
- workflow instance model changes
- role/capability runtime semantics changes
- supervisor infrastructure
- workflow spawning semantics rewrite

### 6.2 What the runtime should eventually provide

The initial proc design should merely reserve a small compatible runtime boundary for future realization:

- a distinct `Proc<A>` runtime representation
- a `run` interpretation boundary
- a future home for process-local mailbox/channel support
- a future home for `par` execution semantics

This runtime boundary should remain narrow enough that the library/type work can move first.

## 7. Opaque Environment Discipline

`ActEnv` remains opaque.
Proc should not expose or mutate raw environment structure directly.

Consequences:

- no public env algebra in this slice
- no public `Proc<F, A>` generalization in this slice
- process/workflow enrichment should later use smart constructors/combinators rather than raw environment manipulation

Neutral `with_*` names remain the best direction for future enrichment hooks:

- `with_capabilities`
- `with_roles`
- `with_requires`
- `with_ensures`

These belong to later process/workflow shaping work, not to the minimal proc-library MVP.

## 8. Compatibility with Later Workflow Goals

This design is intentionally compatible with later workflow elaboration goals.

Target compatibility statement:

- the proc library lands independently
- workflow remains unchanged in the initial slice
- later workflow syntax may elaborate into `Proc` construction plus process/workflow enrichers
- no decision in this design should force workflow to remain separate forever

## 9. Recommended Documentation Split

Use two documents:

1. design doc (this file)
   - architectural rationale
   - runtime-boundary restraint
   - migration compatibility

2. proc library/spec doc
   - `Proc<A>` public type identity
   - proc-library surface
   - algebraic intent (`unit`, `bind`, `then`, `par`, etc.)
   - explicit deferrals for runtime-heavy features

That keeps the spec tight while preserving architectural context.

## 10. Open Questions

1. Should `Proc<A>` be surface-visible immediately, or first land as a library/type contract with delayed runtime backing?
2. What exact type should `par` have in the first spec slice (`Proc<A> -> Proc<B> -> Proc<(A, B)>` is the current leading candidate)?
3. Which proc operations must be runtime-backed in the first implementation slice, and which can remain specified-but-deferred?
4. How should `run` be specified so `Proc` remains distinct from `Act` while still supporting especially valuable cases such as `Proc<Act<A>>`?
5. When mailbox/channel support lands, what is the smallest channel/address model that remains compatible with workflow isolation?

## 11. Current Recommendation

Proceed with a tight proc packet consisting of:

- one design doc for the runtime/library split and workflow compatibility
- one proc spec focused on public types and library surface
- minimal or no immediate runtime changes beyond reserving the abstraction boundary

This keeps the work aligned with current Ash semantics while avoiding premature commitment to full workflow migration or concurrency-runtime details.
