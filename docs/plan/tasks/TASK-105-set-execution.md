# TASK-105: Set Execution

## Status: ✅ Completed

## Description

Implement execution of set statements for output behaviours.

## Specification Reference

- SPEC-016: Output Capabilities - Section 2.3 Semantics

## Requirements

### Functional Requirements

1. Evaluate value expression
2. Lookup settable provider
3. Validate value against schema (if typed)
4. Call provider.set()
5. Handle errors

### Property Requirements

```rust
// Set changes value
let ctx = Context::new();
execute_set(&set_stmt, ctx, &behaviour_ctx).await.unwrap();

// Provider sees new value
let value = provider.sample(&[]).await.unwrap();
assert_eq!(value, expected);
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_set_executes() {
    let mut ctx = BehaviourContext::new();
    let provider = MockSettableProvider::new("hvac", "target");
    ctx.register(Box::new(provider.clone()));
    
    let set = Set {
        capability: "hvac".into(),
        channel: "target".into(),
        value: Expr::Literal(Value::Int(72)),
    };
    
    let workflow_ctx = Context::new();
    execute_set(&set, workflow_ctx, &ctx).await.unwrap();
    
    // Verify provider state changed
    let current = provider.sample(&[]).await.unwrap();
    assert_eq!(current, Value::Int(72));
}

#[tokio::test]
async fn test_set_missing_provider() {
    let ctx = BehaviourContext::new(); // Empty
    
    let set = Set { ... };
    let result = execute_set(&set, Context::new(), &ctx).await;
    
    assert!(result.is_err());
}
```

### Step 2: Verify RED

Expected: FAIL - executor not implemented

### Step 3: Implement (Green)

```rust
pub async fn execute_set(
    set: &Set,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
) -> ExecResult<()> {
    // 1. Evaluate value expression
    let value = eval_expr(&set.value, &ctx)?;
    
    // 2. Get settable provider
    let provider = behaviour_ctx
        .get_settable(&set.capability, &set.channel)
        .ok_or_else(|| ExecError::CapabilityNotAvailable(
            format!("{}:{}", set.capability, set.channel)
        ))?;
    
    // 3. Validate
    provider.validate(&value)
        .map_err(|e| ExecError::ValidationFailed(e.0))?;
    
    // 4. Set value
    provider.set(value).await
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: set statement execution"
```

## Completion Checklist

- [x] Expression evaluation
- [x] Provider lookup
- [x] Validation
- [x] Set call
- [x] Error handling
- [x] Tests pass
- [x] `cargo fmt` clean
- [x] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-101 (Settable provider)
- TASK-103 (Parse set)

## Blocked By

- TASK-103

## Blocks

None
