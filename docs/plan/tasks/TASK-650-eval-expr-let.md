# TASK-650: Evaluator — Handle `Expr::Let` in `eval.rs`

## Status: Planned

## Objective

Implement the `EXPR-LET` evaluation rule in the interpreter's expression evaluator.

## Spec Reference

- SPEC-004 §4.6 — `EXPR-LET` evaluation rule:
  ```
  Γ ⊢e expr ⇓ v
  Γ ⊢p pat ⇐ v ⇓ ΔΓ
  Γ ⊕ ΔΓ ⊢e body ⇓ v'
  ─────────────────────────
  Γ ⊢e Let { pattern = pat, expr, body } ⇓ v'
  ```

## Requirements

1. In `crates/ash-interp/src/eval.rs`, add the `Expr::Let` arm to `eval_expr`:
   ```rust
   Expr::Let { pattern, expr, body } => {
       let value = eval_expr(expr, ctx)?;
       // Pattern match: extend environment with bindings
       let bindings = match_pattern(pattern, &value)?;
       // Bindings are added to context
       for (name, val) in bindings {
           ctx.set(name, val);
       }
       eval_expr(body, ctx)
   }
   ```

2. Handle scoping correctly: `Expr::Let` should NOT leak bindings into the parent scope. The `body` sees the binding, but after evaluation, the parent scope is unchanged. Use a child context or save/restore pattern.

3. Handle pattern match failure: for well-typed programs, patterns are irrefutable. For runtime safety, return `EvalError::PatternBindFailure` on refutable pattern failure (per SPEC-004 §4.6.1).

## TDD Steps

1. Test: `Expr::Let { x, 42, x + 1 }` evaluates to `43`
2. Test: nested let-bindings: `let x = 1; let y = x + 1; y * 2` evaluates to `4`
3. Test: bindings don't leak — after evaluating a let-expression, the binding is not visible in the parent context
4. Test: pattern matching in let — `let Some { value: x } = Some { value: 42 }; x` evaluates to `42`
5. Run `cargo test -p ash-interp`

## Estimated Hours

0.5-1

## Completion Checklist

- [ ] `Expr::Let` arm in `eval_expr` evaluates correctly
- [ ] Scoping: bindings don't leak to parent
- [ ] Pattern match failure produces `EvalError`
- [ ] All interpreter tests pass
