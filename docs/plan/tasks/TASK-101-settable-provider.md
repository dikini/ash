# TASK-101: Settable Behaviour Provider Trait

## Status: ✅ Complete

## Description

Implement the trait for settable (output) behaviour providers.

## Specification Reference

- SPEC-016: Output Capabilities - Section 2.3 Provider Interface

## Requirements

### Functional Requirements

1. `SettableBehaviourProvider` trait extending `BehaviourProvider`
2. `set(&self, value: Value)` method
3. Optional `validate(&self, value: &Value)` for pre-checks
4. `TypedSettableProvider` wrapper with schema validation

### Property Requirements

```rust
// Set changes value
provider.set(Value::Int(42)).await.unwrap();
let value = provider.sample(&[]).await.unwrap();
assert_eq!(value, Value::Int(42));

// Validation can reject
let result = provider.set(Value::String("bad")).await;
assert!(result.is_err());
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_set_changes_value() {
    let provider = MockSettableProvider::new("hvac", "target")
        .with_initial(Value::Int(70));
    
    provider.set(Value::Int(72)).await.unwrap();
    
    let current = provider.sample(&[]).await.unwrap();
    assert_eq!(current, Value::Int(72));
}

#[tokio::test]
async fn test_validate_rejects_invalid() {
    let provider = MockSettableProvider::new("hvac", "target")
        .with_validator(|v| matches!(v, Value::Int(_)));
    
    let result = provider.set(Value::String("hot")).await;
    assert!(result.is_err());
}
```

### Step 2: Verify RED

Expected: FAIL - trait not defined

### Step 3: Implement (Green)

```rust
#[async_trait]
pub trait SettableBehaviourProvider: BehaviourProvider + Send + Sync {
    /// Set the behaviour value
    async fn set(&self, value: Value) -> ExecResult<()>;
    
    /// Validate before setting (default: accept all)
    fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        let _ = value;
        Ok(())
    }
}

pub struct ValidationError(pub String);

impl From<ValidationError> for ExecError {
    fn from(e: ValidationError) -> Self {
        ExecError::ValidationFailed(e.0)
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: settable behaviour provider trait"
```

## Completion Checklist

- [ ] SettableBehaviourProvider trait
- [ ] set method
- [ ] validate method with default
- [ ] MockSettableProvider for tests
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-093 (Behaviour provider)

## Blocked By

- TASK-093

## Blocks

- TASK-105 (Set execution)
- TASK-107 (Bidirectional wrapper)
