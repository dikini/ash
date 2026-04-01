# TASK-360 RuntimeError Type Design

**Date:** 2026-04-01
**Task:** [TASK-360](../plan/tasks/TASK-360-runtime-error-type.md)

## Goal

Align the stdlib `RuntimeError` definition with the canonical single-variant ADT surface required by the Phase 57 entry-point contracts, and update the immediate dependent tests and docs so the repository consistently treats `RuntimeError` as a constructor-bearing ADT instead of a plain record alias.

## Constraints

- Keep scope limited to TASK-360.
- Preserve the public import surface: `use runtime::RuntimeError`.
- Follow TDD: add failing tests before implementation.
- Update `CHANGELOG.md` for the completed task.
- Avoid pulling in blocked runtime/bootstrap behavior from later Phase 57 tasks.

## Chosen Approach

Change `std/src/runtime/error.ash` from:

```ash
pub type RuntimeError = { exit_code: Int, message: String };
```

to the canonical ADT form:

```ash
pub type RuntimeError = RuntimeError {
    exit_code: Int,
    message: String
};
```

Then update the direct dependency surface:

1. parser-facing stdlib parsing tests
2. stdlib surface tests
3. task/docs text that still describe the obsolete record-alias shape
4. changelog entry for TASK-360

## Why This Approach

- It matches the task contract and the existing Phase 57 design notes.
- It keeps the change localized to stdlib syntax and its immediate consumers.
- It creates a concrete regression barrier so later entry-point tasks can rely on `RuntimeError { ... }` constructor syntax.

## Testing Strategy

1. Add failing tests asserting the exact ADT constructor-bearing syntax in the stdlib file.
2. Run focused `ash-parser` stdlib tests to watch them fail.
3. Implement the minimal source and doc updates.
4. Re-run focused tests and a targeted error scan.

## Expected Files

- Modify: `std/src/runtime/error.ash`
- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `docs/plan/tasks/TASK-360-runtime-error-type.md`
- Modify: `CHANGELOG.md`

## Non-Goals

- No runtime bootstrap or supervisor behavior changes.
- No CLI or engine changes.
- No expansion into TASK-362 through TASK-367 unless a focused failing test proves hidden coupling.
