# TASK-651: Type Checker — Handle `Expr::Let` in `check_expr.rs`

## Status: Planned

## Objective

Add type checking support for `Expr::Let` in the type checker, ensuring let-bindings extend the type environment correctly and patterns are type-checked against the bound expression's type.

## Spec Reference

- SPEC-004 §4.6 — `EXPR-LET` evaluation rule (type-checking analogue)
- SPEC-004 §4.6.1 — irrefutable pattern requirement for `Expr::Let`

## Requirements

1. In `crates/ash-typeck/src/check_expr.rs`, add `Expr::Let` arm:
   - Typecheck `expr` to get its type
   - Check `pattern` against the expression's type to extract bindings
   - Extend the type environment with the bindings
   - Typecheck `body` in the extended environment
   - Return the body's type as the overall expression type

2. Pattern irrefutability: `Expr::Let` patterns should be irrefutable for well-typed programs. The type checker should warn or error on refutable patterns at expression-level let-bindings (distinct from `Workflow::Let` which allows refutable patterns).

3. Check other typeck files that match on `Expr` variants: `capability_check.rs`, `effect.rs`, `purity.rs`, `constraints.rs`, `names.rs`, `policy_check.rs`, `instantiate.rs`.

## TDD Steps

1. Test: `let x: Int = 42; x + 1` typechecks as `Int`
2. Test: nested let-bindings typecheck correctly
3. Test: pattern type inference — `let Some { value: x } = expr; ...` extends env with `x: T` where `T` is the field type
4. Test: type error on mismatched pattern — `let x: String = 42; ...` is a type error
5. Run `cargo test -p ash-typeck`

## Estimated Hours

1-2

## Completion Checklist

- [ ] `Expr::Let` typechecks in `check_expr.rs`
- [ ] Type environment extended with pattern bindings
- [ ] All typeck tests pass
- [ ] Other typeck files handle `Expr::Let` (or have a catch-all)
