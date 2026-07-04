# TASK-1894: Contract Predicate Well-Formedness

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Validate contract predicates as stable, total, authority-free observer code.

## Requirements

1. Reject predicates whose expression row is non-empty.
2. Reject row-empty but unstable observers such as time, randomness, pointer identity, and unsafe
   force.
3. Reject operation calls, provider/handler installation, role admission, resource selection, and
   policy admission inside predicates.
4. Validate `old(...)`, boundary snapshots, and `result` scope.
5. Allow pure helper predicates only when their summaries prove stable observer behavior.

## TDD Steps

1. RED: add negative tests for operation calls, provider/handler use, admissions, unstable
   observers, invalid `old(...)`, and invalid `result` scope.
2. RED: add positive tests for pure helper predicates and simple value predicates.
3. GREEN: implement predicate-position validation and diagnostics.
4. Verify the checker rejects before Core/runtime check artifacts are emitted.

## Completion Checklist

- [ ] Authority-acquiring predicate forms fail closed.
- [ ] Unstable row-empty observers fail closed.
- [ ] Snapshot and `result` scope rules are enforced.
- [ ] Pure stable helper predicates are accepted.
- [ ] Diagnostics distinguish predicate well-formedness failures from contract violations.
