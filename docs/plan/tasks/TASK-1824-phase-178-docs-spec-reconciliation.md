# TASK-1824: Reconcile docs/spec/status for Phase 178 boundaries

## Status: ✅ Complete

## Description

Reconcile docs and status surfaces after Phase 178 implementation outcomes are known. This task must clarify exactly which source-to-Core row bridge exists and which target-Ash capabilities remain future work.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)

## Dependencies

- TASK-1818 through TASK-1823 complete or explicitly re-scoped.

## Requirements

### Functional Requirements

1. Update PLAN-178 task statuses and completion evidence according to implementation results.
2. Update PLAN-INDEX Phase 178 status and task table.
3. Update CHANGELOG with implementation entries for completed Phase 178 work.
4. Patch relevant docs/spec notes only where Phase 178 changes current implementation status or prevents stale overclaims.
5. Preserve explicit future-work caveats for row-polymorphic inference, provider/admission runtime wiring, handler execution, and broad corpus migration.

### Property Requirements

- Do not mark target specs fully implemented because Phase 178 closes one bridge.
- Do not remove Phase 177 caveats unless Phase 178 evidence truly retires them.
- Keep orientation indexes current if specs/notes are added, renamed, or meaningfully reclassified.

## TDD Steps

### Step 1: Inspect implementation outcomes

Read completed task evidence from TASK-1818 through TASK-1823.

### Step 2: Patch status surfaces

Update PLAN-178, PLAN-INDEX, task files, CHANGELOG, and any touched specs/notes.

### Step 3: Verify docs

Run docs gates and targeted assertions for Phase 178 status references.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] PLAN-178 status agrees with task statuses.
  - [x] PLAN-INDEX agrees with PLAN-178.
  - [x] CHANGELOG has Phase 178 entries.
  - [x] Future-work caveats remain honest.
```

## Completion Evidence

- Reconciled `SPEC-098c` and `SPEC-100` with the Phase 178 implementation boundary: explicit source callable rows now reach Core function row metadata, while row-polymorphic inference and runtime authority wiring remain future work.
- Updated `SPEC-INDEX` read paths for the current Phase 178 source-to-Core row bridge.
- Verification: `git diff --check`, `python3 tools/docs/validate_orientation_indexes.py --self-test`, and `bash scripts/check-docs-gate.sh`.

## Dependencies for Next Task

This task feeds TASK-1825.
