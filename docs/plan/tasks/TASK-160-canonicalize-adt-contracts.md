# TASK-160: Canonicalize ADT Contracts

## Status: 📝 Planned

## Description

Canonicalize the ADT contract across type-definition syntax, constructor semantics, runtime variant shape, pattern matching, exhaustiveness, and required stdlib helper surface.

This is the final specification-freeze task in Phase A and provides the authoritative upstream contract for the later ADT Rust convergence work.

## Specification Reference

- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-013: Streams and Event Processing
- SPEC-014: Behaviours
- SPEC-020: ADT Types

## Audit Reference

- `docs/audit/2026-03-19-task-consistency-review-non-lean.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Define one canonical syntax and AST shape for type definitions
2. Define one canonical constructor typing story
3. Define one canonical runtime representation for variants
4. Define one canonical relationship between pattern typing, pattern execution, and exhaustiveness
5. Define the required Option/Result helper surface that the standard library must provide
6. Remove or explicitly supersede contradictory ADT wording in the touched specs

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- What is the canonical structural model of ADT definitions?
- What runtime data is stored in a variant value?
- How are patterns typed against variants?
- What does exhaustiveness analyze?
- Which stdlib helpers are required by spec versus optional conveniences?

## Files

- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-014-BEHAVIOURS.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:
- type-definition structure,
- constructor rules,
- runtime variant shape,
- pattern typing,
- exhaustiveness,
- stdlib helper obligations.

### Step 2: Verify RED

Expected failure conditions:
- at least one conflict in ADT structural model
- at least one conflict in runtime variant representation
- at least one mismatch in pattern or exhaustiveness expectations
- at least one mismatch in stdlib helper obligations

### Step 3: Implement the minimal spec repair (Green)

Update only the sections needed to define one canonical ADT story.

### Step 4: Verify GREEN

Expected pass conditions:
- one ADT structural model
- one runtime variant model
- one pattern/exhaustiveness model
- one required stdlib helper surface

### Step 5: Commit

```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-013-STREAMS.md docs/spec/SPEC-014-BEHAVIOURS.md docs/spec/SPEC-020-ADT-TYPES.md CHANGELOG.md
git commit -m "docs: canonicalize adt contracts"
```

## Completion Checklist

- [ ] canonical ADT structure defined
- [ ] canonical runtime variant shape defined
- [ ] canonical pattern/exhaustiveness contract defined
- [ ] required stdlib helper surface defined
- [ ] contradictory wording removed or superseded
- [ ] `CHANGELOG.md` updated

## Non-goals

- No parser, type-checker, interpreter, or stdlib code changes yet
- No new ADT feature expansion beyond convergence needs
- No Lean proof changes

## Dependencies

- Depends on: TASK-156 and any prior workflow-form decisions that affect ADT syntax or runtime values
- Blocks: ADT handoff docs and downstream ADT Rust tasks
