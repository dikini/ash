# TASK-030: OBSERVE Execution (Epistemic)

## Status: ✅ Complete

## Description

Implement the execution of OBSERVE workflows for epistemic (read-only) operations.

## Specification Reference

- SPEC-004: Operational Semantics - Section 4.1 Epistemic Layer
- SPEC-001: IR - Workflow::Observe

## Requirements

### OBSERVE Execution

```rust
/// Execute an OBSERVE workflow
pub async fn execute_observe(
    ctx: &RuntimeContext,
    capability: &Capability,
    pattern: &Pattern,
    continuation: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Check effect level
    if ctx.effect_level > Effect::Epistemic {
        return Err(InterpError::EffectViolation {
            expected: Effect::Epistemic,
            actual: ctx.effect_level,
        });
    }
    
    // Look up capability provider
    let provider = ctx.get_capability(&capability.name)
        .ok_or_else(|| InterpError::UndefinedCapability(capability.name.to_string()))?;
    
    // Prepare arguments
    let args = prepare_capability_args(ctx, &capability.args)?;
    
    // Execute capability (async)
    let value = provider.execute(args).await
        .map_err(|e| InterpError::CapabilityError(e.to_string()))?;
    
    // Record trace event
    let trace_event = TraceEvent::Observation {
        capability: capability.name.clone(),
        value: value.clone(),
        timestamp: Utc::now(),
    };
    
    let mut new_ctx = ctx.clone();
    new_ctx.provenance.record(trace_event);
    
    // Match pattern and bind
    let match_result = match_pattern(pattern, &value)
        .map_err(|e| InterpError::PatternError(e.to_string()))?;
    
    if !match_result.matched {
        return Err(InterpError::PatternMatchFailed {
            pattern: format!("{:?}", pattern),
            value: format!("{:?}", value),
        });
    }
    
    // Apply bindings
    new_ctx.env = apply_bindings(&new_ctx.env, &match_result.bindings);
    
    // Update effect level
    new_ctx.effect_level = Effect::Epistemic;
    
    // Execute continuation
    execute_workflow(&new_ctx, continuation).await
}

/// Prepare arguments for capability execution
fn prepare_capability_args(
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
```

### Trace Event Recording

```rust
/// Trace event for observation
#[derive(Debug, Clone)]
pub enum TraceEvent {
    Observation {
        capability: Box<str>,
        value: Value,
        timestamp: DateTime<Utc>,
    },
    // ... other event types
}

impl Provenance {
    pub fn record(&mut self, event: TraceEvent) {
        self.trace.push(event);
    }
}
```

### Capability Provider Examples

```rust
/// Read file capability
#[derive(Debug)]
pub struct ReadFileCapability;

#[async_trait]
impl CapabilityProvider for ReadFileCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "read_file".into(),
            effect: Effect::Epistemic,
            parameters: vec![
                ParameterSpec {
                    name: "path".into(),
                    ty: Type::String,
                    required: true,
                },
            ],
            return_type: Type::String,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let path = args.get("path")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("path".to_string()))?;
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| CapabilityError::IoError(e.to_string()))?;
        
        Ok(Value::String(content.into_boxed_str()))
    }
}

/// HTTP GET capability
#[derive(Debug)]
pub struct HttpGetCapability;

#[async_trait]
impl CapabilityProvider for HttpGetCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "http_get".into(),
            effect: Effect::Epistemic,
            parameters: vec![
                ParameterSpec {
                    name: "url".into(),
                    ty: Type::String,
                    required: true,
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
        
        // Would use reqwest in real implementation
        // For now, return placeholder
        Ok(Value::String("response".to_string().into_boxed_str()))
    }
}
```

## TDD Steps

### Step 1: Implement execute_observe

Create `crates/ash-interp/src/exec.rs` with OBSERVE execution.

### Step 2: Implement Trace Recording

Add trace event recording to Provenance.

### Step 3: Implement Example Capabilities

Add ReadFileCapability and HttpGetCapability.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observe_execution() {
        let mut caps = CapabilityRegistry::new();
        caps.register("test_cap", Arc::new(TestCapability));
        
        let ctx = RuntimeContext::new(
            Arc::new(caps),
            Arc::new(PolicyRegistry::new()),
        );
        
        let capability = Capability {
            name: "test_cap".into(),
            args: vec![],
        };
        
        let (value, new_ctx) = execute_observe(
            &ctx,
            &capability,
            &Pattern::Variable("x".into()),
            &Workflow::Done,
        ).await.unwrap();
        
        assert_eq!(value, Value::String("test".into()));
        assert_eq!(new_ctx.env.get("x"), Some(&Value::String("test".into())));
    }

    #[tokio::test]
    async fn test_observe_effect_level() {
        let mut ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        ctx.effect_level = Effect::Operational;
        
        let capability = Capability {
            name: "test".into(),
            args: vec![],
        };
        
        let result = execute_observe(
            &ctx,
            &capability,
            &Pattern::Wildcard,
            &Workflow::Done,
        ).await;
        
        assert!(matches!(result, Err(InterpError::EffectViolation { .. })));
    }
}

#[derive(Debug)]
struct TestCapability;

#[async_trait]
impl CapabilityProvider for TestCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "test_cap".into(),
            effect: Effect::Epistemic,
            parameters: vec![],
            return_type: Type::String,
        }
    }
    
    async fn execute(&self, _args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        Ok(Value::String("test".into()))
    }
}
```

## Completion Checklist

- [ ] execute_observe function
- [ ] Effect level checking
- [ ] Capability lookup
- [ ] Argument preparation
- [ ] Trace event recording
- [ ] Pattern matching and binding
- [ ] Example capability providers
- [ ] Unit tests for OBSERVE
- [ ] Effect violation tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Effect checking**: Is epistemic level enforced?
2. **Error handling**: Are capability errors handled?
3. **Tracing**: Are observations recorded?

## Estimated Effort

4 hours

## Dependencies

- TASK-026: Runtime context
- TASK-028: Pattern matching

## Blocked By

- TASK-026, TASK-028

## Blocks

- TASK-034: Control flow (uses OBSERVE)
