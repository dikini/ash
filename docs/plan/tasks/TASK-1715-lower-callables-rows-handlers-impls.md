# TASK-1715: Specify lowering for callables, rows, do, handlers, and impls

## Status: 📝 Planned

## Summary

Specify lowering for callables, rows, do, handlers, and impls. This is a documentation-only task in PLAN-167 and belongs to Phase 2.

## Specification Reference

- PLAN-167: `docs/plan/PLAN-167-TARGET-SURFACE-SEMANTICS-GAP-CLOSURE.md`
- Audit: `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Dependencies

- 📝 TASK-1714: Create surface-to-Core lowering spec scaffold (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Target surface/semantics gap | `docs/audit/2026-06-29-target-spec-notes-gap-audit.md` | Target specs are not implementation-grade for parser/macro/lowering/semantics work | Yes, audit preserved and PLAN-167 created | Implement this docs-only slice now | Docs gate plus task-specific structural assertion |

## Description

Add construct-specific lowering rules for the core surface computation forms: callable declarations,
rows, `do`, handlers, and impl operation identity.

## Files

- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md` if cross-references need tightening
- `CHANGELOG.md`

## Requirements

1. Define lowering for function/callable declarations with inline rows and `where row` layout.
2. Define row inference/defaulting as a lowering input/output contract, leaving detailed type
   inference to TASK-1717.
3. Define `do` sequencing lowering and row accumulation at a spec level.
4. Define `handle expr with name`, `on`, and `done(value)` lowering boundaries.
5. Define `derive handler` as synthesis before or during lowering, with explicit source-origin
   metadata.
6. Define operation identity lowering from abstract `F::op` to concrete `ImplType::op`.

## Docs-only steps

1. Patch `SPEC-098c` with focused sections and examples.
2. Use actual target syntax from `SPEC-095b`/`SPEC-095c`; mark any pseudo-code as pseudo-code.
3. Keep contracts/facts/evidence out of this task except for forward references.
4. Update changelog.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md").read_text(); assert "where row" in s; assert "handle expr with" in s; assert "ImplType::op" in s'
checklist:
  - [ ] Callable/row lowering specified.
  - [ ] Handler lowering specified.
  - [ ] Impl operation identity lowering specified.
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
