# TASK-756: Comprehension Lowering Boundary and Cross-Crate Visitors

## Status: 📝 Planned

## References

- [SPEC-055](../../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) §§5, 8, 11, 13
- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) parser-only lowering boundary

## Objective

Wire all non-typechecking visitors for the new comprehension surface node and enforce that parser-only lowering rejects/defer comprehensions until typed elaboration.

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-typeck/src/names.rs`
- Modify: `crates/ash-typeck/src/capability_check.rs`
- Modify: `crates/ash-typeck/src/purity.rs` only if exhaustive matching requires it; otherwise preserve the existing typed-do purity boundary and document the deferral
- Modify: `crates/ash-lint/src/lib.rs`
- Modify: `crates/ash-repl/src/ast.rs`
- Test: focused parser/lowerer/typeck compile or regression tests as needed

## Requirements

1. `lower_expr(Expr::Comprehension)` must reject with a clear typed-elaboration-required diagnostic, matching `Expr::DoBlock` policy.
2. Name resolution must traverse result expression, qualifier RHS expressions, and let RHS expressions in correct left-to-right scope where applicable.
3. Capability/name/lint/REPL visitors must recurse into all child expressions without inventing comprehension-specific semantics. Purity handling must mirror the current `Expr::DoBlock` policy unless this task explicitly broadens scope to fix typed-do and comprehension purity traversal together.
4. Lint and REPL AST rendering must handle the new node explicitly.
5. No task in this slice may lower to untyped `bind` / `return` calls.

## TDD Steps

1. Add a regression proving parser-only lowering rejects a comprehension.
2. Add traversal regression tests where practical for nested calls inside result and qualifiers, excluding purity recursion if the existing DoBlock purity deferral is preserved.
3. Implement exhaustive visitor handling.
4. Run affected crate checks.

## Verification Checklist

- [ ] Lowerer rejection test passes.
- [ ] Affected crates compile without non-exhaustive matches.
- [ ] Existing `Expr::DoBlock` lowering rejection still passes.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo check -p ash-parser -p ash-typeck -p ash-lint -p ash-repl` passes.
- [ ] Independent review confirms no semantic lowering was added and purity handling remains consistent with the Phase 105 DoBlock boundary.
