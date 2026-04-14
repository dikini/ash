# TASK-553: Type Checker -- FnDef and FnApply Typing

**Phase:** 80
**Spec:** SPEC-031 §6
**Depends on:** TASK-551
**Estimate:** 4 hours

## Description

Add type checking for `Expr::FnDef` and `Expr::FnApply` using the existing `Type::Fn` and `Type::Fun` type variants.

## Requirements

### 1. FnDef Type Checking

In `crates/ash-typeck/src/check_expr.rs`, add an arm for `Expr::FnDef`:

- Extend the type environment with parameter bindings
- Type-check the body in the extended environment
- Result type: `Type::Fn(param_types, return_type)` for pure context
- (Effectful context typing deferred to TASK-558)

### 2. FnApply Type Checking

Add an arm for `Expr::FnApply`:

- Type the `func` expression
- Verify it's `Type::Fn(params, ret)` or `Type::Fun(params, ret, effect)`
- Unify argument types with parameter types
- Return the return type

### 3. Reuse Existing Infrastructure

The existing `lookup_call_target` and `instantiate_fn_call` logic in `check_expr.rs:189-230` already handles `Type::Fn` and `Type::Fun`. The `FnApply` arm should reuse this by:

1. Type the `func` expression to get its type
2. Match against `Type::Fn`/`Type::Fun`
3. Unify argument types using existing `instantiate_fn_call`

## TDD Steps

1. Test: `fn(x: Int) -> Int { x + 1 }` type-checks as `Type::Fn([Int], Int)`
2. Test: `f(1, 2)` where `f: Type::Fn([Int, Int], Int)` type-checks as `Int`
3. Test: `f(1, 2)` where `f: Type::Fn([String, String], String)` produces type error
4. Test: `f(1)` where `f: Type::Fn([Int, Int], Int)` produces arity error
5. Verify `cargo test --all` passes

## Completion Checklist

- [ ] `Expr::FnDef` arm in `check_expr`
- [ ] `Expr::FnApply` arm in `check_expr`
- [ ] Reuses existing `Type::Fn`/`Type::Fun` infrastructure
- [ ] Type checking tests
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
