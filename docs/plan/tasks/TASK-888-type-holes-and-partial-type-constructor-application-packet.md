# TASK-888: Type Holes and Partial Type-Constructor Application Packet

## Status: ✅ Complete

## Description

Promote a future implementation-grade SPEC/PLAN packet for type holes/wildcards in type-expression positions and partial type-constructor application, including forms needed by targets like `Result<_, E>`.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [DESIGN-031](../../design/DESIGN-031-GENERALIZED-DO-NOTATION.md)
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)

## Dependencies

- ✅ Core `Kind` and canonical type-expression substrate
- ✅ Design/spec/plan packet created; feature implementation remains in the new planned task range

## Requirements

1. Add a source and canonical representation for type-expression holes/wildcards that is distinct from type-function pattern wildcards.
2. Define where holes are allowed: signatures, aliases, projections, propositions, do targets, or a narrower MVP subset.
3. Define partial type-constructor application and hole elaboration rules, including arity/kind checking and ambiguity/defaulting boundaries.
4. Specify diagnostics for unsupported hole positions, ambiguous holes, wrong arity, and implicit currying rejection.
5. Preserve no-inversion constraints from SPEC-060/SPEC-063/SPEC-064.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: no-blocking
commands:
  - git diff --check
checklist:
  - [x] Surface grammar and parser carriers specified
  - [x] Canonical/hole representation specified
  - [x] Kind/arity and ambiguity behavior specified
  - [x] Focused parser/typeck diagnostics planned
```

## Notes

This task is a prerequisite candidate for `do:Result<_, E>` and other partially applied computation constructors, but it does not by itself add user-defined `Monad<M>`.

## Completion Notes

Activated as [DESIGN-037](../../design/DESIGN-037-TYPE-HOLES-PARTIAL-TYPE-CONSTRUCTOR-APPLICATION.md), [SPEC-066](../../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md), [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md), with implementation task range TASK-898 through TASK-903. This task completed the docs/spec/plan packet only; feature implementation remains planned in the new task range.
