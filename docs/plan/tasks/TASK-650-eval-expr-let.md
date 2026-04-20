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
   Expr::Let { pattern, expr, body, .. } => {
       let value = eval_expr(expr, ctx)?;
       // Pattern match: create a child context with bindings
       // (follow the same pattern as eval_match / eval_if_let)
       let bindings = match_pattern(pattern, &value)?;
       let mut child_ctx = ctx.extend();  // or ctx.clone() + extend
       for (name, val) in bindings {
           child_ctx.set(name, val);
       }
       eval_expr(body, &child_ctx)
   }
   ```
   IMPORTANT: Use a child context (`ctx.extend()` or equivalent), NOT in-place
   mutation of `ctx`. This matches the existing pattern in `eval_match` and
   `eval_if_let` where pattern bindings go into a fresh child scope. The parent
   context must remain unchanged after evaluating a let-expression.

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
