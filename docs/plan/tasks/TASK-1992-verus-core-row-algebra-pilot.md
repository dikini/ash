# TASK-1992: Verus Pilot 1 — Core Row Algebra

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1991 and the canonical row-rule owner from TASK-1986

## Description

Verify the PLAN-202 closed-row theorem set for `normalize_core_row` and
`core_row_included_in`, then measure proof and maintenance cost.

## Requirements

- Write property tests before proof/implementation adaptations.
- Define and check an explicit `CoreRow`/`CoreRowItem` spec view.
- Prove membership preservation, duplicate elimination, idempotence, normalization invariance,
  stable first-occurrence order, non-increasing length, membership/inclusion-truth permutation
  invariance, and closed inclusion reflexivity/transitivity.
- Keep open tails, ordered output equivalence, and diagnostic-payload equivalence outside this
  closed-row pilot.
- Keep ambiguous group rejection and row-non-authority visible.
- Perform one representation-preserving refactor and record proof maintenance evidence.

## TDD Steps

1. Add failing/strengthened property and mutation fixtures for the theorem set.
2. Define the Verus spec model and checked executable view.
3. Add proofs incrementally and run the pinned verifier.
4. Run focused Cargo tests, formatting, clippy, docs, and TCB gates.

## Completion Checklist

- [ ] All listed theorems are verified, or the pilot records an evidence-backed no-go result and
  conditionally blocks TASK-1993 without blocking programme closeout.
- [ ] No broad unreported assumption establishes correspondence.
- [ ] Existing runtime/property evidence remains green.
- [ ] Maintenance and LLM-repair measurements are recorded.
