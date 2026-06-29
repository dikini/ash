# TASK-1736: Create the Phase 170 plan and task packet

## Status: ✅ Complete

## Summary

Create the Phase 170 implementation packet for expanded-surface integration and notation scoping, registering the plan, task files, changelog entry, and PLAN-INDEX rows before implementation begins.

## Specification Reference

- PLAN-170: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`
- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c: surface AST, macros, and notation
- SPEC-098c: surface-to-Core lowering

## Dependencies

- ✅ TASK-1735: Phase 169 closeout and review

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Expanded-surface integration | PLAN-169 closeout | Phase 169 added helper gates but did not audit all public call sites | Yes | Plan Phase 170 boundary audit and routing tasks | PLAN-170 and TASK-1737/1738 exist |
| Imported/exported notation | PLAN-169 non-goals | Needed summary carrier design | Partial | Plan design plus bounded implementation/deferral tasks | TASK-1739/1740 exist |
| Origin sidecars | PLAN-169 non-goals | Needed separate carrier decision | Partial | Plan narrow design/implementation task | TASK-1741 exists |

## Requirements

1. Create PLAN-170 with goals, non-goals, decision gates, task structure, and verification policy.
2. Create TASK-1736 through TASK-1742 task files.
3. Register Phase 170 in `docs/plan/PLAN-INDEX.md` with `1/7` progress.
4. Add a `CHANGELOG.md` planning entry under `[Unreleased]`.
5. Keep task files rendered at column 0 with executable verification commands.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] PLAN-170 exists and links all Phase 170 task files.
  - [x] TASK-1736 through TASK-1742 files exist.
  - [x] PLAN-INDEX has Phase 170 summary and body rows.
  - [x] CHANGELOG has a Phase 170 planning entry.
```

## Notes

This task is completed by creation of the planning packet. Implementation starts at TASK-1737.
