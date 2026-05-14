# DESIGN-038: Constructor-Kinded Parameters and HKT

**Status:** Promoted to [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) / [PLAN-116](../plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Date:** 2026-05-14
**Origin:** [DESIGN-034 §16.9](DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly), [TASK-889](../plan/tasks/TASK-889-constructor-kinded-parameters-and-hkt-packet.md)
**Related:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)

## Summary

Ash's current kind substrate can represent `* -> *`, but source binders and interface/impl machinery cannot bind variables of that kind. This blocks `Functor<F>`, `Applicative<F>`, `Monad<M>`, user-defined do targets, and generic helper functions over computation constructors.

This design promotes constructor-kinded binders as a type-system feature, not a do-notation tweak. `do:Act` and `do:Proc` currently use compiler-known dictionaries shaped like future `Monad<K>` evidence; SPEC-067 defines the path for replacing that bridge with ordinary interface evidence.

## Core decisions

- Kinded binders such as `M : * -> *` are explicit in the MVP.
- Applying a constructor variable, such as `M<A>`, is a distinct canonical application form checked by kind.
- Higher-kinded interfaces are ordinary interfaces whose parameters may have arrow kinds.
- `Monad<M>`/`Functor<F>`/`Applicative<F>` evidence is resolved through TypeEnv interface/impl coherence, not a parser or do-target special case.
- Higher-rank polymorphism, impredicativity, type lambdas in arbitrary expressions, and do-target inference are deferred.

## Interaction with TASK-888

Unary constructors such as `Option`, `List`, `Act`, and `Proc` can be candidates for `Monad<M>` once SPEC-067 lands. Higher-arity constructors such as `Result<A, E>` need SPEC-066 partial application before they can satisfy a unary `M : * -> *` binder.


## Non-goals

- Do not reopen SPEC-057 through SPEC-064 substrate decisions unless the new spec explicitly names an incompatibility.
- Do not encode type-level computation as ordinary `Type::Constructor` nodes.
- Do not broaden `do` notation or pattern behavior before the owning implementation phase lands.

## Handoff

The normative implementation contract is in [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). The task order is in [PLAN-116](../plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md).
