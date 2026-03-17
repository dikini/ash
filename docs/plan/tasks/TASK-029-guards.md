# TASK-029: Guard Evaluation

## Status: 🔴 Not Started

## Description

Implement guard evaluation for authorization checks and conditional execution.

## Specification Reference

- SPEC-004: Operational Semantics - Guard evaluation
- SPEC-001: IR - Section 2.5 Guards

## Requirements

### Guard Evaluation

```rust
/// Evaluate a guard in a context
pub fn eval_guard(ctx: &RuntimeContext, guard: &Guard) -> Result<bool, GuardError> {
    match guard {
        Guard::Always => Ok(true),
        Guard::Never => Ok(false),
        
        Guard::Pred(predicate) => {
            eval_predicate(ctx, predicate)
        }
        
        Guard::And(left, right) => {
            let left_result = eval_guard(ctx, left)?;
            if !left_result {
                Ok(false) // Short-circuit
            } else {
                eval_guard(ctx, right)
            }
        }
        
        Guard::Or(left, right) => {
            let left_result = eval_guard(ctx, left)?;
            if left_result {
                Ok(true) // Short-circuit
            } else {
                eval_guard(ctx, right)
            }
        }
        
        Guard::Not(inner) => {
            let result = eval_guard(ctx, inner)?;
            Ok(!result)
        }
    }
}

/// Evaluate a predicate
fn eval_predicate(ctx: &RuntimeContext, pred: &Predicate) -> Result<bool, GuardError> {
    match pred {
        Predicate::True => Ok(true),
        Predicate::False => Ok(false),
        
        Predicate::Expr(expr) => {
            let val = eval_expr(ctx, expr)
                .map_err(|e| GuardError::EvalError(e.to_string()))?;
            match val {
                Value::Bool(b) => Ok(b),
                _ => Err(GuardError::TypeMismatch {
                    expected: "bool".to_string(),
                    actual: format!("{:?}", val),
                }),
            }
        }
        
        Predicate::Call { name, args } => {
            // Evaluate arguments
            let arg_vals: Result<Vec<_>, _> = args.iter()
                .map(|arg| eval_expr(ctx, arg))
                .collect();
            let arg_vals = arg_vals.map_err(|e| GuardError::EvalError(e.to_string()))?;
            
            // Call predicate function
            call_predicate(ctx, name, &arg_vals)
        }
        
        Predicate::Comparison { left, op, right } => {
            let left_val = eval_expr(ctx, left)
                .map_err(|e| GuardError::EvalError(e.to_string()))?;
            let right_val = eval_expr(ctx, right)
                .map_err(|e| GuardError::EvalError(e.to_string()))?;
            
            eval_comparison(&left_val, op, &right_val)
        }
    }
}

/// Evaluate a comparison
fn eval_comparison(left: &Value, op: &ComparisonOp, right: &Value) -> Result<bool, GuardError> {
    use ComparisonOp::*;
    
    match (left, op, right) {
        (Value::Int(a), Eq, Value::Int(b)) => Ok(a == b),
        (Value::Int(a), Neq, Value::Int(b)) => Ok(a != b),
        (Value::Int(a), Lt, Value::Int(b)) => Ok(a < b),
        (Value::Int(a), Gt, Value::Int(b)) => Ok(a > b),
        (Value::Int(a), Leq, Value::Int(b)) => Ok(a <= b),
        (Value::Int(a), Geq, Value::Int(b)) => Ok(a >= b),
        
        (Value::String(a), Eq, Value::String(b)) => Ok(a == b),
        (Value::String(a), Neq, Value::String(b)) => Ok(a != b),
        
        (a, Eq, b) => Ok(a == b),
        (a, Neq, b) => Ok(a != b),
        
        _ => Err(GuardError::UnsupportedComparison {
            left: format!("{:?}", left),
            op: format!("{:?}", op),
            right: format!("{:?}", right),
        }),
    }
}

/// Call a predicate function
fn call_predicate(
    ctx: &RuntimeContext,
    name: &str,
    args: &[Value],
) -> Result<bool, GuardError> {
    match name {
        "always" => Ok(true),
        "never" => Ok(false),
        "is_admin" => {
            // Check if current user has admin role
            Ok(false) // Placeholder
        }
        "has_permission" => {
            // Check if current user has permission
            Ok(true) // Placeholder
        }
        "in_trash_hours" => {
            // Check if within trash hours
            Ok(true) // Placeholder
        }
        _ => Err(GuardError::UnknownPredicate(name.to_string())),
    }
}
```

### Guard Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum GuardError {
    #[error("Evaluation error: {0}")]
    EvalError(String),
    
    #[error("Type mismatch: expected {expected}, found {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Unknown predicate: {0}")]
    UnknownPredicate(String),
    
    #[error("Unsupported comparison: {left} {op} {right}")]
    UnsupportedComparison { left: String, op: String, right: String },
    
    #[error("Guard evaluation timeout")]
    Timeout,
}
```

### Authorization Guards

```rust
/// Evaluate an authorization guard
pub fn authorize(
    ctx: &RuntimeContext,
    action: &ActionRef,
    guard: &Guard,
) -> AuthorizationResult {
    // First check guard
    match eval_guard(ctx, guard) {
        Ok(true) => {
            // Guard passed, check policy
            check_policy(ctx, action)
        }
        Ok(false) => {
            AuthorizationResult::Denied(DenialReason::GuardFailed)
        }
        Err(e) => {
            AuthorizationResult::Error(e)
        }
    }
}

/// Check policy for action
fn check_policy(ctx: &RuntimeContext, action: &ActionRef) -> AuthorizationResult {
    // This would check policies registered in the context
    AuthorizationResult::Permitted
}

#[derive(Debug, Clone)]
pub enum AuthorizationResult {
    Permitted,
    Denied(DenialReason),
    Error(GuardError),
}

#[derive(Debug, Clone)]
pub enum DenialReason {
    GuardFailed,
    PolicyDenied,
    MissingCapability,
    InsufficientPermissions,
}
```

### Async Guard Evaluation

```rust
/// Async guard evaluation for long-running guards
pub async fn eval_guard_async(
    ctx: &RuntimeContext,
    guard: &Guard,
    timeout: Duration,
) -> Result<bool, GuardError> {
    tokio::time::timeout(timeout, async {
        eval_guard(ctx, guard)
    })
    .await
    .map_err(|_| GuardError::Timeout)?
}
```

## TDD Steps

### Step 1: Implement eval_guard

Create `crates/ash-interp/src/guard.rs` with guard evaluation.

### Step 2: Implement Predicate Evaluation

Add eval_predicate and call_predicate.

### Step 3: Implement Authorization

Add authorize function and AuthorizationResult.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_guard() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        assert_eq!(eval_guard(&ctx, &Guard::Always).unwrap(), true);
    }

    #[test]
    fn test_never_guard() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        assert_eq!(eval_guard(&ctx, &Guard::Never).unwrap(), false);
    }

    #[test]
    fn test_and_short_circuit() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let guard = Guard::And(
            Box::new(Guard::Never),
            Box::new(Guard::Always), // Should not be evaluated
        );
        
        assert_eq!(eval_guard(&ctx, &guard).unwrap(), false);
    }

    #[test]
    fn test_or_short_circuit() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let guard = Guard::Or(
            Box::new(Guard::Always),
            Box::new(Guard::Never), // Should not be evaluated
        );
        
        assert_eq!(eval_guard(&ctx, &guard).unwrap(), true);
    }

    #[test]
    fn test_not_guard() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let guard = Guard::Not(Box::new(Guard::Never));
        
        assert_eq!(eval_guard(&ctx, &guard).unwrap(), true);
    }

    #[test]
    fn test_predicate_expr() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let pred = Predicate::Expr(Expr::Literal(Value::Bool(true)));
        let guard = Guard::Pred(pred);
        
        assert_eq!(eval_guard(&ctx, &guard).unwrap(), true);
    }
}
```

## Completion Checklist

- [ ] eval_guard for all guard types
- [ ] eval_predicate for predicates
- [ ] eval_comparison for comparisons
- [ ] call_predicate for predicate functions
- [ ] authorize for authorization checks
- [ ] GuardError types
- [ ] Async guard evaluation
- [ ] Unit tests for each guard type
- [ ] Short-circuit tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all guard types evaluable?
2. **Short-circuit**: Do AND/OR short-circuit correctly?
3. **Authorization**: Is policy integration correct?

## Estimated Effort

4 hours

## Dependencies

- TASK-027: Expression evaluator (uses eval_expr)

## Blocked By

- TASK-027: Expression evaluator

## Blocks

- TASK-033: Operational execution (uses guards)
