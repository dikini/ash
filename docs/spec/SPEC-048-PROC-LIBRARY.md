# SPEC-048: Proc Library

**Status:** Draft
**Date:** 2026-04-23
**Related:** DESIGN-030, NOTE-006, SPEC-047, SPEC-004, SPEC-022

## Summary

Introduce `Proc<A>` as a distinct public type constructor and define a minimal `proc` library focused on process-structured composition with minimal immediate runtime interference.

This spec is intentionally narrower than a full process/workflow runtime spec. It focuses on:

- the public identity of `Proc<A>`
- the initial proc-library surface
- the algebraic intent of that surface
- explicit deferrals for runtime-heavy features such as mailbox mechanics, spawning, and full execution semantics

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

This spec does not define `Proc` as reducible to `Act`, even though `Proc<Act<A>>` is recognized as an especially important and useful case.

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
- a `proc` library namespace/module
- initial library surface: `unit`, `bind`, `then`, `par`, `scatter`, `gather`
- algebraic intent for sequential vs. parallel process composition
- explicit deferral of mailbox/spawn/runtime-heavy features

Out of scope for this spec:

- workflow lowering into proc
- supervisor hierarchies
- mailbox address/capability model
- exact `run` operational semantics
- exact environment-distribution law for `par`
- role/capability/process enrichment hooks (`with_roles`, `with_capabilities`, etc.)
- process IR / runtime scheduler design

## 3. Surface Direction

### 3.1 Library namespace

The proc surface is provided through a `proc` library/module.
Inside that namespace, the ordinary unsuffixed names are preferred:

- `unit`
- `bind`
- `then`
- `par`
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

## 4. Initial Library Surface

### 4.1 Core signatures

The initial proc-library surface should support at least the following shapes:

```text
unit   : A -> Proc<A>
bind   : Proc<A> -> (A -> Proc<B>) -> Proc<B>
then   : Proc<A> -> Proc<B> -> Proc<B>
par    : Proc<A> -> Proc<B> -> Proc<(A, B)>
```

Interpretation:

- `unit` lifts a pure value into trivial process structure
- `bind` gives dependent sequential process composition
- `then` sequences while discarding the left value
- `par` composes independent processes in parallel/process structure

### 4.2 Derived or library-level combinators

A plausible initial proc library may also expose:

```text
scatter : List<A> -> (A -> Proc<B>) -> Proc<List<B>>
gather  : List<Proc<A>> -> Proc<List<A>>
```

These are process-oriented collection combinators and may be specified as library-layer combinators over the core surface.

This spec does not require a unique encoding yet, but they belong in the proc-library vocabulary.

## 5. Algebraic Intent

### 5.1 Sequential face

`bind` and `then` define the sequential/dependent face of `Proc`.
This is the part of the process algebra closest to ordinary monadic composition.

### 5.2 Parallel face

`par` defines the independent composition face of `Proc`.
This spec treats `par` as more central to the concurrency/process story than `bind`, without removing `bind`.

Working process-composition intuition:

```text
Act . Act . Act      -- sequential effect composition via bind
Proc || Proc || Proc -- parallel process composition via par
```

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
2. mailbox/channel model
3. `send` / `receive` process operations
4. precise `par` failure and environment rules
5. interaction between proc and workflow execution/failure reporting

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
2. establish the proc library surface and typing intent
3. defer runtime-heavy pieces unless a small runtime hook is absolutely necessary

This preserves the "minimal runtime interference" goal.

## 10. Open Questions

1. Should `Proc<A>` first appear as a surface-visible type constructor or as a type/library contract backed later by runtime representation work?
2. Is `par : Proc<A> -> Proc<B> -> Proc<(A, B)>` the right first canonical type?
3. Which proc combinators can be landed as library declarations before any runtime backing exists?
4. How should `run` be specified later so `Proc` stays distinct from `Act` while still supporting valuable cases such as `Proc<Act<A>>`?
5. How should `scatter` and `gather` relate to `par` in the final proc algebra?

## Changelog

### 2026-04-23

- Initial draft capturing a minimal proc-library/type slice distinct from workflow and `Act`.
