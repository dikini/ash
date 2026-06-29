# TASK-1720: Integrate contracts, traces, monitors, lazy/memo semantics, and close out Phase 167

## Status: ✅ Complete

## Summary

Integrate contracts, traces, monitors, lazy/memo semantics, and close out Phase 167. This is a documentation-only task in PLAN-167 and belongs to Phase 3.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- ✅ TASK-1719: Add target Core big-step and Core/CPS small-step semantics (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Complete the operational-semantics rewrite by integrating contract checks, predicate faults,
structured diagnostics, trace facts, temporal monitors, lazy/memo contract timing, and closeout
status surfaces.

## Files

- `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`
- `docs/spec/SPEC-INDEX.md`
- `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`
- `CHANGELOG.md`

## Requirements

1. Define operational behavior for dynamic contract checks:
   false predicate, predicate evaluator fault, and structured `ContractViolation` trap.
2. Define temporal contract violation versus monitor fault behavior.
3. Define trace fact emission and monitor observation boundaries.
4. Define lazy/memo contract timing and memo replay of terminal diagnostics/blame.
5. Reconcile cross-spec references and stale claims.
6. Update PLAN-167 and PLAN-INDEX status surfaces only when all prior tasks are complete.
7. Record audit gap closure status: closed, partially closed, or deliberately deferred.

## Docs-only steps

1. Patch `SPEC-099b` runtime contract/trace sections.
2. Sweep `SPEC-096b`/`097b`/`098b`/`098c`/`100` for contradictory references.
3. Update indexes, plan status, task statuses, audit status notes, and changelog.
4. Run final docs verification.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md").read_text(); assert "ContractViolation" in s; assert "PredicateFault" in s or "predicate fault" in s; assert "Temporal" in s or "monitor" in s'
checklist:
  - [x] Dynamic contract operational behavior specified.
  - [x] Trace/monitor operational behavior specified.
  - [x] Lazy/memo contract timing specified.
  - [x] Phase 167 closeout surfaces reconciled.
```


## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for next task

This task produces a reviewed documentation slice that the next PLAN-167 task consumes.

## Notes

- This task is documentation-only. Do not add Rust implementation gates.
- Use actual Ash target syntax from existing specs. Mark proposed or illustrative syntax explicitly.
- If an independent review finds blockers, leave the task planned/in progress until those blockers are fixed.
