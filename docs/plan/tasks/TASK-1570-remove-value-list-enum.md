# TASK-1570: Remove Value::List from ash_core::Value Enum

**Status:** ✅ Complete via Phase 176 TASK-1797
**Phase:** [PLAN-157](../PLAN-157-LIST-MIGRATION-HARDENING.md)
**Owner:** Phase 157

## Goal

Completely remove the `Value::List` variant from the `ash_core::Value` enum. After Phase 153, all list operations use `Cons`/`Nil` variants. This task removes the legacy `Value::List` representation entirely.

## Scope

1. Remove `Value::List` from `crates/ash-core/src/value.rs`
2. Update all pattern matches that handle `Value::List` to use `Value::Variant` (Cons/Nil)
3. Update `Literal::List` to convert to `Cons`/`Nil` constructors
4. Remove list-specific builtin dispatch code
5. Update tests that construct `Value::List` directly

## Files to Modify

- `crates/ash-core/src/value.rs` - Remove `Value::List` variant
- `crates/ash-core/src/ast.rs` - Update `Literal::List` if needed
- `crates/ash-parser/src/lower.rs` - Ensure lists lower to Cons/Nil
- `crates/ash-interp/src/eval.rs` - Remove Value::List handling from builtins
- `crates/ash-interp/src/pattern.rs` - Simplify pattern matching (only Variants)
- `crates/ash-interp/src/list_helpers.rs` - May need updates
- Multiple test files that use `Value::List`

## Verification

- `cargo check --workspace` passes
- `cargo test --workspace` passes (or only pre-existing failures)
- `std/src/list.ash` still compiles and runs

## Notes

This is a high-risk change that touches many files. The approach should be:
1. First, audit all `Value::List` usage
2. Update each file systematically
3. Run tests after each major component
4. Commit incrementally


## Phase 176 completion note

TASK-1797 completed this deferred item: `Value::List` was removed from `ash_core::Value`, constructor/pattern call sites were migrated to canonical list helpers over `Cons`/`Nil`, serialized list compatibility was preserved, and repository verification found no remaining `Value::List` references in Rust source under `crates/`.
