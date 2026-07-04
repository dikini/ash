# TASK-1898: Dynamic Contract Runtime Checks

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Execute dynamic contract checks using structured runtime check plans.

## Requirements

1. Evaluate `requires` before the function body with caller-side blame on false predicates.
2. Evaluate `ensures` after the function body with callee/impl-side blame on false predicates.
3. Emit `ContractViolation(ContractDiagnostic)` for false predicates.
4. Emit `ContractPredicateFault(PredicateFaultDiagnostic)` for evaluator traps, partial helper
   faults, missing captured values, or malformed runtime check plans.
5. Keep default contract traps non-resumable unless an explicit `fail` or compensation row is
   present.

## TDD Steps

1. RED: add runtime tests for passing pre/postconditions.
2. RED: add false `requires` and false `ensures` tests with expected blame.
3. RED: add predicate-fault tests distinct from predicate falsehood.
4. GREEN: execute runtime check plans at the right boundaries.
5. Verify recoverable behavior requires explicit failure or compensation rows.

## Completion Checklist

- [ ] Dynamic preconditions execute before body.
- [ ] Dynamic postconditions execute after body and bind `result`.
- [ ] False predicates and predicate faults produce distinct trap payloads.
- [ ] Blame polarity is preserved.
- [ ] Contract traps are not implicit operation effects.
