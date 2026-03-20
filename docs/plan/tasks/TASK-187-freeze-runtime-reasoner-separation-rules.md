# TASK-187: Freeze Runtime-Reasoner Separation Rules

## Status: ✅ Complete

## Description

Freeze the rule set and review protocol that distinguish runtime-only concerns from runtime-to-reasoner
interaction concerns before any further spec revisions are made in this area.

## Specification Reference

- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- SPEC-001: IR
- SPEC-004: Operational Semantics

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md`

## Requirements

### Functional Requirements

1. Define a reusable separation test for classifying candidate features
2. Define the three classification outcomes: runtime-only, interaction-layer, split concern
3. Define explicit review questions and expected evidence for later audits
4. State non-goals that must remain runtime-only, including monitors and exposed workflow views

## Files

- Create: `docs/reference/runtime-reasoner-separation-rules.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit separation test,
- no frozen review protocol,
- runtime-only and interaction-layer concerns still mixed informally in design discussion.

### Step 2: Verify RED

Expected failure conditions:
- at least one feature category still depends on conversational interpretation rather than a written classification rule.

Observed before implementation:
- the repository had a design note for the runtime-versus-reasoner model, but no frozen review
  protocol explaining how to classify candidate features consistently.
- runtime-only concerns such as monitor views and exposed workflow state were at risk of being
  overloaded because the separation test existed only in design discussion, not in a canonical
  review note.

### Step 3: Implement the minimal planning/reference fix (Green)

Add only the classification rules and review protocol needed to guide later audits.

### Step 4: Verify GREEN

Expected pass conditions:
- the separation test is explicit,
- classification outcomes are defined,
- monitor and exposure constructs are explicitly kept runtime-only,
- later audit tasks can cite one stable review protocol.

Verified after implementation:
- [runtime-reasoner-separation-rules.md](../../reference/runtime-reasoner-separation-rules.md)
  now defines the mandatory separation test, the three classification outcomes, review questions,
  and evidence expectations.
- the note explicitly keeps monitor views, `exposes`, workflow observability, and monitor authority
  in the runtime-only bucket unless a future contract proves otherwise.
- later review tasks can now cite one stable protocol instead of re-deriving the rules from design
  discussion.

### Step 5: Commit

```bash
git add docs/reference/runtime-reasoner-separation-rules.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-187-freeze-runtime-reasoner-separation-rules.md docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md
git commit -m "docs: freeze runtime-reasoner separation rules"
```

## Completion Checklist

- [x] separation test documented
- [x] classification outcomes documented
- [x] review protocol documented
- [x] runtime-only non-goals documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No normative spec edits
- No runtime implementation changes
- No surface syntax additions

## Dependencies

- Depends on: TASK-186
- Blocks: TASK-188, TASK-189, TASK-190
