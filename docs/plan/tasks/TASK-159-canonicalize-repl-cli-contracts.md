# TASK-159: Canonicalize REPL and CLI Contracts

## Status: ✅ Complete

## Description

Freeze one authoritative user-facing contract for REPL and CLI behavior, including command set, authority split, history/config behavior, and observable output expectations.

This task ensures later CLI and REPL code work converges on one product contract instead of preserving both current implementations.

## Specification Reference

- SPEC-005: CLI
- SPEC-011: REPL
- SPEC-016: Output

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Define one canonical REPL command set
2. Define which commands belong to CLI shell behavior versus REPL session behavior
3. Define canonical history/config persistence behavior
4. Define canonical type-display and AST-display behavior, if supported
5. Define canonical help and output expectations
6. Remove or explicitly supersede contradictory wording in the touched specs

### Contract Invariants

After completion, a reviewer should be able to answer all of the following with one answer:

- Which commands are supported in the REPL?
- Is there one REPL authority or two?
- How is history stored and configured?
- What does `:type` show?
- What output is required versus optional?

## Files

- Modify: `docs/spec/SPEC-005-CLI.md`
- Modify: `docs/spec/SPEC-011-REPL.md`
- Modify: `docs/spec/SPEC-016-OUTPUT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Create a review checklist covering:

- command set,
- authority boundaries,
- history behavior,
- type display,
- AST display,
- help/output behavior.

### Step 2: Verify RED

Expected failure conditions:

- at least one command mismatch across the touched specs
- at least one ambiguity in CLI versus REPL ownership
- at least one ambiguity in observable output requirements

### Step 3: Implement the minimal spec repair (Green)

Update only the sections needed to define one user-facing REPL/CLI contract.

### Step 4: Verify GREEN

Expected pass conditions:

- one canonical command surface
- one authority split
- one history/config story
- one output story for the touched features

### Step 5: Commit

```bash
git add docs/spec/SPEC-005-CLI.md docs/spec/SPEC-011-REPL.md docs/spec/SPEC-016-OUTPUT.md CHANGELOG.md
git commit -m "docs: canonicalize repl and cli contracts"
```

## Completion Checklist

- [x] canonical command set defined
- [x] CLI/REPL authority split defined
- [x] history/config behavior defined
- [x] type/output behavior defined
- [x] contradictory wording removed or superseded
- [x] `CHANGELOG.md` updated

## Non-goals

- No REPL implementation refactor yet
- No new REPL features beyond canonicalization
- No parser or runtime behavior changes

## Dependencies

- Depends on: TASK-156, TASK-157, and TASK-158 where output semantics overlap
- Blocks: REPL/CLI handoff docs and downstream REPL/CLI Rust tasks
