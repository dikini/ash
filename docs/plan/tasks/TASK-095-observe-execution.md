# TASK-095: Observe Execution

## Status: 🔴 Not Started

## Description

Implement execution of observe and changed constructs.

## Specification Reference

- SPEC-014: Behaviours - Section 4 Semantics

## Requirements

### Functional Requirements

1. Execute `observe` by sampling behaviour provider
2. Apply constraints during sampling
3. Bind result to pattern
4. Execute `changed` check
5. Handle unavailable behaviours

### Property Requirements

```rust
// Observe samples and binds
let ctx = execute_observe(observe, ctx, behaviour_ctx).await.unwrap();
assert!(ctx.get("t").is_some());

// Changed reports correctly
let changed = execute_changed(changed, behaviour_ctx).await.unwrap();
assert_eq!(changed, Value::Bool(true_or_false));

// Missing provider errors
let result = execute_observe(observe, ctx, empty_behaviour_ctx).await;
assert!(result.is_err());
```

## TDD Steps

### Step 1: Write Tests (Red)

Create tests in `crates/ash-interp/src/execute_observe.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_observe_simple() {
        let mut behaviour_ctx = BehaviourContext::new();
        behaviour_ctx.register(Box::new(
            MockBehaviourProvider::new("sensor", "temp")
                .with_value(Value::Int(42))
        ));
        
        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Variable("t".into()),
        };
        
        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx).await.unwrap();
        
        assert_eq!(new_ctx.get("t"), Some(&Value::Int(42)));
    }

    #[tokio::test]
    async fn test_execute_observe_with_constraints() {
        let mut behaviour_ctx = BehaviourContext::new();
        behaviour_ctx.register(Box::new(
            MockBehaviourProvider::new("market", "stock")
                .with_value(Value::Record(hashmap! {
                    "price".into() => Value::Int(150),
                    "symbol".into() => Value::String("AAPL".into()),
                }))
        ));
        
        let observe = Observe {
            capability: "market".into(),
            channel: "stock".into(),
            constraints: vec![Constraint::new("symbol", Value::String("AAPL".into()))],
            pattern: Pattern::Variable("stock".into()),
        };
        
        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx).await.unwrap();
        
        // Should have bound the record
        assert!(matches!(new_ctx.get("stock"), Some(Value::Record(_))));
    }

    #[tokio::test]
    async fn test_execute_observe_destructuring() {
        let mut behaviour_ctx = BehaviourContext::new();
        behaviour_ctx.register(Box::new(
            MockBehaviourProvider::new("sensor", "temp")
                .with_value(Value::Record(hashmap! {
                    "value".into() => Value::Int(25),
                    "unit".into() => Value::String("celsius".into()),
                }))
        ));
        
        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Record(vec![
                ("value".into(), Pattern::Variable("t".into())),
                ("unit".into(), Pattern::Variable("u".into())),
            ]),
        };
        
        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx).await.unwrap();
        
        assert_eq!(new_ctx.get("t"), Some(&Value::Int(25)));
        assert_eq!(new_ctx.get("u"), Some(&Value::String("celsius".into())));
    }

    #[tokio::test]
    async fn test_execute_observe_missing_provider() {
        let behaviour_ctx = BehaviourContext::new(); // Empty
        
        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Variable("t".into()),
        };
        
        let ctx = Context::new();
        let result = execute_observe(&observe, ctx, &behaviour_ctx).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecError::CapabilityNotAvailable(_)));
    }

    #[tokio::test]
    async fn test_execute_changed_true() {
        let mut behaviour_ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::Int(42));
        behaviour_ctx.register(Box::new(provider.clone()));
        
        // First sample to establish baseline
        let _ = behaviour_ctx.sample("sensor", "temp", &[]).await;
        
        // Change value
        provider.set_value(Value::Int(43));
        
        let changed = Changed {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
        };
        
        let ctx = Context::new();
        let new_ctx = execute_changed(&changed, ctx, &behaviour_ctx).await.unwrap();
        
        assert_eq!(new_ctx.get("changed"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_execute_changed_false() {
        let mut behaviour_ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::Int(42));
        behaviour_ctx.register(Box::new(provider.clone()));
        
        // Sample to establish baseline
        let _ = behaviour_ctx.sample("sensor", "temp", &[]).await;
        
        // Value unchanged
        
        let changed = Changed {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
        };
        
        let ctx = Context::new();
        let new_ctx = execute_changed(&changed, ctx, &behaviour_ctx).await.unwrap();
        
        assert_eq!(new_ctx.get("changed"), Some(&Value::Bool(false)));
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-interp execute_observe -- --nocapture`
Expected: FAIL - executor not implemented

### Step 3: Implement (Green)

Create `crates/ash-interp/src/execute_observe.rs`:

```rust
use ash_core::{Observe, Changed, Pattern};
use crate::context::Context;
use crate::behaviour::BehaviourContext;
use crate::pattern::match_pattern;
use crate::ExecResult;
use crate::error::ExecError;

/// Execute an observe statement
pub async fn execute_observe(
    observe: &Observe,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
) -> ExecResult<Context> {
    // Sample the behaviour
    let value = behaviour_ctx.sample(
        &observe.capability,
        &observe.channel,
        &observe.constraints,
    ).await?;
    
    // Match pattern and bind variables
    let bindings = match_pattern(&observe.pattern, &value)
        .map_err(|e| ExecError::PatternMatchFailed {
            pattern: format!("{:?}", observe.pattern),
            value: value.clone(),
        })?;
    
    // Create new context with bindings
    let mut new_ctx = ctx.extend();
    new_ctx.set_many(bindings);
    
    Ok(new_ctx)
}

/// Execute a changed check
pub async fn execute_changed(
    changed: &Changed,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
) -> ExecResult<Context> {
    // Check if value has changed
    let has_changed = behaviour_ctx.has_changed(
        &changed.capability,
        &changed.channel,
        &changed.constraints,
    ).await?;
    
    // Create new context with result bound to "changed"
    let mut new_ctx = ctx.extend();
    new_ctx.set("changed".to_string(), Value::Bool(has_changed));
    
    Ok(new_ctx)
}

/// Execute observe as a workflow step
/// This version is used when observe is part of a larger workflow
pub async fn execute_observe_workflow(
    observe: &Observe,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
    continuation: &Workflow,
) -> ExecResult<Value> {
    let new_ctx = execute_observe(observe, ctx, behaviour_ctx).await?;
    execute_workflow(continuation, new_ctx, cap_ctx, policy_eval).await
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-interp execute_observe -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-interp/src/execute_observe.rs
git commit -m "feat: observe execution with sampling and pattern binding"
```

## Completion Checklist

- [ ] Observe sampling implemented
- [ ] Constraint application
- [ ] Pattern binding after observe
- [ ] Changed detection
- [ ] Missing provider error handling
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-093 (Behaviour provider)
- TASK-094 (Parse observe)

## Blocked By

- TASK-094

## Blocks

None (completes behaviours and Phase 13)
