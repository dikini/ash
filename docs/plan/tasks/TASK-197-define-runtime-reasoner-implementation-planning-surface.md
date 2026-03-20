# TASK-197: Define Runtime-Reasoner Implementation-Planning Surface

## Status: 📝 Planned

## Description

Define the concrete implementation-planning surface implied by the runtime-reasoner docs corpus
without yet creating code-facing tasks.

## Specification Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/reference/surface-guidance-boundary.md`
- `docs/design/LANGUAGE-TERMINOLOGY.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Identify likely runtime entry-point classes that may need interaction-aware handling later
2. Identify which future work is likely runtime, tooling, docs-facing, or surface-docs-only
3. Distinguish implementation-planning surface from still-out-of-scope syntax or redesign work
4. Produce a written planning note that later task synthesis can consume

## Files

- Create: `docs/reference/runtime-reasoner-implementation-planning-surface.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit implementation-planning surface after the docs-only handoff,
- unclear set of likely runtime/tooling/doc integration points,
- risk of creating code-facing tasks from vague intuition instead of the new docs corpus.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one note stating what later implementation planning should treat as in-scope.

### Step 3: Implement the planning-surface note (Green)

Add only the planning-surface note needed for later convergence-map synthesis.

### Step 4: Verify GREEN

Expected pass conditions:
- likely implementation-planning areas are explicit,
- docs-only concerns remain separated,
- the note is usable as direct input to later task-map synthesis.

### Step 5: Commit

```bash
git add docs/reference/runtime-reasoner-implementation-planning-surface.md docs/plan/tasks/TASK-197-define-runtime-reasoner-implementation-planning-surface.md
git commit -m "docs: define runtime-reasoner implementation planning surface"
```

## Completion Checklist

- [ ] implementation-planning surface documented
- [ ] likely work areas classified
- [ ] out-of-scope areas restated

## Non-goals

- No implementation-task creation
- No code changes
- No new syntax design

## Dependencies

- Depends on: TASK-196
- Blocks: TASK-198

