# TASK-106: Send Execution

## Status: 🔴 Not Started

## Description

Implement execution of send statements for output streams.

## Specification Reference

- SPEC-016: Output Capabilities - Section 3.3 Semantics

## Requirements

### Functional Requirements

1. Evaluate value expression
2. Lookup sendable provider
3. Validate value against schema
4. Call provider.send()
5. Handle backpressure (optional wait)
6. Handle errors

### Property Requirements

```rust
// Send succeeds
execute_send(&send_stmt, ctx, &stream_ctx).await.unwrap();

// Value is queued/sent
assert!(provider.received(Value::Record({...})));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_send_executes() {
    let mut ctx = StreamContext::new();
    let provider = MockSendableProvider::new("kafka", "orders");
    ctx.register(Box::new(provider.clone()));
    
    let send = Send {
        capability: "kafka".into(),
        channel: "orders".into(),
        value: Expr::Record(...),
    };
    
    let workflow_ctx = Context::new();
    execute_send(&send, workflow_ctx, &ctx).await.unwrap();
    
    // Verify provider received
    assert!(provider.has_sent());
}

#[tokio::test]
async fn test_send_would_block() {
    let provider = MockSendableProvider::new("kafka", "orders")
        .with_blocking(true);
    
    // Should handle backpressure
}
```

### Step 2: Verify RED

Expected: FAIL - executor not implemented

### Step 3: Implement (Green)

```rust
pub async fn execute_send(
    send: &Send,
    ctx: Context,
    stream_ctx: &StreamContext,
) -> ExecResult<()> {
    // 1. Evaluate value
    let value = eval_expr(&send.value, &ctx)?;
    
    // 2. Get sendable provider
    let provider = stream_ctx
        .get_sendable(&send.capability, &send.channel)
        .ok_or_else(|| ExecError::CapabilityNotAvailable(...))?;
    
    // 3. Optional: wait if would_block
    while provider.would_block() {
        sleep(Duration::from_millis(10)).await;
    }
    
    // 4. Send
    provider.send(value).await
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: send statement execution"
```

## Completion Checklist

- [ ] Expression evaluation
- [ ] Provider lookup
- [ ] Backpressure handling
- [ ] Send call
- [ ] Error handling
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-102 (Sendable provider)
- TASK-104 (Parse send)

## Blocked By

- TASK-104

## Blocks

None
