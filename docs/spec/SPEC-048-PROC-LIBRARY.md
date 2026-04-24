# SPEC-048: Proc Library

**Status:** Draft
**Date:** 2026-04-23
**Related:** DESIGN-030, NOTE-006, NOTE-007, NOTE-008, SPEC-047, SPEC-004, SPEC-022

## Summary

Introduce `Proc<A>` as a distinct public type constructor and define a minimal `proc` library focused on process-structured composition with minimal immediate runtime interference.

This spec is intentionally narrower than a full process/workflow runtime spec. It focuses on:

- the public identity of `Proc<A>`
- the initial proc-library surface
- the algebraic intent of that surface
- explicit deferrals for runtime-heavy features such as mailbox mechanics, spawning, bottom/failure handling, and full execution semantics

## Motivation

Ash now has a clearer separation between:

- pure functions
- `Act<A>` for sequential effectful computation
- workflows, which are increasingly understood operationally as isolated process-like units

A dedicated `Proc<A>` layer is valuable because it provides a process-oriented library/type substrate without forcing the full workflow model, scheduler model, or supervision model to land at the same time.

## 1. Core Position

### 1.1 `Proc<A>` is distinct from `Act<A>`

Normative position:

- `Proc` and `Act` are different monads/algebras
- `Act<A>` remains the sequential effectful-computation type
- `Proc<A>` is the public type of process-structured computation

This spec does not define `Proc` as reducible to `Act`, even though `Proc<Act<A>>` remains a valid nested type.

Design-note refinement: `Act<A>` is currently best understood as the `Effectful`/sequential stratum below `Proc<A>` in the semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

This spec preserves the public `Act`/`Proc` distinction because the strata differ in both admissible power and environment model. The common embedding direction is from sequential effectful computation into process composition. A later proc surface may expose this as an explicit operation such as:

```text
from_act : Act<A> -> Proc<A>
```

This does not imply that `Proc<Act<A>>` implicitly flattens. `Proc<Act<A>>` means “a process computation whose normal result is a suspended effectful computation.”

### 1.2 Public type form

The public type form is:

```text
Proc<A>
```

This spec does **not** introduce a higher-kinded public form such as `Proc<F, A>`.

### 1.3 Opaque execution substrate

`Proc<A>` is a public type identity with an opaque runtime interpretation boundary.
This spec does not expose raw `ActEnv` structure or any lower-level process-environment representation.

## 2. Scope

In scope:

- `Proc<A>` as a draft public type constructor
- `P<A>` as a draft public running-process handle type, used by async `par`/`join`/message operations
- a `proc` library namespace/module
- initial library surface: `unit`, `bind`, `then`, `par`, `join`, `scatter`, `gather`
- algebraic intent for sequential vs. parallel process composition
- explicit deferral of mailbox/spawn/full runtime-heavy features

Out of scope for this spec:

- workflow lowering into proc
- supervisor hierarchies
- mailbox address/capability model
- exact `run` operational semantics
- exact environment-distribution law for `par`
- role/capability/process enrichment hooks (`with_roles`, `with_capabilities`, etc.)
- process IR / runtime scheduler design
- exact operational bottom/failure handling semantics (`fail`, `with_error`) beyond compatibility constraints in this spec (see NOTE-008)

## 3. Surface Direction

### 3.1 Library namespace

The proc surface is provided through a `proc` library/module.
Inside that namespace, the ordinary unsuffixed names are preferred:

- `unit`
- `bind`
- `then`
- `par`
- `join`
- `scatter`
- `gather`

This mirrors the `act` library choice where unsuffixed names live under `act::...`.

### 3.2 Public type usage

The intended user-facing type spelling is:

```ash
Proc<A>
```

This spec intentionally keeps the public type shape simple.
Any deeper semantic relation to `Act<A>` remains specification prose or implementation detail, not a surface kind/generalization commitment.

The proc surface also reserves a public running-process handle type:

```ash
P<A>
```

`P<A>` is an opaque handle to a running process that may eventually produce `A`. It is not a special join token; the same handle identity may be used by `join`, `gather`, `send`, cancellation, and later mailbox/channel operations.

## 4. Initial Library Surface

### 4.1 Core signatures

The initial proc-library surface should support at least the following shapes:

```text
unit   : A -> Proc<A>
bind   : Proc<A> -> (A -> Proc<B>) -> Proc<B>
then   : Proc<A> -> Proc<B> -> Proc<B>
par    : Proc<A> -> Proc<B> -> Proc<(P<A>, P<B>)>
join   : P<A> -> P<B> -> Proc<(A, B)>
```

Interpretation:

- `unit` lifts a pure value into trivial process structure
- `bind` gives dependent sequential process composition
- `then` sequences while discarding the left value
- `par` starts/adopts independent process computations and returns their running process handles
- `join` observes/synchronizes two running process handles and returns their completed values, or propagates observed process failure according to the later proc failure semantics

### 4.2 Derived or library-level combinators

A plausible initial proc library may also expose:

```text
scatter : List<A> -> (A -> Proc<B>) -> Proc<List<P<B>>>
gather  : List<P<A>> -> Proc<List<A>>
```

These are process-oriented collection combinators over running process handles. `scatter` starts/adopts one process per input element and returns handles. `gather` observes a collection of process handles and returns their completed values, or propagates observed process failure according to later proc failure semantics.

This spec does not require a unique encoding yet, but they belong in the proc-library vocabulary.

## 5. Algebraic Intent

### 5.1 Sequential face

`bind` and `then` define the sequential/dependent face of `Proc`.
This is the part of the process algebra closest to ordinary monadic composition.

### 5.2 Parallel face

`par` defines the independent process-start face of `Proc`.
This spec treats `par` as more central to the concurrency/process story than `bind`, without removing `bind`.

Working process-composition intuition:

```text
Act . Act . Act          -- sequential effect composition via bind
Proc || Proc || Proc     -- async process start/composition via par
join(P<A>, P<B>)         -- later observation/synchronization point
```

Because `par` is asynchronous, a failure inside one of the returned running processes does not retroactively fail the lexical `par` call after it has returned. Such failures are attached to the running process identity and are observed by later `join`, `gather`, cancellation, supervision, or workflow boundary operations.

Consequently, a scoped bottom handler around `par` handles only failure to start/admit/create the running process handles:

```ash
with_error {
    par(p1, p2)
} handle {
    _ => fallback_handles
}
```

Branch/process failures should be handled around observation points such as `join` or inside the processes themselves:

```ash
handles = par(p1, p2);
with_error {
    join(handles.0, handles.1)
} handle {
    _ => fallback_result
}
```

The `with_error`/`fail` syntax above is included as semantic direction only. NOTE-008 records the current bottom/failure handling model; a dedicated normative failure spec must define exact parser, typing, and runtime behavior before implementation.

### 5.3 Applicative / monoidal reading

This spec explicitly leaves room for `Proc` to be understood not only as a monad, but also through applicative/monoidal structure.
That is likely the more natural interface for independent concurrency than monadic sequencing alone.

This spec does not require higher-kinded interface syntax to land now. The point is semantic guidance for the proc-library design.

## 6. Runtime Boundary

This spec deliberately minimizes runtime commitments.

### 6.0 Coordination / non-interference rule

Act semantics are active concurrent work. This spec must therefore avoid claiming ownership of Act runtime behavior.

This spec owns only:

- the public identity of `Proc<A>`
- the proc-library surface
- the algebraic intent of proc combinators
- explicit deferrals needed to keep later runtime integration open

This spec does **not** own:

- hidden `ActEnv` threading semantics
- the concrete Act runtime carrier meaning
- exact `run` semantics
- concurrency scheduler or mailbox runtime design

Any later proc/runtime implementation must integrate with the landed Act semantics rather than preempt them here.

### 6.1 Not required in the initial proc-library slice

This spec does not require immediate landing of:

- `spawn`
- `run`
- `send`
- `receive`
- mailbox/channel runtime structures
- scheduler/pinning policies
- process IDs / workflow addresses

Those operations remain explicitly deferred to a later proc/runtime slice.

### 6.2 Compatibility requirement

Even though these features are deferred, the proc-library design must remain compatible with later additions of:

- process-local mailbox support
- channel-based communication
- process execution via `run`
- workflow elaboration into proc machinery
- identity-indexed process handles usable by `join`, `gather`, message send/receive, cancellation, and workflow reporting

## 7. Relation to Workflow

This spec intentionally avoids defining workflow in terms of proc today.
However, compatibility with that future direction is a requirement.

Compatibility statement:

- workflow remains unchanged by this spec
- proc is introduced as its own library/type layer
- later workflow elaboration into `Proc` is permitted and should not be blocked by the initial proc-library design

## 8. Required Follow-On Surfaces

A later proc/runtime spec should define:

1. `run` semantics
2. `P<A>` runtime representation and process identity discipline (see NOTE-007)
3. mailbox/channel model
4. `send` / `receive` process operations over process handles
5. precise `par` start/admission failure vs process-observation failure rules (see NOTE-008)
6. `join`/`gather` observation, cancellation, and ownership/consumption semantics
7. interaction between proc and workflow execution/failure reporting

## 9. Implementation Guidance

### 9.1 Parser/type-system expectation

The intended type shape is an ordinary constructor-form type:

```text
Proc<A>
```

No new exotic syntax is required for the type itself beyond ordinary type-constructor usage.
If the current parser/typechecker cannot yet represent the necessary public type identity cleanly, that limitation should be handled as an implementation prerequisite rather than by broadening the public proc type form.

### 9.2 Keep the first slice library-first

The recommended implementation order is:

1. establish the public `Proc<A>` type identity
2. establish the public `P<A>` process-handle type identity
3. establish the proc library surface and typing intent
4. defer runtime-heavy pieces unless a small runtime hook is absolutely necessary

This preserves the "minimal runtime interference" goal.

## 10. Current Answers and Open Questions

1. Public type surface: keep `Act<A>` and `Proc<A>` for now. This remains open to future extension with explicit `Pure`/`Workflow` computation names or indexed `Comp<K, A>` only if implementation pressure justifies it.
2. Process-handle spelling: current preference is `Process<A>`, but this spec continues to use `P<A>` as short draft notation until naming is finalized.
3. `par` identity creation: current direction is that `par` creates new child processes or workflows depending on the level where it is interpreted; the returned values are process handles.
4. `join` shape: `join` is binary. The core primitive is probably an await-like operation over one running process handle, from which `join` and `gather` can be built.
5. Which proc combinators can be landed as library declarations before any runtime backing exists?
6. How should `run` be specified later so `Proc` stays distinct from `Act` while still supporting the embedding of sequential `Act<A>` computations into process composition?
7. How exactly should `scatter`/`gather` relate to `par`/`join`/await in the final proc algebra?
8. Which bottom/failure semantics (`fail`, `with_error`, process-observation failure) belong in this spec vs a dedicated failure semantics spec? Current direction: DESIGN-030 records the relation, NOTE-008 records the draft model, and a future normative spec should harden parser/type/runtime behavior.

## Changelog

### 2026-04-24

- Aligned the proc library draft with DESIGN-030's semantic tower. Added `P<A>` running process handles, changed `par` to asynchronous process start returning handles, added `join`, updated `scatter`/`gather` around handles, clarified `Act<A>` as the effectful/sequential stratum below `Proc<A>`, recorded current answers for public type surface/process-handle naming/`par` identity creation/`join` shape, and deferred exact `fail`/`with_error` bottom semantics to NOTE-008 and a future normative spec.

### 2026-04-23

- Initial draft capturing a minimal proc-library/type slice distinct from workflow and `Act`.
