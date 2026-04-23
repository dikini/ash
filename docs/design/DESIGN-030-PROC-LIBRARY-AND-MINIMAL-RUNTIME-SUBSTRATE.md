# DESIGN-030: Proc Library and Minimal Runtime Substrate

**Status:** Draft
**Date:** 2026-04-23
**Related:** NOTE-006, SPEC-047, SPEC-022, SPEC-004, THREADING_MODEL.md, WORKFLOW_SPAWNING_AND_SUPERVISION.md

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

### 4.1 `Proc<A>` is distinct from `Act<A>`

Current position:

- `Act` and `Proc` are different monads/algebras
- `Act` is the monad of sequential effectful computation
- `Proc` is the algebra of runnable/composable processes

This distinction should be strengthened rather than weakened.
`Proc` is not merely "Act with mailbox" and should not be defined away as reducible to `Act`.

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
