//! Expression operator evaluation helpers.

use ash_core::{BinaryOp, UnaryOp, Value};

use crate::EvalResult;
use crate::error::EvalError;

/// Evaluate a unary operation
pub(super) fn eval_unary_op(op: UnaryOp, operand: Value) -> EvalResult<Value> {
    match op {
        UnaryOp::Not => match operand {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(EvalError::InvalidUnaryOp {
                op: "not".to_string(),
                operand: format!("{:?}", operand),
            }),
        },
        UnaryOp::Neg => match operand {
            Value::Int(i) => Ok(Value::Int(-i)),
            _ => Err(EvalError::InvalidUnaryOp {
                op: "neg".to_string(),
                operand: format!("{:?}", operand),
            }),
        },
    }
}

/// Evaluate a binary operation
pub(super) fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
    match op {
        // Arithmetic
        BinaryOp::Add => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "add".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Sub => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "sub".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Mul => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "mul".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Div => match (&left, &right) {
            (Value::Int(_), Value::Int(r)) if *r == 0 => Err(EvalError::DivisionByZero),
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l / r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "div".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Mod => match (&left, &right) {
            (Value::Int(_), Value::Int(r)) if *r == 0 => Err(EvalError::DivisionByZero),
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l % r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "mod".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },

        // Logical — NOTE: And/Or are handled with short-circuit evaluation in
        // the Expr::Binary arm of eval_expr (SPEC-004 EXPR-AND-FALSE, EXPR-OR-TRUE).
        // These arms are only reachable if eval_binary_op is called directly.
        BinaryOp::And | BinaryOp::Or => {
            unreachable!(
                "and/or are handled with short-circuit in eval_expr; \
                 eval_binary_op should never be called for {:?}",
                op
            )
        }

        // Comparison
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => eval_comparison(left, right, |o| o == std::cmp::Ordering::Less),
        BinaryOp::Gt => eval_comparison(left, right, |o| o == std::cmp::Ordering::Greater),
        BinaryOp::Le => eval_comparison(left, right, |o| {
            o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal
        }),
        BinaryOp::Ge => eval_comparison(left, right, |o| {
            o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal
        }),

        // Membership
        BinaryOp::In => match right {
            Value::List(list) => Ok(Value::Bool(list.contains(&left))),
            Value::String(s) => match left.as_string() {
                Some(substr) => Ok(Value::Bool(s.contains(substr))),
                None => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", left),
                }),
            },
            _ => Err(EvalError::InvalidBinaryOp {
                op: "in".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Pipe => Err(EvalError::InvalidBinaryOp {
            op: "pipe".to_string(),
            left: format!("{:?}", left),
            right: format!("{:?}", right),
        }),
    }
}

/// Helper to evaluate comparison operations
fn eval_comparison<F>(left: Value, right: Value, check: F) -> EvalResult<Value>
where
    F: Fn(std::cmp::Ordering) -> bool,
{
    let ordering = compare_values(&left, &right)?;
    Ok(Value::Bool(check(ordering)))
}

/// Compare two values for ordering
fn compare_values(left: &Value, right: &Value) -> EvalResult<std::cmp::Ordering> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(l.cmp(r)),
        (Value::String(l), Value::String(r)) => Ok(l.cmp(r)),
        (Value::Bool(l), Value::Bool(r)) => Ok(l.cmp(r)),
        (Value::Time(l), Value::Time(r)) => Ok(l.cmp(r)),
        _ => Err(EvalError::InvalidBinaryOp {
            op: "comparison".to_string(),
            left: format!("{:?}", left),
            right: format!("{:?}", right),
        }),
    }
}
