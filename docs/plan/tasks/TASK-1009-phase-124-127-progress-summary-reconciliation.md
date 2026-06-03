# TASK-1009: Phase 124/127/128 Progress Summary Reconciliation

## Status: ✅ Complete

## Description

Reconcile PLAN-INDEX progress summary drift discovered during the post-Phase-131 roadmap inventory. This is an interphase documentation/status maintenance task, not a new language/runtime implementation phase.

## Scope

- Fix the Phase 124 progress-table row so it matches the completed Phase 124 section, completed TASK-946 through TASK-953 rows, and SPEC-071 Implemented MVP status.
- Clarify the Phase 127 and Phase 128 progress-table rows so Phase 127 remains the historical partial SPEC-073 first-slice closeout while Phase 128 owns the deferred-row closure and SPEC-073 Implemented MVP promotion.
- Verify the dirty root reference-document edits do not change roadmap, PLAN-INDEX, phase/task/spec status, or next-task conclusions.

## Non-Goals

- Do not create Phase 132.
- Do not reopen or reclassify Phase 76B, Phase 89, TASK-063, TASK-368b, or TASK-599.
- Do not modify runtime, parser, typechecker, stdlib, or reference-language semantics.
- Do not overwrite the dirty root reference-document changes from the Phase 131 / TASK-1008 documentation follow-up.

## Requirements

1. Inspect both PLAN-INDEX progress-table surfaces and patch only the current summary rows that carry Phase 124/127/128 drift.
2. Preserve the Phase 124 section and task rows if they are already internally consistent.
3. Preserve Phase 127 as historical partial closeout language rather than pretending TASK-974 promoted SPEC-073 to Implemented MVP.
4. Preserve Phase 128 as the closure/promotion owner for the deferred SPEC-073 rows.
5. Add a matching CHANGELOG.md entry.
6. Run scoped documentation verification after editing.

## Evidence

- Root dirty-reference-doc audit found changes only in CHANGELOG.md, docs/reference/*, and reference/language/functions/* around SPEC-076/TASK-1008 semantics; no dirty docs/plan or docs/spec files changed roadmap/status surfaces.
- Phase 124 PLAN-INDEX section already marks Status as complete and lists TASK-946 through TASK-953 complete.
- SPEC-071 already records Implemented MVP status.
- TASK-974 records Phase 127 closeout as complete while preserving SPEC-073 Draft/deferred rows.
- TASK-986 and SPEC-073 record the Phase 128 Implemented MVP promotion.

## Verification

- Dirty root audit command scope: `git status --short --branch`, `git diff --stat`, `git diff --name-status`, and targeted dirty-diff searches in `/home/dikini/Projects/ash`; result showed dirty changes only in CHANGELOG.md, docs/reference/*, and reference/language/functions/* with no dirty docs/plan or docs/spec roadmap/status edits.
- Scoped docs gate command: `bash scripts/check-docs-gate.sh`; result: `git diff --check` passed, changelog policy regression tests passed, Markdown links checked 2547 with 0 missing, `docs-gate: OK`.

## Completion Checklist

- [x] Phase 124 summary row reconciled to 8/8 complete.
- [x] Phase 127 summary row clarified as historical partial with all first-slice task rows complete.
- [x] Phase 128 summary row clarified as the deferred-row closure and SPEC-073 Implemented MVP owner.
- [x] Dirty root reference-document changes audited for roadmap/status impact.
- [x] CHANGELOG.md updated.
- [x] Scoped docs verification run on edited files.
