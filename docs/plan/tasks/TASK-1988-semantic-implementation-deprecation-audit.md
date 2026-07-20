# TASK-1988: Semantic Implementation and Deprecation Audit

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1986 and Phase 201 reconciliation

## Description

Map canonical semantic rules to Rust symbols, tests, examples, diagnostics, and runtime artifacts;
then produce evidence-led delete/fold/retain/decision tasks for conflicts and orphaned mechanisms.

## Requirements

- Use Rust language-aware symbol/reference/diagnostic tools before broad tracing.
- Reconcile TASK-1971/TASK-1972 and the Phase 201 worktree before assigning overlap.
- Distinguish semantic deletion from rename-only cleanup.
- Require behavior-level parity or absence evidence for every removal.
- Create task files before any resulting implementation work begins.

## TDD Steps

1. Define mapping and classification fixtures.
2. Audit one canonical vertical slice at a time.
3. Verify every public semantic implementation has an owner or private-machinery rationale.
4. Publish the deprecation/removal packet and run docs/index gates.

## Completion Checklist

- [ ] Canonical-to-Rust/test mapping is complete for programme scope.
- [ ] Conflicts, partial implementations, and unmapped code are explicit.
- [ ] Every removal/refactor has an evidence requirement and task owner.
- [ ] No Phase 201 evidence or in-flight user change is overwritten.
