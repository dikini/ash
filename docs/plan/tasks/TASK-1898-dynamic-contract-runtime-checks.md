# TASK-1898: Dynamic Contract Runtime Checks

**Status:** Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Execute dynamic contract checks with distinct `ContractViolation` and `ContractPredicateFault` traps, preserving the authority-free predicate evaluator boundary.

## Requirements

1. Evaluate dynamic predicates over captured boundary environments and snapshots.
2. Distinguish predicate falsehood (`ContractViolation`) from predicate evaluator fault (`ContractPredicateFault`).
3. Keep the evaluator authority-free: no operation calls, handler dispatch, or row admission during predicate evaluation.
4. Insert checks at the correct boundary (function entry for `requires`, return boundary for `ensures`).
5. Respect recoverability only when an explicit `fail` or compensation operation row is present.

## TDD Steps

1. Add runtime tests for precondition false traps with caller-side blame.
2. Add runtime tests for postcondition false traps with callee/impl-side blame.
3. Add runtime tests proving predicate evaluator faults are distinct from false predicates.
4. Add authority-neutrality tests proving predicate evaluation performs no operations or authority acquisition.

## Completion Checklist

- [x] Dynamic predicate evaluator executes over captured environments and snapshots.
- [x] `ContractViolation` and `ContractPredicateFault` produce distinct traps.
- [x] Predicate evaluator remains authority-free.
- [x] Check insertion timing correct for `requires` and `ensures`.
- [ ] Recoverability gated by explicit `fail`/compensation row.
- [x] Focused runtime tests pass.

## Notes

Recoverability is wired as `TrapDefault` in the current plans. The remaining recoverability gating is tracked as a follow-up under PLAN-194 close-out work.
