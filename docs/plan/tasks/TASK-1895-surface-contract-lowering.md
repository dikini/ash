# TASK-1895: Surface Contract Lowering

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Lower surface contracts into Core predicate sidecars, snapshots, runtime check plans, and discharge
metadata.

## Requirements

1. Lower each checked `requires`/`ensures` clause to a structured `LoweredPredicate` or equivalent
   live carrier.
2. Record `PredicateRef`, boundary id, clause kind, binder environment, snapshot refs, and source
   origin.
3. Emit runtime check plans for dynamically checked predicates.
4. Preserve proof/discharge slots without requiring a solver implementation.
5. Ensure Core contains no raw source-text predicate re-evaluation path.

## TDD Steps

1. RED: add lowering tests that inspect Core predicate sidecars and runtime check plan metadata.
2. RED: add negative tests proving rejected predicates do not lower.
3. GREEN: implement lowering from surface contract carriers to Core artifacts.
4. Verify Core text/summary round trips preserve stable references where supported.

## Completion Checklist

- [ ] Lowered predicates preserve boundary and source identity.
- [ ] Runtime check plans reference structured predicate artifacts.
- [ ] Snapshot refs are boundary-local.
- [ ] Rejected predicates do not produce runtime artifacts.
- [ ] No source-text predicate evaluator path is introduced.
