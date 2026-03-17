# TASK-091: Mailbox Implementation

## Status: 🔴 Not Started

## Description

Implement the mailbox with size limits and overflow strategies.

## Specification Reference

- SPEC-013: Streams and Event Processing - Section 7 Mailbox Configuration

## Requirements

### Functional Requirements

1. `Mailbox` with configurable size limit
2. Overflow strategies: `DropOldest`, `DropNewest`, `Error`
3. Pattern matching search across mailbox entries
4. Thread-safe operations (for async execution)
5. Entry tracking (source, channel, timestamp)

### Property Requirements

```rust
// FIFO order preserved
mailbox.push(entry1);
mailbox.push(entry2);
assert_eq!(mailbox.pop().unwrap(), entry1);

// Limit enforced
mailbox.set_limit(5);
for i in 0..10 { mailbox.push(entry); }
assert_eq!(mailbox.len(), 5);

// Pattern matching finds matches
mailbox.push(entry_from_kafka);
mailbox.push(entry_from_sensor);
assert!(mailbox.find_matching(|e| e.source == "kafka").is_some());
```

## TDD Steps

### Step 1: Write Tests (Red)

Create tests in `crates/ash-interp/src/mailbox.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_push_and_pop() {
        let mut mailbox = Mailbox::new();
        let entry = MailboxEntry::new("kafka", "orders", Value::Int(1));
        
        mailbox.push(entry.clone());
        assert_eq!(mailbox.len(), 1);
        
        let popped = mailbox.pop().unwrap();
        assert_eq!(popped.value, Value::Int(1));
        assert_eq!(mailbox.len(), 0);
    }

    #[test]
    fn test_mailbox_drop_oldest() {
        let mut mailbox = Mailbox::with_limit(3, OverflowStrategy::DropOldest);
        
        mailbox.push(MailboxEntry::new("a", "b", Value::Int(1)));
        mailbox.push(MailboxEntry::new("a", "b", Value::Int(2)));
        mailbox.push(MailboxEntry::new("a", "b", Value::Int(3)));
        mailbox.push(MailboxEntry::new("a", "b", Value::Int(4)));
        
        assert_eq!(mailbox.len(), 3);
        // Oldest (1) should be dropped
        assert_eq!(mailbox.pop().unwrap().value, Value::Int(2));
    }

    #[test]
    fn test_mailbox_find_matching() {
        let mut mailbox = Mailbox::new();
        mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1)));
        mailbox.push(MailboxEntry::new("sensor", "temp", Value::Int(2)));
        
        let found = mailbox.find_matching(|e| e.source == "kafka");
        assert!(found.is_some());
        assert_eq!(found.unwrap().value, Value::Int(1));
    }

    #[test]
    fn test_mailbox_remove_matching() {
        let mut mailbox = Mailbox::new();
        mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1)));
        mailbox.push(MailboxEntry::new("sensor", "temp", Value::Int(2)));
        
        let removed = mailbox.remove_matching(|e| e.source == "kafka");
        assert!(removed.is_some());
        assert_eq!(mailbox.len(), 1);
    }

    #[tokio::test]
    async fn test_mailbox_thread_safe() {
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        
        let m1 = mailbox.clone();
        let t1 = tokio::spawn(async move {
            for i in 0..100 {
                m1.lock().await.push(MailboxEntry::new("a", "b", Value::Int(i)));
            }
        });
        
        let m2 = mailbox.clone();
        let t2 = tokio::spawn(async move {
            for _ in 0..100 {
                let _ = m2.lock().await.pop();
            }
        });
        
        let (r1, r2) = tokio::join!(t1, t2);
        assert!(r1.is_ok() && r2.is_ok());
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-interp mailbox -- --nocapture`
Expected: FAIL - implementation missing

### Step 3: Implement (Green)

Create `crates/ash-interp/src/mailbox.rs`:

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ash_core::{Name, Value};

/// Entry in the mailbox
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxEntry {
    pub source: Name,
    pub channel: Name,
    pub value: Value,
    pub timestamp: Instant,
}

impl MailboxEntry {
    pub fn new(source: impl Into<Name>, channel: impl Into<Name>, value: Value) -> Self {
        Self {
            source: source.into(),
            channel: channel.into(),
            value,
            timestamp: Instant::now(),
        }
    }
}

/// Overflow strategy when mailbox is full
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OverflowStrategy {
    #[default]
    DropOldest,
    DropNewest,
    Error,
}

/// Thread-safe mailbox for event buffering
pub struct Mailbox {
    entries: VecDeque<MailboxEntry>,
    limit: usize,
    overflow: OverflowStrategy,
}

impl Mailbox {
    pub fn new() -> Self {
        Self::with_limit(1000, OverflowStrategy::default())
    }
    
    pub fn with_limit(limit: usize, overflow: OverflowStrategy) -> Self {
        Self {
            entries: VecDeque::with_capacity(limit.min(1024)),
            limit,
            overflow,
        }
    }
    
    pub fn push(&mut self, entry: MailboxEntry) -> Result<(), MailboxError> {
        if self.entries.len() >= self.limit {
            match self.overflow {
                OverflowStrategy::DropOldest => {
                    self.entries.pop_front();
                }
                OverflowStrategy::DropNewest => {
                    return Ok(()); // Silently drop
                }
                OverflowStrategy::Error => {
                    return Err(MailboxError::Overflow);
                }
            }
        }
        self.entries.push_back(entry);
        Ok(())
    }
    
    pub fn pop(&mut self) -> Option<MailboxEntry> {
        self.entries.pop_front()
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    pub fn find_matching<F>(&self, predicate: F) -> Option<&MailboxEntry>
    where
        F: Fn(&MailboxEntry) -> bool,
    {
        self.entries.iter().find(|e| predicate(e))
    }
    
    pub fn remove_matching<F>(&mut self, predicate: F) -> Option<MailboxEntry>
    where
        F: Fn(&MailboxEntry) -> bool,
    {
        if let Some(pos) = self.entries.iter().position(|e| predicate(e)) {
            return self.entries.remove(pos);
        }
        None
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MailboxError {
    #[error("mailbox overflow")]
    Overflow,
}

/// Thread-safe mailbox wrapper
pub type SharedMailbox = Arc<Mutex<Mailbox>>;
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-interp mailbox -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-interp/src/mailbox.rs
git commit -m "feat: mailbox implementation with limits and overflow"
```

## Completion Checklist

- [ ] MailboxEntry struct with metadata
- [ ] Mailbox with configurable limit
- [ ] Overflow strategies implemented
- [ ] Pattern matching search methods
- [ ] Thread-safe wrapper
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-088 (Stream AST types)

## Blocked By

- TASK-088

## Blocks

- TASK-092 (Stream execution)
