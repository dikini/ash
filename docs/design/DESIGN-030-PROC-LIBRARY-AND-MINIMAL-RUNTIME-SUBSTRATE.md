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

### 4.5 Resolved `par` semantics slice

This section records the current resolved direction for the next process-runtime spec. It is intentionally narrower than a full runtime spec, but it closes the blockers that would otherwise make `par`, process handles, and process failure observation ambiguous.

#### 4.5.1 Identity splitting

Normatively, `par` creates child process identities. Given current process `P0`:

```text
par(pa, pb)
  creates child ProcessId P1 under P0 for pa
  creates child ProcessId P2 under P0 for pb
  starts pa in P1
  starts pb in P2
  returns (P<A>{P1}, P<B>{P2})
```

The returned handles denote `ProcessId`s, not `BranchId`s. `BranchId` remains available as a subordinate/internal runtime identity for trace segments, branch-local facts, or lower-level scheduling structure inside a process, but it is not the public identity of `P<A>`.

Consequences:

- `ProcessId` owns observable lifecycle.
- `ProcessId` is what `P<A>` points at.
- process failure is indexed to `ProcessId`.
- cancellation/supervision targets `ProcessId`.
- `join`, `gather`, and await-like observation observe `ProcessId` completion/failure.
- `BranchId` may index internal branch-local environment, trace, or facts, but not public process handles.

#### 4.5.2 Child environment projection

`par` must not clone a monolithic context. It derives child process environments by typed projection:

```text
derive_child_env(parent_env, child_process_id, child_index)
```

Each component declares its split behavior. Initial classification:

| Component | Split behavior | Notes |
| --- | --- | --- |
| provider registry | `CopyReadOnly` | runtime/provider handles are visible but not child-owned authority minting |
| capability definitions | `CopyReadOnly` | definitions are copied/read, not mutated |
| static policy definitions | `CopyReadOnly` | policy definitions are shared as immutable context |
| admitted capability surface | `CopyReadOnly` or `RefinedChildLocal` | child may receive the same or narrower surface, never wider |
| sequential effect state | `ChildLocal` | parent sequential effect state is not shared mutably across children |
| provenance/audit sink | `AppendMerge` | children append child-local segments merged/reported by process identity |
| effect invocation scope | `ChildLocal` | new effect scopes are created under each child process |
| effect-level failure channel | `ChildLocal` | failures are attributed through child process/effect identities |
| linear/exclusive resources | `ForbiddenInPar` unless explicitly partitioned or moved | no implicit cloning |
| parent `ProcessId` | `CopyReadOnly` parent link | child knows parentage for attribution/supervision |
| current `ProcessId` | `ChildLocal` | `P1` for left child, `P2` for right child |
| child registry | parent-owned `Append/Track` | parent records started children |
| scheduler handle | `SharedConcurrent` runtime substrate | scheduler is shared by runtime, not cloned user state |
| mailbox identity | `ChildLocal` | each child receives its own mailbox identity when mailboxes exist |
| channel endpoints | `SharedConcurrent`, `ExplicitMove`, or `ForbiddenInPar` | only share endpoints with explicit concurrent endpoint semantics |
| cancellation scope | `ChildLocal` with parent propagation link | parent cancellation may propagate to children |
| process failure scope | `ChildLocal` | observed through process handles |
| join/observation registry | observer-side / parent-side | not copied into children as ordinary mutable state |
| process-local resources | `ChildLocal` or explicitly partitioned `Exclusive` | no ambient shared mutable process-local state |

Invariant:

```text
Child process env may be equal-or-less-authorized than parent env.
It may not manufacture new authority.
```

#### 4.5.3 Failure timing

A handler around the `par` expression catches only failures that occur before `par` successfully returns handles:

```ash
with_error {
  par(pa, pb)
} handle {
  _ => recover_start_failure
}
```

Catchable at the `par` call site:

- parent process lacks authority to start/admit child processes
- child `ProcessId` allocation fails
- child environment projection fails
- exclusive/linear resource split is invalid
- scheduler/admission refuses child start
- process handle allocation or registration fails
- initial child start fails before a handle becomes valid

Not catchable at the `par` call site:

- `pa` fails after `P1` has started
- `pb` fails after `P2` has started
- an effect invocation fails inside `P1` or `P2`
- a child process is cancelled after handles are returned
- a child process deadlocks, times out, or violates a process-local invariant after handles are returned

After `par` returns handles, child failures belong to child `ProcessId`s and become visible only at observation or boundary points such as `await`, `join`, `gather`, cancellation/supervision, or workflow reporting.

#### 4.5.4 Linear process handles

The first normative process model treats `P<A>` as an affine process handle. It may be moved, dropped/detached only under explicit rules, cancelled, or observed once.

Observation consumes the handle:

```text
await  : P<A> -> Proc<A>
join   : P<A> -> P<B> -> Proc<(A, B)>
gather : List<P<A>> -> Proc<List<A>>
```

A successful observation consumes the handle and returns the child result. A failed observation consumes the handle and raises the child process failure in the observing process.

Multiple observation, cached/replayable results, `dup`, shared process handles, monitors, and supervisor subscriptions are deferred to later supervision semantics. They should not be part of the first process-runtime spec unless implementation pressure forces them.

#### 4.5.5 `join` and `gather` as observation barriers

`join` is a wait-for-both observation barrier, not a left-then-right sequential await. It consumes both handles, observes both child processes to terminality, and then:

- returns `(a, b)` if both complete normally
- raises the single child failure if one child fails
- raises an aggregate process failure preserving both child failures if both fail

The observer-visible failure should preserve the child process failures and their identities. `gather` generalizes this rule to collections: consume all handles, wait for all terminal states, return ordered results if all succeed, and raise an aggregate failure containing every observed child failure if one or more fail.

#### 4.5.6 Cooperative scheduling

The proc operation set should include an explicit cooperative scheduling point:

```text
yield : Proc<Unit>
```

`yield` voluntarily suspends the current process and permits the scheduler to make progress on other runnable processes. When resumed, it returns `Unit` in the same process identity.

Identity and environment behavior:

- current `ProcessId` before and after `yield` is unchanged
- `yield` does not split `EffEnv` or `ProcEnv`
- `yield` preserves the current process environment

Failure behavior:

- `yield` normally returns `Unit`
- if the current process has been cancelled or the scheduler refuses resumption, `yield` may surface the relevant process failure/cancellation

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
- `yield`
- `await` or an equivalent single-handle observation primitive
- `join`
- `scatter`
- `gather`
- later: `send`, `receive`, `spawn`, `run`, shared-handle/supervision operations such as `dup` or `monitor`

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
- an opaque running-process handle representation (`P<A>` in current draft notation, with `Process<A>` as the likely eventual spelling)
- a `run` interpretation boundary
- a future home for process-local mailbox/channel support
- a future home for `par` execution semantics, process identity creation, and observation via `join`/`gather`/await-like operations

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
- workflow must still be tracked in its own normative spec, covering semantic behavior and not only surface syntax
- no decision in this design should force workflow to remain separate forever

Workflow surface syntax may remain close to the current form, but that does not make workflow a mere syntax layer. The separate workflow spec should own the governed-execution semantics above `Proc`: admission, role/capability context, `requires`, `ensures`, workflow failure/reporting boundaries, and the rules by which unhandled lower-level failures become workflow-level failures.

## 9. Recommended Documentation Split

Use a staged documentation split:

1. design doc (this file)
   - architectural rationale
   - runtime-boundary restraint
   - migration compatibility

2. proc library/spec doc
   - `Proc<A>` public type identity
   - proc-library surface
   - algebraic intent (`unit`, `bind`, `then`, `par`, etc.)
   - explicit deferrals for runtime-heavy features

3. environment and failure design notes
   - NOTE-007 owns the current runtime identity/component model: workflow/run identity, process identity, branch/effect/lexical identities, and identity-indexed component lookup
   - NOTE-008 owns the current operational-bottom model: `fail`, scoped `with_error`, tower/entity-indexed failure objects, and async process-failure observation

4. later normative runtime specs
   - process runtime semantics: `P<A>`/`Process<A>` representation, `par` identity splitting, branch-local `EffEnv`/`ProcEnv`, `join`/`gather` observation, cancellation, and process-failure propagation
   - workflow semantics: a separate workflow spec, not just a surface-syntax spec, covering workflow as governed proc execution, admission, roles/capabilities, `requires`/`ensures`, `WorkflowFailure`, reporting, and lower-failure reinterpretation at the workflow boundary

That keeps SPEC-048 tight while preserving architectural context and avoiding premature workflow/runtime hardening.

## 10. Open Questions

1. Should `Proc<A>` be surface-visible immediately, or first land as a library/type contract with delayed runtime backing?
2. Which proc operations must be runtime-backed in the first implementation slice, and which can remain specified-but-deferred?
3. How should `run` be specified so `Proc` remains distinct from `Act` while still supporting explicit embedding of sequential `Act<A>` computations into process composition?
4. When mailbox/channel support lands, what is the smallest channel/address model that remains compatible with workflow isolation?
5. What are the exact cancellation and detach/drop semantics for linear `P<A>` handles?
6. What supervision model justifies later shared-handle operations such as `dup`, `monitor`, cached observation, or multiple observers?
7. What is the exact boundary between the process-runtime spec and the separate workflow-semantics spec, especially for admission failures, completion failures, and lower-failure reinterpretation?

## 11. Current Recommendation

Proceed with a tight proc packet consisting of:

- one design doc for the runtime/library split and workflow compatibility
- one proc spec focused on public types and library surface
- NOTE-007 and NOTE-008 as the current exploratory records for environment identity and operational failure
- a separate workflow-semantics spec once the process/failure substrate is sufficiently stable; this spec should cover semantics as well as any surface syntax cleanup
- minimal or no immediate runtime changes beyond reserving the abstraction boundary

The `par` semantics slice is now resolved enough to seed the next normative process-runtime spec:

- `par` normatively creates child `ProcessId`s and returns handles to those child processes; `BranchId` is subordinate/internal, not the public identity of `P<A>`
- child `EffEnv`/`ProcEnv` values are derived by typed projection from the parent environment, not by cloning a monolithic context
- handlers around `par` catch only start/admission/handle-creation failures; failures inside already-started children are observed at `await`, `join`, `gather`, supervision, or workflow reporting boundaries
- `P<A>` is affine/linear in the first normative model; observation consumes the handle, while `dup`, monitoring, cached observation, and multiple observers are deferred to supervision semantics
- `join` and `gather` are wait-for-all observation barriers that preserve child process failure identities and aggregate multiple failures
- the proc operation set should include `yield : Proc<Unit>` as an explicit cooperative scheduling point

This keeps the work aligned with current Ash semantics while avoiding premature commitment to full workflow migration or supervision/runtime details.
