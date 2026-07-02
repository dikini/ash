# TASK-1817: Create the Phase 178 source-to-Core row lowering bridge packet

## Status: ✅ Complete

## Description

Create and register the Phase 178 planning packet for bridging parsed target row syntax into source-to-typechecker/Core lowering. This task is documentation/planning only and does not implement Rust behavior.

## Specification Reference

- [PLAN-178: Source-to-Core Row Lowering Bridge](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100: Core Type Checking](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [NOTE-020: Computation Row Taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)

## Dependencies

- ✅ Phase 177 closeout and TASK-1816 review remediation are complete.
- ✅ No existing Phase 178 packet exists.

## Requirements

### Functional Requirements

1. Create `PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md` with scope, non-goals, decision gates, tasks, and verification baseline.
2. Create TASK-1817 through TASK-1825 task files with dependencies, requirements, dispatch metadata, and verification commands.
3. Register Phase 178 in PLAN-INDEX progress and detail sections.
4. Add a CHANGELOG.md planning entry under `[Unreleased]`.

### Property Requirements

- The plan must preserve the requested scope: source-to-typechecker/Core row lowering bridge.
- Planning language must not include row-polymorphic inference or provider/admission runtime wiring as Phase 178 implementation work.
- Rows must be framed as requirements, not authority grants.

## TDD Steps

### Step 1: Inspect current planning state

Read PLAN-177, TASK-1814, TASK-1815, TASK-1816, PLAN-INDEX, CHANGELOG, and relevant target specs before assigning globally unique task IDs.

### Step 2: Write planning artifacts

Create the Phase 178 plan and task files with audit-first source-to-Core row bridge scope.

### Step 3: Register planning surfaces

Update PLAN-INDEX and CHANGELOG after all task files exist.

### Step 4: Verify structure

Run structural checks that every task link resolves and PLAN-INDEX/CHANGELOG mention Phase 178.

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
  - [x] Task files TASK-1817 through TASK-1825 exist.
  - [x] PLAN-INDEX row and phase section exist.
  - [x] CHANGELOG entry exists.
```

## Dependencies for Next Task

This task feeds the following Phase 178 tasks according to the dependency table in PLAN-178.

## Completion Evidence

Created the Phase 178 planning packet, task files, PLAN-INDEX entries, and CHANGELOG planning entry. Implementation tasks remain planned.
