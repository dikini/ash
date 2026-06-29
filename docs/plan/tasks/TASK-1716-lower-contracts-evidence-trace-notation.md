# TASK-1716: Specify lowering for contracts, evidence, trace contracts, and notation erasure

## Status: 📝 Planned

## Summary

Specify lowering for contracts, evidence, trace contracts, and notation erasure. This is a documentation-only task in PLAN-167 and belongs to Phase 2.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1715: Specify lowering for callables, rows, do, handlers, and impls (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Complete `SPEC-098c` lowering coverage for facts, evidence, contract predicates, trace contracts,
macros, notation, and operator sections.

## Files

- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`
- `CHANGELOG.md`

## Requirements

1. Define lowering for local facts, proofs, evidence declarations, and row evidence references.
2. Define direct contract row sugar lowering to canonical fact/evidence/check artifacts.
3. Define `old(...)` snapshot lowering boundary by reference to NOTE-031/SPEC-100.
4. Define trace contract lowering to `TraceContract` sidecars and monitor plans.
5. Define notation and operator-section erasure before Core, preserving source-origin metadata.
6. State that macros are expanded before lowering and do not reach Core.

## Docs-only steps

1. Patch `SPEC-098c` with contract/evidence/trace/notation sections.
2. Cross-check against NOTE-033 and the Phase 165 carrier names in `SPEC-098b`/`SPEC-100`.
3. Patch `SPEC-096b` or `SPEC-100` only for cross-reference consistency.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md").read_text(); assert "LoweredPredicate" in s; assert "TraceContract" in s; assert "operator section" in s'
checklist:
  - [ ] Fact/evidence lowering specified.
  - [ ] Contract and trace-contract lowering specified.
  - [ ] Macro/notation/section erasure before Core specified.
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
