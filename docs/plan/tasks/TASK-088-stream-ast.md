# TASK-088: Stream AST Types

## Status: ✅ Complete

## Description

Implement AST types for streams and the mailbox structure.

## Specification Reference

- SPEC-013: Streams and Event Processing - Section 3 Syntax

## Requirements

### Functional Requirements

1. `StreamRef` struct - reference to a stream source
2. `Receive` struct - the receive construct with arms
3. `ReceiveArm` struct - pattern + guard + body
4. `ReceiveMode` enum - blocking vs non-blocking
5. `Mailbox` struct - message buffer with limits
6. Update `Workflow` enum with `Receive` and `ForStream` variants

### Property Requirements

```rust
// Receive arms are ordered
receive.arms.len() >= 1

// Mailbox respects limits
mailbox.len() <= mailbox.limit()

// Pattern matching covers all cases or has wildcard
receive.has_wildcard() || receive.is_exhaustive()
```

## TDD Steps

### Step 1: Write Tests (Red)

Create `crates/ash-core/src/stream.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_ref_creation() {
        let stream = StreamRef::new("kafka", "orders");
        assert_eq!(stream.capability(), "kafka");
        assert_eq!(stream.channel(), "orders");
    }

    #[test]
    fn test_receive_mode_blocking() {
        let mode = ReceiveMode::Blocking(None); // wait forever
        assert!(mode.is_blocking());
    }

    #[test]
    fn test_receive_mode_timeout() {
        let mode = ReceiveMode::Blocking(Some(Duration::from_secs(30)));
        assert!(mode.is_blocking());
        assert_eq!(mode.timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_mailbox_respects_limit() {
        let mut mailbox = Mailbox::with_limit(10);
        for i in 0..15 {
            mailbox.push(MailboxEntry::new(Value::Int(i)));
        }
        assert_eq!(mailbox.len(), 10); // Oldest dropped
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-core stream::tests -- --nocapture`
Expected: FAIL - types not defined

### Step 3: Implement Types (Green)

```rust
/// Reference to a stream source
#[derive(Debug, Clone, PartialEq)]
pub struct StreamRef {
    pub capability: Box<str>,
    pub channel: Box<str>,
}

/// Receive mode: non-blocking, blocking forever, or blocking with timeout
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiveMode {
    NonBlocking,
    Blocking(Option<Duration>),
}

/// A receive arm: pattern + optional guard + body
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Workflow,
}

/// The receive construct
#[derive(Debug, Clone, PartialEq)]
pub struct Receive {
    pub mode: ReceiveMode,
    pub arms: Vec<ReceiveArm>,
    pub control_arms: Option<Vec<ReceiveArm>>, // For receive control
}

/// Mailbox entry
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxEntry {
    pub source: Box<str>,
    pub channel: Box<str>,
    pub value: Value,
    pub timestamp: Instant,
}

/// Mailbox with size limit
#[derive(Debug, Clone)]
pub struct Mailbox {
    entries: VecDeque<MailboxEntry>,
    limit: usize,
    overflow: OverflowStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverflowStrategy {
    DropOldest,
    DropNewest,
    Error,
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-core stream::tests -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-core/src/stream.rs
git commit -m "feat: stream AST types and mailbox structure"
```

## Completion Checklist

- [ ] StreamRef struct defined
- [ ] ReceiveMode enum defined
- [ ] ReceiveArm struct defined
- [ ] Receive struct defined
- [ ] Mailbox with limit and overflow strategy
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-089 (Stream provider)
- TASK-090 (Parse receive)
- TASK-091 (Mailbox implementation)
