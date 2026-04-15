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
3. `PolicyExpr::Var(Name)` becomes `PolicyExpr::Var { name: Name, span: Span }` in `surface.rs` only (`PolicyExpr` is surface-only).
4. Parser captures `current_span()` when parsing identifiers into variable expressions/patterns/policy vars.
5. Lowering threads the span through from surface to core.
6. Update `impl Spanned for Expr` and `impl Spanned for PolicyExpr` in `crates/ash-parser/src/surface.rs` to return the new `span` fields instead of `Span::default()`.
7. `ast::Span` derives `Hash` and `Eq` (required for downstream Salsa usage in SPEC-043; not strictly required for SPEC-039/TASK-570 itself).
8. All match sites updated (~400+ call sites across the workspace), including:
   - `ash-parser/src/desugar.rs`
   - `ash-parser/src/parse_workflow.rs`
   - `ash-typeck/src/constraints.rs`
   - `ash-typeck/src/capability_check.rs`
   - `ash-typeck/src/policy_check.rs`
   - `ash-typeck/src/effect.rs`
   - `ash-typeck/src/solver.rs`
   - `ash-interp/src/execute.rs`
   - `ash-interp/src/execute_observe.rs`
   - `ash-interp/src/execute_stream.rs`
   - `ash-interp/src/policy.rs`
   - `ash-interp/src/pattern.rs`
   - `ash-interp/src/guard.rs`
   - `ash-interp/src/lib.rs`
   - `ash-core/src/visualize.rs`
   - `ash-core/src/stream.rs`
   - `ash-core/src/test_helpers.rs`
   - `ash-bench/benches/core.rs`
   - `ash-bench/benches/interp.rs`
   - plus `check_expr.rs`, `check_pattern.rs`, `lib.rs`, `names.rs`, `purity.rs`, `eval.rs`, `repl/ast.rs`, `proptest_helpers.rs`, `fuzz_targets/typeck.rs`, and all tests constructing these variants
9. Parser identifier capture must leave a clear hook for TASK-571: after parsing a token that yields a span, the site should be structured so that `set_last_token(span)` can be inserted without re-refactoring the same code.

## TDD Steps

### Red
- Change enum definitions; observe compilation failures across workspace.

### Green
- Fix parser, lowering, type checker, interpreter, REPL, and tests.
- Verify `cargo test --all` passes.

## Completion Checklist

- [ ] `Expr::Variable { name, span }` in surface and core AST
- [ ] `Pattern::Variable { name, span }` in surface and core AST
- [ ] `PolicyExpr::Var { name, span }` in surface AST only
- [ ] `impl Spanned for Expr` and `impl Spanned for PolicyExpr` updated in `surface.rs`
- [ ] `ast::Span` derives `Hash` and `Eq`
- [ ] Parser and lowering updated
- [ ] All downstream match sites fixed (~400+ call sites)
- [ ] Parser span-capture sites structured to accommodate `set_last_token(span)` protocol from TASK-571
- [ ] All tests updated and passing
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean
