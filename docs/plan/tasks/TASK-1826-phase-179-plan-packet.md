# TASK-1826: Create the Phase 179 explicit row admission wiring packet

## Status: ✅ Complete

## Description

Create and register the Phase 179 planning packet for explicit row admission/runtime wiring. This task is planning-only and must not implement runtime behavior.

## Specification Reference

- [PLAN-179](../PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md)
- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-099b](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Requirements

1. Add `docs/plan/PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md`.
2. Add TASK-1826 through TASK-1834 files.
3. Register Phase 179 in `docs/plan/PLAN-INDEX.md`.
4. Update orientation/read paths where Phase 179 prevents stale target-Ash planning turns.
5. Update `CHANGELOG.md` with a planning entry.
6. Preserve explicit non-goals for row-polymorphic inference, handler execution, provider registration, and corpus migration.

## Verification

```bash
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Completion Checklist

- [x] PLAN-179 exists.
- [x] TASK-1826 through TASK-1834 exist.
- [x] PLAN-INDEX references Phase 179.
- [x] Relevant orientation indexes point to Phase 179.
- [x] CHANGELOG records the planning packet.
