# TASK-570: Parser — Add Spans to Variable Bindings

**Phase:** 84
**Spec:** SPEC-039 §3
**Related:** TASK-571
**Estimate:** 6 hours
**Status:** 📝 Planned

## Description

Add spans to `Expr::Variable`, `Pattern::Variable`, and `PolicyExpr::Var` in both the surface AST and core AST, then fix all downstream match sites. `Literal` span work is explicitly deferred per SPEC-039 §3.5.

## Requirements

1. `Expr::Variable(Name)` becomes `Expr::Variable { name: Name, span: Span }` in `surface.rs` and `ast.rs`.
2. `Pattern::Variable(Name)` becomes `Pattern::Variable { name: Name, span: Span }` in `surface.rs` and `ast.rs`.
3. `PolicyExpr::Var(Name)` becomes `PolicyExpr::Var { name: Name, span: Span }` in `surface.rs` and `ast.rs`.
4. Parser captures `current_span()` when parsing identifiers into variable expressions/patterns/policy vars.
5. Lowering threads the span through from surface to core.
6. `ast::Span` derives `Hash` and `Eq` (prerequisite for TASK-571 / CommentTable usage in core AST).
7. All match sites updated in:
   - `ash-typeck/src/check_expr.rs`
   - `ash-typeck/src/check_pattern.rs`
   - `ash-typeck/src/lib.rs`
   - `ash-typeck/src/names.rs`
   - `ash-typeck/src/purity.rs`
   - `ash-interp/src/eval.rs`
   - `ash-repl/src/ast.rs`
   - `ash-core/src/proptest_helpers.rs`
   - `ash-fuzz/fuzz_targets/typeck.rs`
   - All tests constructing these variants
8. Parser identifier capture must leave a clear hook for TASK-571: after parsing a token that yields a span, the site should be structured so that `set_last_token(span)` can be inserted without re-refactoring the same code.

## TDD Steps

### Red
- Change enum definitions; observe compilation failures across workspace.

### Green
- Fix parser, lowering, type checker, interpreter, REPL, and tests.
- Verify `cargo test --all` passes.

## Completion Checklist

- [ ] `Expr::Variable { name, span }` in surface and core AST
- [ ] `Pattern::Variable { name, span }` in surface and core AST
- [ ] `PolicyExpr::Var { name, span }` in surface and core AST
- [ ] `ast::Span` derives `Hash` and `Eq`
- [ ] Parser and lowering updated
- [ ] All downstream match sites fixed
- [ ] Parser span-capture sites structured to accommodate `set_last_token(span)` protocol from TASK-571
- [ ] All tests updated and passing
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean
