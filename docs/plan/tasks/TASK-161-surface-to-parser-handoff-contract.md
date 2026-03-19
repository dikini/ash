# TASK-161: Surface-to-Parser Handoff Contract

## Status: ✅ Complete

## Description

Write the explicit handoff contract between the stabilized surface syntax and the parser layer.

This task freezes which surface forms are accepted, which surface AST nodes must be produced,
and which parser failures are legal for the convergence features.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-013: Streams and Event Processing
- SPEC-020: ADT Types

## Audit Reference

- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`
- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`

## Requirements

### Functional Requirements

1. Define accepted syntax forms for stabilized workflow, policy, `receive`, and ADT features
2. Define the surface AST nodes the parser must produce for those forms
3. Define parser-rejection cases that are legal and expected
4. Make the handoff reference the single parser authority for downstream tasks

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- Which surface forms are accepted for `check`, `decide`, `receive`, policies, and ADTs?
- Which surface AST node is produced for each accepted form?
- Which invalid forms must be rejected by the parser?
- Which failures belong to parsing versus later lowering or type checking?

## Files

- Create: `docs/reference/surface-to-parser-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:
- accepted surface syntax,
- produced surface AST nodes,
- rejected syntax,
- parser versus later-phase responsibilities.

### Step 2: Verify RED

Expected failure conditions:
- no single reference currently covers the parser handoff,
- at least one stabilized form still requires inference across multiple docs,
- parser rejection boundaries are not fully centralized.

Observed before implementation:
- `docs/reference/` did not exist, so there was no parser-handoff reference file.
- accepted `check`, `decide`, `receive`, policy, and ADT parser expectations had to be inferred
  across `SPEC-002`, `SPEC-013`, `SPEC-020`, and current parser AST code.
- parser-vs-lowering boundaries were implicit rather than written in one authority file.

### Step 3: Implement the minimal reference (Green)

Document only the handoff needed between surface syntax and parser AST.

### Step 4: Verify GREEN

Expected pass conditions:
- one explicit reference covers the stabilized forms,
- parser outputs are named concretely,
- parser failure boundaries are explicit.

Verified after implementation:
- `docs/reference/surface-to-parser-contract.md` is now the single parser-handoff reference.
- required parser outputs are named concretely using `ash_parser::surface` and
  `ash_parser::parse_type_def` node names.
- parser rejection cases and parser-vs-later boundaries are explicitly listed.

### Step 5: Commit

```bash
git add docs/reference/surface-to-parser-contract.md CHANGELOG.md
git commit -m "docs: add surface to parser contract"
```

## Completion Checklist

- [x] accepted surface forms documented
- [x] parser AST outputs documented
- [x] legal parser failures documented
- [x] parser/later-phase boundary documented
- [x] `CHANGELOG.md` updated

## Non-goals

- No parser implementation changes
- No lowering contract yet
- No type-checker or interpreter changes

## Dependencies

- Depends on: TASK-156 through TASK-160
- Blocks: TASK-164, TASK-165, TASK-166, TASK-167
