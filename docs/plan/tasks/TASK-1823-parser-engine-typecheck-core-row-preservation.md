# TASK-1823: Add parser -> engine/typecheck -> Core row preservation tests

## Status: ✅ Complete

## Description

Add end-to-end tests proving explicit callable rows survive from source parsing through engine/module validation, typechecker-facing summaries, and Core callable rows. This task is the proof that Phase 178 actually closes the Phase 177 validation-only bridge.

## Specification Reference

- [PLAN-178](../PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md)
- [SPEC-095b](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- TASK-1819 through TASK-1822 complete or explicitly re-scoped.

## Requirements

### Functional Requirements

1. Add an end-to-end inline-row fixture that proves the same row reaches Core callable rows.
2. Add an end-to-end `where row` fixture that proves the same row reaches Core callable rows.
3. Add an imported/exported callable fixture proving row requirements survive module boundaries.
4. Add a rowless function fixture proving existing behavior remains stable.
5. Add an open-row or whole-row-variable fixture if TASK-1821 supports lowering it; otherwise add an explicit fail-closed/deferred test.
6. Add assertions that inspect actual row structures, not just successful checks.
7. Combine these with TASK-1822 non-authority assertions where possible.

### Property Requirements

- End-to-end evidence must include parser, engine/typecheck, and Core.
- A parser-only or Core-only test is insufficient for this task.
- Any unsupported row family must be documented with current fail-closed behavior.

## TDD Steps

### Step 1: Write failing end-to-end tests

Add tests in the crate(s) identified by TASK-1818 that can inspect parser output, engine/typechecker summaries, and Core rows.

### Step 2: Verify RED

Run focused tests and confirm the current failure is row loss at the source-to-Core bridge.

### Step 3: Patch integration gaps

Patch only the summary/lowering seams needed for end-to-end preservation.

### Step 4: Verify GREEN

Run focused tests plus affected crate suites.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo test -p ash-core
  - git diff --check
checklist:
  - [x] Inline row fixture reaches Core row.
  - [x] `where row` fixture reaches Core row.
  - [x] Imported/exported callable rows are preserved.
  - [x] Rowless functions remain stable.
```

## Completion Evidence

- Added `crates/ash-engine/tests/task_1823_parser_engine_typecheck_core_row_preservation.rs`.
- Covered inline rows, expanded `where row` rows, imported/exported callable rows across module boundaries, rowless functions, and open row tails through parser inspection, `Engine::parse`, `Engine::check`, workflow row summaries, and Core callable rows.
- Verification: `cargo test -p ash-engine --test task_1823_parser_engine_typecheck_core_row_preservation -- --nocapture`, `cargo test -p ash-parser`, `cargo test -p ash-engine`, `cargo test -p ash-typeck`, `cargo test -p ash-core`, `cargo fmt --check`, `git diff --check`, and `python3 tools/docs/validate_orientation_indexes.py --self-test`.

## Dependencies for Next Task

This task feeds TASK-1824 and TASK-1825.
