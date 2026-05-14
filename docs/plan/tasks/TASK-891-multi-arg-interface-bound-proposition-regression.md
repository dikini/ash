# TASK-891: Multi-Argument Interface-Bound Proposition Regression

## Status: ✅ Complete

## Description

Add focused parser and typechecker regression evidence that the existing SPEC-H proposition surface preserves and lowers multi-argument interface-bound proposition tails without broadening solver behavior.

## Specification Reference

- [SPEC-064 §5.2](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md#52-source-propositions)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)
- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)

## Dependencies

- ✅ TASK-874: Parser proposition surface
- ✅ TASK-875: TypeEnv proposition environment

## Requirements

1. Add parser coverage for an interface-bound proposition whose interface application has at least two type arguments.
2. Add typechecker lowering coverage proving `InterfaceBoundProposition::interface_args` preserves both arguments in order.
3. Do not add new solver behavior, HKT, holes, or partial type-constructor application.

## Dispatch

```
agent: hermes
reasoning: low
max_turns: 8
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-parser --test task_874_proposition_surface task_874_parses_multi_argument_interface_bound_proposition_tail
  - cargo test -p ash-typeck --test task_875_proposition_environment task_875_lowers_multi_argument_interface_bound_proposition_terms
checklist:
  - [x] Parser test added
  - [x] TypeEnv lowering test added
  - [x] Focused parser test passes
  - [x] Focused typechecker test passes
```

## Notes

This task hardens evidence for existing SPEC-H behavior only. It does not make generalized HKT/interface-application constraints more expressive than current type-expression syntax already supports.
