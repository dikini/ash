# TASK-107: Bidirectional Provider Wrappers

## Status: 🔴 Not Started

## Description

Create wrappers that combine input/output capabilities for bidirectional providers.

## Specification Reference

- SPEC-016: Output Capabilities - Section 4 Bidirectional Capabilities

## Requirements

### Functional Requirements

1. `BidirectionalBehaviourProvider` combining observe + set
2. `BidirectionalStreamProvider` combining receive + send
3. Single schema for both directions (or separate read/write schemas)
4. Unified registration

### Property Requirements

```rust
let provider = BidirectionalBehaviourProvider::new(
    inner,
    read_schema: Type::Int,
    write_schema: Type::Int,
);

// Can observe
let value = provider.sample(&[]).await.unwrap();

// Can set
provider.set(Value::Int(42)).await.unwrap();
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_bidirectional_observe_and_set() {
    let inner = MockBidirectionalProvider::new("hvac", "target");
    let provider = BidirectionalBehaviourProvider::new(
        inner,
        Type::Int,
        Type::Int,
    );
    
    // Observe
    let v1 = provider.sample(&[]).await.unwrap();
    assert_eq!(v1, Value::Int(70));
    
    // Set
    provider.set(Value::Int(72)).await.unwrap();
    
    // Observe new value
    let v2 = provider.sample(&[]).await.unwrap();
    assert_eq!(v2, Value::Int(72));
}
```

### Step 2: Verify RED

Expected: FAIL - wrapper not defined

### Step 3: Implement (Green)

```rust
pub struct BidirectionalBehaviourProvider {
    inner: Box<dyn BidirectionalBehaviour>,
    read_schema: Type,
    write_schema: Type,
}

impl BidirectionalBehaviourProvider {
    pub fn new<B>(inner: B, read_schema: Type, write_schema: Type) -> Self
    where
        B: BidirectionalBehaviour + 'static,
    {
        Self {
            inner: Box::new(inner),
            read_schema,
            write_schema,
        }
    }
}

#[async_trait]
impl BehaviourProvider for BidirectionalBehaviourProvider {
    fn capability_name(&self) -> &str { self.inner.capability_name() }
    fn channel_name(&self) -> &str { self.inner.channel_name() }
    
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.inner.sample(constraints).await?;
        // Validate against read_schema
        Ok(value)
    }
}

#[async_trait]
impl SettableBehaviourProvider for BidirectionalBehaviourProvider {
    async fn set(&self, value: Value) -> ExecResult<()> {
        // Validate against write_schema
        self.inner.set(value).await
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: bidirectional provider wrappers"
```

## Completion Checklist

- [ ] BidirectionalBehaviourProvider
- [ ] BidirectionalStreamProvider
- [ ] Separate read/write schemas
- [ ] Unified registration
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

- TASK-101 (Settable provider)
- TASK-102 (Sendable provider)

## Blocked By

- TASK-101
- TASK-102

## Blocks

None (completes output capabilities)
