# TASK-193: Tighten Projection and Monitorability Terminology

## Status: ✅ Complete

## Description

Freeze the terminology needed to keep runtime-to-reasoner projection, monitorability, exposed
workflow views, and observation distinct.

## Specification Reference

- `docs/design/LANGUAGE-TERMINOLOGY.md`
- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md`
- `docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md`

## Requirements

### Functional Requirements

1. Reserve `projection`, `monitorability`, and `exposed workflow view` as distinct terms
2. Clarify the overloaded use of `observe`
3. Add an explicit non-overlap note between runtime visibility and reasoner projection
4. Keep the terminology pass framing-only

## Files

- Modify: `docs/design/LANGUAGE-TERMINOLOGY.md`
- Modify: `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- projection terminology not yet reserved,
- monitorability terminology not yet reserved,
- `observe` still carrying both workflow-input and generic monitor-access wording without clarification.

### Step 2: Verify RED

Expected failure conditions:
- the repository still lacks one frozen terminology pass separating runtime visibility from runtime-to-reasoner projection.

Observed before implementation:
- `LANGUAGE-TERMINOLOGY` reserved policy, scheduling, and link terms, but did not yet freeze
  `projection`, `monitorability`, or `exposed workflow view` as distinct terms.
- `RUNTIME_REASONER_INTERACTION_MODEL` described projection and runtime exposure separately in
  prose, but it did not yet contain a compact non-overlap reminder.
- the word `observe` still carried workflow-input meaning without an explicit constraint against
  reusing it as a monitor-view synonym in terminology guidance.

### Step 3: Implement the minimal terminology pass (Green)

Add only the terminology reservations and non-overlap notes needed to prevent drift.

### Step 4: Verify GREEN

Expected pass conditions:
- reserved terms are explicit,
- projection and monitorability are distinguished,
- the `observe` overload is constrained by wording.

Verified after implementation:
- [LANGUAGE-TERMINOLOGY.md](/home/dikini/Projects/ash/docs/design/LANGUAGE-TERMINOLOGY.md) now
  reserves `projection`, `monitorability`, and `exposed workflow view` as distinct terms and
  constrains `observe` to workflow input acquisition.
- [RUNTIME_REASONER_INTERACTION_MODEL.md](/home/dikini/Projects/ash/docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md)
  now includes an explicit non-overlap note stating that runtime visibility is separate from
  reasoner projection.
- the terminology pass remains framing-only and does not change runtime semantics.

### Step 5: Commit

```bash
git add docs/design/LANGUAGE-TERMINOLOGY.md docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md CHANGELOG.md docs/plan/tasks/TASK-193-tighten-projection-and-monitorability-terminology.md
git commit -m "docs: tighten projection and monitorability terminology"
```

## Completion Checklist

- [x] terminology reserved
- [x] projection versus monitorability separated
- [x] non-overlap note added
- [x] `CHANGELOG.md` updated

## Non-goals

- No normative spec changes
- No surface syntax changes
- No implementation work

## Dependencies

- Depends on: TASK-191
- Blocks: TASK-194, TASK-195
