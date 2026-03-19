# TASK-178: Normalize Phase Judgments and Rejection Boundaries

## Status: ✅ Complete

## Description

Define explicit phase-owned judgments and rejection boundaries for parsing, lowering, typing,
runtime execution, and observable behavior so later implementation work does not rely on local
interpretation of where failures belong.

## Specification Reference

- SPEC-001: IR
- SPEC-003: Type System
- SPEC-004: Operational Semantics

## Requirements

### Functional Requirements

1. Define explicit phase-owned judgment boundaries
2. Separate parser, lowering, typing, runtime, and observable-behavior rejection classes
3. Keep normative specs free of implementation-drift commentary
4. Align reference docs with the tightened boundaries

## Files

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/reference/surface-to-parser-contract.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:
- implicit phase ownership,
- overlapping rejection classes,
- phase boundaries explained only in prose.

### Step 2: Verify RED

Expected failure conditions:
- at least one rejection class still depends on interpretation across multiple docs.

Observed before implementation:
- `SPEC-001`, `SPEC-003`, and `SPEC-004` described phase behavior, but parser, lowering, type, and
  runtime rejection ownership was still partly implicit.
- the reference docs mixed canonical contracts with implementation commentary in the type-to-runtime
  handoff.

### Step 3: Implement the minimal spec/reference fix (Green)

Tighten only the phase judgments and rejection boundaries.

### Step 4: Verify GREEN

Expected pass conditions:
- phase-owned rejection boundaries are explicit,
- reference docs align to the same split,
- specs state truth while tasks/plans carry migration commentary.

Verified after implementation:
- `SPEC-001` now states that the IR contract owns lowering/type/runtime boundaries, not parser
  acceptance.
- `SPEC-003` now separates type-layer rejection from parser, lowering, and runtime failures.
- `SPEC-004` now distinguishes runtime boundary failures from parser, lowering, and type failures.
- the three reference docs now keep canonical contract text separate from convergence notes.

### Step 5: Commit

```bash
git add docs/spec/SPEC-001-IR.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/reference/surface-to-parser-contract.md docs/reference/parser-to-core-lowering-contract.md docs/reference/type-to-runtime-contract.md CHANGELOG.md
git commit -m "docs: normalize phase judgments and rejection boundaries"
```

## Completion Checklist

- [x] phase judgments documented
- [x] rejection boundaries documented
- [x] reference docs aligned
- [x] `CHANGELOG.md` updated

## Non-goals

- No implementation changes
- No new runtime features

## Dependencies

- Depends on: TASK-177
- Blocks: TASK-179, TASK-180, TASK-181, TASK-182, TASK-183, TASK-184
