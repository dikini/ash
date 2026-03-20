# TASK-215: Normalize Residual Spec Hygiene

## Status: ✅ Complete

## Description

Perform the remaining low-severity cleanup pass on the spec set so the residual documentation debt
from the final audit is reduced to zero or clearly bounded.

## Specification Reference

- affected files in `docs/spec/`

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-20-final-convergence-audit.md`

## Requirements

### Functional Requirements

1. Normalize residual example-type names in the affected specs where the canonical type names are
   already known
2. Reduce uneven status/editorial formatting across the touched spec set where that can be done
   mechanically and safely
3. Record any hygiene issue intentionally left alone rather than silently skipping it

## Files

- Modify: affected files under `docs/spec/`
- Modify: `docs/audit/2026-03-20-final-convergence-audit.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Enumerate the remaining low-severity hygiene issues to be touched in this pass.

### Step 2: Verify RED

Confirm the listed files still contain those issues.

### Step 3: Implement the cleanup (Green)

Apply the minimum safe normalization and formatting cleanup needed for the targeted files.

### Step 4: Verify GREEN

Re-read the changed sections and confirm the targeted hygiene issues are either resolved or
explicitly deferred.

### Step 5: Commit

```bash
git add docs/spec CHANGELOG.md
git commit -m "docs: normalize residual spec hygiene"
```

## Completion Checklist

- [x] residual low-severity hygiene pass applied
- [x] untouched issues, if any, explicitly recorded
- [x] `CHANGELOG.md` updated

## Outcome

Applied:

- normalized the remaining non-canonical `Number` examples in `SPEC-015`
- updated the final convergence audit to record that the residual spec-only findings from Phase 34
  are now closed

Explicitly left out:

- CLI `json` examples and file names, which describe data formats rather than Ash type names
- Lean reference syntax in `SPEC-021-LEAN-REFERENCE`, which is not Ash surface syntax
- broad cross-corpus editorial restyling, which would exceed the “minimum safe normalization”
  boundary of this task

## Non-goals

- No new semantics
- No attempt to rewrite the entire spec corpus stylistically

## Dependencies

- Depends on: TASK-213, TASK-214
- Blocks: residual spec-only audit closure
