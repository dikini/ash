# TASK-653: Fix `and`/`or` Short-Circuit Evaluation

## Status: Planned

## Objective

Fix the expression evaluator to short-circuit `and` and `or` binary operations per SPEC-004 §4.6 `EXPR-AND-FALSE`, `EXPR-AND-TRUE`, `EXPR-OR-TRUE`, `EXPR-OR-FALSE`.

## Problem

`eval.rs` currently evaluates both operands eagerly before calling `eval_binary_op`:
```rust
Expr::Binary { op, left, right, .. } => {
    let left_val = eval_expr(left, ctx)?;
    let right_val = eval_expr(right, ctx)?;
    eval_binary_op(*op, left_val, right_val)
}
```

This violates SPEC-004 which requires:
- `EXPR-AND-FALSE`: if left is `false`, don't evaluate right, return `false`
- `EXPR-AND-TRUE`: if left is `true`, evaluate right, return right's value
- `EXPR-OR-TRUE`: if left is `true`, don't evaluate right, return `true`
- `EXPR-OR-FALSE`: if left is `false`, evaluate right, return right's value

## Requirements

1. Add special-case handling for `BinaryOp::And` and `BinaryOp::Or` before the general binary evaluation:
   ```rust
   BinaryOp::And => {
       let left_val = eval_expr(left, ctx)?;
       match left_val {
           Value::Bool(false) => Ok(Value::Bool(false)),
           Value::Bool(true) => eval_expr(right, ctx),
           _ => Err(EvalError::TypeError { ... }),
       }
   }
   BinaryOp::Or => {
       let left_val = eval_expr(left, ctx)?;
       match left_val {
           Value::Bool(true) => Ok(Value::Bool(true)),
           Value::Bool(false) => eval_expr(right, ctx),
           _ => Err(EvalError::TypeError { ... }),
       }
   }
   ```

2. This is independent of TASK-648 through TASK-652 — it can be done in parallel.

## TDD Steps

1. Test: `false && panic "should not evaluate"` returns `false` (not panic)
2. Test: `true || panic "should not evaluate"` returns `true`
3. Test: `true && false` returns `false`
4. Test: `false || true` returns `true`
5. Test: side effects in short-circuited operand are not observed
6. Run `cargo test -p ash-interp`

## Estimated Hours

0.5-1

## Completion Checklist

- [ ] `and` short-circuits on `false`
- [ ] `or` short-circuits on `true`
- [ ] Non-boolean operands produce type errors
- [ ] All interpreter tests pass
