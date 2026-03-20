# TASK-203: Audit Trace Export and Presentation Surfaces

## Status: ✅ Complete

## Description

Audit trace export and presentation surfaces that later tooling planning may need to treat
explicitly while preserving runtime-owned provenance and observability.

## Specification Reference

- `docs/spec/SPEC-005-CLI.md`
- `docs/spec/SPEC-016-OUTPUT-CAPABILITIES.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/surface-guidance-boundary.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`

## Plan Reference

- `docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Audit the current trace formatting, export, and presentation surfaces named in the
   implementation-planning corpus
2. Identify where later tooling planning may need stage-aware or interaction-aware wording
3. Preserve runtime-owned provenance and observability as the semantic authority
4. Produce an audit document that later tooling/surface synthesis can consume directly

## Files

- Create: `docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit audit of trace export and presentation surfaces after TASK-198,
- unclear distinction between runtime provenance and presentation-level stage wording,
- risk of using trace presentation as a proxy for hidden reasoner state.

### Step 2: Verify RED

Expected failure conditions:
- the repository lacks a dedicated audit of trace export and presentation surfaces for later
  tooling planning.

### Step 3: Implement the audit (Green)

Create only the audit needed for later tooling/surface task synthesis.

### Step 4: Verify GREEN

Expected pass conditions:
- trace/export presentation surfaces are mapped,
- runtime-owned provenance remains authoritative,
- the audit is usable as direct input to TASK-204.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md
git commit -m "docs: audit trace presentation surfaces"
```

## Completion Checklist

- [x] trace/export presentation surfaces audited
- [x] provenance authority protections restated
- [x] likely wording/presentation follow-ups identified
- [x] audit output written for TASK-204

## Evidence

- [docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md](/home/dikini/Projects/ash/docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md)
  now classifies the CLI trace command, provenance recorder, and export helpers as runtime-only
  and identifies stage-aware wording as tooling-level follow-up rather than semantic change.
- The audit keeps trace export and presentation separate from reasoner projection, preserving
  runtime-owned observability and provenance.
- The only remaining pressure is presentation-level wording for accepted versus rejected
  progression, which belongs to the later tooling/surface steering brief.

## Non-goals

- No code changes
- No implementation-task creation
- No provenance semantic redesign

## Dependencies

- Depends on: TASK-198
- Blocks: TASK-204
