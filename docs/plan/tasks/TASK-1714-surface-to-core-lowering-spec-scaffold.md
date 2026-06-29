# TASK-1714: Create surface-to-Core lowering spec scaffold

## Status: ✅ Complete

## Summary

Create surface-to-Core lowering spec scaffold. This is a documentation-only task in PLAN-167 and belongs to Phase 2.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- ✅ TASK-1713: Reconcile Phase 1 cross-references and stale claims (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Create the general surface-to-Core lowering spec scaffold. This task defines the lowering pipeline,
input/output boundaries, invariants, and non-goals before adding construct-specific lowering rules.

## Files

- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/spec/SPEC-INDEX.md`
- `CHANGELOG.md`

## Requirements

1. Define lowering input as expanded surface AST from `SPEC-095c`, not raw parser syntax.
2. Define lowering output as Core AST plus sidecars suitable for `SPEC-100` Core checking and
   `SPEC-098b` CPS lowering.
3. State invariants: macros/notation/operator sections are erased before Core; source origins are
   preserved; rows/facts/evidence get stable identities.
4. State non-goals: parser implementation, macro expander implementation, and runtime behavior.
5. Add cross-references from `SPEC-098b` to the new lowering spec.
6. Add `SPEC-098c` to `SPEC-INDEX.md` with read path placement.

## Docs-only steps

1. Create `SPEC-098c` with frontmatter matching nearby specs.
2. Patch `SPEC-098b` only enough to point to `SPEC-098c` for surface lowering.
3. Update `SPEC-INDEX.md` and changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md").read_text(); assert "expanded surface AST" in s; assert "Core AST" in s; assert "sidecar" in s'
checklist:
  - [x] SPEC-098c exists.
  - [x] Lowering boundary is explicit.
  - [x] SPEC-098b points to SPEC-098c.
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
