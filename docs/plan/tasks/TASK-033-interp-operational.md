# TASK-033: ACT/OBLIG Execution (Operational)

## Status: ✅ Complete

## Description

Implement the execution of ACT and OBLIG workflows for operational (side-effecting) operations.

## Specification Reference

- SPEC-004: Operational Semantics - Section 4.4 Operational Layer
- SPEC-001: IR - Workflow::Act, Workflow::Oblig

## Requirements

### ACT Execution

```rust
/// Execute an ACT workflow
pub async fn execute_act(
    ctx: &RuntimeContext,
    action: &ActionRef,
    guard: &Guard,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Check authorization
    let auth_result = authorize(ctx, action, guard)?;
    
    match auth_result {
        AuthorizationResult::Permitted => {
            // Look up capability provider
            let provider = ctx.get_capability(&action.name)
                .ok_or_else(|| InterpError::UndefinedCapability(action.name.to_string()))?;
            
            // Prepare arguments
            let args = prepare_action_args(ctx, &action.args)?;
            
            // Execute capability
            let value = provider.execute(args).await
                .map_err(|e| InterpError::CapabilityError(e.to_string()))?;
            
            // Record trace event
            let trace_event = TraceEvent::Action {
                action: action.clone(),
                value: value.clone(),
                guard: guard.clone(),
                timestamp: Utc::now(),
            };
            
            let mut new_ctx = ctx.clone();
            new_ctx.provenance.record(trace_event);
            
            // Update effect level
            new_ctx.effect_level = Effect::Operational;
            
            Ok((value, new_ctx))
        }
        AuthorizationResult::Denied(reason) => {
            Err(InterpError::AuthorizationDenied {
                action: action.name.to_string(),
                reason: format!("{:?}", reason),
            })
        }
        AuthorizationResult::Error(e) => {
            Err(InterpError::AuthorizationError(e.to_string()))
        }
    }
}

/// Prepare arguments for action execution
fn prepare_action_args(
    ctx: &RuntimeContext,
    args: &[(Box<str>, Expr)],
) -> Result<HashMap<Box<str>, Value>, InterpError> {
    let mut result = HashMap::new();
    
    for (name, expr) in args {
        let value = eval_expr(ctx, expr)
            .map_err(|e| InterpError::EvalError(e.to_string()))?;
        result.insert(name.clone(), value);
    }
    
    Ok(result)
}

/// Authorization check
fn authorize(
    ctx: &RuntimeContext,
    action: &ActionRef,
    guard: &Guard,
) -> Result<AuthorizationResult, InterpError> {
    // Evaluate guard
    match eval_guard(ctx, guard) {
        Ok(true) => {
            // Guard passed, additional policy checks could go here
            Ok(AuthorizationResult::Permitted)
        }
        Ok(false) => {
            Ok(AuthorizationResult::Denied(DenialReason::GuardFailed))
        }
        Err(e) => {
            Ok(AuthorizationResult::Error(e))
        }
    }
}
```

### OBLIG Execution

```rust
/// Execute an OBLIG workflow
pub async fn execute_oblig(
    ctx: &RuntimeContext,
    role: &Role,
    workflow: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Incur obligations for the role
    let mut new_ctx = ctx.clone();
    
    for obligation in &role.obligations {
        let obl = Obligation::Obliged {
            role: role.name.clone(),
            condition: obligation.condition.clone(),
        };
        new_ctx.obligations.push(obl);
    }
    
    // Record trace event
    let trace_event = TraceEvent::ObligationIncurred {
        role: role.name.clone(),
        obligations: role.obligations.clone(),
        timestamp: Utc::now(),
    };
    new_ctx.provenance.record(trace_event);
    
    // Update effect level
    new_ctx.effect_level = Effect::Evaluative.join(new_ctx.effect_level);
    
    // Execute scoped workflow
    execute_workflow(&new_ctx, workflow).await
}
```

### Operational Capabilities

```rust
/// Write file capability
#[derive(Debug)]
pub struct WriteFileCapability;

#[async_trait]
impl CapabilityProvider for WriteFileCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "write_file".into(),
            effect: Effect::Operational,
            parameters: vec![
                ParameterSpec {
                    name: "path".into(),
                    ty: Type::String,
                    required: true,
                },
                ParameterSpec {
                    name: "content".into(),
                    ty: Type::String,
                    required: true,
                },
            ],
            return_type: Type::Null,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let path = args.get("path")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("path".to_string()))?;
        
        let content = args.get("content")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("content".to_string()))?;
        
        tokio::fs::write(path, content).await
            .map_err(|e| CapabilityError::IoError(e.to_string()))?;
        
        Ok(Value::Null)
    }
}

/// Send email capability
#[derive(Debug)]
pub struct SendEmailCapability;

#[async_trait]
impl CapabilityProvider for SendEmailCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "send_email".into(),
            effect: Effect::Operational,
            parameters: vec![
                ParameterSpec {
                    name: "to".into(),
                    ty: Type::String,
                    required: true,
                },
                ParameterSpec {
                    name: "subject".into(),
                    ty: Type::String,
                    required: true,
                },
                ParameterSpec {
                    name: "body".into(),
                    ty: Type::String,
                    required: true,
                },
            ],
            return_type: Type::Null,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        // Placeholder for email sending
        // In real implementation, use an email library
        let to = args.get("to")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("to".to_string()))?;
        
        tracing::info!("Sending email to: {}", to);
        
        Ok(Value::Null)
    }
}

/// HTTP POST capability
#[derive(Debug)]
pub struct HttpPostCapability;

#[async_trait]
impl CapabilityProvider for HttpPostCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "http_post".into(),
            effect: Effect::Operational,
            parameters: vec![
                ParameterSpec {
                    name: "url".into(),
                    ty: Type::String,
                    required: true,
                },
                ParameterSpec {
                    name: "body".into(),
                    ty: Type::String,
                    required: false,
                },
            ],
            return_type: Type::String,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let url = args.get("url")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("url".to_string()))?;
        
        tracing::info!("POST to: {}", url);
        
        Ok(Value::String("response".to_string().into_boxed_str()))
    }
}
```

## TDD Steps

### Step 1: Implement execute_act

Add ACT execution to exec.rs.

### Step 2: Implement execute_oblig

Add OBLIG execution.

### Step 3: Implement Operational Capabilities

Add WriteFileCapability, SendEmailCapability, HttpPostCapability.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_act_with_guard() {
        let mut caps = CapabilityRegistry::new();
        caps.register("test_act", Arc::new(TestActionCapability));
        
        let ctx = RuntimeContext::new(
            Arc::new(caps),
            Arc::new(PolicyRegistry::new()),
        );
        
        let action = ActionRef {
            name: "test_act".into(),
            args: vec![],
        };
        
        let (value, new_ctx) = execute_act(
            &ctx,
            &action,
            &Guard::Always,
        ).await.unwrap();
        
        assert_eq!(value, Value::String("acted".into()));
        assert_eq!(new_ctx.effect_level, Effect::Operational);
    }

    #[tokio::test]
    async fn test_act_guard_fail() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let action = ActionRef {
            name: "test".into(),
            args: vec![],
        };
        
        let result = execute_act(
            &ctx,
            &action,
            &Guard::Never,
        ).await;
        
        assert!(matches!(result, Err(InterpError::AuthorizationDenied { .. })));
    }
}

#[derive(Debug)]
struct TestActionCapability;

#[async_trait]
impl CapabilityProvider for TestActionCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "test_act".into(),
            effect: Effect::Operational,
            parameters: vec![],
            return_type: Type::String,
        }
    }
    
    async fn execute(&self, _args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        Ok(Value::String("acted".into()))
    }
}
```

## Completion Checklist

- [ ] execute_act function
- [ ] execute_oblig function
- [ ] authorize helper
- [ ] WriteFileCapability
- [ ] SendEmailCapability
- [ ] HttpPostCapability
- [ ] Trace events for actions
- [ ] Unit tests for ACT
- [ ] Unit tests for OBLIG
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Authorization**: Are guards checked before execution?
2. **Effect tracking**: Is operational effect recorded?
3. **Safety**: Are side effects properly isolated?

## Estimated Effort

6 hours

## Dependencies

- TASK-026: Runtime context
- TASK-029: Guards

## Blocked By

- TASK-026, TASK-029

## Blocks

- TASK-034: Control flow
