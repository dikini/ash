# TASK-1573: Add Performance Benchmarks for List Operations

**Status:** 📝 Planned
**Phase:** [PLAN-157](../PLAN-157-LIST-MIGRATION-HARDENING.md)
**Owner:** Phase 157

## Goal

Add performance benchmarks to measure the impact of the Cons/Nil representation compared to the old `Value::List` representation.

## Benchmarks to Add

1. **List construction:** `[]` vs `Nil`, `[1, 2, 3]` vs nested `Cons`
2. **List traversal:** `len`, `head`, `tail` on lists of varying sizes
3. **List concatenation:** `concat` on lists of varying sizes
4. **List mapping:** `map` with simple function
5. **List reversal:** `reverse` on lists of varying sizes

## Implementation

Add benchmarks to `crates/ash-bench/benches/` or create a new benchmark crate.

## Expected Results

- `len`: O(n) instead of O(1) — expected slowdown
- `head`: O(1) — same performance
- `tail`: O(1) — same performance
- `concat`: O(n) instead of O(n) — similar performance
- `reverse`: O(n) instead of O(n) — similar performance

## Verification

- `cargo bench` runs successfully
- Results are documented in a benchmark report

## Notes

The main performance concern is `len` going from O(1) to O(n). This is acceptable for the purity gain, but should be documented.
