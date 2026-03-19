# TASK-099: Runtime Validation in Providers

## Status: ✅ Complete

## Description

Add runtime validation to `TypedBehaviourProvider` and `TypedStreamProvider` that validates values against schemas.

## Specification Reference

- SPEC-015: Typed Providers - Section 3.3 Validation

## Requirements

### Functional Requirements

1. Override `sample()` to validate before returning
2. Override `recv()`/`try_recv()` to validate before returning
3. Clear error messages on mismatch
4. Option to skip validation (performance mode)

### Property Requirements

```rust
// Valid value passes through
let value = typed.sample(&[]).await.unwrap();
assert_eq!(value, expected);

// Invalid value returns error
let result = bad_typed.sample(&[]).await;
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("type mismatch"));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_validation_passes() {
    let inner = MockBehaviourProvider::new("sensor", "temp")
        .with_value(Value::Int(42));
    let typed = TypedBehaviourProvider::new(inner, Type::Int);
    
    let result = typed.sample(&[]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validation_fails() {
    let inner = MockBehaviourProvider::new("sensor", "temp")
        .with_value(Value::String("not a number"));
    let typed = TypedBehaviourProvider::new(inner, Type::Int);
    
    let result = typed.sample(&[]).await;
    assert!(result.is_err());
    
    let err = result.unwrap_err().to_string();
    assert!(err.contains("type mismatch"));
    assert!(err.contains("sensor:temp"));
}
```

### Step 2: Verify RED

Expected: FAIL - validation not implemented

### Step 3: Implement (Green)

```rust
#[async_trait]
impl BehaviourProvider for TypedBehaviourProvider {
    // ... capability_name, channel_name ...
    
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.inner.sample(constraints).await?;
        
        if !self.schema.matches(&value) {
            return Err(ExecError::TypeMismatch {
                provider: format!("{}:{}", self.capability_name(), self.channel_name()),
                expected: self.schema.to_string(),
                actual: value.to_string(),
            });
        }
        
        Ok(value)
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: runtime type validation in providers"
```

## Completion Checklist

- [ ] Behaviour provider validates
- [ ] Stream provider validates
- [ ] Clear error messages
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-097 (Schema validation logic)
- TASK-098 (Typed registry)

## Blocked By

- TASK-098

## Blocks

- TASK-100 (Error reporting)
