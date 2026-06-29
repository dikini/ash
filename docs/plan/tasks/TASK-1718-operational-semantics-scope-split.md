# TASK-1718: Rewrite SPEC-099b scope and preserve Phase 159 interpreter semantics as context

## Status: ✅ Complete

## Summary

Rewrite SPEC-099b scope and preserve Phase 159 interpreter semantics as context. This is a documentation-only task in PLAN-167 and belongs to Phase 3.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- ✅ TASK-1717: Tighten surface type inference for rows, evidence, handlers, operation identity, and notation (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Rescope `SPEC-099b` so it owns current target operational semantics rather than only Phase 159 CPS
interpreter behavior. Preserve the Phase 159 semantics as historical/implementation context instead
of deleting useful material.

## Files

- `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- optional `docs/reference/phase-159-cps-interpreter-semantics.md` or appendix section
- `docs/spec/SPEC-INDEX.md`
- `CHANGELOG.md`

## Requirements

1. Rewrite the scope/header to target current Core/CPS semantics.
2. Move, demote, or clearly label Phase 159 CPS-interpreter semantics as implementation context.
3. Define the intended split: Core big-step semantics, Core/CPS small-step semantics, and runtime
   monitor/provider behavior.
4. Remove the live claim that full contract discharge is deferred.
5. Add cross-references to `SPEC-098c`, `SPEC-100`, and the Phase 167 audit.

## Docs-only steps

1. Read the current `SPEC-099b` sections before patching.
2. Avoid rewriting all rules in this task; focus on scope, ownership, and section scaffolding.
3. Update `SPEC-INDEX.md` semantics read path if needed.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md").read_text(); assert "Phase 159" in s; assert "full contract discharge" not in s.lower() or "deferred" not in s.lower()'
checklist:
  - [x] SPEC-099b scope is current target semantics.
  - [x] Phase 159 interpreter semantics are preserved as context.
  - [x] Contract discharge is no longer live-deferred.
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
