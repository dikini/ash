# TASK-191: Define Runtime-to-Reasoner Interaction Contract

## Status: 📝 Planned

## Description

Define the missing runtime-to-reasoner interaction contract as a separate authoritative note so
projection, advisory outputs, and runtime acceptance boundaries are explicit without overloading
runtime-only specs.

## Specification Reference

- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- `docs/reference/runtime-reasoner-separation-rules.md`
- SPEC-004: Operational Semantics

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md`
- `docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md`

## Requirements

### Functional Requirements

1. Define projected or injected runtime context as a governed output
2. Define advisory outputs as non-authoritative artifacts
3. Define acceptance boundaries for artifacts returning to runtime
4. State explicitly that monitor views and `exposes` are not projection machinery
5. Keep the interaction transport abstract while allowing current tool-call boundaries

## Files

- Create: `docs/reference/runtime-to-reasoner-interaction-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no standalone interaction contract,
- projection and advisory boundaries existing only in design notes,
- monitor and exposure non-overlap not yet explicit in one contract note.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one authoritative interaction-facing document that later specs can cite.

### Step 3: Implement the minimal contract note (Green)

Add only the interaction contract and the minimal planning/changelog updates needed to track it.

### Step 4: Verify GREEN

Expected pass conditions:
- the interaction contract is explicit,
- monitor views and `exposes` are explicitly excluded from projection semantics,
- later framing and terminology tasks can cite one stable note.

### Step 5: Commit

```bash
git add docs/reference/runtime-to-reasoner-interaction-contract.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-191-define-runtime-to-reasoner-interaction-contract.md docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md
git commit -m "docs: define runtime-to-reasoner interaction contract"
```

## Completion Checklist

- [ ] interaction contract documented
- [ ] projection/acceptance boundaries documented
- [ ] monitor/exposes non-overlap documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No normative runtime semantics changes
- No surface syntax changes
- No implementation work

## Dependencies

- Depends on: TASK-190
- Blocks: TASK-192, TASK-193, TASK-194, TASK-195

