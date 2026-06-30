# TASK-1751: Create the Phase 172 parser-first macro execution MVP plan packet

## Status: ✅ Complete

## Description

Create the Phase 172 plan and task packet for a conservative parser-first macro execution MVP. The packet must not implement macro execution; it defines an implementation-grade sequence that starts from Phase 171 fail-closed carriers and preserves all hygiene, origin, and scope-boundary invariants.

## Specification Reference

- PLAN-172: `docs/plan/PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`

## Dependencies

- ✅ TASK-1750: Phase 171 closeout

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expander | SPEC-095c / PLAN-171 | No hygiene/execution substrate | Partially | Plan only parser-first expression MVP | TASK-1752 audit + TASK-1753 spec patch |
| Imported macro activation | SPEC-095c import/export model | No summary carriers | No | Keep rejected | TASK-1755/TASK-1758 negative tests |
| Binder-introducing macros | SPEC-095c §7.4 | Binder hygiene absent | No | Reject in MVP | TASK-1756 whitelist + tests |

## Requirements

1. Create `PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md` with explicit goals, non-goals, decision gates, task graph, and acceptance criteria.
2. Create TASK-1751 through TASK-1759 files.
3. Update `PLAN-INDEX.md` with a Phase 172 row and section.
4. Update `CHANGELOG.md` under `[Unreleased]` with a planning-packet entry.
5. Mark only this packet task complete; all implementation tasks remain planned.

## Verification

```yaml
strictness: clean
commands:
  - python3 -c 'from pathlib import Path; root=Path("."); plan=(root/"docs/plan/PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md").read_text(); index=(root/"docs/plan/PLAN-INDEX.md").read_text(); changelog=(root/"CHANGELOG.md").read_text(); assert all(f"TASK-{n}" in plan for n in range(1751,1760)); assert all(len(list((root/"docs/plan/tasks").glob(f"TASK-{n}-*.md"))) == 1 for n in range(1751,1760)); assert "Phase 172" in index and "TASK-1751 through TASK-1759" in changelog; print("phase-172-packet-structure: OK")'
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Plan file exists.
  - [x] TASK-1751 through TASK-1759 files exist.
  - [x] PLAN-INDEX includes Phase 172 with 1/9 progress.
  - [x] CHANGELOG includes the packet creation entry.
```

## Completion Evidence

Created the Phase 172 parser-first macro execution MVP packet with TASK-1751 through TASK-1759, PLAN-INDEX registration, and changelog entry. Implementation remains intentionally unstarted.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides the task packet consumed by TASK-1752.
