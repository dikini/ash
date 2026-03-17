# TASK-031: ORIENT/PROPOSE Execution (Deliberative)

## Status: 🟢 Complete

## Description

Implement the execution of ORIENT and PROPOSE workflows for deliberative (analysis/planning) operations.

## Specification Reference

- SPEC-004: Operational Semantics - Section 4.2 Deliberative Layer
- SPEC-001: IR - Workflow::Orient, Workflow::Propose

## Requirements

### ORIENT Execution

```rust
/// Execute an ORIENT workflow
pub async fn execute_orient(
    ctx: &RuntimeContext,
    expr: &Expr,
    pattern: &Pattern,
    continuation: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Evaluate expression
    let value = eval_expr(ctx, expr)
        .map_err(|e| InterpError::EvalError(e.to_string()))?;
    
    // Apply analysis (if any)
    let analyzed = analyze_value(&value)?;
    
    // Record trace event
    let trace_event = TraceEvent::Orientation {
        expr: format!("{:?}", expr),
        input: value.clone(),
        output: analyzed.clone(),
        timestamp: Utc::now(),
    };
    
    let mut new_ctx = ctx.clone();
    new_ctx.provenance.record(trace_event);
    
    // Match pattern and bind
    let match_result = match_pattern(pattern, &analyzed)
        .map_err(|e| InterpError::PatternError(e.to_string()))?;
    
    if !match_result.matched {
        return Err(InterpError::PatternMatchFailed {
            pattern: format!("{:?}", pattern),
            value: format!("{:?}", analyzed),
        });
    }
    
    // Apply bindings
    new_ctx.env = apply_bindings(&new_ctx.env, &match_result.bindings);
    
    // Update effect level
    new_ctx.effect_level = Effect::Deliberative.join(new_ctx.effect_level);
    
    // Execute continuation
    execute_workflow(&new_ctx, continuation).await
}

/// Analyze a value (placeholder for analysis operations)
fn analyze_value(value: &Value) -> Result<Value, InterpError> {
    // This could perform various analyses:
    // - Type analysis
    // - Content analysis
    // - Statistical analysis
    // - etc.
    
    // For now, return as-is
    Ok(value.clone())
}
```

### PROPOSE Execution

```rust
/// Execute a PROPOSE workflow
pub async fn execute_propose(
    ctx: &RuntimeContext,
    action: &ActionRef,
    pattern: &Pattern,
    continuation: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Record proposal (advisory, non-binding)
    let proposal = Proposal {
        action: action.clone(),
        timestamp: Utc::now(),
        context: ctx.env.clone(),
    };
    
    // Record trace event
    let trace_event = TraceEvent::Proposal {
        action: action.clone(),
        proposal: proposal.clone(),
    };
    
    let mut new_ctx = ctx.clone();
    new_ctx.provenance.record(trace_event);
    
    // Create proposal value
    let proposal_value = Value::Record([
        ("action".into(), Value::String(action.name.clone())),
        ("timestamp".into(), Value::String(proposal.timestamp.to_rfc3339().into_boxed_str())),
    ].into_iter().collect());
    
    // Match pattern and bind
    let match_result = match_pattern(pattern, &proposal_value)
        .map_err(|e| InterpError::PatternError(e.to_string()))?;
    
    if !match_result.matched {
        return Err(InterpError::PatternMatchFailed {
            pattern: format!("{:?}", pattern),
            value: format!("{:?}", proposal_value),
        });
    }
    
    // Apply bindings
    new_ctx.env = apply_bindings(&new_ctx.env, &match_result.bindings);
    
    // Update effect level (propose is deliberative)
    new_ctx.effect_level = Effect::Deliberative.join(new_ctx.effect_level);
    
    // Store proposal for later review
    new_ctx.provenance.proposals.push(proposal);
    
    // Execute continuation
    execute_workflow(&new_ctx, continuation).await
}

/// A proposed action (advisory)
#[derive(Debug, Clone)]
pub struct Proposal {
    pub action: ActionRef,
    pub timestamp: DateTime<Utc>,
    pub context: Environment,
}
```

### Analysis Capabilities

```rust
/// Analyze text capability
#[derive(Debug)]
pub struct AnalyzeSentimentCapability;

#[async_trait]
impl CapabilityProvider for AnalyzeSentimentCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "analyze_sentiment".into(),
            effect: Effect::Deliberative,
            parameters: vec![
                ParameterSpec {
                    name: "text".into(),
                    ty: Type::String,
                    required: true,
                },
            ],
            return_type: Type::Record(vec![
                ("sentiment".into(), Type::String),
                ("confidence".into(), Type::Int),
            ]),
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        // Placeholder for sentiment analysis
        Ok(Value::Record([
            ("sentiment".into(), Value::String("neutral".into())),
            ("confidence".into(), Value::Int(80)),
        ].into_iter().collect()))
    }
}

/// Summarize capability
#[derive(Debug)]
pub struct SummarizeCapability;

#[async_trait]
impl CapabilityProvider for SummarizeCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "summarize".into(),
            effect: Effect::Deliberative,
            parameters: vec![
                ParameterSpec {
                    name: "text".into(),
                    ty: Type::String,
                    required: true,
                },
            ],
            return_type: Type::String,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        // Placeholder for summarization
        args.get("text")
            .cloned()
            .ok_or_else(|| CapabilityError::MissingArgument("text".to_string()))
    }
}
```

## TDD Steps

### Step 1: Implement execute_orient

Add ORIENT execution to exec.rs.

### Step 2: Implement execute_propose

Add PROPOSE execution.

### Step 3: Implement Proposal Tracking

Add Proposal struct and tracking.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orient_execution() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let expr = Expr::Literal(Value::Int(42));
        let (value, new_ctx) = execute_orient(
            &ctx,
            &expr,
            &Pattern::Variable("x".into()),
            &Workflow::Done,
        ).await.unwrap();
        
        assert_eq!(value, Value::Int(42));
        assert_eq!(new_ctx.env.get("x"), Some(&Value::Int(42)));
        assert!(new_ctx.effect_level >= Effect::Deliberative);
    }

    #[tokio::test]
    async fn test_propose_execution() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let action = ActionRef {
            name: "send_email".into(),
            args: vec![],
        };
        
        let (value, new_ctx) = execute_propose(
            &ctx,
            &action,
            &Pattern::Variable("proposal".into()),
            &Workflow::Done,
        ).await.unwrap();
        
        assert!(matches!(value, Value::Record(_)));
        assert_eq!(new_ctx.provenance.proposals.len(), 1);
    }
}
```

## Completion Checklist

- [ ] execute_orient function
- [ ] execute_propose function
- [ ] analyze_value placeholder
- [ ] Proposal struct and tracking
- [ ] Trace events for orient and propose
- [ ] Effect level updates
- [ ] Example analysis capabilities
- [ ] Unit tests for ORIENT
- [ ] Unit tests for PROPOSE
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Effect tracking**: Is deliberative effect recorded?
2. **Proposal tracking**: Are proposals stored for review?
3. **Analysis**: Is the analysis hook extensible?

## Estimated Effort

4 hours

## Dependencies

- TASK-026: Runtime context
- TASK-027: Expression evaluator
- TASK-028: Pattern matching

## Blocked By

- TASK-026, TASK-027, TASK-028

## Blocks

- TASK-034: Control flow
