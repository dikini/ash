# TASK-200: Audit Runtime Trace and Provenance Surfaces

## Status: ✅ Complete

## Description

Audit the trace, provenance, and workflow-wrapper surfaces that later implementation planning may
need to treat explicitly without changing runtime-owned authority.

## Specification Reference

- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/reference/runtime-to-reasoner-interaction-contract.md`
- `docs/reference/runtime-reasoner-implementation-planning-surface.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Plan Reference

- `docs/plan/2026-03-20-runtime-boundary-implementation-planning-plan.md`

## Requirements

### Functional Requirements

1. Audit the current trace, provenance, export, and workflow-wrapper surfaces named in the
   implementation-planning surface
2. Identify where later planning may need stage-aware or acceptance-aware visibility
3. Preserve runtime-only ownership of observability, provenance, and official workflow history
4. Produce an audit document that later runtime-boundary synthesis can consume directly

## Files

- Create: `docs/audit/2026-03-20-runtime-trace-and-provenance-planning-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- no explicit audit of trace/provenance surfaces after TASK-198,
- unclear distinction between runtime-owned observability and later interaction-aware planning,
- risk of overloading provenance or wrapper hooks with projection semantics.

### Step 2: Verify RED

Expected failure conditions:
- the repository lacks a dedicated audit of runtime trace and provenance surfaces for later
  planning.

### Step 3: Implement the audit (Green)

Create only the audit needed for later runtime-boundary task synthesis.

### Step 4: Verify GREEN

Expected pass conditions:
- trace/provenance and wrapper surfaces are mapped,
- runtime-owned observability is protected,
- the audit is usable as direct input to TASK-201.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-runtime-trace-and-provenance-planning-review.md docs/plan/PLAN-INDEX.md CHANGELOG.md docs/plan/tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md
git commit -m "docs: audit runtime trace and provenance surfaces"
```

## Completion Checklist

- [x] trace/provenance surfaces audited
- [x] wrapper boundaries reviewed
- [x] runtime-owned observability protections restated
- [x] audit output written for TASK-201

## Evidence

- The runtime trace recorder, trace event, export helpers, and workflow wrapper hooks remain
  meaningful without any reasoner present, so they stay in the runtime-only bucket.
- The new audit explicitly separates runtime observability from later presentation-level planning
  and keeps projection semantics out of trace/provenance ownership.

## Non-goals

- No code changes
- No implementation-task creation
- No CLI/REPL wording work

## Dependencies

- Depends on: TASK-198
- Blocks: TASK-201
