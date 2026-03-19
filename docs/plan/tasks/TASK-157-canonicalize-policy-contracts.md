# TASK-157: Canonicalize Policy Contracts

## Status: ✅ Complete

## Description

Choose and document one policy model that spans syntax, AST/lowering expectations, type checking, and runtime verification.

This task removes the current parallel policy stories from the spec set and replaces them with one continuous contract that downstream Rust work can implement.

## Specification Reference

- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-006: Policy Definitions
- SPEC-007: Policy Combinators
- SPEC-008: Dynamic Policies
- SPEC-017: Capability Integration
- SPEC-018: Capability Matrix

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Define one canonical policy representation story
2. Define how policy expressions differ from policy instances, if both exist
3. Define what lowering must preserve for policies
4. Define what type checking validates for policies
5. Define what runtime verification consumes as policy decisions
6. Remove or explicitly supersede conflicting policy interpretations in the touched specs

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- What is the canonical syntax form for policies?
- What is the canonical lowered/core representation of a policy?
- Are policy expressions first-class values, static declarations, both, or neither?
- How are dynamic policy outcomes represented at runtime?
- Which parts are compile-time validated versus runtime enforced?

## Files

- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- Modify: `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- Modify: `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:
- policy syntax,
- policy composition,
- lowering/core representation,
- type rules,
- runtime outcomes,
- SMT/static validation relationship.

### Step 2: Verify RED

Expected failure conditions:
- at least one place where policy expressions and policy instances are mixed ambiguously
- at least one place where runtime policy outcomes do not match the earlier policy model
- at least one place where lowering expectations are unclear or contradictory

### Step 3: Implement the minimal spec repair (Green)

Edit only the necessary sections to establish one policy pipeline from syntax through runtime verification.

### Step 4: Verify GREEN

Expected pass conditions:
- one continuous policy story from parse through runtime
- policy roles for type checking and runtime verification are explicit
- no conflicting policy vocabulary remains in the touched sections

### Step 5: Commit

```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-006-POLICY-DEFINITIONS.md docs/spec/SPEC-007-POLICY-COMBINATORS.md docs/spec/SPEC-008-DYNAMIC-POLICIES.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md CHANGELOG.md
git commit -m "docs: canonicalize policy contracts"
```

## Completion Checklist

- [x] one canonical policy model documented
- [x] lowering expectations for policies defined
- [x] type-checking expectations for policies defined
- [x] runtime policy outcomes defined
- [x] contradictory policy wording removed or superseded
- [x] `CHANGELOG.md` updated

## Non-goals

- No parser or lowering code changes
- No SMT implementation changes
- No runtime verifier code changes
- No REPL/CLI or ADT work

## Dependencies

- Depends on: TASK-156
- Blocks: policy-related handoff docs and all downstream policy Rust tasks
