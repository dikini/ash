# TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics

## Status: ✅ Complete

## Description

Revise [docs/spec/SPEC-004-SEMANTICS.md](../../spec/SPEC-004-SEMANTICS.md) into a complete, proof-suitable big-step core semantics for Ash. The revised spec must expose explicit judgments for the pure/core fragment, isolate runtime-bearing helper contracts, and classify determinism boundaries clearly enough for Lean proofs and later small-step refinement work.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [Formalization Boundary and Proof Targets](../../reference/formalization-boundary.md)

## Requirements

### Functional Requirements

1. Define explicit subjudgments for workflow evaluation, expression evaluation, and pattern matching.
2. Complete the pure expression semantics in one canonical section.
3. Complete pattern/binding semantics in one canonical section.
4. Formalize propagation, failure ownership, and lookup-failure conventions.
5. Extract semantically significant helpers into explicit helper contracts with determinism and failure notes.
6. Add deterministic/nondeterministic fragment classification and semantic invariants.
7. Preserve execution-neutrality and runtime authority while making the pure/core fragment proof-shaped.

### Non-Functional Requirements

1. Keep the canonical core aligned with `SPEC-001` naming and forms.
2. Avoid over-specifying provider internals; helper contracts should constrain outcomes, not implementation strategy.
3. Keep the resulting spec readable for both implementation and proof audiences.
4. Update `CHANGELOG.md` with the planning/spec revision work.

## TDD Evidence

### Red

Before this task, `SPEC-004` is strong as a runtime-facing draft but still leaves several proof-shape gaps:

- `eval(...)` and `bind(...)` carry semantically significant behavior without one fully normalized judgment inventory;
- expression semantics are split across workflow rules and later constructor/match sections;
- propagation and failure ownership still rely partly on prose rather than centralized conventions;
- helper relations such as `select_receive_outcome(...)` and `combine_parallel_outcomes(...)` are not yet uniformly contract-shaped;
- deterministic and nondeterministic fragments are not classified centrally enough for theorem scoping.

### Green

This task is complete when:

- `SPEC-004` declares all core judgments explicitly;
- the pure expression and pattern fragments are defined in one proof-friendly place each;
- failure ownership and propagation are centralized and explicit;
- helper-backed rules cite helper contracts with determinism/failure semantics;
- theorem scope can distinguish deterministic core behavior from runtime-owned nondeterminism without informal guesswork.

## Files

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-001-IR.md` (if alignment edits are needed)
- Modify: `docs/reference/formalization-boundary.md` (if theorem targets or boundary wording need alignment)
- Modify: `CHANGELOG.md`
- Reference: `docs/plans/2026-03-29-spec-004-big-step-core-design.md`
- Reference: `docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:

- undefined or mixed semantic helper vocabulary,
- incomplete pure expression semantics,
- incomplete pattern semantics,
- implicit propagation/failure ownership,
- missing determinism boundary.

### Step 2: Verify RED

Expected failure conditions:

- at least one workflow rule still depends on implicit `eval(...)`, `bind(...)`, or helper behavior not defined as a first-class judgment or contract.

### Step 3: Implement the minimal semantic backbone fix

Add the front-matter algebra, judgment inventory, and propagation/failure conventions needed to make the rest of the document structurally complete.

### Step 4: Implement the pure-core completion

Complete the expression and pattern semantics so the pure fragment is proof-shaped and internally consistent.

### Step 5: Implement the helper-contract and meta-property cleanup

Extract runtime-bearing helpers into explicit contracts and add determinism, invariants, and conformance/proof-target sections.

### Step 6: Verify GREEN

Expected pass conditions:

- the revised document reads as one complete big-step core semantics, with explicit proof boundaries and no semantically meaningful hidden shorthand.

### Step 7: Commit

```bash
git add docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-001-IR.md docs/reference/formalization-boundary.md docs/plans/2026-03-29-spec-004-big-step-core-design.md docs/plans/2026-03-29-spec-004-big-step-core-implementation-plan.md docs/plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md CHANGELOG.md
git commit -m "docs: plan complete big-step core semantics for SPEC-004"
```

## Completion Checklist

- [x] explicit judgment inventory added or planned against exact gaps
- [x] pure expression semantics unified
- [x] pure pattern semantics unified
- [x] propagation and failure ownership explicit
- [x] helper contracts explicit
- [x] determinism / nondeterminism boundary explicit
- [x] semantic invariants and proof targets explicit
- [x] `CHANGELOG.md` updated

## Completion Notes

- Added proof-shaped workflow, expression, and pattern judgments to `SPEC-004`.
- Completed the canonical pure expression and pattern sections and centralized failure ownership.
- Extracted helper contracts, determinism boundaries, semantic invariants, and conformance targets.
- Aligned adjacent semantics docs and the formalization boundary note with the revised vocabulary.

## Dependencies

- Depends on: current hardened semantic corpus, especially TASK-177 through TASK-185 and later `SPEC-004` cleanup tasks.
- Informs: future Lean formalization and any later small-step semantics work.

## Non-goals

- No Rust implementation edits
- No Lean theorem implementation
- No immediate small-step semantics document
