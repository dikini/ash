# TASK-1713: Reconcile Phase 1 cross-references and stale claims

## Status: 📝 Planned

## Summary

Reconcile Phase 1 cross-references and stale claims. This is a documentation-only task in PLAN-167 and belongs to Phase 1.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1712: Specify operator sections as callable sugar (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Close Phase 1 by sweeping target specs and indexes for stale surface claims. This is a docs-only
review/remediation task, not a new design task.

## Files

- `docs/spec/SPEC-095b-TARGET-GRAMMAR.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/spec/SPEC-INDEX.md`
- `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`
- `CHANGELOG.md`

## Requirements

1. Search for stale closed-operator, inline-handler, and contract-as-handler claims.
2. Ensure `SPEC-095b` and `SPEC-095c` do not contradict each other.
3. Ensure `SPEC-096b` trace-contract wording either links to grammar spelling or names the deferred
   surface syntax boundary.
4. Ensure `SPEC-097b` has at least a forward pointer to notation/section typing if detailed rules
   wait for TASK-1717.
5. Update `SPEC-INDEX.md` read paths for target surface grammar and AST/macros/notation.

## Docs-only steps

1. Run scoped searches before editing.
2. Patch stale live normative claims only; historical changelog context may remain.
3. Record residual follow-ups in the Phase 167 plan if discovered.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; specs="
".join(p.read_text() for p in Path("docs/spec").glob("SPEC-09*b*.md")); assert "requires ->" not in specs; assert "SPEC-095c" in Path("docs/spec/SPEC-INDEX.md").read_text()'
checklist:
  - [ ] Phase 1 stale-claim sweep completed.
  - [ ] SPEC-INDEX read paths updated.
  - [ ] Residual follow-ups are documented.
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
