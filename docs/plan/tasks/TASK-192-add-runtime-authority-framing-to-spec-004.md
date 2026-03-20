# TASK-192: Add Runtime-Authority Framing to SPEC-004

## Status: ✅ Complete

## Description

Add the smallest possible framing update to `SPEC-004` so the operational semantics explicitly
state runtime authority and advisory-boundary ownership without changing the canonical evaluation
rules.

## Specification Reference

- SPEC-004: Operational Semantics
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md`
- `docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md`

## Requirements

### Functional Requirements

1. Add a short runtime-authority framing section near the front of `SPEC-004`
2. Clarify that advisory interaction remains outside authoritative state transition until accepted
3. Preserve the execution-neutral semantics and existing primitive rules
4. Avoid redefining monitors, `exposes`, or capability verification as interaction machinery

## Files

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- `SPEC-004` silent on runtime authority versus advisory interaction,
- no explicit statement that advisory outputs are external until accepted.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks a runtime-semantics framing section that cleanly references the interaction contract without absorbing it.

Observed before implementation:
- `SPEC-004` defined the operational rules and execution-neutral big-step semantics, but it did not
  yet explicitly state that authoritative runtime transition remains separate from advisory
  interaction until accepted.
- the new interaction contract existed, but the runtime semantics file did not yet point at that
  boundary in a short framing section.

### Step 3: Implement the minimal framing delta (Green)

Add only the framing text needed to clarify authority and acceptance boundaries.

### Step 4: Verify GREEN

Expected pass conditions:
- runtime authority is explicit,
- advisory interaction is acknowledged but not operationalized in the core rules,
- no runtime-only feature is overloaded.

Verified after implementation:
- [docs/spec/SPEC-004-SEMANTICS.md](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md)
  now includes a minimal `1.1 Runtime Authority and Advisory Interaction` subsection.
- the new framing states that the runtime owns authoritative state, validation, rejection,
  commitment, trace, and provenance.
- the new framing also states that external reasoner outputs remain advisory until runtime
  acceptance and that the spec stays execution-neutral.

### Step 5: Commit

```bash
git add docs/spec/SPEC-004-SEMANTICS.md CHANGELOG.md docs/plan/tasks/TASK-192-add-runtime-authority-framing-to-spec-004.md
git commit -m "docs: add runtime authority framing to spec-004"
```

## Completion Checklist

- [x] framing section added
- [x] runtime authority explicit
- [x] advisory boundary explicit
- [x] `CHANGELOG.md` updated

## Non-goals

- No new operational rules
- No surface syntax changes
- No implementation work

## Dependencies

- Depends on: TASK-191
- Blocks: TASK-195
