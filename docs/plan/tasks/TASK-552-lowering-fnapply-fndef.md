# TASK-552: Lowering -- Built-in Registry, FnApply, FnDef

**Phase:** 80
**Spec:** SPEC-031 §5.4, §9.1, §9.2
**Depends on:** TASK-551
**Estimate:** 4 hours

## Description

Wire `Expr::FnDef` and `Expr::FnApply` through the lowering step. Add a built-in function registry to disambiguate `Call` vs `FnApply` at lowering time.

## Requirements

### 1. Built-in Registry

Add `BUILTIN_FUNCTIONS: &[&str]` to `crates/ash-parser/src/lower.rs`. Extract the complete list from `eval_function_call` in `crates/ash-interp/src/eval.rs`.

### 2. lower_expr Updates

- Surface `Expr::FnDef` -> `CoreExpr::FnDef { params, return_type, body: lower_expr(body) }`
- Surface `Expr::Call` where callee is NOT in `BUILTIN_FUNCTIONS` -> `CoreExpr::FnApply { func: Box::new(Expr::Variable(name)), args }`
- Surface `Expr::Call` where callee IS in `BUILTIN_FUNCTIONS` -> `CoreExpr::Call { func, arguments }` (unchanged)

### 3. lower_fn_def

Add function to lower surface `FnDef` bodies, extending the parameter environment for type-checking.

### 4. Temporary Expr::Call Closure Fallback

Add a fallback in the interpreter's `Expr::Call` handler: if the callee is not a built-in, check if it resolves to a `Value::Closure` in context and apply it. This handles the transition period where lowering may not yet produce `FnApply` for all cases.

## TDD Steps

1. Test: lowering `fn(x) { x + 1 }` produces `CoreExpr::FnDef`
2. Test: lowering `f(1, 2)` where `f` is not a built-in produces `CoreExpr::FnApply`
3. Test: lowering `len(xs)` produces `CoreExpr::Call` (built-in)
4. Test: built-in registry is complete (compare against `eval_function_call`)
5. Verify `cargo test --all` passes

## Completion Checklist

- [ ] `BUILTIN_FUNCTIONS` registry in `lower.rs`
- [ ] `lower_expr` produces `FnApply` for user calls
- [ ] `lower_fn_def` for FnDef body lowering
- [ ] Temporary `Expr::Call` closure fallback in interpreter
- [ ] Tests for lowering correctness
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
