# TASK-198: Synthesize Revised Runtime-Reasoner Convergence Map

## Status: ✅ Complete

## Description

Synthesize the planned-task impact audit and the implementation-planning surface into one revised
convergence map that states what existing tasks need updates and where new code-facing task
clusters should later be introduced.

## Specification Reference

- `docs/audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md`
- `docs/reference/runtime-reasoner-implementation-planning-surface.md`
- `docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Merge findings from TASK-196 and TASK-197
2. State which current planned tasks should be updated in place
3. State which new task clusters should be created later
4. Keep the output planning-only and do not open implementation tasks yet

## Files

- Create: `docs/plan/2026-03-20-runtime-reasoner-revised-convergence-map.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no unified revised convergence map,
- unclear relationship between the old convergence queue and the new runtime-reasoner docs corpus,
- no decision on where later code-facing task clusters should be introduced.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one planning output that revises the convergence map after the runtime-reasoner docs phase.

### Step 3: Implement the revised convergence map (Green)

Create only the revised convergence map and the minimal planning/changelog updates needed to close the phase.

### Step 4: Verify GREEN

Expected pass conditions:
- the revised convergence map explains old-task impact clearly,
- later code-facing task-cluster openings are explicit,
- the planning phase closes without creating implementation tasks prematurely.

## Evidence

- Red: the impact audit established that TASK-164 through TASK-171 remain unchanged and that only
  TASK-172 and TASK-173 need reference-only updates, while the planning surface note identified
  runtime, tooling, and provenance/trace entry-point classes for later grouping.
- Green: the new revised convergence map records those outcomes in one place, keeps the existing
  convergence queue intact, and defers code-facing clusters until a later implementation-planning
  pass.

### Step 5: Commit

```bash
git add docs/plan/2026-03-20-runtime-reasoner-revised-convergence-map.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-198-synthesize-revised-runtime-reasoner-convergence-map.md
git commit -m "docs: synthesize revised runtime-reasoner convergence map"
```

## Completion Checklist

- [x] revised convergence map documented
- [x] existing-task impact synthesized
- [x] later task-cluster openings identified
- [x] `CHANGELOG.md` updated

## Non-goals

- No implementation-task creation
- No code changes
- No spec redesign

## Dependencies

- Depends on: TASK-196, TASK-197
- Blocks: later code-facing task creation after this planning phase
