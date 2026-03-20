# TASK-202: Audit CLI and REPL Surfaces for Interaction Planning

## Status: 📝 Planned

## Description

Audit the user-facing CLI and REPL surfaces that later implementation planning may need to align
with the runtime-reasoner docs corpus without changing runtime semantic authority.

## Specification Reference

- `docs/spec/SPEC-005-CLI.md`
- `docs/spec/SPEC-011-REPL.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/surface-guidance-boundary.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`

## Plan Reference

- `docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Audit the current CLI and REPL surfaces named in the implementation-planning corpus
2. Distinguish runtime-observable behavior from explanatory-only stage guidance
3. Identify where later planning may need wording, ordering, prompt, or reporting adjustments
4. Produce an audit document that later tooling/surface synthesis can consume directly

## Files

- Create: `docs/audit/2026-03-20-cli-and-repl-interaction-planning-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit audit of CLI and REPL surfaces after TASK-198,
- unclear separation between runtime-observable behavior and explanatory stage guidance,
- risk of treating prompts or output text as semantic authority.

### Step 2: Verify RED

Expected failure conditions:
- the repository lacks a dedicated audit of CLI and REPL surfaces for tooling planning.

### Step 3: Implement the audit (Green)

Create only the audit needed for later tooling/surface task synthesis.

### Step 4: Verify GREEN

Expected pass conditions:
- CLI and REPL surfaces are mapped,
- runtime-observable behavior remains authoritative,
- explanatory-only guidance stays separate,
- the audit is usable as direct input to TASK-204.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-cli-and-repl-interaction-planning-review.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-202-audit-cli-and-repl-surfaces-for-interaction-planning.md docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md
git commit -m "docs: audit cli and repl surfaces for planning"
```

## Completion Checklist

- [ ] CLI and REPL surfaces audited
- [ ] runtime-observable versus explanatory-only boundary restated
- [ ] likely wording/presentation follow-ups identified
- [ ] audit output written for TASK-204

## Non-goals

- No code changes
- No implementation-task creation
- No new syntax or stage markers

## Dependencies

- Depends on: TASK-198
- Blocks: TASK-204
