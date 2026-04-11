# TASK-505: Pure Functions Contract Lowering and Stage 1 Constraints

## Status: ✅ Passed

## Description

Implement fn contract validation and lowering for Stage 1 arithmetic constraints, including
`NotEq`, `Modulo`, repeated/comma-separated contract normalization, and ensures validation.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [SPEC-028: Function Constraint System](../../spec/SPEC-028-FUNCTION-CONSTRAINT-SYSTEM.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)

## Requirements

1. Validate fn `requires` and `ensures` clauses against the fn contract subset.
2. Normalize repeated and comma-separated contract clause forms to a canonical shape.
3. Lower Stage 1 arithmetic predicates into core constraints.
4. Add `NotEq` and `Modulo` to the arithmetic vocabulary and cover them in tests.
5. Define the ensures lowering/evaluation boundary needed by runtime checking.

## Dependencies

- [TASK-502](TASK-502-pure-functions-parser-and-ast-foundation.md)
- [TASK-504](TASK-504-pure-functions-type-system-and-purity.md)

## Likely Files

- Modify: contract lowering / validation code
- Modify: `ash-core` arithmetic constraint definitions
- Modify: tests for `requires`, `ensures`, `NotEq`, `Modulo`, and clause normalization

## Completion Checklist

- [x] fn `requires` subset enforced
- [x] fn `ensures` subset enforced
- [x] repeated/comma-separated clause forms normalize consistently
- [x] `NotEq` and `Modulo` implemented
- [x] lowering/tests cover Stage 1 predicates
