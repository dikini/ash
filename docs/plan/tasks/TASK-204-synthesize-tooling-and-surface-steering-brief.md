# TASK-204: Synthesize Tooling and Surface Steering Brief

## Status: 📝 Planned

## Description

Synthesize the CLI/REPL and trace-presentation audits into one steering brief for later tooling and
surface code-facing task creation.

## Specification Reference

- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/surface-guidance-boundary.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Plan Reference

- `docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Merge the findings from TASK-202 and TASK-203
2. Define the later tooling/surface task clusters that should exist without opening them
3. State the explicit review checkpoint and steering questions for the tooling/surface phase
4. Preserve the separation between presentation-level work and runtime semantic authority

## Files

- Create: `docs/plan/2026-03-20-tooling-surface-steering-brief.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- tooling/surface audits exist but no single steering brief merges them,
- later tooling/surface code-facing clusters remain implicit,
- no explicit review checkpoint exists before tooling task creation.

### Step 2: Verify RED

Expected failure conditions:
- the repository lacks one synthesized tooling/surface steering brief.

### Step 3: Implement the steering brief (Green)

Create only the steering brief needed to review the tooling/surface planning phase.

### Step 4: Verify GREEN

Expected pass conditions:
- tooling/surface task clusters are explicit,
- the review checkpoint is explicit,
- runtime-observable authority remains intact,
- the brief is usable as the phase-end review artifact.

### Step 5: Commit

```bash
git add docs/plan/2026-03-20-tooling-surface-steering-brief.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-204-synthesize-tooling-and-surface-steering-brief.md docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md
git commit -m "docs: synthesize tooling and surface steering brief"
```

## Completion Checklist

- [ ] TASK-202 and TASK-203 findings merged
- [ ] later tooling/surface task clusters stated
- [ ] review checkpoint and steering questions documented
- [ ] runtime authority protections restated

## Non-goals

- No code changes
- No implementation-task creation
- No runtime-boundary planning

## Dependencies

- Depends on: TASK-202, TASK-203
