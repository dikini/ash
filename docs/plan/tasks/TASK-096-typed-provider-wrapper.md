# TASK-096: Typed Provider Wrapper

## Status: 🔴 Not Started

## Description

Implement the `TypedBehaviourProvider` and `TypedStreamProvider` wrappers that carry type schemas.

## Specification Reference

- SPEC-015: Typed Providers - Section 4 API Design

## Requirements

### Functional Requirements

1. `TypedBehaviourProvider` struct wrapping `BehaviourProvider` with `Type` schema
2. `TypedStreamProvider` struct wrapping `StreamProvider` with `Type` schema
3. Constructor that accepts provider and schema
4. Schema accessor method
5. Delegation of trait methods to inner provider

### Property Requirements

```rust
let typed = TypedBehaviourProvider::new(provider, Type::Int);
assert_eq!(typed.schema(), &Type::Int);
assert_eq!(typed.capability_name(), provider.capability_name());
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_typed_provider_creation() {
    let inner = MockBehaviourProvider::new("sensor", "temp");
    let typed = TypedBehaviourProvider::new(inner, Type::Int);
    
    assert_eq!(typed.schema(), &Type::Int);
    assert_eq!(typed.capability_name(), "sensor");
    assert_eq!(typed.channel_name(), "temp");
}

#[tokio::test]
async fn test_typed_provider_delegates_sample() {
    let inner = MockBehaviourProvider::new("sensor", "temp")
        .with_value(Value::Int(42));
    let typed = TypedBehaviourProvider::new(inner, Type::Int);
    
    let value = typed.sample(&[]).await.unwrap();
    assert_eq!(value, Value::Int(42));
}
```

### Step 2: Verify RED

Expected: FAIL - types not defined

### Step 3: Implement (Green)

```rust
pub struct TypedBehaviourProvider {
    inner: Box<dyn BehaviourProvider>,
    schema: Type,
}

impl TypedBehaviourProvider {
    pub fn new<P>(provider: P, schema: Type) -> Self
    where
        P: BehaviourProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            schema,
        }
    }
    
    pub fn schema(&self) -> &Type {
        &self.schema
    }
}

#[async_trait]
impl BehaviourProvider for TypedBehaviourProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }
    
    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }
    
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        self.inner.sample(constraints).await
    }
    
    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        self.inner.has_changed(constraints).await
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: typed provider wrapper structs"
```

## Completion Checklist

- [ ] TypedBehaviourProvider struct
- [ ] TypedStreamProvider struct  
- [ ] Constructor with schema
- [ ] Schema accessor
- [ ] Trait delegation
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-093 (Behaviour provider)
- TASK-089 (Stream provider)

## Blocked By

- TASK-093

## Blocks

- TASK-097 (Schema validation)
