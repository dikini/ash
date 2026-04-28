# TASK-757: Comprehension Typed Elaboration

## Status: ✅ Complete

## References

- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) §§5-8, 10, 13
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)

## Objective

Type-check and elaborate comprehensions by reusing the Phase 105 generalized typed-do target resolution and elaboration path.

## Files

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/do_target.rs` only if target helpers need small public factoring
- Test: `crates/ash-typeck/tests/task_757_comprehension_elaboration.rs`

## Requirements

1. Require explicit target annotations in the MVP unless target inference is implemented and tested in this task.
2. Resolve the target through existing `resolve_do_target` / dictionary evidence.
3. Check qualifiers left-to-right using the same rules as do statements.
4. Synthesize `K<A>` from the result expression.
5. Elaborate to the same effective core shape as the equivalent explicit `do:K` block.
6. Reject pure RHS with `<-`, wrong constructor RHS, wrong target kind, missing dictionary, and implicit Act-to-Proc lifting.
7. Preserve existing Phase 105 typed-do behavior.

## TDD Steps

1. Add equivalence tests comparing `Expr::Comprehension` with equivalent `Expr::DoBlock` for `Act` and `Proc` targets.
2. Add negative tests for pure RHS with `<-`, Act RHS in Proc target, wrong target kind, and missing target annotation.
3. Implement typed normalization through existing do elaboration helper(s).
4. Run focused typechecker tests.

## Verification Checklist

- [x] Tests fail before implementation.
- [x] Focused comprehension elaboration tests pass.
- [x] Existing TASK-749/TASK-752 typed-do tests still pass.
- [x] `cargo fmt --check` passes.
- [x] `cargo test -p ash-typeck --test task_757_comprehension_elaboration` passes.
- [x] `cargo test -p ash-typeck --test task_749_typed_do` passes.
- [x] Independent review confirms typed-do semantics were reused rather than forked.
