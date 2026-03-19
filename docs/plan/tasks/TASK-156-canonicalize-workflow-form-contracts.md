# TASK-156: Canonicalize Workflow Form Contracts

## Status: ✅ Complete

## Description

Canonicalize the written contracts for core workflow forms so the spec set gives one stable answer for `check`, `decide`, `receive`, and workflow effect vocabulary.

This is a documentation-first convergence task. It does not change Rust code. Its purpose is to freeze the upstream contract that later parser, AST, lowering, type-checking, and interpreter tasks will follow.

## Specification Reference

- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-017: Capability Integration
- SPEC-018: Capability Matrix

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Specify one canonical surface form for `check`
2. Specify one canonical AST/core meaning for `check`
3. Specify one canonical surface form for `decide`
4. Specify one canonical AST/core meaning for `decide`
5. Specify one canonical surface form for `receive`
6. Specify one canonical AST/core meaning for `receive`
7. Use one consistent effect vocabulary across the touched spec sections
8. Remove or explicitly supersede conflicting interpretations in the touched specs

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with a single unambiguous answer:

- Is `check` obligation-only, policy-only, or both?
- Is `decide` allowed without an explicit policy?
- What is the canonical shape of `receive` at the syntax and IR levels?
- Which effect level is assigned to `receive` and why?
- Which spec is authoritative when a neighboring spec references these forms?

## Files

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a short review checklist covering:
- syntax shape,
- AST shape,
- lowering expectations,
- effect classification,
- execution meaning.

Use the checklist to review the target spec sections and mark every place where multiple answers currently exist.

### Step 2: Verify RED

Expected failure conditions:
- at least one contradictory description of `check`
- at least one contradictory description of `decide`
- at least one contradictory description of `receive`
- at least one inconsistent effect naming or effect assignment

### Step 3: Implement the minimal spec repair (Green)

Edit only the sections needed to produce one canonical contract. Prefer tightening existing wording over adding broad new material.

### Step 4: Verify GREEN

Re-run the checklist against all touched sections.

Expected pass conditions:
- one answer for `check`
- one answer for `decide`
- one answer for `receive`
- one consistent effect vocabulary
- no stale cross-references to superseded behavior in the touched sections

### Step 5: Commit

```bash
git add docs/spec/SPEC-001-IR.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-018-CAPABILITY-MATRIX.md CHANGELOG.md
git commit -m "docs: canonicalize workflow form contracts"
```

## Completion Checklist

- [x] `check` contract canonicalized
- [x] `decide` contract canonicalized
- [x] `receive` contract canonicalized
- [x] effect vocabulary consistent in touched sections
- [x] no conflicting wording remains in touched sections
- [x] `CHANGELOG.md` updated

## Non-goals

- No Rust parser, AST, or interpreter changes
- No full policy-model redesign beyond what these workflow forms require
- No CLI/REPL contract work
- No ADT contract work

## Dependencies

- Depends on: audit reports already written
- Blocks: TASK-157, TASK-158, and all downstream Rust convergence tasks
