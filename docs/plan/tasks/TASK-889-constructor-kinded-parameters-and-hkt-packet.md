# TASK-889: Constructor-Kinded Parameters and HKT Packet

## Status: ✅ Complete

## Description

Promote a future implementation-grade SPEC/PLAN packet for constructor-kinded type/interface parameters such as `M : * -> *`, higher-kinded abstractions, and user-defined unary computation interfaces such as future `Monad<M>`.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [DESIGN-031](../../design/DESIGN-031-GENERALIZED-DO-NOTATION.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)

## Dependencies

- ✅ Core `Kind::{Type, Arrow}` substrate
- ✅ SPEC-066 / PLAN-115 owns partial application and holes; SPEC-067 / PLAN-116 records that dependency for higher-arity do targets
- ✅ Design/spec/plan packet created; feature implementation remains in the new planned task range

## Requirements

1. Define source syntax for kinded binders in interfaces, impls, function/type-function parameters, and proposition predicates.
2. Extend parser carriers beyond `domain: Option<Type>` where a real kind binder is needed.
3. Specify typechecker representation for constructor variables, constructor application, unification, and interface constraints.
4. Define user-defined computation-constructor evidence boundaries without implicitly importing target-specific operations into `do:K` blocks.
5. Preserve the existing Act/Proc builtin dictionary bridge until full user-defined dictionaries are implemented.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 24
toolsets: [terminal, file]
```

## Verification

```
strictness: no-blocking
commands:
  - git diff --check
checklist:
  - [x] Syntax and carriers specified
  - [x] Typechecker kinded-variable representation specified
  - [x] Interface/impl coherence rules specified
  - [x] do-notation interaction specified
```

## Notes

Do not implement constructor-kinded parameters as a local tweak to `do` notation. This is a cross-cutting type-system feature.

## Completion Notes

Activated as [DESIGN-038](../../design/DESIGN-038-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md), with implementation task range TASK-904 through TASK-911. This task completed the docs/spec/plan packet only; feature implementation remains planned in the new task range.
