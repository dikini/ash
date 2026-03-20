# TASK-189: Audit Surface and Observability Docs for Reasoner Boundaries

## Status: ✅ Complete

## Description

Audit the workflow-facing and observability-facing documents to ensure runtime-only constructs such
as monitoring and exposed workflow views are not overloaded with runtime-to-reasoner meaning, and
to identify where future human-facing guidance will be needed.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-021: Runtime Observable Behavior
- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md`

## Requirements

### Functional Requirements

1. Review surface and observability docs using the frozen separation test
2. Distinguish projection from monitorability and exposed workflow views
3. Identify terminology collisions that could mislead later surface syntax work
4. Produce a concrete findings report with file and section references

## Files

- Create: `docs/audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md`

## Review Targets

- `docs/spec/SPEC-002-SURFACE.md`
- `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
- `docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md`
- `docs/design/LANGUAGE-TERMINOLOGY.md`
- `docs/plan/tasks/TASK-186-monitor-authority-and-exposed-workflow-view.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- projection and exposure terminology overlapping,
- monitorability and reasoner visibility not clearly separated,
- workflow-facing wording that lacks a clean runtime-only interpretation,
- missing guidance for future human-facing explanation of advisory versus authoritative stages.

### Step 2: Verify RED

Expected failure conditions:
- at least one workflow-facing or observability-facing feature still lacks a clear classification against the separation rules.

Observed before implementation:
- `SPEC-002`, `SPEC-021`, and `TASK-186` already keep monitor authority and exposed workflow
  views on the runtime side, but the terminology guide does not yet reserve the projection and
  monitorability vocabulary.
- the word `observe` is used both for workflow input acquisition and for observing an exposed
  monitor view, which is acceptable runtime behavior but still a terminology collision.

### Step 3: Implement the audit report (Green)

Write only the audit and findings report. Do not edit normative spec meaning in this task.

### Step 4: Verify GREEN

Expected pass conditions:
- monitorability and projection are explicitly distinguished,
- findings identify terminology or design tensions precisely,
- runtime-only features remain runtime-only in the audit outcome.

Verified after implementation:
- [surface-and-observability-reasoner-boundaries-review.md](/home/dikini/Projects/ash/docs/audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md)
  records the audit findings with file and line references.
- the report preserves the runtime-only meaning of `exposes`, monitor views, and `MonitorLink`
  while flagging the remaining vocabulary drift risk as a terminology issue rather than a semantic
  failure.
- no normative spec files were modified.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md docs/plan/tasks/TASK-189-audit-surface-and-observability-docs-for-reasoner-boundaries.md
git commit -m "docs: audit surface and observability reasoner boundaries"
```

## Completion Checklist

- [x] review scope covered
- [x] classifications documented
- [x] projection versus exposure separated
- [x] findings include file references

## Non-goals

- No normative spec edits
- No runtime implementation changes
- No final surface syntax redesign

## Dependencies

- Depends on: TASK-187
- Blocks: TASK-190
