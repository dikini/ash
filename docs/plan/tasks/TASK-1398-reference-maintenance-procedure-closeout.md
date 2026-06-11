# TASK-1398: Reference maintenance procedure and closeout

## Status: ✅ Complete

## Description

Document a repeatable post-phase closeout reference refresh procedure, update CHANGELOG, and reconcile PLAN-INDEX.

## Specification Reference

- [PLAN-139: Reference Maintenance and Staleness Remediation](../PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md)

## Dependencies

- TASK-1395, TASK-1396, TASK-1397 complete.

## Requirements

### Functional Requirements

- Update `reference/maintenance/refresh-procedure.md` with a repeatable post-phase reference refresh checklist.
- Add CHANGELOG entry under `[Unreleased]` for Phase 139.
- Update PLAN-INDEX summary table: Phase 139 → Complete.
- Update PLAN-INDEX phase body: all tasks → Complete.

## Files

- Modify: `reference/maintenance/refresh-procedure.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md`

## Verification

- [ ] Procedure is repeatable by future agents without asking questions.
- [ ] CHANGELOG entry follows Common Changelog format.
- [ ] PLAN-INDEX summary and phase tables are consistent.
- [ ] Markdown link check passes.
- [ ] Docs gate passes.
