# TASK-1974: Historical Reference Routing

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Quarantine historical workflow/tower reference pages so agent-facing and user-facing indexes do not
present removed Act/Proc/Workflow tower surfaces as current productive Ash guidance.

## Requirements

- Mark or route historical workflow/tower reference pages away from current feature/status indexes.
- Update agent cards that still classify Act/Proc/Workflow pages as current guidance.
- Keep historical prose available for context, but label it as historical and point current readers
  to target functions, rows/effects, process/channel helpers, application runtime reports, and
  productive examples.
- Add or preserve docs-gate evidence that links remain valid and orientation indexes stay healthy.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Search reference indexes, status pages, and agent cards for current routes to historical
   workflow/tower pages.
2. Patch status and card metadata/prose so those pages are historical-only and no current feature
   matrix row promotes removed tower APIs.
3. Verify markdown links and orientation indexes through the docs gate.
4. Record the completed routing evidence in the Phase 201 audit and changelog.

## Completion Checklist

- [x] Historical workflow/tower pages are not listed as current productive feature guidance.
- [x] Agent cards for Act/Proc/Workflow no longer claim current status for removed tower surfaces.
- [x] Current routing points to target functions/effects/process/application runtime guidance.
- [x] Docs gate and orientation index validation pass.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

- Retargeted `reference/status/feature-matrix.md` from Act/Proc/Workflow/tower rows to effect-row
  admission, process/channel helpers, application reports, productive stdlib helpers, and Result.
- Moved historical Act/Proc/Workflow/generalized-do and stdlib tower cards out of the normal agent
  context-pack retrieval order.
- Updated `reference/INDEX.md`, `reference/agents/README.md`, and
  `reference/getting-started/next-steps.md` so current readers are routed to target functions,
  runtime admission, runtime reports, checked examples, Result, and algebra pages.
- Marked Act/Proc/Workflow/generalized-do agent cards as `superseded` and historical-only.
- Verification:
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
