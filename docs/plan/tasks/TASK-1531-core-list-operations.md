# TASK-1531: Core List Operations in Pure Ash

## Status: ✅ Complete

## Description

Implement the 7 core list operations in pure Ash: `len`, `head`, `tail`, `append`, `concat`, `map`, `filter`. Replace the `pub builtin fn` declarations with `pub fn` implementations.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
- [TASK-1530](TASK-1530-list-type-definition-and-parsing.md) — Type definition dependency

## Acceptance Criteria

- [ ] All 7 operations implemented in pure Ash
- [ ] No `builtin` declarations remain for list operations
- [ ] Operations work with `Nil` and `Cons` pattern matching
- [ ] Recursive implementations are correct
- [ ] Tests verify correctness

## Verification

- `cargo test -p ash-cli --test list_ops_e2e` passes
- New property tests for list operations pass
- No regressions in existing tests
