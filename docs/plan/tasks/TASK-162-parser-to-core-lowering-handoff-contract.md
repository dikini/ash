# TASK-162: Parser-to-Core Lowering Handoff Contract

## Status: ✅ Complete

## Description

Write the explicit contract between parser output and canonical core/lowering forms for the
stabilized convergence features.

This task defines which surface nodes lower to which core forms and which combinations must be
rejected before or during lowering.

## Specification Reference

- SPEC-001: IR
- SPEC-002: Surface Language
- SPEC-006: Policy Definitions
- SPEC-013: Streams and Event Processing
- SPEC-020: ADT Types

## Audit Reference

- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`

## Requirements

### Functional Requirements

1. Map accepted surface nodes to canonical core forms
2. Define lowering-time rejection cases for invalid combinations
3. Cover `check`, `decide`, policies, `receive`, and ADT declarations
4. Give downstream Rust tasks one authoritative lowering reference

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- Which surface forms lower into which canonical core nodes?
- Which invalid combinations are rejected during lowering?
- Which features preserve information from surface AST into core AST?
- Which contract owns the lowering meaning for `receive` and policy forms?

## Files

- Create: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:
- surface-to-core mappings,
- invalid lowering combinations,
- feature-specific lowering preservation,
- policy and `receive` lowering meaning.

### Step 2: Verify RED

Expected failure conditions:
- no single lowering contract exists,
- at least one stabilized feature still relies on implied lowering behavior,
- lowering rejection boundaries are not explicit.

Observed before implementation:
- there was no `docs/reference/parser-to-core-lowering-contract.md`.
- lowering meaning for `decide`, `check`, `receive`, policy expressions, and ADT forms had to be
  inferred from `SPEC-001`, `SPEC-006`, `SPEC-013`, `SPEC-020`, and the current lowering code.
- current lowering still exposes placeholder behavior such as `"default"` policy fallback,
  dummy-obligation lowering for policy-target checks, and `receive` lowering to `Done`.

### Step 3: Implement the minimal reference (Green)

Document only the lowering contract required by downstream parser/core work.

### Step 4: Verify GREEN

Expected pass conditions:
- one explicit surface-to-core mapping exists for the stabilized features,
- invalid lowering cases are named,
- policy and `receive` lowering are explicitly covered.

Verified after implementation:
- `docs/reference/parser-to-core-lowering-contract.md` now centralizes the required surface-to-core
  mappings.
- lowering-time rejection cases are explicitly listed.
- policy lowering, `receive` lowering, and ADT lowering preservation rules are covered directly.

### Step 5: Commit

```bash
git add docs/reference/parser-to-core-lowering-contract.md CHANGELOG.md
git commit -m "docs: add parser to core lowering contract"
```

## Completion Checklist

- [x] surface-to-core mappings documented
- [x] lowering rejection cases documented
- [x] policy lowering documented
- [x] `receive` lowering documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No lowering implementation changes
- No type-checking rules yet
- No runtime behavior changes

## Dependencies

- Depends on: TASK-161
- Blocks: TASK-166, TASK-167
