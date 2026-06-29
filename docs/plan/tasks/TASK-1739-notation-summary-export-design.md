# TASK-1739: Specify notation summary/export and visibility semantics

## Status: ✅ Complete

## Summary

Design the module-summary, visibility, import, and export semantics for notation declarations before implementing any cross-module notation propagation.

## Specification Reference

- PLAN-170: notation scoping track
- SPEC-095c §7 and §10: notation declarations and active notation tables
- SPEC-098c §11: type inference interface for notation target resolution
- PLAN-169 TASK-1732: local-only notation table

## Dependencies

- ✅ TASK-1736: Phase 170 packet created
- ✅ TASK-1737: Boundary audit identified module-summary consumers

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported notation propagation | PLAN-169 non-goal | No honest summary/visibility design | Design required | Specify before implementation | Design note and tests planned in TASK-1740 |
| Inline-module notation leakage | Phase 169 review | Local table flattened scopes incorrectly before fix | Fixed locally | Preserve no-leakage invariant in cross-module design | Positive/negative scope matrix |

## Requirements

1. Create `docs/design/phase-170-notation-summary-export-semantics.md`.
2. Define whether notation declarations are exportable, importable, re-exportable, or strictly local.
3. Define how visibility applies to notation declarations and their callable targets.
4. Define conflict resolution for local vs imported notation and imported-vs-imported duplicates.
5. Define summary carrier requirements if notation is transported across modules.
6. Define negative leakage invariants for parent/inline/import scopes.
7. Decide whether TASK-1740 implements propagation or formalizes non-propagation.

## TDD Steps

1. Write a scope matrix in the design note before code changes.
2. Map each row to a positive or negative test planned for TASK-1740.
3. Update TASK-1740 with the selected implementation branch.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Design note covers local, inline, imported, exported, and re-exported cases.
  - [x] Decision is explicit: preserve non-propagation.
  - [x] TASK-1740 test matrix is concrete.
```

## Closeout evidence

- Design note: `docs/design/phase-170-notation-summary-export-semantics.md`.
- Decision: Phase 170 preserves explicit non-propagation; notation remains module-local until summary carriers can transport notation metadata honestly.
- TASK-1740 patched with the selected non-propagation branch and concrete scope-matrix tests.
- Fresh verification:
  - `git diff --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
