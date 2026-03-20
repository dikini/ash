# TASK-214: Fix Residual Policy and Typed-Provider Spec Drift

## Status: ✅ Complete

## Description

Fix the remaining spec-only drift around the stale typed-provider forward reference, the incorrect
policy-conflict example, and the still-underspecified note about provider effect granularity.

## Specification Reference

- `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- `docs/spec/SPEC-010-EMBEDDING.md`
- `docs/spec/SPEC-015-TYPED-PROVIDERS.md`
- `docs/spec/SPEC-016-OUTPUT.md`

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-20-final-convergence-audit.md`

## Requirements

### Functional Requirements

1. Remove or correct the stale `SPEC-015` forward reference that still points future schema-first
   work at `SPEC-016`
2. Correct the incorrect policy-conflict example in `SPEC-007` so it no longer contradicts the SMT
   constraints it shows
3. Clarify the intended provider effect-granularity boundary between the embedding/provider docs
   and the canonical effect model without introducing new implementation requirements

## Files

- Modify: `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- Modify: `docs/spec/SPEC-010-EMBEDDING.md`
- Modify: `docs/spec/SPEC-015-TYPED-PROVIDERS.md`
- Modify: `docs/spec/SPEC-016-OUTPUT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

List the exact stale reference, contradictory example, and effect-granularity ambiguity.

### Step 2: Verify RED

Confirm the current text still exhibits those issues.

### Step 3: Implement the spec fixes (Green)

Correct the references/examples and add the minimum clarifying text needed to remove the ambiguity.

### Step 4: Verify GREEN

Re-read the changed sections and confirm the drift is gone without expanding the runtime/model
scope.

### Step 5: Commit

```bash
git add docs/spec/SPEC-007-POLICY-COMBINATORS.md docs/spec/SPEC-010-EMBEDDING.md docs/spec/SPEC-015-TYPED-PROVIDERS.md docs/spec/SPEC-016-OUTPUT.md CHANGELOG.md
git commit -m "docs: fix residual policy and provider drift"
```

## Completion Checklist

- [x] stale typed-provider forward reference corrected
- [x] policy-conflict example corrected
- [x] provider effect-granularity boundary clarified
- [x] `CHANGELOG.md` updated

## Non-goals

- No policy-engine redesign
- No embedding or provider implementation changes

## Dependencies

- Depends on: TASK-176
- Blocks: residual spec-only audit closure
