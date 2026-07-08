# TASK-1960: Deprecated Functionality Removal Plan Packet

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Create the Phase 201 plan and task packet for completely removing deprecated Ash functionality
after Phase 200 demoted legacy/deprecated forms from productive surfaces.

## Requirements

- Add a Phase 201 plan that treats complete removal as the product goal.
- Include a required audit before removal work begins.
- Require deprecated Ash forms to be absent from repository code while disallowing them as valid
  Ash.
- Split the phase into task files under `docs/plan/tasks/`.
- Update `PLAN-INDEX.md` and `CHANGELOG.md`.

## TDD Steps

1. Add the Phase 201 plan and task files.
2. Update PLAN-INDEX summary and detailed phase sections.
3. Update CHANGELOG with the new phase packet.
4. Run docs orientation and docs gates.

## Completion Checklist

- [x] Phase 201 plan exists.
- [x] TASK-1960 through TASK-1968 task files exist.
- [x] PLAN-INDEX includes Phase 201 summary and task entries.
- [x] CHANGELOG records the Phase 201 packet.
- [x] Docs gates are recorded after the packet is written.

## Evidence

- `python3 tools/docs/validate_orientation_indexes.py --self-test` passed with
  `orientation-index-check: OK`.
- `bash scripts/check-docs-gate.sh` passed with markdown links checked=1436 missing=0 and
  `docs-gate: OK`.
- `git diff --check` passed.
