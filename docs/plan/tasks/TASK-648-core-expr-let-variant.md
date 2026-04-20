# TASK-648: Add `Expr::Let` to Core IR + Fix All Exhaustive Matches

## Status: Planned

## Objective

Add `Expr::Let { pattern, expr, body }` to the core `Expr` enum in `ash-core/src/ast.rs`, then fix all exhaustive match sites across the workspace to handle the new variant.

## Spec Reference

- SPEC-001 §2.0 — canonical core expression forms (amended)
- SPEC-001 §2.6 — Expr enum definition (amended)
- NOTE-003 — design rationale

## Requirements

1. Add `Expr::Let` variant to `ash_core::ast::Expr`:
   ```rust
   Let {
       pattern: Pattern,
       expr: Box<Expr>,
       body: Box<Expr>,
       span: Span,
   }
   ```
   The `span` field is required for diagnostics when pattern matching fails
   at runtime (see SPEC-004 §4.6.1 — `PatternBindFailure` must report location).

2. Fix all exhaustive `match` sites on `Expr` across these crates:
   - `ash-interp/src/eval.rs` (~208 sites) — add `Expr::Let` arm that evaluates `expr`, matches `pattern`, extends env, evaluates `body`. Return `todo!()` initially if evaluator work is deferred to TASK-650.
   - `ash-typeck/src/check_expr.rs` (~494 sites across multiple files) — add `Expr::Let` arm. Typecheck `expr`, check pattern against its type, extend environment, typecheck `body`. Return `todo!()` initially if typechecker work is deferred to TASK-651.
   - `ash-engine/src/lib.rs`, `module_loader.rs`, `monomorphize.rs`, `providers/mod.rs` (~28 sites) — add appropriate handling.
   - `ash-parser/src/lower.rs` — this file will be fully handled in TASK-649, but must compile after the variant addition.

3. Compilation gate: `cargo check --workspace` must pass after all sites are updated.

## TDD Steps

1. Add the variant to `Expr` enum
2. Run `cargo check --workspace 2>&1` to get the full list of broken match sites
3. Fix each site — for evaluator and typechecker, use `todo!("TASK-650/651")` initially
4. Verify `cargo check --workspace` passes

## Estimated Hours

2-3

## Completion Checklist

- [ ] `Expr::Let` variant exists in `ash-core/src/ast.rs`
- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --all-targets` clean
- [ ] All match sites handle `Expr::Let` (even if via `todo!()`)
