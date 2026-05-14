# DESIGN-037: Type Holes and Partial Type-Constructor Application

**Status:** Promoted to [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) / [PLAN-115](../plan/PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
**Date:** 2026-05-14
**Origin:** [DESIGN-034 §16.9](DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly), [TASK-888](../plan/tasks/TASK-888-type-holes-and-partial-type-constructor-application-packet.md)
**Related:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)

## Summary

Ash's generalized `do:K` design wants computation constructors of kind `* -> *`. Some useful constructors are already unary (`Act`, `Proc`, `Option`, `List`), but others are higher arity (`Result<A, E>`, `Reader<Env, A>`). The missing bridge is an explicit source hole and partial-application elaboration model.

The design chooses explicit holes over implicit currying. A target such as `Result<_, ParseError>` elaborates to a unary constructor equivalent to `λA. Result<A, ParseError>`. Bare `Result` remains a wrong-kind error rather than being silently curried.

## Core decisions

- `_` in a type-expression position is a source hole with a scoped purpose, not an inference meta that may solve arbitrarily later.
- The first public partial-constructor surface allows exactly one value-position hole in do targets and explicitly audited type positions.
- Partial constructor applications elaborate to canonical constructor-lambda or partial-application carriers, not debug strings and not ordinary saturated nominal constructors.
- Holes must not trigger type-function inversion, associated-family inversion, or output-driven solving.
- Type-function pattern wildcards remain a separate construct; they do not become general source holes.

## Dependency on HKT

TASK-888 / SPEC-066 makes `Result<_, E>` representable as a unary constructor. It does not, by itself, make `Monad<Result<_, E>>` meaningful. That requires TASK-889 / SPEC-067 constructor-kinded parameters and dictionary/evidence resolution.


## Non-goals

- Do not reopen SPEC-057 through SPEC-064 substrate decisions unless the new spec explicitly names an incompatibility.
- Do not encode type-level computation as ordinary `Type::Constructor` nodes.
- Do not broaden `do` notation or pattern behavior before the owning implementation phase lands.

## Handoff

The normative implementation contract is in [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). The task order is in [PLAN-115](../plan/PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md).
