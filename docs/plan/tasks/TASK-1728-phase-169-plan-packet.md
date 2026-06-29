# TASK-1728: Create the Phase 169 plan and task packet

## Status: ✅ Complete

## Summary

Create the Phase 169 implementation plan for surface expansion and notation elaboration, including
ordered task files, dependencies, verification commands, and honest deferrals from Phase 168.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 168 lowering inventory: `docs/audit/phase-168-surface-to-core-lowering-inventory.md`

## Dependencies

- ✅ Phase 168: Surface AST, notation, and lowering substrate

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Surface expansion and notation elaboration | Phase 168 closeout | Needed source-preserving carriers first | Yes | Plan this implementation packet | Plan, tasks, PLAN-INDEX, and CHANGELOG agree |
| Full macro expansion | SPEC-095c / Phase 168 | Hygiene and typed macro design are larger than notation substrate | No | Keep deferred | Plan non-goals name macro deferral |

## Files

- `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- `docs/plan/tasks/TASK-1728-phase-169-plan-packet.md`
- `docs/plan/tasks/TASK-1729-surface-expansion-traversal-api.md`
- `docs/plan/tasks/TASK-1730-notation-declaration-parser-ast.md`
- `docs/plan/tasks/TASK-1731-built-in-operator-token-normalization.md`
- `docs/plan/tasks/TASK-1732-local-notation-table-resolution.md`
- `docs/plan/tasks/TASK-1733-operator-section-elaboration.md`
- `docs/plan/tasks/TASK-1734-expanded-surface-lowering-gate.md`
- `docs/plan/tasks/TASK-1735-phase-169-closeout.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

## Requirements

1. Allocate globally unique task IDs after TASK-1727.
2. Preserve the Phase 168 recommended order unless a dependency audit justifies a change.
3. Mark the packet as planned, not implemented.
4. Keep non-goals explicit: macro hygiene, generalized mixfix, imported notation, and full lowering.
5. Include exact verification commands for implementation and docs-only slices.

## Work steps

1. Read Phase 168 closeout and lowering inventory.
2. Draft the Phase 169 plan with goals, non-goals, dependency graph, and acceptance criteria.
3. Create task files with concrete Current → Target handoff notes.
4. Update `PLAN-INDEX.md` and `CHANGELOG.md`.
5. Run docs validation.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; p=Path("docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md"); s=p.read_text(); assert "TASK-1728" in s and "TASK-1735" in s and "Non-goals" in s'
checklist:
  - [x] Plan file exists.
  - [x] All Phase 169 task files exist.
  - [x] PLAN-INDEX references Phase 169.
  - [x] CHANGELOG records the planning packet.
```

Verified on 2026-06-29 with:

```bash
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 - <<'PY'
from pathlib import Path
root=Path('docs/plan')
plan=root/'PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md'
assert plan.exists()
s=plan.read_text()
for n in range(1728,1736):
    assert f'TASK-{n}' in s, n
    matches=list((root/'tasks').glob(f'TASK-{n}-*.md'))
    assert len(matches)==1, (n,matches)
idx=(root/'PLAN-INDEX.md').read_text()
assert 'PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md' in idx
ch=Path('CHANGELOG.md').read_text()
assert 'PLAN-169 surface expansion and notation elaboration packet' in ch
PY
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the task packet consumed by TASK-1729 through TASK-1735.
