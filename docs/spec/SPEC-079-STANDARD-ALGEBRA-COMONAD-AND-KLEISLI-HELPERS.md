# SPEC-079: Standard Algebra Comonad and Kleisli Helper Surfaces

**Status:** Implemented MVP
**Date:** 2026-06-07
**Plan:** [PLAN-129](../plan/PLAN-129-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md)
**Implementation Tasks:** [TASK-1030](../plan/tasks/TASK-1030-comonad-kleisli-packet.md) through [TASK-1037](../plan/tasks/TASK-1037-comonad-kleisli-closeout.md)
**Related:** [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-078](SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md), [SPEC-077](SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)

## Summary

This specification extends the `std::algebra` roadmap beyond the Phase 133 `Semigroup`, `Monoid`, `Functor`, `Applicative`, and `Monad` MVP. Phase 134 implements the current source-visible `Comonad` interface and concrete Option/Result Kleisli helper module, while recording explicit Cokleisli and Coapplicative deferrals. Ash must not add vague or mathematically unstable interfaces merely because names are dual to existing algebra names.

The phase does not add syntax. It must use existing modules, imports, interfaces, impls, ordinary functions, function types, constructor-kinded parameters, and current standard-library evidence. If the live language cannot express a logical signature directly, the audit gate must translate it into the exact accepted Ash surface or defer that row.

Kleisli and Cokleisli are helper surfaces in this packet, not a general category hierarchy. `std::category`, `Category`, arrows, profunctors, and general categorical instances remain deferred unless a later packet introduces them explicitly.

## Motivation

SPEC-078 intentionally left category-level algebra work for a later phase once `std::algebra` stabilized. The implemented algebra namespace now gives Ash a place to define the dual operations that are useful for context-dependent computation: extracting from a total context, extending a context-aware function, and composing functions through monadic or comonadic structure.

The same stability creates a risk. `Comonad<Option>`, `Comonad<Result<_, E>>`, or `Comonad<Act>` may look appealing by symmetry, but each would be unsound without a total extraction semantics. `Option` and `Result` can be empty or failed; `Act`, `Proc`, and `Workflow` are opaque runtime-managed carriers that cannot reveal values without executing or crossing tower boundaries. This spec makes those negative cases explicit so the library grows by lawful, final-surface evidence rather than by names alone.

## Relationship to SPEC-078

SPEC-078 remains the authority for the implemented Phase 133 algebra MVP. It introduced `std::algebra` and deliberately deferred category-level abstractions. SPEC-079 partially retires only the `comonad` portion of that future-work sentence by implementing the current `Comonad` interface and recording Cokleisli helper blockers. It does not retire the broader category, arrow, bifunctor, or profunctor deferrals.

The current source implementation of Phase 133 is an MVP surface, not the fully polymorphic logical target. In particular, current `Functor`, `Applicative`, and `Monad` source interfaces use constructor-kinded parameters but monomorphic `Int` method payloads where method-level generics and higher-rank helpers are not yet accepted. SPEC-079 follows the same discipline: logical signatures describe the intended contract, while TASK-1031 must freeze exact live Ash syntax before any source implementation task starts.

## Namespace decision

The canonical namespace remains `std::algebra`.

Logical target files:

```text
std/src/algebra/comonad.ash
std/src/algebra/kleisli.ash
std/src/algebra/cokleisli.ash
std/src/algebra/coapplicative.ash   # only if TASK-1035 validates a precise first slice
```

`std/src/algebra/mod.ash` must re-export only surfaces that the implementation actually adds. The audit may decide to plan `coapplicative.ash` as a deferred placeholder document rather than as source code.

`std::category` is out of scope for this phase. Kleisli and Cokleisli helpers may use category language in documentation, but they must be ordinary functions/modules rather than instances of a `Category` interface.

## Logical algebra contracts

The following blocks are logical targets. They are not permission to write unsupported Ash syntax. The audit gate must inspect the live parser, module loader, interface registration, impl registration, function type support, and helper lowering before implementation.

### `Comonad`

Preferred logical contract:

```text
interface Comonad<W : * -> *> extends Functor<W> {
  extract : W<A> -> A
  extend  : W<A> -> (W<A> -> B) -> W<B>
}
```

`duplicate` is a helper or derived operation:

```text
duplicate : W<A> -> W<W<A>>
duplicate(wa) = extend(wa, identity)
```

If the implementation chooses `extract + duplicate` as primitives instead, it must still expose `extend` where the language can express it, because Cokleisli composition is clearest in terms of `extend`.

Current-MVP source may need to be narrower, for example:

```ash
pub interface Comonad<W : * -> *> {
    extract(W<Int>) -> Int
    extend(W<Int>, W<Int> -> Int) -> W<Int>
}
```

The exact accepted spelling is owned by TASK-1031.

### Cokleisli helpers

Cokleisli helpers compose context-consuming functions through `Comonad<W>` evidence.

Logical target:

```text
id<W, A>      : W<A> -> A
compose<W,A,B,C> : (W<A> -> B) -> (W<B> -> C) -> W<A> -> C
compose(f, g)(wa) = g(extend(wa, f))
```

The MVP may provide concrete helper wrappers if generic higher-rank helper functions cannot be expressed in current Ash. Helper examples must import the final stdlib paths and must not be local-only fixtures.

### Kleisli helpers

Kleisli helpers compose value-to-computation functions through existing `Monad<M>` evidence from SPEC-078.

Logical target:

```text
id<M, A>      : A -> M<A>
compose<M,A,B,C> : (A -> M<B>) -> (B -> M<C>) -> A -> M<C>
compose(f, g)(a) = bind(f(a), g)
```

These helpers do not change `do:K` or comprehension lowering. They reuse the selected `Monad<M>` evidence path already implemented by Phase 133. If current Ash cannot express generic function-returning helpers, the task must either add concrete Option/Result wrappers or defer the helper with a named follow-up.

### `Coapplicative` decision gate

`Coapplicative` is not as settled as `Comonad`. The term can refer to different duals depending on whether the design starts from applicative functors, monoidal functors, contravariant `Divisible`, context splitting, or a separate coalgebraic structure.

This packet therefore requires a decision before implementation:

1. define a precise Ash-facing `Coapplicative` interface with method names, laws, and at least one lawful candidate carrier; or
2. defer `Coapplicative` explicitly and keep this phase focused on `Comonad`, Kleisli helpers, and Cokleisli helpers.

A vague placeholder interface is not acceptable. If no lawful final-surface instance exists, `Coapplicative` must remain a planned design note or follow-up task, not an implemented stdlib module.

## Instance policy

A `Comonad<W>` instance requires total extraction from `W<A>` to `A`. The audit gate must classify candidate carriers before implementation.

Expected first-slice policy:

| Carrier | Default decision | Reason |
|---|---|---|
| `Option` | Reject as Comonad | `None` has no `A` to extract. |
| `Result<_, E>` | Reject as Comonad | `Err` has no `A` to extract. |
| `List` | Reject unless refined to non-empty/focused carrier | Ordinary lists may be empty and do not choose a focus. |
| `Act` | Reject as Comonad | Extraction would inspect or run opaque effectful computation. |
| `Proc` | Reject as Comonad | Extraction would violate process/runtime opacity. |
| `Workflow` | Reject as Comonad | Extraction would cross governance/runtime boundaries. |
| `Identity` | Candidate if added or already available | Total wrapper with lawful extraction. |
| `NonEmpty`, `Store`, `Env`, zipper/focused contexts | Candidate follow-up | Needs concrete carrier and lawful operations. |

If no lawful carrier is currently present, the first implementation slice may still add the interface and helpers only if final-surface import/typecheck evidence exists and the plan records the lack of instances honestly. It must not add fake instances to satisfy symmetry.

## Laws and generated-test handoff

Comonad laws are normative contracts:

```text
extend(wa, extract) == wa
extract(extend(wa, f)) == f(wa)
extend(extend(wa, f), g) == extend(wa, fn(wb) { g(extend(wb, f)) })
```

Equivalent duplicate-form laws may also be recorded:

```text
extract . duplicate == id
map extract . duplicate == id
duplicate . duplicate == map duplicate . duplicate
```

Kleisli helper laws follow from Monad laws but must be listed for generated-test reporting:

```text
compose(unit, f) == f
compose(f, unit) == f
compose(compose(f, g), h) == compose(f, compose(g, h))
```

Cokleisli helper laws follow from Comonad laws:

```text
compose(extract, f) == f
compose(f, extract) == f
compose(compose(f, g), h) == compose(f, compose(g, h))
```

Coapplicative laws remain intentionally unspecified for Phase 134 because TASK-1035 chose explicit deferral pending an accepted formulation and lawful carrier.

Generated law execution remains tied to SPEC-077-style generated tests. TASK-1029 owns generated law tests for the Phase 133 algebra set and was extended by TASK-1036 to cover Comonad, Kleisli, and Cokleisli law profiles. Phase 134 records law-profile ownership but does not implement generated law execution.

## Required acceptance matrix

| ID | Requirement | Evidence |
|---|---|---|
| A79-1 | SPEC-079, PLAN-129, PLAN-INDEX, spec README, task files, and CHANGELOG are created coherently | Docs packet and link checks |
| A79-2 | Audit gate freezes exact live Ash syntax for Comonad, Kleisli, Cokleisli, and any Coapplicative decision | TASK-1031 audit artifact |
| A79-3 | `std::algebra::comonad` is either importable as a final source module or explicitly blocked with a named source-syntax reason | Final-surface import/typecheck test or fail-closed audit row |
| A79-4 | Kleisli helpers reuse existing `Monad<M>` evidence and do not introduce hidden category/runtime authority | Helper tests or deferred helper row |
| A79-5 | Cokleisli helpers reuse `Comonad<W>` evidence and do not introduce a general `Category` interface | Helper tests or deferred helper row |
| A79-6 | Coapplicative is precisely defined with laws and a lawful carrier, or deferred explicitly | TASK-1035 decision record |
| A79-7 | Unsound Comonad instances for Option, Result, ordinary List, Act, Proc, and Workflow are rejected or remain absent with negative coverage | Negative evidence/audit tests |
| A79-8 | Law-profile handoff covers Comonad, Kleisli, and Cokleisli laws and names generated-test ownership | Updated law-profile artifact/task |
| A79-9 | Reference docs describe only implemented surfaces and planned/deferred surfaces accurately | `reference/stdlib/algebra.md` sweep |
| A79-10 | Closeout runs focused gates, broad cargo/doc gates, `git diff --check`, link/status checks, and independent review | [TASK-1037 closeout evidence](../plan/audits/TASK-1037-comonad-kleisli-closeout.md) |

Filtered cargo commands must prove non-zero execution or be paired with artifact assertions. A filter that can pass with zero matching tests is not acceptable closeout evidence.

## Non-goals

- No new Ash syntax.
- No general `std::category` hierarchy.
- No `Category`, `Arrow`, `Profunctor`, or broad category-law implementation.
- No fake `Comonad` instances for partial or opaque runtime carriers.
- No law proof checker in this phase.
- No generated law execution in this phase; TASK-1029 remains the concrete generated-test follow-up owner.
- No self-hosting of `ActEnv`, process scheduler state, workflow admission state, or other opaque runtime internals.
- No unrestricted type lambdas, higher-rank polymorphism, or broad multi-parameter constructor classes beyond the live substrate.

## Implementation tasks

- TASK-1030: Create the SPEC-079/PLAN-129 packet and task skeletons.
- TASK-1031: Audit live syntax, module/evidence seams, lawful candidate carriers, and downstream verification commands.
- TASK-1032: Add the `Comonad` namespace/interface surface if the audit validates exact syntax.
- TASK-1033: Add Kleisli helper module plans/implementation using existing Monad evidence.
- TASK-1034: Add Cokleisli helper module plans/implementation using Comonad evidence.
- TASK-1035: Decide and either implement or defer Coapplicative with precise laws and carrier evidence.
- TASK-1036: Extend law-profile/generated-test handoff and reference/corpus docs.
- TASK-1037: Closeout, broad verification, independent review, and status reconciliation.

## Changelog

### 2026-06-07

- Promoted to Implemented MVP after Phase 134 landed the current `std::algebra::comonad` interface, concrete Option/Result Kleisli helpers, explicit Cokleisli/Coapplicative deferrals, generated-law handoff ownership, and closeout evidence without adding new syntax or a general category hierarchy.
