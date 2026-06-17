# TASK-1571: Fix QuickCheck Combinator Test Failure

**Status:** 📝 Planned
**Phase:** [PLAN-157](../PLAN-157-LIST-MIGRATION-HARDENING.md)
**Owner:** Phase 157

## Goal

Fix the pre-existing test failure in `crates/ash-engine/tests/phase151_quickcheck_stdlib.rs` where `one_of` is not found in `test::quickcheck::combinator`.

## Background

The test `quickcheck_prelude_and_canonical_submodule_imports_resolve` tries to import `one_of` from `test::quickcheck::combinator`, but the function doesn't exist in `std/src/test/quickcheck/combinator.ash`.

## Scope

1. Implement `one_of` in `std/src/test/quickcheck/combinator.ash`
2. Implement any other missing combinators that tests expect
3. Verify the test passes

## Expected Implementation

```ash
pub fn one_of<T>(strategies: List<Strategy<T>>) -> Strategy<T> {
    -- Choose one strategy from a list uniformly at random
    Strategy {
        gen: fn(ctx) { 
            let index = choose_int(ctx, 0, len(strategies) - 1)
            index(strategies, index).gen(ctx)
        },
        shrink: fn(t) { [] }
    }
}
```

## Verification

- `cargo test -p ash-engine --test phase151_quickcheck_stdlib` passes

## Notes

This is a pre-existing failure that was not caused by Phase 153. Fixing it as part of hardening.
