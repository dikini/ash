//! Guard evaluation for conditional workflow execution
//!
//! Guards are boolean conditions that control whether actions can proceed.

use ash_core::{Expr, Guard, Predicate, Value};

use crate::context::Context;
use crate::error::EvalError;
use crate::eval::eval_expr;

/// Evaluate a guard in the given context
///
/// Returns `Ok(true)` if the guard passes, `Ok(false)` if it fails,
/// or an error if evaluation fails.
///
/// # Examples
/// ```
/// use ash_core::Guard;
/// use ash_interp::context::Context;
/// use ash_interp::guard::eval_guard;
///
/// let ctx = Context::new();
/// let guard = Guard::Always;
/// assert!(eval_guard(&guard, &ctx).unwrap());
/// ```
pub fn eval_guard(guard: &Guard, ctx: &Context) -> Result<bool, EvalError> {
    match guard {
        Guard::Always => Ok(true),
        Guard::Never => Ok(false),

        Guard::Pred(pred) => eval_predicate(pred, ctx),

        Guard::And(left, right) => {
            let left_result = eval_guard(left, ctx)?;
            if !left_result {
                Ok(false) // Short-circuit
            } else {
                eval_guard(right, ctx)
            }
        }

        Guard::Or(left, right) => {
            let left_result = eval_guard(left, ctx)?;
            if left_result {
                Ok(true) // Short-circuit
            } else {
                eval_guard(right, ctx)
            }
        }

        Guard::Not(inner) => {
            let inner_result = eval_guard(inner, ctx)?;
            Ok(!inner_result)
        }
    }
}

/// Evaluate a predicate in the given context
///
/// Predicates are named boolean functions with arguments.
/// Built-in predicates: eq, ne, lt, le, gt, ge, contains
fn eval_predicate(pred: &Predicate, ctx: &Context) -> Result<bool, EvalError> {
    match pred.name.as_str() {
        // Equality comparison
        "eq" => {
            if pred.arguments.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: pred.arguments.len(),
                });
            }
            let left = eval_expr(&pred.arguments[0], ctx)?;
            let right = eval_expr(&pred.arguments[1], ctx)?;
            Ok(left == right)
        }

        // Not equal
        "ne" => {
            if pred.arguments.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: pred.arguments.len(),
                });
            }
            let left = eval_expr(&pred.arguments[0], ctx)?;
            let right = eval_expr(&pred.arguments[1], ctx)?;
            Ok(left != right)
        }

        // Less than
        "lt" => compare_values(&pred.arguments, ctx, |ord| ord == std::cmp::Ordering::Less),

        // Less than or equal
        "le" => compare_values(&pred.arguments, ctx, |ord| {
            ord == std::cmp::Ordering::Less || ord == std::cmp::Ordering::Equal
        }),

        // Greater than
        "gt" => compare_values(&pred.arguments, ctx, |ord| {
            ord == std::cmp::Ordering::Greater
        }),

        // Greater than or equal
        "ge" => compare_values(&pred.arguments, ctx, |ord| {
            ord == std::cmp::Ordering::Greater || ord == std::cmp::Ordering::Equal
        }),

        // Contains - check if collection contains element
        "contains" => {
            if pred.arguments.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: pred.arguments.len(),
                });
            }
            let collection = eval_expr(&pred.arguments[0], ctx)?;
            let element = eval_expr(&pred.arguments[1], ctx)?;

            match collection {
                Value::List(list) => Ok(list.contains(&element)),
                Value::String(s) => match element.as_string() {
                    Some(substr) => Ok(s.contains(substr)),
                    None => Err(EvalError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: format!("{:?}", element),
                    }),
                },
                _ => Err(EvalError::TypeMismatch {
                    expected: "list or string".to_string(),
                    actual: format!("{:?}", collection),
                }),
            }
        }

        // Is null predicate
        "is_null" => {
            if pred.arguments.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: pred.arguments.len(),
                });
            }
            let value = eval_expr(&pred.arguments[0], ctx)?;
            Ok(matches!(value, Value::Null))
        }

        // Is empty predicate
        "is_empty" => {
            if pred.arguments.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: pred.arguments.len(),
                });
            }
            let value = eval_expr(&pred.arguments[0], ctx)?;
            match value {
                Value::List(list) => Ok(list.is_empty()),
                Value::String(s) => Ok(s.is_empty()),
                Value::Record(r) => Ok(r.is_empty()),
                _ => Err(EvalError::TypeMismatch {
                    expected: "list, string, or record".to_string(),
                    actual: format!("{:?}", value),
                }),
            }
        }

        // Unknown predicate
        _ => Err(EvalError::UnknownFunction(pred.name.clone())),
    }
}

/// Helper to compare two values
fn compare_values<F>(args: &[Expr], ctx: &Context, check: F) -> Result<bool, EvalError>
where
    F: Fn(std::cmp::Ordering) -> bool,
{
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
        });
    }

    let left = eval_expr(&args[0], ctx)?;
    let right = eval_expr(&args[1], ctx)?;

    let ordering = compare_values_ord(&left, &right)?;
    Ok(check(ordering))
}

/// Compare two values for ordering
fn compare_values_ord(left: &Value, right: &Value) -> Result<std::cmp::Ordering, EvalError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Expr;

    #[test]
    fn test_guard_always() {
        let ctx = Context::new();
        assert!(eval_guard(&Guard::Always, &ctx).unwrap());
    }

    #[test]
    fn test_guard_never() {
        let ctx = Context::new();
        assert!(!eval_guard(&Guard::Never, &ctx).unwrap());
    }

    #[test]
    fn test_guard_not() {
        let ctx = Context::new();
        let guard = Guard::Not(Box::new(Guard::Always));
        assert!(!eval_guard(&guard, &ctx).unwrap());
    }

    #[test]
    fn test_guard_and() {
        let ctx = Context::new();

        // true && true = true
        let guard = Guard::And(Box::new(Guard::Always), Box::new(Guard::Always));
        assert!(eval_guard(&guard, &ctx).unwrap());

        // true && false = false
        let guard = Guard::And(Box::new(Guard::Always), Box::new(Guard::Never));
        assert!(!eval_guard(&guard, &ctx).unwrap());

        // false && true = false (short-circuit)
        let guard = Guard::And(Box::new(Guard::Never), Box::new(Guard::Always));
        assert!(!eval_guard(&guard, &ctx).unwrap());
    }

    #[test]
    fn test_guard_and_short_circuit() {
        let ctx = Context::new();
        // false && <anything> should short-circuit to false
        // without evaluating the right side
        let guard = Guard::And(Box::new(Guard::Never), Box::new(Guard::Always));
        assert!(!eval_guard(&guard, &ctx).unwrap());
    }

    #[test]
    fn test_guard_or() {
        let ctx = Context::new();

        // true || false = true
        let guard = Guard::Or(Box::new(Guard::Always), Box::new(Guard::Never));
        assert!(eval_guard(&guard, &ctx).unwrap());

        // false || true = true
        let guard = Guard::Or(Box::new(Guard::Never), Box::new(Guard::Always));
        assert!(eval_guard(&guard, &ctx).unwrap());

        // false || false = false
        let guard = Guard::Or(Box::new(Guard::Never), Box::new(Guard::Never));
        assert!(!eval_guard(&guard, &ctx).unwrap());
    }

    #[test]
    fn test_guard_or_short_circuit() {
        let ctx = Context::new();
        // true || <anything> should short-circuit to true
        let guard = Guard::Or(Box::new(Guard::Always), Box::new(Guard::Never));
        assert!(eval_guard(&guard, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_eq() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(42));

        let pred = Predicate {
            name: "eq".to_string(),
            arguments: vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Value::Int(42)),
            ],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "eq".to_string(),
            arguments: vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Value::Int(43)),
            ],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_ne() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "ne".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1)), Expr::Literal(Value::Int(2))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "ne".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1)), Expr::Literal(Value::Int(1))],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_lt() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "lt".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1)), Expr::Literal(Value::Int(2))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "lt".to_string(),
            arguments: vec![Expr::Literal(Value::Int(2)), Expr::Literal(Value::Int(1))],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_le() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "le".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1)), Expr::Literal(Value::Int(1))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_gt() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "gt".to_string(),
            arguments: vec![Expr::Literal(Value::Int(2)), Expr::Literal(Value::Int(1))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_ge() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "ge".to_string(),
            arguments: vec![Expr::Literal(Value::Int(2)), Expr::Literal(Value::Int(2))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_contains_list() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "contains".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
                Expr::Literal(Value::Int(1)),
            ],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "contains".to_string(),
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
                Expr::Literal(Value::Int(3)),
            ],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_contains_string() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "contains".to_string(),
            arguments: vec![
                Expr::Literal(Value::String("hello world".to_string())),
                Expr::Literal(Value::String("world".to_string())),
            ],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_is_null() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "is_null".to_string(),
            arguments: vec![Expr::Literal(Value::Null)],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "is_null".to_string(),
            arguments: vec![Expr::Literal(Value::Int(0))],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_is_empty() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "is_empty".to_string(),
            arguments: vec![Expr::Literal(Value::List(Box::default()))],
        };
        assert!(eval_predicate(&pred, &ctx).unwrap());

        let pred = Predicate {
            name: "is_empty".to_string(),
            arguments: vec![Expr::Literal(Value::List(Box::new(vec![Value::Int(1)])))],
        };
        assert!(!eval_predicate(&pred, &ctx).unwrap());
    }

    #[test]
    fn test_predicate_unknown() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "unknown_pred".to_string(),
            arguments: vec![],
        };
        assert!(eval_predicate(&pred, &ctx).is_err());
    }

    #[test]
    fn test_predicate_wrong_arity() {
        let ctx = Context::new();

        let pred = Predicate {
            name: "eq".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1))],
        };
        assert!(eval_predicate(&pred, &ctx).is_err());
    }

    #[test]
    fn test_complex_guard() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(10));
        ctx.set("y".to_string(), Value::Int(20));

        // (x < y) && (x > 5)
        let guard = Guard::And(
            Box::new(Guard::Pred(Predicate {
                name: "lt".to_string(),
                arguments: vec![
                    Expr::Variable("x".to_string()),
                    Expr::Variable("y".to_string()),
                ],
            })),
            Box::new(Guard::Pred(Predicate {
                name: "gt".to_string(),
                arguments: vec![
                    Expr::Variable("x".to_string()),
                    Expr::Literal(Value::Int(5)),
                ],
            })),
        );

        assert!(eval_guard(&guard, &ctx).unwrap());
    }
}
