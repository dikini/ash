# TASK-102: Sendable Stream Provider Trait

## Status: ✅ Complete

## Description

Implement the trait for sendable (output) stream providers.

## Specification Reference

- SPEC-016: Output Capabilities - Section 3.3 Provider Interface

## Requirements

### Functional Requirements

1. `SendableStreamProvider` trait extending `StreamProvider`
2. `send(&self, value: Value)` method
3. `would_block(&self) -> bool` for backpressure
4. `flush(&self)` for buffered sends
5. `TypedSendableProvider` wrapper with schema validation

### Property Requirements

```rust
// Send produces event
provider.send(Value::Record({...})).await.unwrap();

// would_block indicates backpressure
if provider.would_block() {
    sleep(10ms).await;
}
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_send_accepts_value() {
    let provider = MockSendableProvider::new("kafka", "orders");
    
    let result = provider.send(Value::Int(42)).await;
    assert!(result.is_ok());
}

#[test]
fn test_would_block_default() {
    let provider = MockSendableProvider::new("kafka", "orders");
    assert!(!provider.would_block()); // Default: never blocks
}
```

### Step 2: Verify RED

Expected: FAIL - trait not defined

### Step 3: Implement (Green)

```rust
#[async_trait]
pub trait SendableStreamProvider: StreamProvider + Send + Sync {
    /// Send an event to the stream
    async fn send(&self, value: Value) -> ExecResult<()>;
    
    /// Check if send would block (backpressure)
    fn would_block(&self) -> bool {
        false // Default: never blocks
    }
    
    /// Flush any buffered sends
    async fn flush(&self) -> ExecResult<()> {
        Ok(()) // Default: no-op
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: sendable stream provider trait"
```

## Completion Checklist

- [ ] SendableStreamProvider trait
- [ ] send method
- [ ] would_block with default
- [ ] flush with default
- [ ] MockSendableProvider for tests
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-089 (Stream provider)

## Blocked By

- TASK-089

## Blocks

- TASK-106 (Send execution)
- TASK-107 (Bidirectional wrapper)
