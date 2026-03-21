//! Expression evaluation
//!
//! Evaluates expressions in a runtime context, producing values.

use ash_core::{BinaryOp, Expr, UnaryOp, Value, WorkflowId, ast::MatchArm, ast::Pattern};
use ash_core::{ControlLink, Instance, InstanceAddr};
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
                    let removed = fields.remove(field);
                    if removed.is_none() {
                        return Err(EvalError::FieldNotFound {
                            field: field.clone(),
                            value: Value::Record(fields),
                        });
                    }
                    Ok(removed.unwrap())
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
            // Evaluate each field expression and collect into a vector of (name, value) pairs
            let evaluated_fields: Vec<(String, Value)> = fields
                .iter()
                .map(|(field_name, expr)| {
                    eval_expr(expr, ctx).map(|value| (field_name.clone(), value))
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Value::Variant {
                name: name.clone(),
                fields: Box::new(evaluated_fields),
            })
        }

        Expr::Match { scrutinee, arms } => eval_match(scrutinee, arms, ctx),

        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        } => eval_if_let(pattern, expr, then_branch, else_branch, ctx),

        Expr::Spawn {
            workflow_type,
            init: _,
        } => eval_spawn(workflow_type),

        Expr::Split(expr) => eval_split(expr, ctx),
    }
}

/// Generate a fresh instance ID
fn fresh_instance_id() -> WorkflowId {
    WorkflowId::new()
}

/// Evaluate a spawn expression
/// Creates a new Instance value with a fresh address and control link
fn eval_spawn(workflow_type: &str) -> EvalResult<Value> {
    let instance_id = fresh_instance_id();

    let addr = InstanceAddr {
        workflow_type: workflow_type.to_string(),
        instance_id,
    };

    let control = Some(ControlLink { instance_id });

    Ok(Value::Instance(Box::new(Instance { addr, control })))
}

/// Evaluate a split expression
/// Splits an Instance into a tuple (InstanceAddr, Option<ControlLink>)
fn eval_split(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match value {
        Value::Instance(instance) => {
            let addr = Value::InstanceAddr(instance.addr);
            let control = instance.control.map(Value::ControlLink);
            // Return as a tuple: (addr, control)
            Ok(Value::List(Box::new(vec![
                addr,
                control.unwrap_or(Value::Null),
            ])))
        }
        _ => Err(EvalError::TypeMismatch {
            expected: "Instance".to_string(),
            actual: format!("{:?}", value),
        }),
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

/// Evaluate a match expression
///
/// Tries each arm in order, returning the result of the first matching arm.
/// If no arm matches, returns a non-exhaustive match error.
fn eval_match(scrutinee: &Expr, arms: &[MatchArm], ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(scrutinee, ctx)?;

    for arm in arms {
        match crate::pattern::match_pattern(&arm.pattern, &value) {
            Ok(bindings) => {
                // Create a new context with the bindings
                let mut new_ctx = ctx.extend();
                for (name, val) in bindings {
                    new_ctx.set(name, val);
                }
                return eval_expr(&arm.body, &new_ctx);
            }
            Err(_) => {
                // Pattern didn't match, try next arm
                continue;
            }
        }
    }

    // No arm matched
    Err(EvalError::NonExhaustiveMatch {
        value: format!("{:?}", value),
    })
}

/// Evaluate an if-let expression
///
/// If the pattern matches the expression value, evaluates the then branch with bindings.
/// Otherwise evaluates the else branch.
fn eval_if_let(
    pattern: &Pattern,
    expr: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    ctx: &Context,
) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match crate::pattern::match_pattern(pattern, &value) {
        Ok(bindings) => {
            // Pattern matched - evaluate then branch with bindings
            let mut new_ctx = ctx.extend();
            for (name, val) in bindings {
                new_ctx.set(name, val);
            }
            eval_expr(then_branch, &new_ctx)
        }
        Err(_) => {
            // Pattern didn't match - evaluate else branch
            eval_expr(else_branch, ctx)
        }
    }
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
                    let mut new_list = list.to_vec();
                    new_list.push(elem.clone());
                    Ok(Value::List(Box::new(new_list)))
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
                    let mut new_list = l1.to_vec();
                    new_list.extend(l2.iter().cloned());
                    Ok(Value::List(Box::new(new_list)))
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
                    Ok(Value::List(Box::new(keys)))
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
                    Ok(Value::List(Box::new(values)))
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
            Ok(Value::Record(Box::new(fields)))
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
        ctx.set("person".to_string(), Value::Record(Box::new(record)));

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
            expr: Box::new(Expr::Literal(Value::Record(Box::new(record)))),
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
            expr: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(10),
                Value::Int(20),
                Value::Int(30),
            ])))),
            index: Box::new(Expr::Literal(Value::Int(1))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_eval_index_out_of_bounds() {
        let ctx = Context::new();
        let expr = Expr::IndexAccess {
            expr: Box::new(Expr::Literal(Value::List(Box::new(vec![Value::Int(10)])))),
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
            right: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_call_len() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "len".to_string(),
            arguments: vec![Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
            ])))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_call_append() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "append".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1)]))),
                Expr::Literal(Value::Int(2)),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))
        );
    }

    #[test]
    fn test_eval_call_concat() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "concat".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1)]))),
                Expr::Literal(Value::List(Box::new(vec![Value::Int(2)]))),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))
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

    // ============================================================
    // TASK-131: Constructor Evaluation Tests
    // ============================================================

    #[test]
    fn test_eval_constructor_some_with_value() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_none_empty() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "None".to_string(),
            fields: vec![],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }

    #[test]
    fn test_eval_match_wildcard_fallback() {
        let ctx = Context::new();

        // match 2 { 1 => "one", _ => "other" } → "other"
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Value::Int(1)),
                body: Expr::Literal(Value::String("one".to_string())),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Expr::Literal(Value::String("other".to_string())),
            },
        ];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::Int(2))),
            arms,
        };

        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("other".to_string())
        );
    }

    #[test]
    fn test_eval_constructor_ok_with_string() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Ok".to_string(),
            fields: vec![(
                "value".to_string(),
                Expr::Literal(Value::String("hello".to_string())),
            )],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Ok".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::String("hello".to_string())
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_err_with_value() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Err".to_string(),
            fields: vec![(
                "error".to_string(),
                Expr::Literal(Value::String("not found".to_string())),
            )],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Err".to_string(),
                fields: Box::new(vec![(
                    "error".to_string(),
                    Value::String("not found".to_string())
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_nested() {
        let ctx = Context::new();
        // Some { value: Ok { value: 42 } }
        let inner = Expr::Constructor {
            name: "Ok".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };
        let outer = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), inner)],
        };
        let result = eval_expr(&outer, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::Variant {
                        name: "Ok".to_string(),
                        fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
                    }
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_with_variable() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(100));

        let expr = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), Expr::Variable("x".to_string()))],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(100))]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_with_expression_in_field() {
        let ctx = Context::new();
        // Point { x: 1 + 2, y: 3 * 4 }
        let expr = Expr::Constructor {
            name: "Point".to_string(),
            fields: vec![
                (
                    "x".to_string(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Literal(Value::Int(1))),
                        right: Box::new(Expr::Literal(Value::Int(2))),
                    },
                ),
                (
                    "y".to_string(),
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Literal(Value::Int(3))),
                        right: Box::new(Expr::Literal(Value::Int(4))),
                    },
                ),
            ],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Point".to_string(),
                fields: Box::new(vec![
                    ("x".to_string(), Value::Int(3)),
                    ("y".to_string(), Value::Int(12)),
                ]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_multiple_fields() {
        let ctx = Context::new();
        // Person { name: "Alice", age: 30, active: true }
        let expr = Expr::Constructor {
            name: "Person".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Expr::Literal(Value::String("Alice".to_string())),
                ),
                ("age".to_string(), Expr::Literal(Value::Int(30))),
                ("active".to_string(), Expr::Literal(Value::Bool(true))),
            ],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Person".to_string(),
                fields: Box::new(vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                    ("active".to_string(), Value::Bool(true)),
                ]),
            }
        );
    }

    #[test]
    fn test_value_variant_helpers() {
        // Test Value::variant helper
        let v = Value::variant("Some", vec![("value", Value::Int(42))]);
        assert_eq!(
            v,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            }
        );

        // Test Value::unit_variant helper
        let v = Value::unit_variant("None");
        assert_eq!(
            v,
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }

    #[test]
    fn test_eval_match_list_destructure() {
        let ctx = Context::new();

        // match [1, 2, 3] { [a, b, ..] => a + b, _ => 0 } → 3
        let arms = vec![
            MatchArm {
                pattern: Pattern::List(
                    vec![
                        Pattern::Variable("a".to_string()),
                        Pattern::Variable("b".to_string()),
                    ],
                    Some("_".to_string()),
                ),
                body: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("a".to_string())),
                    right: Box::new(Expr::Variable("b".to_string())),
                },
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Expr::Literal(Value::Int(0)),
            },
        ];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))),
            arms,
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_match_tuple_destructure() {
        let ctx = Context::new();

        // match (1, 2) { (x, y) => x + y } → 3
        let arms = vec![MatchArm {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("x".to_string()),
                Pattern::Variable("y".to_string()),
            ]),
            body: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Variable("y".to_string())),
            },
        }];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
            ])))),
            arms,
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_match_option_some_branch_binds_value() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            },
        );

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable("x".to_string()),
                        )]),
                    },
                    body: Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Literal(Value::Int(2))),
                    },
                },
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "None".to_string(),
                        fields: None,
                    },
                    body: Expr::Literal(Value::Int(0)),
                },
            ],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(84));
    }

    #[test]
    fn test_eval_match_option_none_branch_selected() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            },
        );

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable("x".to_string()),
                        )]),
                    },
                    body: Expr::Variable("x".to_string()),
                },
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "None".to_string(),
                        fields: None,
                    },
                    body: Expr::Literal(Value::Int(0)),
                },
            ],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_eval_if_let_option_some_then_branch_binds_value() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(99))]),
            },
        );

        let expr = Expr::IfLet {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Variable("x".to_string()),
                )]),
            },
            expr: Box::new(Expr::Variable("opt".to_string())),
            then_branch: Box::new(Expr::Variable("x".to_string())),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(99));
    }

    // ============================================================
    // TASK-134: Spawn and Split Tests
    // ============================================================

    #[test]
    fn test_eval_spawn_returns_instance() {
        let ctx = Context::new();

        // spawn Worker with { init: 42 }
        let expr = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(42))),
        };

        let result = eval_expr(&expr, &ctx).unwrap();

        // Should return an Instance value
        match result {
            Value::Instance(instance) => {
                assert_eq!(instance.addr.workflow_type, "Worker");
                assert!(instance.control.is_some());
                assert_eq!(
                    instance.control.unwrap().instance_id,
                    instance.addr.instance_id
                );
            }
            _ => panic!("Expected Instance, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_spawn_creates_unique_ids() {
        let ctx = Context::new();

        // spawn two instances
        let expr1 = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(1))),
        };
        let expr2 = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(2))),
        };

        let result1 = eval_expr(&expr1, &ctx).unwrap();
        let result2 = eval_expr(&expr2, &ctx).unwrap();

        let id1 = match &result1 {
            Value::Instance(inst) => inst.addr.instance_id,
            _ => panic!("Expected Instance"),
        };
        let id2 = match &result2 {
            Value::Instance(inst) => inst.addr.instance_id,
            _ => panic!("Expected Instance"),
        };

        // IDs should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_eval_split_returns_tuple() {
        let ctx = Context::new();

        // First spawn an instance
        let spawn_expr = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(42))),
        };

        // Then split it
        let split_expr = Expr::Split(Box::new(spawn_expr));

        let result = eval_expr(&split_expr, &ctx).unwrap();

        // Should return a tuple (InstanceAddr, ControlLink)
        match result {
            Value::List(tuple) => {
                assert_eq!(tuple.len(), 2);
                // First element should be InstanceAddr
                assert!(matches!(tuple[0], Value::InstanceAddr(_)));
                // Second element should be ControlLink
                assert!(matches!(tuple[1], Value::ControlLink(_)));
            }
            _ => panic!("Expected tuple (List), got {:?}", result),
        }
    }

    #[test]
    fn test_eval_split_type_mismatch() {
        let ctx = Context::new();

        // Try to split a non-instance value
        let split_expr = Expr::Split(Box::new(Expr::Literal(Value::Int(42))));

        let result = eval_expr(&split_expr, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_instance_addr_display() {
        let id = WorkflowId::new();
        let addr = InstanceAddr {
            workflow_type: "Worker".to_string(),
            instance_id: id,
        };
        let display = format!("{}", addr);
        assert!(display.starts_with("InstanceAddr<Worker:"));
        assert!(display.ends_with(">"));
    }

    #[test]
    fn test_control_link_display() {
        let link = ControlLink {
            instance_id: WorkflowId::new(),
        };
        let display = format!("{}", link);
        assert!(display.starts_with("ControlLink<"));
        assert!(display.ends_with(">"));
    }

    #[test]
    fn test_instance_display() {
        let id = WorkflowId::new();
        let instance = Instance {
            addr: InstanceAddr {
                workflow_type: "Worker".to_string(),
                instance_id: id,
            },
            control: Some(ControlLink { instance_id: id }),
        };
        let display = format!("{}", instance);
        assert!(display.contains("Instance {"));
        assert!(display.contains("addr:"));
        assert!(display.contains("control: Some(ControlLink"));
    }

    #[test]
    fn test_instance_display_no_control() {
        let instance = Instance {
            addr: InstanceAddr {
                workflow_type: "Worker".to_string(),
                instance_id: WorkflowId::new(),
            },
            control: None,
        };
        let display = format!("{}", instance);
        assert!(display.contains("control: None"));
    }
}
