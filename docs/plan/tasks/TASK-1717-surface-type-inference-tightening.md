# TASK-1717: Tighten surface type inference for rows, evidence, handlers, operation identity, and notation

## Status: ✅ Complete

## Summary

Tighten surface type inference for rows, evidence, handlers, operation identity, and notation. This is a documentation-only task in PLAN-167 and belongs to Phase 2.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- ✅ TASK-1716: Specify lowering for contracts, evidence, trace contracts, and notation erasure (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Patch `SPEC-097b` with the surface type-inference rules needed by the new surface AST and lowering
specs. This task does not alter Core type checking; `SPEC-100` remains annotation-led Core checking.

## Files

- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `CHANGELOG.md`

## Requirements

1. Specify missing row inference/defaulting rules for callable surfaces.
2. Specify how inline row and `where row` normalize to one callable row.
3. Specify fact/evidence inference and discharge boundaries at a high level.
4. Specify handler marker subtyping: `handler τ <: τ`, not conversely.
5. Specify operation identity inference and specialization for abstract/concrete impl types.
6. Specify notation and operator-section typing after notation resolution.
7. Preserve the boundary that Core checking is not full HM inference.

## Docs-only steps

1. Patch the relevant `SPEC-097b` type-inference sections.
2. Add cross-references from `SPEC-095c` and `SPEC-098c` if needed.
3. Run stale-claim searches for handler-marker-as-alias or closed operator typing claims.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md").read_text(); assert "handler" in s and "<:" in s; assert "operator section" in s; assert "where row" in s'
checklist:
  - [x] Row inference/defaulting specified.
  - [x] Handler marker subtyping specified.
  - [x] Notation and operator-section typing specified.
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
