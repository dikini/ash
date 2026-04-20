# TASK-649: Lowerer — Desugar `Expr::Block` → Nested `Expr::Let`, Delete Module Loader Workaround

## Status: Planned

## Objective

Replace the lowerer's rejection of `Expr::Block` with a desugaring pass that converts `{ [let x = e1; ...], tail }` into nested `CoreExpr::Let { pattern, expr, body }`. Delete the `normalize_imported_callable_expr` workaround from `module_loader.rs` since the lowerer now handles this uniformly.

## Spec Reference

- SPEC-001 §2.6 — lowering rule: `Expr::Block → nested Expr::Let`

## Requirements

1. In `crates/ash-parser/src/lower.rs`, replace the `Expr::Block` rejection arm:
   ```rust
   // BEFORE:
   Expr::Block { .. } => Err(LoweringError::ExprNotLowerable { kind: "block" }),
   
   // AFTER:
   Expr::Block { statements, tail_expr, .. } => {
       let tail = tail_expr.as_deref().map_or_else(
           || Ok(CoreExpr::Literal(ash_core::Value::Null)),
           |e| lower_expr(e),
       )?;
       let mut result = tail;
       for stmt in statements.iter().rev() {
           match stmt {
               BlockStmt::Let { pattern, expr, .. } => {
                   result = CoreExpr::Let {
                       pattern: lower_pattern(pattern)?,
                       expr: Box::new(lower_expr(expr)?),
                       body: Box::new(result),
                   };
               }
               // Handle any other BlockStmt variants
           }
       }
       Ok(result)
   }
   ```

2. Verify `lower_fn_def` still works — it calls `lower_expr(body)` which will now handle `Expr::Block` through the new desugaring.

3. In `crates/ash-engine/src/module_loader.rs`:
   - Delete `normalize_imported_callable_expr` function entirely
   - Remove all calls to it — imported callables should now use their raw `Expr::Block` bodies, which will be desugared during lowering.

4. Also handle `Expr::Block` in `lower_expr_for_module` (the module-scope variant that rejects `FnDef`) if it exists.

## TDD Steps

1. Write a test: `lower_expr` on `Expr::Block { [Let x = 1], tail_expr: Some(x + 2) }` produces `CoreExpr::Let { pattern: x, expr: 1, body: x + 2 }`
2. Write a test: nested let-bindings produce nested `CoreExpr::Let`
3. Implement the desugaring
4. Run `cargo test -p ash-parser`
5. Delete `normalize_imported_callable_expr` and its calls
6. Run `cargo test --workspace`

## Estimated Hours

1-2

## Completion Checklist

- [ ] `Expr::Block` desugars to nested `Expr::Let` in the lowerer
- [ ] `normalize_imported_callable_expr` deleted from module_loader
- [ ] All existing tests pass
- [ ] New unit tests for the desugaring pass
- [ ] `cargo clippy --all-targets` clean
