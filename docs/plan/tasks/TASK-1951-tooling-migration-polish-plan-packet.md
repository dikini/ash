# TASK-1951: Tooling/Migration Polish Plan Packet

**Status:** Complete
**Phase:** [PLAN-200: Tooling And Migration Polish](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md)

## Description

Create and index the Phase 200 plan and task packet for migration-first tooling polish, with
legacy and deprecated form elimination as the primary objective.

## Requirements

- Add PLAN-200 after Phase 199.
- Create task files for every Phase 200 implementation and closeout task.
- Update PLAN-INDEX and CHANGELOG.
- Make legacy/deprecated-form elimination the first implementation task and the design lock for
  diagnostics, formatter, LSP, examples, and docs.

## TDD Steps

1. Add the phase plan and task files.
2. Run documentation orientation and gate checks.
3. Verify PLAN-INDEX links resolve.

## Completion Checklist

- [x] PLAN-200 exists.
- [x] TASK-1951 through TASK-1959 exist.
- [x] PLAN-INDEX references the phase and task files.
- [x] CHANGELOG.md records the planning packet.

## Evidence

- PLAN-200 and TASK-1951 through TASK-1959 were added in commit `b0d356f6`.
- PLAN-INDEX and CHANGELOG include the Phase 200 planning packet.
- TASK-1952 is sequenced as the first implementation task so legacy/deprecated-form elimination
  drives diagnostics, formatter, LSP, examples, and docs polish.
