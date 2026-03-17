# TASK-032: DECIDE/CHECK Execution (Evaluative)

## Status: 🟢 Complete

## Description

Implement the execution of DECIDE and CHECK workflows for evaluative (policy/governance) operations.

## Specification Reference

- SPEC-004: Operational Semantics - Section 4.3 Evaluative Layer
- SPEC-001: IR - Workflow::Decide, Workflow::Check

## Requirements

### DECIDE Execution

```rust
/// Execute a DECIDE workflow
pub async fn execute_decide(
    ctx: &RuntimeContext,
    expr: &Expr,
    policy: &str,
    then_branch: &Workflow,
    else_branch: Option<&Workflow>,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Evaluate condition expression
    let value = eval_expr(ctx, expr)
        .map_err(|e| InterpError::EvalError(e.to_string()))?;
    
    // Evaluate policy
    let decision = ctx.evaluate_policy(policy, &value);
    
    // Record trace event
    let trace_event = TraceEvent::Decision {
        policy: policy.into(),
        input: value.clone(),
        decision: decision.clone(),
        timestamp: Utc::now(),
    };
    
    let mut new_ctx = ctx.clone();
    new_ctx.provenance.record(trace_event);
    
    // Update effect level
    new_ctx.effect_level = Effect::Evaluative.join(new_ctx.effect_level);
    
    // Execute based on decision
    match decision {
        Decision::Permit => {
            execute_workflow(&new_ctx, then_branch).await
        }
        Decision::Deny => {
            match else_branch {
                Some(else_workflow) => execute_workflow(&new_ctx, else_workflow).await,
                None => Err(InterpError::PolicyViolation {
                    policy: policy.to_string(),
                    reason: "Denied".to_string(),
                }),
            }
        }
        Decision::RequireApproval { role } => {
            // Log approval requirement
            new_ctx.provenance.record(TraceEvent::ApprovalRequired {
                policy: policy.into(),
                role,
                input: value.clone(),
            });
            
            // For now, treat as deny if no approval mechanism
            Err(InterpError::ApprovalRequired {
                policy: policy.to_string(),
                role: role.to_string(),
            })
        }
        Decision::Escalate => {
            new_ctx.provenance.record(TraceEvent::Escalated {
                policy: policy.into(),
                input: value.clone(),
            });
            
            Err(InterpError::Escalated {
                policy: policy.to_string(),
            })
        }
    }
}
```

### CHECK Execution

```rust
/// Execute a CHECK workflow (obligation verification)
pub async fn execute_check(
    ctx: &RuntimeContext,
    obligation: &Obligation,
    continuation: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Verify the obligation
    let satisfied = verify_obligation(ctx, obligation).await?;
    
    // Record trace event
    let trace_event = TraceEvent::ObligationCheck {
        obligation: obligation.clone(),
        satisfied,
        timestamp: Utc::now(),
    };
    
    let mut new_ctx = ctx.clone();
    new_ctx.provenance.record(trace_event);
    
    // Update effect level
    new_ctx.effect_level = Effect::Evaluative.join(new_ctx.effect_level);
    
    if satisfied {
        // Discharge the obligation
        new_ctx.obligations.retain(|o| o != obligation);
        
        // Execute continuation
        execute_workflow(&new_ctx, continuation).await
    } else {
        Err(InterpError::ObligationViolation {
            obligation: format!("{:?}", obligation),
        })
    }
}

/// Verify an obligation
async fn verify_obligation(
    ctx: &RuntimeContext,
    obligation: &Obligation,
) -> Result<bool, InterpError> {
    match obligation {
        Obligation::Obliged { role, condition } => {
            verify_condition(ctx, role, condition).await
        }
        Obligation::Permitted { .. } => {
            // Permission is already granted if we're here
            Ok(true)
        }
        Obligation::Prohibited { role, action } => {
            // Check if action was performed
            let performed = ctx.provenance.has_action(action);
            Ok(!performed) // Satisfied if NOT performed
        }
        _ => Ok(false),
    }
}

/// Verify a condition
async fn verify_condition(
    ctx: &RuntimeContext,
    _role: &str,
    condition: &Condition,
) -> Result<bool, InterpError> {
    match condition {
        Condition::Expr(expr) => {
            let val = eval_expr(ctx, expr)
                .map_err(|e| InterpError::EvalError(e.to_string()))?;
            match val {
                Value::Bool(b) => Ok(b),
                _ => Ok(false),
            }
        }
        Condition::Check { predicate, args } => {
            // Call predicate
            let arg_vals: Result<Vec<_>, _> = args.iter()
                .map(|arg| eval_expr(ctx, arg))
                .collect();
            let arg_vals = arg_vals.map_err(|e| InterpError::EvalError(e.to_string()))?;
            
            call_check_predicate(ctx, predicate, &arg_vals).await
        }
        _ => Ok(false),
    }
}
```

### Policy Implementations

```rust
/// Simple threshold policy
#[derive(Debug)]
pub struct ThresholdPolicy {
    pub threshold: i64,
    pub field: Box<str>,
}

impl Policy for ThresholdPolicy {
    fn evaluate(&self, input: &Value, _env: &Environment) -> Decision {
        match input {
            Value::Int(n) => {
                if *n >= self.threshold {
                    Decision::Permit
                } else {
                    Decision::Deny
                }
            }
            Value::Record(fields) => {
                match fields.get(&self.field) {
                    Some(Value::Int(n)) if *n >= self.threshold => Decision::Permit,
                    _ => Decision::Deny,
                }
            }
            _ => Decision::Deny,
        }
    }
}

/// Role-based policy
#[derive(Debug)]
pub struct RoleBasedPolicy {
    pub allowed_roles: Vec<Box<str>>,
}

impl Policy for RoleBasedPolicy {
    fn evaluate(&self, input: &Value, _env: &Environment) -> Decision {
        match input {
            Value::String(role) if self.allowed_roles.contains(role) => Decision::Permit,
            Value::Record(fields) => {
                match fields.get("role") {
                    Some(Value::String(role)) if self.allowed_roles.contains(role) => {
                        Decision::Permit
                    }
                    _ => Decision::Deny,
                }
            }
            _ => Decision::Deny,
        }
    }
}
```

## TDD Steps

### Step 1: Implement execute_decide

Add DECIDE execution to exec.rs.

### Step 2: Implement execute_check

Add CHECK execution.

### Step 3: Implement Policy Implementations

Add ThresholdPolicy and RoleBasedPolicy.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decide_permit() {
        let mut policies = PolicyRegistry::new();
        policies.register("allow_all", Arc::new(AllowAllPolicy));
        
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(policies),
        );
        
        let then_workflow = Workflow::Ret { expr: Expr::Literal(Value::Int(1)) };
        
        let (value, _) = execute_decide(
            &ctx,
            &Expr::Literal(Value::Bool(true)),
            "allow_all",
            &then_workflow,
            None,
        ).await.unwrap();
        
        assert_eq!(value, Value::Int(1));
    }

    #[tokio::test]
    async fn test_decide_deny() {
        let mut policies = PolicyRegistry::new();
        policies.register("deny_all", Arc::new(DenyAllPolicy));
        
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(policies),
        );
        
        let result = execute_decide(
            &ctx,
            &Expr::Literal(Value::Bool(true)),
            "deny_all",
            &Workflow::Done,
            None,
        ).await;
        
        assert!(matches!(result, Err(InterpError::PolicyViolation { .. })));
    }

    #[tokio::test]
    async fn test_check_satisfied() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let obligation = Obligation::Obliged {
            role: "admin".into(),
            condition: Condition::Expr(Expr::Literal(Value::Bool(true))),
        };
        
        let (value, _) = execute_check(
            &ctx,
            &obligation,
            &Workflow::Ret { expr: Expr::Literal(Value::Int(1)) },
        ).await.unwrap();
        
        assert_eq!(value, Value::Int(1));
    }
}

#[derive(Debug)]
struct AllowAllPolicy;

impl Policy for AllowAllPolicy {
    fn evaluate(&self, _input: &Value, _env: &Environment) -> Decision {
        Decision::Permit
    }
}

#[derive(Debug)]
struct DenyAllPolicy;

impl Policy for DenyAllPolicy {
    fn evaluate(&self, _input: &Value, _env: &Environment) -> Decision {
        Decision::Deny
    }
}
```

## Completion Checklist

- [ ] execute_decide function
- [ ] execute_check function
- [ ] verify_obligation helper
- [ ] verify_condition helper
- [ ] ThresholdPolicy implementation
- [ ] RoleBasedPolicy implementation
- [ ] Trace events for decisions
- [ ] Unit tests for DECIDE
- [ ] Unit tests for CHECK
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Policy enforcement**: Are decisions enforced correctly?
2. **Obligation tracking**: Are obligations discharged properly?
3. **Error handling**: Are violations reported clearly?

## Estimated Effort

6 hours

## Dependencies

- TASK-026: Runtime context
- TASK-027: Expression evaluator
- TASK-029: Guards

## Blocked By

- TASK-026, TASK-027, TASK-029

## Blocks

- TASK-033: Operational execution
- TASK-034: Control flow
