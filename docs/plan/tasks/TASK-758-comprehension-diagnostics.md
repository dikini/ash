# TASK-758: Comprehension Diagnostics

## Status: ✅ Complete

## References

- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) §12
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §13

## Objective

Add comprehension-specific hard errors and non-fatal teaching diagnostics while reusing the SPEC-054 diagnostic families.

## Files

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/error.rs` only if new hard error variants are required
- Test: `crates/ash-typeck/tests/task_758_comprehension_diagnostics.rs`

## Requirements

1. Missing/ambiguous target diagnostic names comprehension syntax and suggests `[expr | x <- xs]: K`.
2. Wrong-kind target diagnostic names the target and expected `* -> *`.
3. Pure RHS with `<-` suggests `let`.
4. Wrong constructor RHS names expected/found constructors and suggests explicit lift for Act-to-Proc when applicable.
5. Monadic value bound with `let` is reported through the non-fatal diagnostic carrier if general warning emission is still unavailable.
6. Bare boolean qualifiers are rejected at parse/recovery time or diagnosed as unsupported Monad-only guards; the implementation must not accept them as valid qualifiers.
7. Diagnostics must not claim unavailable target inference, guard semantics, or pure List/Option/Result dictionaries.

## TDD Steps

1. Add focused substring/golden-style tests for each diagnostic family.
2. Implement diagnostics with comprehension-specific wording.
3. Re-run existing do-notation diagnostic tests to prevent wording regressions.

## Verification Checklist

- [x] Every SPEC-055 §12 diagnostic family has a test.
- [x] Existing SPEC-054/TASK-752 diagnostics still pass.
- [x] `cargo fmt --check` passes.
- [x] `cargo test -p ash-typeck --test task_758_comprehension_diagnostics` passes.
- [x] `cargo test -p ash-typeck --test task_752_do_diagnostics` passes.
- [x] Independent review confirms no diagnostic overclaims.
