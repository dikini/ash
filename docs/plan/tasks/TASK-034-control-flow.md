# TASK-034: Control Flow Execution

## Status: ✅ Complete

## Description

Implement the execution of control flow constructs: SEQ, PAR, IF, LET, FOREACH, and other control flow workflows.

## Specification Reference

- SPEC-004: Operational Semantics - Section 4.5 Control Flow
- SPEC-001: IR - Control flow variants

## Requirements

### Main Workflow Executor

```rust
/// Execute any workflow
pub async fn execute_workflow(
    ctx: &RuntimeContext,
    workflow: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    match workflow {
        Workflow::Observe { capability, pattern, continuation } => {
            execute_observe(ctx, capability, pattern, continuation).await
        }
        Workflow::Orient { expr, pattern, continuation } => {
            execute_orient(ctx, expr, pattern, continuation).await
        }
        Workflow::Propose { action, pattern, continuation } => {
            execute_propose(ctx, action, pattern, continuation).await
        }
        Workflow::Decide { expr, policy, continuation } => {
            execute_decide(ctx, expr, policy, continuation, None).await
        }
        Workflow::Act { action, guard, .. } => {
            execute_act(ctx, action, guard).await
        }
        Workflow::Let { pattern, expr, continuation } => {
            execute_let(ctx, pattern, expr, continuation.as_deref()).await
        }
        Workflow::If { condition, then_branch, else_branch } => {
            execute_if(ctx, condition, then_branch, else_branch.as_deref()).await
        }
        Workflow::Seq { first, second } => {
            execute_seq(ctx, first, second).await
        }
        Workflow::Par { workflows } => {
            execute_par(ctx, workflows).await
        }
        Workflow::ForEach { pattern, collection, body } => {
            execute_foreach(ctx, pattern, collection, body).await
        }
        Workflow::With { capability, workflow } => {
            execute_with(ctx, capability, workflow).await
        }
        Workflow::Ret { expr } => {
            let val = eval_expr(ctx, expr)
                .map_err(|e| InterpError::EvalError(e.to_string()))?;
            Ok((val, ctx.clone()))
        }
        Workflow::Done => {
            Ok((Value::Null, ctx.clone()))
        }
        _ => Err(InterpError::UnsupportedWorkflow(format!("{:?}", workflow))),
    }
}
```

### Control Flow Implementations

```rust
/// Execute LET
pub async fn execute_let(
    ctx: &RuntimeContext,
    pattern: &Pattern,
    expr: &Expr,
    continuation: Option<&Workflow>,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Evaluate expression
    let value = eval_expr(ctx, expr)
        .map_err(|e| InterpError::EvalError(e.to_string()))?;
    
    // Match pattern
    let match_result = match_pattern(pattern, &value)
        .map_err(|e| InterpError::PatternError(e.to_string()))?;
    
    if !match_result.matched {
        return Err(InterpError::PatternMatchFailed {
            pattern: format!("{:?}", pattern),
            value: format!("{:?}", value),
        });
    }
    
    // Apply bindings
    let mut new_ctx = ctx.clone();
    new_ctx.env = apply_bindings(&new_ctx.env, &match_result.bindings);
    
    // Execute continuation
    match continuation {
        Some(cont) => execute_workflow(&new_ctx, cont).await,
        None => Ok((value, new_ctx)),
    }
}

/// Execute IF
pub async fn execute_if(
    ctx: &RuntimeContext,
    condition: &Expr,
    then_branch: &Workflow,
    else_branch: Option<&Workflow>,
) -> Result<(Value, RuntimeContext), InterpError> {
    let cond_val = eval_expr(ctx, condition)
        .map_err(|e| InterpError::EvalError(e.to_string()))?;
    
    match cond_val {
        Value::Bool(true) => execute_workflow(ctx, then_branch).await,
        Value::Bool(false) => {
            match else_branch {
                Some(else_workflow) => execute_workflow(ctx, else_workflow).await,
                None => Ok((Value::Null, ctx.clone())),
            }
        }
        _ => Err(InterpError::TypeMismatch {
            expected: "bool".to_string(),
            actual: format!("{:?}", cond_val),
        }),
    }
}

/// Execute SEQ (sequential composition)
pub async fn execute_seq(
    ctx: &RuntimeContext,
    first: &Workflow,
    second: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Execute first
    let (_, ctx_after_first) = execute_workflow(ctx, first).await?;
    
    // Execute second with updated context
    execute_workflow(&ctx_after_first, second).await
}

/// Execute PAR (parallel composition)
pub async fn execute_par(
    ctx: &RuntimeContext,
    workflows: &[Workflow],
) -> Result<(Value, RuntimeContext), InterpError> {
    // Fork context for each branch
    let branches: Vec<_> = workflows.iter()
        .map(|wf| {
            let forked_ctx = ctx.fork();
            execute_workflow(forked_ctx, wf)
        })
        .collect();
    
    // Execute all branches concurrently
    let results: Vec<_> = futures::future::join_all(branches).await;
    
    // Collect results
    let mut values = Vec::with_capacity(results.len());
    let mut contexts = Vec::with_capacity(results.len());
    
    for result in results {
        let (val, branch_ctx) = result?;
        values.push(val);
        contexts.push(branch_ctx);
    }
    
    // Merge contexts
    let merged_ctx = ctx.merge(&contexts);
    
    // Return list of values
    Ok((Value::List(values.into()), merged_ctx))
}

/// Execute FOREACH
pub async fn execute_foreach(
    ctx: &RuntimeContext,
    pattern: &Pattern,
    collection: &Expr,
    body: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Evaluate collection
    let coll_val = eval_expr(ctx, collection)
        .map_err(|e| InterpError::EvalError(e.to_string()))?;
    
    let items = match &coll_val {
        Value::List(items) => items.as_ref(),
        _ => return Err(InterpError::TypeMismatch {
            expected: "list".to_string(),
            actual: format!("{:?}", coll_val),
        }),
    };
    
    let mut results = Vec::with_capacity(items.len());
    let mut current_ctx = ctx.clone();
    
    for item in items.iter() {
        // Match pattern
        let match_result = match_pattern(pattern, item)
            .map_err(|e| InterpError::PatternError(e.to_string()))?;
        
        if !match_result.matched {
            return Err(InterpError::PatternMatchFailed {
                pattern: format!("{:?}", pattern),
                value: format!("{:?}", item),
            });
        }
        
        // Apply bindings and execute body
        let branch_ctx = apply_bindings(&current_ctx.env, &match_result.bindings);
        current_ctx.env = branch_ctx;
        
        let (val, new_ctx) = execute_workflow(&current_ctx, body).await?;
        results.push(val);
        current_ctx = new_ctx;
    }
    
    Ok((Value::List(results.into()), current_ctx))
}

/// Execute WITH (capability scoping)
pub async fn execute_with(
    ctx: &RuntimeContext,
    capability: &Capability,
    workflow: &Workflow,
) -> Result<(Value, RuntimeContext), InterpError> {
    // Add capability to context
    let mut new_ctx = ctx.clone();
    // In real implementation, would add to available capabilities
    
    // Execute workflow with scoped capability
    execute_workflow(&new_ctx, workflow).await
}
```

### InterpError Extensions

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum InterpError {
    // ... existing errors
    
    #[error("Unsupported workflow: {0}")]
    UnsupportedWorkflow(String),
    
    #[error("Recursion limit exceeded")]
    RecursionLimitExceeded,
    
    #[error("Execution timeout")]
    Timeout,
    
    #[error("Concurrent execution failed: {0}")]
    ConcurrentFailure(String),
}
```

## TDD Steps

### Step 1: Implement execute_workflow

Create main dispatcher in exec.rs.

### Step 2: Implement Control Flow

Add execute_let, execute_if, execute_seq, execute_par, execute_foreach.

### Step 3: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_seq_execution() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let seq = Workflow::Seq {
            first: Box::new(Workflow::Let {
                pattern: Pattern::Variable("x".into()),
                expr: Expr::Literal(Value::Int(1)),
                continuation: Some(Box::new(Workflow::Done)),
            }),
            second: Box::new(Workflow::Ret {
                expr: Expr::Var("x".into()),
            }),
        };
        
        let (value, _) = execute_workflow(&ctx, &seq).await.unwrap();
        assert_eq!(value, Value::Int(1));
    }

    #[tokio::test]
    async fn test_if_true() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let if_wf = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Some(Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            })),
        };
        
        let (value, _) = execute_workflow(&ctx, &if_wf).await.unwrap();
        assert_eq!(value, Value::Int(1));
    }

    #[tokio::test]
    async fn test_foreach() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let foreach = Workflow::ForEach {
            pattern: Pattern::Variable("x".into()),
            collection: Expr::Literal(Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ].into())),
            body: Box::new(Workflow::Ret {
                expr: Expr::Var("x".into()),
            }),
        };
        
        let (value, _) = execute_workflow(&ctx, &foreach).await.unwrap();
        
        match value {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[tokio::test]
    async fn test_parallel() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let par = Workflow::Par {
            workflows: vec![
                Workflow::Ret { expr: Expr::Literal(Value::Int(1)) },
                Workflow::Ret { expr: Expr::Literal(Value::Int(2)) },
                Workflow::Ret { expr: Expr::Literal(Value::Int(3)) },
            ],
        };
        
        let (value, _) = execute_workflow(&ctx, &par).await.unwrap();
        
        match value {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }
}
```

## Completion Checklist

- [ ] execute_workflow dispatcher
- [ ] execute_let
- [ ] execute_if
- [ ] execute_seq
- [ ] execute_par
- [ ] execute_foreach
- [ ] execute_with
- [ ] InterpError extensions
- [ ] Unit tests for each control flow
- [ ] Integration tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all control flow constructs implemented?
2. **Parallel safety**: Is context merging correct?
3. **Error handling**: Are errors propagated correctly?

## Estimated Effort

6 hours

## Dependencies

- All previous interpreter tasks

## Blocked By

- TASK-030 through TASK-033

## Blocks

- TASK-036: Policy runtime
- TASK-037: Async runtime
