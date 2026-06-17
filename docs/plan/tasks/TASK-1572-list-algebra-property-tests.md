# TASK-1572: Add Property Tests for List Algebraic Laws

**Status:** 📝 Planned
**Phase:** [PLAN-157](../PLAN-157-LIST-MIGRATION-HARDENING.md)
**Owner:** Phase 157

## Goal

Add property tests that verify the algebraic laws for list operations implemented in `std/src/list.ash`.

## Laws to Verify

### Functor Laws (via `list_functor_map` / `map`)
1. **Identity:** `map(list, fn(x) { x }) == list`
2. **Composition:** `map(list, f . g) == map(map(list, g), f)`

### Semigroup Laws (via `list_semigroup_append` / `concat`)
1. **Associativity:** `concat(concat(a, b), c) == concat(a, concat(b, c))`

### Monoid Laws (via `list_monoid_empty` / `[]`)
1. **Left identity:** `concat([], list) == list`
2. **Right identity:** `concat(list, []) == list`

## Implementation

Add tests to `crates/ash-engine/tests/list_ops_e2e.rs` or create a new test file.

## Verification

- `cargo test -p ash-engine --test <test_file>` passes
- Tests use `proptest` or QuickCheck infrastructure where available

## Notes

These tests verify that the pure Ash implementations satisfy the same laws as the old Rust builtins.
