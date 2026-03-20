# TASK-194: Define Human-Facing Surface Guidance Boundary

## Status: 📝 Planned

## Description

Define what human-facing guidance belongs in the surface-language documentation for advisory,
gated, and committed stages, without introducing syntax unless the docs-only review proves it is
needed later.

## Specification Reference

- SPEC-002: Surface Language
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/design/LANGUAGE-TERMINOLOGY.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md`
- `docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md`

## Requirements

### Functional Requirements

1. Decide whether the needed guidance is explanatory only or normative
2. Define what stage guidance belongs in `SPEC-002`
3. Explicitly avoid overloading `exposes`, monitor visibility, or runtime-only constructs
4. Defer syntax work unless a separate future task intentionally opens it

## Files

- Create: `docs/reference/surface-guidance-boundary.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit boundary for human-facing advisory/gated/committed guidance,
- no written decision on whether this belongs in explanatory or normative text,
- risk of surface guidance drifting into syntax design prematurely.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one note that decides the surface-guidance boundary before `SPEC-002` is edited.

### Step 3: Implement the minimal boundary note (Green)

Add only the boundary decision and guidance ownership note needed for later surface-spec work.

### Step 4: Verify GREEN

Expected pass conditions:
- the human-facing guidance boundary is explicit,
- the decision on explanatory versus normative text is explicit,
- runtime-only constructs remain protected.

### Step 5: Commit

```bash
git add docs/reference/surface-guidance-boundary.md CHANGELOG.md docs/plan/tasks/TASK-194-define-human-facing-surface-guidance-boundary.md
git commit -m "docs: define human-facing surface guidance boundary"
```

## Completion Checklist

- [ ] guidance boundary documented
- [ ] explanatory versus normative decision documented
- [ ] runtime-only constructs protected
- [ ] `CHANGELOG.md` updated

## Non-goals

- No surface syntax additions
- No normative `SPEC-002` changes yet
- No implementation work

## Dependencies

- Depends on: TASK-191, TASK-193
- Blocks: TASK-195

