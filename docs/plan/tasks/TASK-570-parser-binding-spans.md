# TASK-570: Parser — Add Spans to Variable Bindings

**Phase:** 84
**Spec:** SPEC-039 §3
**Related:** TASK-571
**Estimate:** 6 hours
**Status:** 📝 Planned

## Description

Add `span: ash_parser::token::Span` to `Expr::Variable` and `Pattern::Variable` in both the surface AST and core AST, then fix all downstream match sites.

## Requirements

1. `Expr::Variable(Name)` becomes `Expr::Variable(Name, Span)` in `surface.rs` and `ast.rs`.
2. `Pattern::Variable(Name)` becomes `Pattern::Variable(Name, Span)` in `surface.rs` and `ast.rs`.
3. Parser captures `current_span()` when parsing identifiers into variable expressions/patterns.
4. Lowering threads the span through from surface to core.
5. All match sites updated in:
   - `ash-typeck/src/check_expr.rs`
   - `ash-typeck/src/check_pattern.rs`
   - `ash-typeck/src/names.rs`
   - `ash-typeck/src/purity.rs`
   - `ash-interp/src/eval.rs`
   - `ash-repl/src/ast.rs`
   - All tests constructing `Expr::Variable` or `Pattern::Variable`

## TDD Steps

### Red
- Change enum definitions; observe compilation failures across workspace.

### Green
- Fix parser, lowering, type checker, interpreter, REPL, and tests.
- Verify `cargo test --all` passes.

## Completion Checklist

- [ ] `Expr::Variable(Name, Span)` in surface and core AST
- [ ] `Pattern::Variable(Name, Span)` in surface and core AST
- [ ] Parser and lowering updated
- [ ] All downstream match sites fixed
- [ ] All tests updated and passing
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean
