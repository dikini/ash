# TASK-1915: Legacy Workflow Form Boundary Reconciliation

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Reconcile specs, notes, and orientation indexes so the legacy `workflow` form is compatibility-only
for target planning.

## Requirements

- Mark legacy `workflow` form references as historical, compatibility, or current-state only.
- Ensure target runtime guidance routes through application entrypoints over checked computations.
- Preserve compatibility diagnostics without treating `workflow` syntax as a new target semantic
  foundation.
- Update `docs/spec/SPEC-INDEX.md` and `docs/notes/NOTE-INDEX.md` when normative routing changes.

## TDD Steps

1. Add failing stale-claim search evidence for live target docs.
2. Patch specs/notes/indexes to fence legacy workflow form language.
3. Re-run docs gates and stale-claim searches.

## Completion Checklist

- [x] Target specs no longer describe legacy `workflow` form as the application runtime foundation.
- [x] Compatibility references are clearly labeled.
- [x] Orientation indexes route application runtime work through PLAN-196.
- [x] Docs gates pass.

## Evidence

- [AUDIT-196: Legacy Workflow Boundary Reconciliation](../audits/AUDIT-196-legacy-workflow-boundary-reconciliation.md)
  records pre-patch stale-claim evidence and the routing changes made for TASK-1915.
- Updated `SPEC-096b`, `SPEC-097b`, `NOTE-016`, `SPEC-INDEX.md`, and `NOTE-INDEX.md` so
  application/runtime guidance routes through PLAN-196 and legacy `workflow` remains
  compatibility-only.
- Docs verification passed for stale-claim searches, `cargo fmt --check`,
  `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash scripts/check-docs-gate.sh`,
  and `git diff --check`.
