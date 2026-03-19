# TASK-089: Stream Provider Trait

## Status: ✅ Complete

## Description

Implement the stream provider trait and registry for event sources.

## Specification Reference

- SPEC-013: Streams and Event Processing - Section 8 Stream Provider Interface

## Requirements

### Functional Requirements

1. `StreamProvider` trait with `try_recv`, `recv`, `is_closed`
2. `StreamRegistry` for managing providers
3. `StreamContext` for execution-time access
4. Support for multiple channels per capability

### Property Requirements

```rust
// Provider can be registered and retrieved
registry.register(provider);
registry.get("kafka", "orders").is_some()

// Closed stream returns None
test_provider.close();
assert!(test_provider.is_closed());
assert!(test_provider.try_recv().is_none());
```

## TDD Steps

### Step 1: Write Tests (Red)

Create tests in `crates/ash-interp/src/stream.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_recv() {
        let provider = MockStreamProvider::new("kafka", "orders")
            .with_values(vec![Value::Int(1), Value::Int(2)]);
        
        let v1 = provider.recv().await.unwrap();
        assert_eq!(v1, Value::Int(1));
        
        let v2 = provider.recv().await.unwrap();
        assert_eq!(v2, Value::Int(2));
    }

    #[test]
    fn test_stream_registry() {
        let mut registry = StreamRegistry::new();
        let provider = MockStreamProvider::new("kafka", "orders");
        
        registry.register(Box::new(provider));
        
        assert!(registry.has("kafka", "orders"));
        assert!(!registry.has("kafka", "metrics"));
    }

    #[test]
    fn test_stream_context_get() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::new("sensor", "temp");
        ctx.register(Box::new(provider));
        
        let stream = ctx.get("sensor", "temp");
        assert!(stream.is_some());
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-interp stream::tests -- --nocapture`
Expected: FAIL - trait not defined

### Step 3: Implement (Green)

Create `crates/ash-interp/src/stream.rs`:

```rust
/// Stream provider trait
#[async_trait]
pub trait StreamProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    
    /// Try to receive without blocking
    fn try_recv(&self) -> Option<ExecResult<Value>>;
    
    /// Block until message available
    async fn recv(&self) -> ExecResult<Value>;
    
    /// Check if stream is closed
    fn is_closed(&self) -> bool;
}

/// Registry of stream providers
#[derive(Default)]
pub struct StreamRegistry {
    providers: HashMap<(Name, Name), Box<dyn StreamProvider>>,
}

impl StreamRegistry {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, provider: Box<dyn StreamProvider>) { ... }
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn StreamProvider> { ... }
    pub fn has(&self, cap: &str, channel: &str) -> bool { ... }
}

/// Context for stream operations
pub struct StreamContext {
    registry: StreamRegistry,
}

impl StreamContext {
    pub fn new() -> Self { ... }
    pub fn with_registry(registry: StreamRegistry) -> Self { ... }
    pub fn register(&mut self, provider: Box<dyn StreamProvider>) { ... }
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn StreamProvider> { ... }
}

/// Mock provider for testing
pub struct MockStreamProvider {
    name: (String, String),
    values: Mutex<VecDeque<Value>>,
    closed: AtomicBool,
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-interp stream::tests -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-interp/src/stream.rs
git commit -m "feat: stream provider trait and registry"
```

## Notes

This task creates the base trait. See TASK-096 for typed wrapper with schema validation. Providers will eventually need to declare their Ash type schema for runtime validation.

## Completion Checklist

- [ ] StreamProvider trait defined
- [ ] StreamRegistry implemented
- [ ] StreamContext implemented
- [ ] MockStreamProvider for testing
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-088 (Stream AST types)

## Blocked By

- TASK-088

## Blocks

- TASK-092 (Stream execution)
- TASK-096 (Typed provider wrapper)
