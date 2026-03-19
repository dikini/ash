# TASK-158: Canonicalize Streams and Runtime Verification Contracts

## Status: ✅ Complete

## Description

Canonicalize stream semantics and runtime-verification semantics, with emphasis on `receive`, control arms, declaration requirements, runtime context shape, and verification outcomes.

This task establishes the contract that later parser, lowering, type-checking, and interpreter/runtime tasks must follow.

## Specification Reference

- SPEC-004: Operational Semantics
- SPEC-013: Streams and Event Processing
- SPEC-014: Behaviours
- SPEC-017: Capability Integration
- SPEC-018: Capability Matrix

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Define canonical `receive` behavior for non-blocking, blocking, and timed variants
2. Define canonical control-arm semantics
3. Define declaration requirements for stream operations and `receive`
4. Define the canonical runtime context responsibilities
5. Define canonical verification outcomes for proceed, deny, warning, approval, and transform behaviors
6. Remove or explicitly supersede contradictory descriptions in the touched specs

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- What does `receive` do at runtime?
- How do control-stream arms differ from normal stream arms?
- What must be declared statically before a receive/send/observe/set operation is legal?
- What fields or responsibilities belong to runtime context?
- Which verification outcomes are errors, warnings, denials, or transforms?

## Files

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-014-BEHAVIOURS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:
- receive mode semantics,
- control-stream behavior,
- declaration rules,
- runtime context shape,
- aggregate verification behavior,
- outcome taxonomy.

### Step 2: Verify RED

Expected failure conditions:
- at least one mismatch on `receive`
- at least one mismatch on control-arm semantics
- at least one mismatch on runtime verification outcomes or context shape

### Step 3: Implement the minimal spec repair (Green)

Update only the sections needed to produce one end-to-end stream and verification contract.

### Step 4: Verify GREEN

Expected pass conditions:
- `receive` has one end-to-end story
- runtime context has one stable responsibility set
- verification outcomes are consistently named and scoped
- touched sections do not conflict with each other

### Step 5: Commit

```bash
git add docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-014-BEHAVIOURS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md CHANGELOG.md
git commit -m "docs: canonicalize streams and runtime verification"
```

## Completion Checklist

- [x] `receive` modes canonicalized
- [x] control-arm semantics canonicalized
- [x] declaration requirements canonicalized
- [x] runtime context contract canonicalized
- [x] verification outcome taxonomy canonicalized
- [x] `CHANGELOG.md` updated

## Non-goals

- No parser or interpreter implementation changes
- No provider API redesign
- No REPL/CLI contract work
- No ADT contract work

## Dependencies

- Depends on: TASK-156 and TASK-157
- Blocks: stream/runtime handoff docs and all downstream `receive` and runtime-verification Rust tasks
