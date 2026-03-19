//! Expression evaluation
//!
//! Evaluates expressions in a runtime context, producing values.

use ash_core::{BinaryOp, Expr, UnaryOp, Value};
use std::collections::HashMap;

use crate::EvalResult;
use crate::context::Context;
use crate::error::EvalError;

/// Evaluate an expression in the given context
///
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `ctx` - The runtime context with variable bindings
///
/// # Returns
/// The evaluated value or an error
///
/// # Examples
/// ```
/// use ash_core::{Expr, Value};
/// use ash_interp::context::Context;
/// use ash_interp::eval::eval_expr;
///
/// let ctx = Context::new();
/// let expr = Expr::Literal(Value::Int(42));
/// let value = eval_expr(&expr, &ctx).unwrap();
/// assert_eq!(value, Value::Int(42));
/// ```
pub fn eval_expr(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),

        Expr::Variable(name) => ctx
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

        Expr::FieldAccess { expr, field } => {
            let value = eval_expr(expr, ctx)?;
            match value {
                Value::Record(mut fields) => {
                    fields
                        .remove(field)
                        .ok_or_else(|| EvalError::FieldNotFound {
                            field: field.clone(),
                            value: Value::Record(fields),
                        })
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", value),
                }),
            }
        }

        Expr::IndexAccess { expr, index } => {
            let collection = eval_expr(expr, ctx)?;
            let idx_val = eval_expr(index, ctx)?;

            match idx_val {
                Value::Int(i) => {
                    let idx = i as usize;
                    match collection {
                        Value::List(list) => {
                            if idx < list.len() {
                                Ok(list[idx].clone())
                            } else {
                                Err(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: list.len(),
                                })
                            }
                        }
                        Value::String(s) => {
                            if let Some(c) = s.chars().nth(idx) {
                                Ok(Value::String(c.to_string()))
                            } else {
                                Err(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: s.len(),
                                })
                            }
                        }
                        _ => Err(EvalError::TypeMismatch {
                            expected: "list or string".to_string(),
                            actual: format!("{:?}", collection),
                        }),
                    }
                }
                _ => Err(EvalError::InvalidIndexType(format!("{:?}", idx_val))),
            }
        }

        Expr::Unary { op, expr } => {
            let value = eval_expr(expr, ctx)?;
            eval_unary_op(*op, value)
        }

        Expr::Binary { op, left, right } => {
            let left_val = eval_expr(left, ctx)?;
            let right_val = eval_expr(right, ctx)?;
            eval_binary_op(*op, left_val, right_val)
        }

        Expr::Call { func, arguments } => {
            let args: Vec<Value> = arguments
                .iter()
                .map(|arg| eval_expr(arg, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            eval_function_call(func, &args)
        }

        Expr::Constructor { name, fields } => {
            // TODO: Implement constructor evaluation in TASK-131
            Err(EvalError::NotImplemented(format!(
                "Constructor {} not yet implemented",
                name
            )))
        }

        Expr::Match { scrutinee, arms } => {
            // TODO: Implement match evaluation in TASK-133
            Err(EvalError::NotImplemented(format!(
                "Match expression with {} arms not yet implemented",
                arms.len()
            )))
        }

        Expr::IfLet { pattern, expr, then_branch, else_branch } => {
            // TODO: Implement if-let evaluation in TASK-133
            Err(EvalError::NotImplemented(
                "If-let expression not yet implemented".to_string()
            ))
        }
    }
}

/// Evaluate a unary operation
fn eval_unary_op(op: UnaryOp, operand: Value) -> EvalResult<Value> {
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
fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
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

        // Logical
        BinaryOp::And => match (&left, &right) {
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l && *r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "and".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Or => match (&left, &right) {
            (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l || *r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "or".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },

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

/// Evaluate a built-in function call
fn eval_function_call(func: &str, args: &[Value]) -> EvalResult<Value> {
    match func {
        // List operations
        "len" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            match &args[0] {
                Value::List(list) => Ok(Value::Int(list.len() as i64)),
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                _ => Err(EvalError::TypeMismatch {
                    expected: "list or string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        "append" => {
            if args.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: args.len(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::List(list), elem) => {
                    let mut new_list = list.clone();
                    new_list.push(elem.clone());
                    Ok(Value::List(new_list))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        "concat" => {
            if args.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: args.len(),
                });
            }
            match (&args[0], &args[1]) {
                (Value::List(l1), Value::List(l2)) => {
                    let mut new_list = l1.clone();
                    new_list.extend(l2.clone());
                    Ok(Value::List(new_list))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "list, list".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }

        // Record operations
        "keys" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            match &args[0] {
                Value::Record(fields) => {
                    let keys: Vec<Value> =
                        fields.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::List(keys))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        "values" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            match &args[0] {
                Value::Record(fields) => {
                    let values: Vec<Value> = fields.values().cloned().collect();
                    Ok(Value::List(values))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        // Type checking
        "is_int" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::Int(_))))
        }

        "is_string" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::String(_))))
        }

        "is_bool" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
        }

        "is_list" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::List(_))))
        }

        "is_record" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::Record(_))))
        }

        "is_null" => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                });
            }
            Ok(Value::Bool(matches!(args[0], Value::Null)))
        }

        // Record constructor
        "record" => {
            let mut fields = HashMap::new();
            // Arguments come in pairs: key, value, key, value, ...
            if !args.len().is_multiple_of(2) {
                return Err(EvalError::ExecutionFailed(
                    "record requires even number of arguments (key, value pairs)".to_string(),
                ));
            }
            for i in (0..args.len()).step_by(2) {
                let key = match &args[i] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::TypeMismatch {
                            expected: "string".to_string(),
                            actual: format!("{:?}", args[i]),
                        });
                    }
                };
                fields.insert(key, args[i + 1].clone());
            }
            Ok(Value::Record(fields))
        }

        // Unknown function
        _ => Err(EvalError::UnknownFunction(func.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_literal() {
        let ctx = Context::new();
        let expr = Expr::Literal(Value::Int(42));
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_variable_found() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(42));
        let expr = Expr::Variable("x".to_string());
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_variable_not_found() {
        let ctx = Context::new();
        let expr = Expr::Variable("x".to_string());
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_field_access() {
        let mut ctx = Context::new();
        let mut record = HashMap::new();
        record.insert("name".to_string(), Value::String("Alice".to_string()));
        ctx.set("person".to_string(), Value::Record(record));

        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Variable("person".to_string())),
            field: "name".to_string(),
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("Alice".to_string())
        );
    }

    #[test]
    fn test_eval_field_access_not_found() {
        let ctx = Context::new();
        let mut record = HashMap::new();
        record.insert("x".to_string(), Value::Int(1));
        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Literal(Value::Record(record))),
            field: "missing".to_string(),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_field_access_not_record() {
        let ctx = Context::new();
        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Literal(Value::Int(42))),
            field: "x".to_string(),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_index_list() {
        let ctx = Context::new();
        let expr = Expr::IndexAccess {
            expr: Box::new(Expr::Literal(Value::List(vec![
                Value::Int(10),
                Value::Int(20),
                Value::Int(30),
            ]))),
            index: Box::new(Expr::Literal(Value::Int(1))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_eval_index_out_of_bounds() {
        let ctx = Context::new();
        let expr = Expr::IndexAccess {
            expr: Box::new(Expr::Literal(Value::List(vec![Value::Int(10)]))),
            index: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_unary_not() {
        let ctx = Context::new();
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_unary_neg() {
        let ctx = Context::new();
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Literal(Value::Int(42))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(-42));
    }

    #[test]
    fn test_eval_binary_arithmetic() {
        let ctx = Context::new();

        // Addition
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(15));

        // Subtraction
        let expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(5));

        // Multiplication
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(50));

        // Division
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_binary_div_by_zero() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(0))),
        };
        assert!(matches!(
            eval_expr(&expr, &ctx),
            Err(EvalError::DivisionByZero)
        ));
    }

    #[test]
    fn test_eval_binary_logical() {
        let ctx = Context::new();

        // AND
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));

        // OR
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_comparison() {
        let ctx = Context::new();

        // Less than
        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        // Greater than
        let expr = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Literal(Value::Int(2))),
            right: Box::new(Expr::Literal(Value::Int(1))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        // Equal
        let expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Value::Int(42))),
            right: Box::new(Expr::Literal(Value::Int(42))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_in_list() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::In,
            left: Box::new(Expr::Literal(Value::Int(2))),
            right: Box::new(Expr::Literal(Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_call_len() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "len".to_string(),
            arguments: vec![Expr::Literal(Value::List(vec![
                Value::Int(1),
                Value::Int(2),
            ]))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_call_append() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "append".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(vec![Value::Int(1)])),
                Expr::Literal(Value::Int(2)),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2)])
        );
    }

    #[test]
    fn test_eval_call_concat() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "concat".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(vec![Value::Int(1)])),
                Expr::Literal(Value::List(vec![Value::Int(2)])),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2)])
        );
    }

    #[test]
    fn test_eval_call_unknown() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "unknown".to_string(),
            arguments: vec![],
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_call_wrong_arity() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "len".to_string(),
            arguments: vec![],
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_nested_expr() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(5));

        // (x + 3) * 2
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Literal(Value::Int(3))),
            }),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(16));
    }

    #[test]
    fn test_eval_string_concat() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::String("hello ".to_string()))),
            right: Box::new(Expr::Literal(Value::String("world".to_string()))),
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_eval_type_checks() {
        let ctx = Context::new();

        let expr = Expr::Call {
            func: "is_int".to_string(),
            arguments: vec![Expr::Literal(Value::Int(42))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        let expr = Expr::Call {
            func: "is_string".to_string(),
            arguments: vec![Expr::Literal(Value::Int(42))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }
}
