# Residual Spec-Audit Follow-up Plan

**Goal:** Close the explicit spec-only documentation debt that remained after the final
implementation convergence audit.

**Scope:** Docs/spec/reference work only. No Rust code changes. No new feature design beyond
resolving the residual findings recorded in
`docs/audit/2026-03-20-final-convergence-audit.md`.

**Success criteria:**

- the still-open spec-only findings from the final audit are assigned to explicit task owners
- low-severity spec hygiene work is scoped separately from the higher-priority spec inconsistencies
- `PLAN-INDEX.md` shows a bounded post-convergence docs-only phase rather than leaving residual
  audit items implicit

## Task Set

1. [TASK-213](tasks/TASK-213-reconcile-module-and-import-spec-scope.md)
   Resolve the remaining scope conflict between `SPEC-009` and `SPEC-012`, including any directly
   related example-type normalization in those specs.
2. [TASK-214](tasks/TASK-214-fix-residual-policy-and-typed-provider-spec-drift.md)
   Fix the remaining stale forward reference in `SPEC-015`, correct the policy-conflict example in
   `SPEC-007`, and document any still-intended limitation around provider effect granularity.
3. [TASK-215](tasks/TASK-215-normalize-residual-spec-hygiene.md)
   Clean up the residual low-severity spec hygiene findings such as example type normalization and
   uneven status/editorial formatting across the affected spec set.

## Execution Notes

- `TASK-213` and `TASK-214` can run in parallel.
- `TASK-215` should follow them so the hygiene pass reflects the corrected canonical text.
- After those tasks, the final convergence audit can be amended or superseded only if needed; do
  not reopen the closed implementation convergence path unless a new contradiction is discovered.
