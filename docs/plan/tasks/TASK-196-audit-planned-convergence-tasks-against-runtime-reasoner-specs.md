# TASK-196: Audit Planned Convergence Tasks Against Runtime-Reasoner Specs

## Status: 📝 Planned

## Description

Audit the already-planned convergence tasks against the newly completed runtime-reasoner spec
corpus to determine which tasks are unchanged, which need updated references or scope notes, and
which are blocked pending new implementation-planning work.

## Specification Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- SPEC-004: Operational Semantics

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Review at least TASK-164 through TASK-173 against the new runtime-reasoner docs
2. Classify each task as unchanged, reference-update-only, scope-adjustment-needed, or blocked
3. Record task-by-task findings with explicit reasons
4. Preserve valid existing work instead of replacing it unnecessarily

## Files

- Create: `docs/audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- existing planned tasks not yet reviewed against the new runtime-reasoner docs,
- no explicit classification of task impact,
- unclear downstream effect on the current convergence queue.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one audit explaining how the existing planned convergence tasks relate to the new interaction-facing contracts.

### Step 3: Implement the audit report (Green)

Write only the impact audit and the minimal planning/changelog updates needed to track it.

### Step 4: Verify GREEN

Expected pass conditions:
- reviewed tasks have explicit impact classifications,
- existing valid tasks are preserved where appropriate,
- the audit provides enough detail for later convergence-map synthesis.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-196-audit-planned-convergence-tasks-against-runtime-reasoner-specs.md docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md
git commit -m "docs: audit planned convergence tasks against runtime-reasoner specs"
```

## Completion Checklist

- [ ] reviewed tasks classified
- [ ] task-by-task findings documented
- [ ] valid existing tasks preserved where appropriate
- [ ] `CHANGELOG.md` updated

## Non-goals

- No implementation-task creation
- No code changes
- No runtime/spec redesign

## Dependencies

- Depends on: TASK-195
- Blocks: TASK-197, TASK-198

