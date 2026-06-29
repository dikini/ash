# TASK-1726: Inventory and scope surface-to-Core lowering implementation seams

## Status: 📝 Planned

## Summary

Map the current lowering path against `SPEC-098c` after the Phase 168 parser/surface substrate is in
place. This task creates exact implementation ownership for the next lowering phase rather than
trying to lower every target surface form in Phase 168.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- TASK-1725 expanded-surface-AST boundary

## Dependencies

- 📝 TASK-1725: Expanded surface AST boundary

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| General surface-to-Core lowering | SPEC-098c | Needed expanded surface substrate first | Partial | Inventory and scope next implementation packet | Artifact names exact lowering seams and follow-on task IDs/placeholders |

## Files

- `docs/audit/phase-168-surface-to-core-lowering-inventory.md`
- Current parser lowering modules identified by TASK-1721
- Current engine adapter modules identified by TASK-1721
- Relevant parser/engine tests for existing lowerer behavior

## Requirements

1. Identify the live lowering entry points from parser surface AST toward Core or engine forms.
2. Compare live behavior to `SPEC-098c` sections for callables/rows, do, handlers, impls,
   contracts/evidence/trace contracts, notation erasure, and source origins.
3. Classify each surface family as implemented, partially implemented, rejected, or unowned.
4. Recommend the next implementation packet and task ordering.
5. Do not mark deferred lowering families complete just because this inventory exists.

## Work steps

1. Inspect the post-TASK-1725 code and focused tests.
2. Write `docs/audit/phase-168-surface-to-core-lowering-inventory.md`.
3. Include a lowering-family matrix and exact proposed next-phase task list.
4. Run focused existing lowerer/parser tests sufficient to prove the inventory references live seams.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo fmt --check
  - git diff --check
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; s=Path("docs/audit/phase-168-surface-to-core-lowering-inventory.md").read_text(); assert "SPEC-098c" in s and "lowering-family matrix" in s.lower()'
checklist:
  - [ ] Lowering inventory artifact exists.
  - [ ] Every `SPEC-098c` lowering family is classified.
  - [ ] Next lowering phase/task ordering is explicit.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the lowering inventory consumed by TASK-1727.
