# PLAN-156: Parser Blocker Resolution for List Migration

**Status:** ✅ Complete; Parser blockers resolved, Phase 153 unblocked
**Spec:** [SPEC-092: Parser Blocker Resolution](../spec/SPEC-092-PARSER-BLOCKER-RESOLUTION.md)
**Blocked Phase:** [PLAN-153: List Builtin to Stdlib](PLAN-153-LIST-BUILTIN-TO-STDLIB.md) (now unblocked)
**Completion Date:** 2026-06-17
**Commit:** `55d69387`
**Task range:** TASK-1560 through TASK-1564

## Goal

Fix three parser blockers that prevent Phase 153 (List Builtin to Stdlib) from proceeding. The blockers are:
1. `if`/`else` with `match` in `else` branch
2. Variant patterns with record payloads (`Cons { head: h, tail: rest }`)
3. List literal patterns in `match` (`[h, ..rest]`)

## Core Design

### B1: `if`/`else` with `match`

The `parse_fn_if_expr` function in `parse_module/fn_defs.rs` needs to correctly handle `else` followed by `match`. The issue is likely in how `parse_fn_block_or_expr` handles the `else` branch after parsing the `then` branch.

### B2: Variant patterns with record payloads

The `parse_variant_pattern` function calls `parse_variant_fields` for record payloads. The field parsing requires `field: pattern` syntax. The issue may be in backtracking or whitespace handling.

### B3: List literal patterns

The `parse_list_pattern` function exists but may not be reached in the `match` context. The `pattern()` function's `alt` combinator order may need adjustment.

## Non-Goals

- No changes to list literal syntax (`[1, 2, 3]`)
- No changes to `List<T>` type representation
- No runtime changes (those are in Phase 153)

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Fix `if`/`else` with `match` | TASK-1560 |
| D2 | Fix variant patterns with record payloads | TASK-1561 |
| D3 | Fix list literal patterns in `match` | TASK-1562 |
| D4 | Add regression tests for all blockers | TASK-1563 |
| D5 | Verify Phase 153 unblocked | TASK-1564 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1560](tasks/TASK-1560-fix-if-else-match.md) | Fix `if`/`else` with `match` in `else` branch | 📝 Planned |
| [TASK-1561](tasks/TASK-1561-fix-variant-record-patterns.md) | Fix variant patterns with record payloads | 📝 Planned |
| [TASK-1562](tasks/TASK-1562-fix-list-patterns.md) | Fix list literal patterns in `match` | 📝 Planned |
| [TASK-1563](tasks/TASK-1563-regression-tests.md) | Add regression tests for all blockers | 📝 Planned |
| [TASK-1564](tasks/TASK-1564-verify-phase-153-unblocked.md) | Verify Phase 153 is unblocked | 📝 Planned |

## Implementation Order

1. TASK-1560: Fix `if`/`else` with `match` (most critical for list operations)
2. TASK-1561: Fix variant patterns with record payloads (parallel)
3. TASK-1562: Fix list literal patterns (parallel)
4. TASK-1563: Add regression tests (depends on all fixes)
5. TASK-1564: Verify Phase 153 unblocked (depends on tests)

## Verification Strategy

Every fix must include:
- Focused parser test for the specific blocker
- End-to-end test with `ash check` on `.ash` file
- `cargo fmt --check`, `cargo test -p ash-parser` gates
- `git diff --check`

## Closeout Criteria

- All TASK-1560 through TASK-1563 tasks are complete
- SPEC-092, PLAN-156, and PLAN-INDEX agree on scope/status
- Phase 153 can proceed with `std/src/list.ash` implementation
- All existing parser tests still pass
- Stdlib corpus check passes

## Notes

This phase is a **prerequisite** for Phase 153. Without these parser fixes, the list operations cannot be written in idiomatic Ash.

The risk is low — all changes are localized to the parser crate. No runtime or typechecker changes needed.

## Relationship to Other Phases

| Phase | Relationship |
-------|-------------|
| Phase 153 | Unblocks: List Builtin to Stdlib |
| Phase 151 | Enables: QuickCheck combinators using list operations |
| Phase 152 | Consistent: Closure refinement works with list patterns |
