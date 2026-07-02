# TASK-1806: Create the Phase 177 target-Ash row syntax and Core/CPS alignment packet

## Status: ✅ Complete

## Description

Create and register the Phase 177 planning packet for target-Ash computation-row/effect syntax integration and Core/CPS row taxonomy alignment. This task is documentation/planning only and does not implement Rust behavior.

## Specification Reference

- [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-020: Computation Row Taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- [NOTE-022: Effects as Interfaces](../../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-023: Handler Surface](../../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- [NOTE-025: Effect Identity via Sorts and Impls](../../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)

## Dependencies

- ✅ Phase 176 closeout committed on `main`.
- ✅ TASK-1803 through TASK-1805 interphase docs/status reconciliation completed.

## Requirements

### Functional Requirements

1. Create `PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md` with scope, non-goals, decision gates, tasks, and verification baseline.
2. Create TASK-1806 through TASK-1815 task files with dependencies, requirements, dispatch metadata, and verification commands.
3. Register Phase 177 in PLAN-INDEX progress and detail sections.
4. Add a CHANGELOG.md planning entry under `[Unreleased]`.

### Property Requirements

- Phase 177 must preserve the original requested scope: surface computation-row/effect syntax integration plus Core/CPS row taxonomy alignment.
- Planning language must not declare full target Ash implemented.
- Row syntax must be framed as requirement metadata, not authority grants.

## TDD Steps

### Step 1: Inspect current planning state

Read PLAN-176, PLAN-INDEX, CHANGELOG, target specs, and relevant target-Ash notes before assigning globally unique task IDs.

### Step 2: Write planning artifacts

Create the Phase 177 plan and task files with an audit-first row/identity/core/CPS scope.

### Step 3: Register planning surfaces

Update PLAN-INDEX and CHANGELOG after all task files exist.

### Step 4: Verify structure

Run structural checks that every task link resolves and PLAN-INDEX/CHANGELOG mention Phase 177.

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
  - [x] Plan file exists.
  - [x] Task files TASK-1806 through TASK-1815 exist.
  - [x] PLAN-INDEX row and phase section exist.
  - [x] CHANGELOG entry exists.
```

## Dependencies for Next Task

This task feeds the following Phase 177 tasks according to the dependency table in PLAN-177.

## Completion Evidence

Created the Phase 177 planning packet, task files, PLAN-INDEX entries, and CHANGELOG planning entry. Implementation tasks remain planned.
