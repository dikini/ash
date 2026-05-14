# SPEC-067: Constructor-Kinded Parameters and HKT

**Status:** Draft
**Date:** 2026-05-14
**Promotes:** [DESIGN-038](../design/DESIGN-038-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Origin:** [TASK-889](../plan/tasks/TASK-889-constructor-kinded-parameters-and-hkt-packet.md)
**Builds on:** [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-064](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
**Related:** [SPEC-033](SPEC-033-MULTI-PARAMETER-INTERFACES.md), [SPEC-034](SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-035](SPEC-035-ASSOCIATED-TYPES.md), [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-066](SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
**Plan:** [PLAN-116](../plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Implementation Tasks:** [TASK-904](../plan/tasks/TASK-904-hkt-audit-gate.md) through [TASK-911](../plan/tasks/TASK-911-hkt-closeout.md)

## 1. Summary

SPEC-067 adds constructor-kinded binders and higher-kinded interface support. It is the packet that enables Ash to express interfaces such as `Functor<F>`, `Applicative<F>`, and `Monad<M>` where the parameter has kind `* -> *`.

This spec is not a local do-notation patch. It extends source binders, core/typechecker representation, interface/impl coherence, and TypeEnv evidence resolution. Generalized `do` then consumes the resulting `Monad<K>` evidence.

## 2. Baseline

Live substrate:

- `ash-core::Kind` can represent arrow kinds such as `* -> *`;
- current source type parameters and interface parameters are effectively proper-type parameters;
- current do targets use compiler-known hidden dictionaries for `Act`, `Proc`, and `Workflow`;
- `do:Result<_, E>` is explicitly deferred pending partial application and Monad evidence;
- higher-kinded interface declarations, constructor-variable application, and impl heads are not source-supported.

## 3. Scope

In scope:

1. source kinded binders such as `M : * -> *`;
2. constructor variables and constructor-variable applications such as `M<A>`;
3. interface/type/function/type-function binders carrying arrow kinds;
4. impl heads for higher-kinded interfaces such as `impl Monad<Option>`;
5. TypeEnv kind checking, constructor unification, and evidence lookup for kinded binders;
6. replacement path from hidden do dictionaries to `Monad<K>` evidence;
7. diagnostics and acceptance tests for `Functor`, `Applicative`, and `Monad`-shaped contracts.

Out of scope:

- higher-rank polymorphism and impredicative types;
- unrestricted type lambdas in source syntax;
- automatic do-target inference;
- proof of Monad/Functor/Applicative laws;
- arbitrary associated-type-family inversion during evidence search;
- multi-parameter constructor classes beyond the audited MVP.

## 4. Source Syntax

The audit gate must freeze the exact grammar. The intended logical forms are:

```ash
interface Functor<F : * -> *> {
    map<A, B>(fa: F<A>, f: A -> B) -> F<B>;
}

interface Monad<M : * -> *> {
    return<A>(a: A) -> M<A>;
    bind<A, B>(ma: M<A>, f: A -> M<B>) -> M<B>;
}

impl Monad<Option> { ... }
impl Monad<Result<_, E>> { ... }
```

`Result<_, E>` requires SPEC-066. Without SPEC-066, higher-arity constructors remain wrong-kind for `M : * -> *`.

## 5. Core and TypeEnv Model

Required logical capabilities:

- a binder records `name`, `kind`, source span, and optional bounds;
- `CanonicalTypeExpr` or an adjacent core carrier can represent applying a constructor variable to a proper type argument;
- TypeEnv tracks variables at both kind `*` and arrow kinds;
- unification distinguishes proper type metas from constructor metas;
- interface evidence keys include kinded argument spines.

Implementations must not lower `M<A>` into a nominal type named `M` with argument `A`. It is constructor-variable application and must remain diagnosable as such.

## 6. Interface and Impl Coherence

Rules:

1. interface type parameters may have proper or arrow kinds;
2. method signatures may apply constructor parameters according to their kind;
3. impl heads must match the interface's expected kinded argument shape;
4. overlapping higher-kinded impls are rejected unless a future coherence spec defines specialization;
5. generic impls over constructor variables require where-bound evidence and must not select by expected output.

SPEC-034 where-bounded generic impls remain relevant. If the live generic-impl substrate is still narrower than this spec needs, TASK-904 must bind a prerequisite or narrow the first implementation slice.

## 7. Do-Notation Evidence

`do:K` resolves in this order after this spec lands:

1. elaborate target `K` to a unary constructor expression;
2. require `Monad<K>` evidence from TypeEnv;
3. elaborate `return` and `<-` through the selected evidence;
4. keep tower-specific effects for `Act`, `Proc`, and `Workflow` explicit.

The hidden Act/Proc/Workflow dictionaries may remain as compiler-prelude evidence during migration, but they must be shaped as ordinary `Monad<K>` entries at the TypeEnv boundary.

## 8. Diagnostics

Required diagnostics:

- kinded binder syntax unsupported at non-enabled sites;
- applying a proper type variable as a constructor;
- constructor variable applied to wrong number/kind of arguments;
- impl head wrong kind for an interface parameter;
- missing `Monad<K>` evidence for do target;
- ambiguous or overlapping higher-kinded impl evidence;
- attempted law proof or automatic law assumption beyond type-shape checking.

## 9. Acceptance Matrix

| ID | Case | Expected result |
|----|------|-----------------|
| HKT-1 | parse `interface Functor<F : * -> *>` | kind binder preserved |
| HKT-2 | typecheck `F<A>` in method signature | constructor application accepted |
| HKT-3 | `impl Monad<Option>` | evidence registered if methods match |
| HKT-4 | `impl Monad<Result<_, E>>` | requires SPEC-066 partial target support |
| HKT-5 | `M` used where `M<A>` required | wrong-kind diagnostic |
| HKT-6 | overlapping `Monad<Option>` impls | coherence rejection |
| HKT-7 | `do:Option` after evidence | typed elaboration uses Monad evidence |
| HKT-8 | `do:List` without evidence | missing Monad evidence diagnostic |

## 10. Implementation Tasks

See [PLAN-116](../plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md).
