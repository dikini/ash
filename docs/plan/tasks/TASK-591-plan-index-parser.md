# TASK-591: PLAN-INDEX Parser and Coherence Checker

## Status: 📝 Planned

## Description

Parse `docs/plan/PLAN-INDEX.md` and detect missing task files, orphaned tasks, and status inconsistencies.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track A

## Dependencies

- TASK-590 (file collector)

## Requirements

1. Extract all `TASK-NNN` references from `PLAN-INDEX.md`.
2. Verify that each referenced task has a corresponding `docs/plan/tasks/TASK-NNN-*.md` file.
3. Detect task files that exist but are not referenced in the index.
4. Emit `IndexIncoherence` findings.

## TDD Steps

### Step 1: Write failing test

Mock `PLAN-INDEX.md` with a missing task file reference. Assert the checker finds it.

### Step 2: Implement

Create `apps/spec_processor/src/plan_index.ash` with `check(path: String) -> List<SpecFinding>`.

### Step 3: Verify

Run against real `PLAN-INDEX.md`. Report count of findings.

## Verification Steps

- [ ] Mock tests pass
- [ ] Real repo run produces expected findings
- [ ] Codex verification: VERIFIED
