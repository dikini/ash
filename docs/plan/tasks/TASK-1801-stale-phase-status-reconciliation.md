# TASK-1801: Reconcile stale Phase 151/152/157/158 status surfaces

## Status: ✅ Complete

## Description

Reconcile historical status drift after Phase 176 implementation decisions are known. This task should not pre-mark old work complete; it should make old plans/tasks/index rows agree with the actual current state.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1797, TASK-1798, and TASK-1800 complete or explicitly re-scoped

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Complete via TASK-1797 | Mark historical deferral complete via Phase 176 | Reference classification and removal tests passed |
| TASK-1580 | PLAN-158 | Needed power-tower lifting / pure-vs-Act distinction | Complete via TASK-1798 runtime callable environments | Mark historical deferral complete via Phase 176 | Closure lookup positive and private-helper non-leakage tests passed |
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Public API/config landed; execution re-scoped by TASK-1800 | Mark stale blocker text reconciled | Final-surface QuickCheck import/check fixtures pass |
| Phase 152 status drift | PLAN-152 vs PLAN-INDEX | Historical status drift | Reconciled | PLAN-152 task table now matches task files/PLAN-INDEX and notes QuickCheck re-scope | Docs gate |

## Requirements

### Functional Requirements

1. Patch PLAN-151/TASK-1511 closeout text after QuickCheck combinator decisions.
2. Patch PLAN-152 and TASK-1520 through TASK-1524 if they still show planned rows while PLAN-INDEX says complete, or annotate them as historical/superseded if appropriate.
3. Patch PLAN-157/TASK-1570 and PLAN-158/TASK-1580 to point to Phase 176 outcomes.
4. Update PLAN-INDEX summary/detail rows and CHANGELOG if status changed.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Audit status surfaces

Read the plan, task, index, and changelog surfaces for Phases 151, 152, 157, and 158.

### Step 2: Patch only after implementation outcome

Use Phase 176 task results as evidence; do not invent completion claims.

### Step 3: Run consistency assertions

Assert old rows and Phase 176 rows no longer contradict each other.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Old phase status drift reconciled
  - [x] No stale planned rows contradict PLAN-INDEX completion claims for targeted tasks
  - [x] Docs gate passes
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

This is the right place to handle Phase 152 drift; doing it before implementation would obscure whether it is merely historical or still behaviorally relevant.


## Completion Evidence

TASK-1801 reconciled:

- PLAN-151 planned task table drift for TASK-1497 through TASK-1506, TASK-1510, and TASK-1511, including the Phase 176 recursive-combinator re-scope.
- PLAN-152 planned task table drift for TASK-1520 through TASK-1524.
- PLAN-157 and TASK-1570 after TASK-1797 removed `Value::List`.
- PLAN-158 and TASK-1580 after TASK-1798 fixed module-level function visibility in closures.
- TASK-1511 after TASK-1800 landed the recursive-combinator public names/config but re-scoped execution fail-closed.

Verification run during completion:

```text
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```
