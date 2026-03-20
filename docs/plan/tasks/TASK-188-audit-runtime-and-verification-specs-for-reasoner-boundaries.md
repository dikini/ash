# TASK-188: Audit Runtime and Verification Specs for Reasoner Boundaries

## Status: ✅ Complete

## Description

Audit the core runtime-facing specifications against the frozen runtime-reasoner separation rules
to identify where authority, projection, advisory output, validation, and commitment boundaries are
already explicit, missing, or blurred.

## Specification Reference

- SPEC-001: IR
- SPEC-004: Operational Semantics
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Plan Reference

- `docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md`

## Requirements

### Functional Requirements

1. Review the core runtime and verification docs using the frozen separation test
2. Classify constructs as runtime-only, interaction-layer, or split concern
3. Identify missing interaction-boundary statements that can be added without overloading runtime-only features
4. Produce a concrete findings report with file and section references

## Files

- Create: `docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md`

## Review Targets

- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- implicit or missing projection boundaries,
- advisory output and committed state not clearly separated,
- runtime-only validation concerns mixed with interaction concerns,
- unclear ownership between `DECIDE`, capability verification, and `ACT`.

### Step 2: Verify RED

Expected failure conditions:
- at least one core runtime or verification feature still lacks a clear classification against the separation rules.

Observed before implementation:
- the reviewed runtime docs were already strongly runtime-centered, but they did not yet have an
  audit report recording the separation-rule classification against the frozen review protocol.

### Step 3: Implement the audit report (Green)

Write only the audit and findings report. Do not edit normative spec meaning in this task.

### Step 4: Verify GREEN

Expected pass conditions:
- each reviewed area has an explicit classification,
- findings identify real tensions and not merely vocabulary differences,
- runtime-only features remain runtime-only in the audit outcome.

Verified after implementation:
- [docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md](/home/dikini/Projects/ash/docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md)
  now records the review scope, line references, classifications, and the conclusion that all four
  reviewed specs are runtime-only and aligned or silent rather than tense.
- the audit explicitly distinguishes `Aligned` and `Silent` outcomes and records that no
  interaction-layer or split concerns were found in the runtime contract set.

### Step 5: Commit

```bash
git add docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md docs/plan/tasks/TASK-188-audit-runtime-and-verification-specs-for-reasoner-boundaries.md
git commit -m "docs: audit runtime and verification reasoner boundaries"
```

## Completion Checklist

- [x] review scope covered
- [x] classifications documented
- [x] findings include file references
- [x] runtime-only boundaries preserved

## Non-goals

- No normative spec edits
- No runtime implementation changes
- No surface syntax redesign

## Dependencies

- Depends on: TASK-187
- Blocks: TASK-190
