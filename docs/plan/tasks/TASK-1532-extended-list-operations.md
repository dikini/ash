# TASK-1532: Extended List Operations

## Status: ✅ Complete

## Description

Implement additional list operations needed for QuickCheck combinators and general use: `index`, `take`, `drop`, `reverse`, `prepend`.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
- [TASK-1531](TASK-1531-core-list-operations.md) — Core operations dependency

## Acceptance Criteria

- [ ] `index(list, n)` — get element at index n (O(n))
- [ ] `take(n, list)` — first n elements
- [ ] `drop(n, list)` — all but first n elements
- [ ] `reverse(list)` — reversed list
- [ ] `prepend(item, list)` — add to front (O(1))
- [ ] All operations tested

## Verification

- Property tests for each operation
- Edge cases: empty lists, out-of-bounds, negative indices
- No regressions in existing tests
