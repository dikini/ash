# TASK-027: Expression Evaluator

## Status: 🟢 Complete

## Description

Implement the expression evaluator that computes values from expressions in the runtime context.

## Specification Reference

- SPEC-004: Operational Semantics - Section 5. Auxiliary Functions
- SHARO_CORE_LANGUAGE.md - Section 9. Execution Engine

## Requirements

### Expression Evaluation

```rust
/// Evaluate an expression in a context
pub fn eval_expr(ctx: &RuntimeContext, expr: &Expr) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(val) => Ok(val.clone()),
        
        Expr::Var(name) => {
            ctx.env.get(name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedVariable(name.to_string()))
        }
        
        Expr::Input(name) => {
            // Input references come from special input environment
            ctx.env.get(&format!("${}", name))
                .cloned()
                .ok_or_else(|| EvalError::UndefinedInput(name.to_string()))
        }
        
        Expr::Field { base, field } => {
            let base_val = eval_expr(ctx, base)?;
            match base_val {
                Value::Record(fields) => {
                    fields.get(field)
                        .cloned()
                        .ok_or_else(|| EvalError::MissingField(field.to_string()))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", base_val),
                }),
            }
        }
        
        Expr::Index { base, index } => {
            let base_val = eval_expr(ctx, base)?;
            let index_val = eval_expr(ctx, index)?;
            
            match (base_val, index_val) {
                (Value::List(items), Value::Int(i)) => {
                    let idx = if i < 0 {
                        items.len().saturating_add(i as usize)
                    } else {
                        i as usize
                    };
                    items.get(idx)
                        .cloned()
                        .ok_or_else(|| EvalError::IndexOutOfBounds { index: i, len: items.len() })
                }
                (Value::Record(fields), Value::String(key)) => {
                    fields.get(&key)
                        .cloned()
                        .ok_or_else(|| EvalError::MissingField(key.to_string()))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "list or record".to_string(),
                    actual: format!("cannot index"),
                }),
            }
        }
        
        Expr::BinOp { op, left, right } => {
            let left_val = eval_expr(ctx, left)?;
            let right_val = eval_expr(ctx, right)?;
            eval_binop(op, left_val, right_val)
        }
        
        Expr::UnOp { op, operand } => {
            let val = eval_expr(ctx, operand)?;
            eval_unop(op, val)
        }
        
        Expr::Ternary { condition, then_expr, else_expr } => {
            let cond_val = eval_expr(ctx, condition)?;
            match cond_val {
                Value::Bool(true) => eval_expr(ctx, then_expr),
                Value::Bool(false) => eval_expr(ctx, else_expr),
                _ => Err(EvalError::TypeMismatch {
                    expected: "bool".to_string(),
                    actual: format!("{:?}", cond_val),
                }),
            }
        }
        
        Expr::Call { func, args } => {
            // Evaluate arguments
            let arg_vals: Result<Vec<_>, _> = args.iter()
                .map(|arg| eval_expr(ctx, arg))
                .collect();
            let arg_vals = arg_vals?;
            
            // Look up function
            if let Some(cap) = ctx.get_capability(func) {
                // Convert args to map
                let arg_map: HashMap<_, _> = cap.metadata().parameters.iter()
                    .zip(arg_vals.iter())
                    .map(|(p, v)| (p.name.clone(), v.clone()))
                    .collect();
                
                // This would need to be async in practice
                // For now, we'll use a placeholder
                Ok(Value::Null)
            } else {
                Err(EvalError::UndefinedFunction(func.to_string()))
            }
        }
    }
}
```

### Binary Operations

```rust
fn eval_binop(op: &BinOp, left: Value, right: Value) -> Result<Value, EvalError> {
    use BinOp::*;
    
    match (op, left, right) {
        // Arithmetic
        (Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (Div, Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Int(a / b))
            }
        }
        
        // String concatenation
        (Add, Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b).into_boxed_str()))
        }
        
        // Comparison
        (Eq, a, b) => Ok(Value::Bool(a == b)),
        (Neq, a, b) => Ok(Value::Bool(a != b)),
        (Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (Leq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (Geq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
        
        // Logical
        (And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
        (Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
        
        // Membership
        (In, item, Value::List(items)) => {
            Ok(Value::Bool(items.iter().any(|i| i == &item)))
        }
        (In, item, Value::Record(fields)) => {
            if let Value::String(key) = item {
                Ok(Value::Bool(fields.contains_key(&key)))
            } else {
                Err(EvalError::TypeMismatch {
                    expected: "string key".to_string(),
                    actual: format!("{:?}", item),
                })
            }
        }
        
        _ => Err(EvalError::UnsupportedOperation {
            op: format!("{:?}", op),
            left: format!("{:?}", left),
            right: format!("{:?}", right),
        }),
    }
}
```

### Unary Operations

```rust
fn eval_unop(op: &UnOp, val: Value) -> Result<Value, EvalError> {
    use UnOp::*;
    
    match (op, val) {
        (Neg, Value::Int(n)) => Ok(Value::Int(-n)),
        (Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        (Len, Value::String(s)) => Ok(Value::Int(s.len() as i64)),
        (Len, Value::List(items)) => Ok(Value::Int(items.len() as i64)),
        (Empty, Value::String(s)) => Ok(Value::Bool(s.is_empty())),
        (Empty, Value::List(items)) => Ok(Value::Bool(items.is_empty())),
        (Empty, Value::Record(fields)) => Ok(Value::Bool(fields.is_empty())),
        (Empty, Value::Null) => Ok(Value::Bool(true)),
        
        _ => Err(EvalError::UnsupportedUnaryOp {
            op: format!("{:?}", op),
            operand: format!("{:?}", val),
        }),
    }
}
```

### Evaluation Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    
    #[error("Undefined input: {0}")]
    UndefinedInput(String),
    
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),
    
    #[error("Missing field: {0}")]
    MissingField(String),
    
    #[error("Index out of bounds: index {index}, length {len}")]
    IndexOutOfBounds { index: i64, len: usize },
    
    #[error("Type mismatch: expected {expected}, found {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Unsupported operation: {op} on {left} and {right}")]
    UnsupportedOperation { op: String, left: String, right: String },
    
    #[error("Unsupported unary operation: {op} on {operand}")]
    UnsupportedUnaryOp { op: String, operand: String },
}
```

## TDD Steps

### Step 1: Implement eval_expr

Create `crates/ash-interp/src/eval.rs` with expression evaluation.

### Step 2: Implement Binary Operations

Add eval_binop for all binary operators.

### Step 3: Implement Unary Operations

Add eval_unop for all unary operators.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_literal() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let expr = Expr::Literal(Value::Int(42));
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_variable() {
        let mut ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        ctx.env = ctx.env.bind("x", Value::Int(42));
        let expr = Expr::Var("x".into());
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_arithmetic() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_field_access() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let record = Value::Record(vec![
            ("name".into(), Value::String("test".into())),
        ].into_iter().collect());
        ctx.env = ctx.env.bind("obj", record);
        
        let expr = Expr::Field {
            base: Box::new(Expr::Var("obj".into())),
            field: "name".into(),
        };
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::String("test".into()));
    }

    #[test]
    fn test_eval_comparison() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let expr = Expr::BinOp {
            op: BinOp::Lt,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_ternary() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let expr = Expr::Ternary {
            condition: Box::new(Expr::Literal(Value::Bool(true))),
            then_expr: Box::new(Expr::Literal(Value::Int(1))),
            else_expr: Box::new(Expr::Literal(Value::Int(2))),
        };
        
        assert_eq!(eval_expr(&ctx, &expr).unwrap(), Value::Int(1));
    }
}
```

## Completion Checklist

- [ ] eval_expr for all expression types
- [ ] eval_binop for all binary operators
- [ ] eval_unop for all unary operators
- [ ] EvalError types
- [ ] Unit tests for each expression type
- [ ] Unit tests for operators
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all expression types evaluated?
2. **Type safety**: Are type mismatches caught?
3. **Error handling**: Are errors informative?

## Estimated Effort

6 hours

## Dependencies

- TASK-026: Runtime context (uses RuntimeContext)

## Blocked By

- TASK-026: Runtime context

## Blocks

- TASK-028: Pattern matching (uses expression evaluation)
- TASK-029: Guards (uses expression evaluation)
- All workflow execution tasks
