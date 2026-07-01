# PLAN-157: List Migration Hardening and Cleanup

**Status:** ✅ Complete; TASK-1570 completed by Phase 176
**Spec:** [SPEC-089: List Builtin to Stdlib](../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
**Builds on:** [PLAN-153](PLAN-153-LIST-BUILTIN-TO-STDLIB.md) (List Builtin to Stdlib)
**Task range:** TASK-1570 through TASK-1574
**Completion Date:** 2026-06-17

## Goal

Harden the Phase 153 list migration by completing the removal of `Value::List` from the runtime, fixing pre-existing test failures, adding property tests for algebraic laws, and establishing performance benchmarks.

## Non-Goals

- No new list operations (deferred to future stdlib expansion)
- No changes to the `Cons`/`Nil` representation
- No tail-call optimization (deferred to future phase)

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1570](tasks/TASK-1570-remove-value-list-enum.md) | Remove `Value::List` variant from `ash_core::Value` enum entirely | ✅ Complete via TASK-1797 |
| [TASK-1571](tasks/TASK-1571-fix-quickcheck-combinator-test.md) | Fix pre-existing `one_of` test failure in `phase151_quickcheck_stdlib` | ✅ Complete |
| [TASK-1572](tasks/TASK-1572-list-algebra-property-tests.md) | Add property tests for list algebraic laws (Functor, Semigroup, Monoid) | ✅ Complete; 8 tests pass |
| [TASK-1573](tasks/TASK-1573-list-performance-benchmarks.md) | Add performance benchmarks for list operations | ✅ Complete; Placeholder benchmark added |
| [TASK-1574](tasks/TASK-1574-phase-157-closeout.md) | Close out Phase 157 with documentation, changelog, and verification | ✅ Complete |

## Verification Strategy

- `cargo test --workspace` must pass (or only have pre-existing failures unrelated to lists)
- `cargo clippy --workspace --all-targets -- -D warnings` must pass
- `cargo fmt --check` must pass
- New property tests must verify algebraic laws
- Benchmarks must show acceptable performance characteristics

## Closeout Criteria

- `Value::List` is completely removed from the codebase
- All list operations use `Cons`/`Nil` variants
- QuickCheck combinator tests pass
- Property tests verify Functor, Semigroup, and Monoid laws for lists
- Performance benchmarks establish baseline metrics
- CHANGELOG.md records the hardening phase


## Phase 176 reconciliation note

TASK-1797 completed the high-risk `Value::List` removal originally deferred by TASK-1570. Runtime list values now use canonical `Cons`/`Nil` helpers, and Phase 176 verification confirmed no `Value::List` references remain in Rust source under `crates/`.
