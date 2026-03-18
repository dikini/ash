//! Stream AST types and mailbox structure
//!
//! This module provides types for stream processing in Ash workflows,
//! including stream references, receive constructs, and mailbox management.

use crate::{Expr, Pattern, Value, Workflow};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Reference to a stream source
///
/// A `StreamRef` identifies a stream by its capability and channel name.
/// The capability represents the stream provider (e.g., "kafka", "rabbitmq"),
/// while the channel identifies the specific stream or topic within that provider.
///
/// # Examples
///
/// ```
/// use ash_core::stream::StreamRef;
///
/// let stream = StreamRef::new("kafka", "orders");
/// assert_eq!(stream.capability(), "kafka");
/// assert_eq!(stream.channel(), "orders");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamRef {
    capability: Box<str>,
    channel: Box<str>,
}

impl StreamRef {
    /// Creates a new stream reference
    ///
    /// # Arguments
    ///
    /// * `capability` - The stream provider capability name
    /// * `channel` - The channel or topic name within the provider
    pub fn new(capability: impl Into<Box<str>>, channel: impl Into<Box<str>>) -> Self {
        Self {
            capability: capability.into(),
            channel: channel.into(),
        }
    }

    /// Returns the capability name
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Returns the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }
}

/// Receive mode for stream message consumption
///
/// Determines how the receive operation should behave when no messages
/// are immediately available in the mailbox.
///
/// # Variants
///
/// * `NonBlocking` - Return immediately if no messages are available
/// * `Blocking` - Wait indefinitely or for a specified duration
///
/// # Examples
///
/// ```
/// use ash_core::stream::ReceiveMode;
/// use std::time::Duration;
///
/// // Non-blocking receive
/// let mode = ReceiveMode::NonBlocking;
/// assert!(!mode.is_blocking());
///
/// // Blocking receive with no timeout
/// let mode = ReceiveMode::Blocking(None);
/// assert!(mode.is_blocking());
/// assert_eq!(mode.timeout(), None);
///
/// // Blocking receive with 30-second timeout
/// let mode = ReceiveMode::Blocking(Some(Duration::from_secs(30)));
/// assert!(mode.is_blocking());
/// assert_eq!(mode.timeout(), Some(Duration::from_secs(30)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReceiveMode {
    /// Return immediately if no messages available
    NonBlocking,
    /// Block until a message arrives, optionally with timeout
    Blocking(Option<Duration>),
}

impl ReceiveMode {
    /// Returns true if this mode blocks for messages
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Blocking(_))
    }

    /// Returns true if this mode does not block
    pub fn is_non_blocking(&self) -> bool {
        matches!(self, Self::NonBlocking)
    }

    /// Returns the timeout duration if blocking with timeout
    pub fn timeout(&self) -> Option<Duration> {
        match self {
            Self::Blocking(timeout) => *timeout,
            Self::NonBlocking => None,
        }
    }
}

/// A receive arm: pattern + optional guard + body
///
/// Represents one branch of a receive expression. When a message matches
/// the pattern and satisfies the guard (if present), the body workflow
/// is executed.
///
/// # Examples
///
/// ```
/// use ash_core::stream::ReceiveArm;
/// use ash_core::{Pattern, Workflow, Expr};
///
/// let arm = ReceiveArm {
///     pattern: Pattern::Variable("msg".to_string()),
///     guard: None,
///     body: Workflow::Done,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiveArm {
    /// Pattern to match against incoming messages
    pub pattern: Pattern,
    /// Optional guard expression that must evaluate to true
    pub guard: Option<Expr>,
    /// Workflow to execute when pattern matches and guard passes
    pub body: Workflow,
}

/// The receive construct for stream message handling
///
/// A receive expression waits for messages from streams and dispatches
/// to the appropriate arm based on pattern matching.
///
/// # Fields
///
/// * `mode` - Blocking or non-blocking receive behavior
/// * `arms` - List of receive arms (must have at least one)
/// * `control_arms` - Optional arms for control messages
///
/// # Examples
///
/// ```
/// use ash_core::stream::{Receive, ReceiveArm, ReceiveMode};
/// use ash_core::{Pattern, Workflow};
///
/// let receive = Receive {
///     mode: ReceiveMode::NonBlocking,
///     arms: vec![ReceiveArm {
///         pattern: Pattern::Wildcard,
///         guard: None,
///         body: Workflow::Done,
///     }],
///     control_arms: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receive {
    /// Receive mode (blocking/non-blocking)
    pub mode: ReceiveMode,
    /// Arms for handling regular messages
    pub arms: Vec<ReceiveArm>,
    /// Optional arms for control messages
    pub control_arms: Option<Vec<ReceiveArm>>,
}

impl Receive {
    /// Returns true if any arm has a wildcard pattern
    pub fn has_wildcard(&self) -> bool {
        let check_arms = |arms: &[ReceiveArm]| {
            arms.iter()
                .any(|arm| matches!(arm.pattern, Pattern::Wildcard))
        };

        check_arms(&self.arms) || self.control_arms.as_ref().is_some_and(|ca| check_arms(ca))
    }

    /// Returns the total number of arms (including control arms)
    pub fn arm_count(&self) -> usize {
        self.arms.len() + self.control_arms.as_ref().map_or(0, |ca| ca.len())
    }
}

/// Entry in the mailbox buffer
///
/// Represents a single message stored in a mailbox, including metadata
/// about its source and when it was received.
///
/// # Examples
///
/// ```
/// use ash_core::stream::MailboxEntry;
/// use ash_core::Value;
///
/// let entry = MailboxEntry::new("kafka", "orders", Value::Int(42));
/// assert_eq!(entry.source(), "kafka");
/// assert_eq!(entry.channel(), "orders");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MailboxEntry {
    source: Box<str>,
    channel: Box<str>,
    /// The message value
    pub value: Value,
    /// When the message was received (runtime-only, not serialized)
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
}

impl MailboxEntry {
    /// Creates a new mailbox entry with the current timestamp
    ///
    /// # Arguments
    ///
    /// * `source` - The source capability name
    /// * `channel` - The channel name
    /// * `value` - The message value
    pub fn new(source: impl Into<Box<str>>, channel: impl Into<Box<str>>, value: Value) -> Self {
        Self {
            source: source.into(),
            channel: channel.into(),
            value,
            timestamp: Instant::now(),
        }
    }

    /// Returns the source capability name
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }
}

/// Strategy for handling mailbox overflow
///
/// Determines what to do when the mailbox reaches its capacity limit
/// and a new message arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverflowStrategy {
    /// Drop the oldest message to make room
    DropOldest,
    /// Drop the new incoming message
    DropNewest,
    /// Return an error instead of accepting the message
    Error,
}

/// Mailbox for buffering stream messages
///
/// A bounded buffer that stores messages with configurable overflow
/// behavior when the limit is reached.
///
/// # Examples
///
/// ```
/// use ash_core::stream::{Mailbox, MailboxEntry, OverflowStrategy};
/// use ash_core::Value;
///
/// // Create a mailbox with default limit and strategy
/// let mut mailbox = Mailbox::new();
///
/// // Create with specific limit
/// let mut mailbox = Mailbox::with_limit(100);
///
/// // Create with custom strategy
/// let mut mailbox = Mailbox::with_strategy(100, OverflowStrategy::DropOldest);
///
/// // Push entries
/// mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1)));
/// ```
#[derive(Debug, Clone)]
pub struct Mailbox {
    entries: VecDeque<MailboxEntry>,
    limit: usize,
    overflow: OverflowStrategy,
}

impl Mailbox {
    /// Default mailbox size limit
    pub const DEFAULT_LIMIT: usize = 1024;

    /// Creates a new mailbox with default limit and DropOldest strategy
    pub fn new() -> Self {
        Self::with_limit(Self::DEFAULT_LIMIT)
    }

    /// Creates a new mailbox with the specified limit and DropOldest strategy
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of entries to store
    pub fn with_limit(limit: usize) -> Self {
        Self::with_strategy(limit, OverflowStrategy::DropOldest)
    }

    /// Creates a new mailbox with specified limit and overflow strategy
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of entries to store
    /// * `overflow` - Strategy when limit is exceeded
    pub fn with_strategy(limit: usize, overflow: OverflowStrategy) -> Self {
        Self {
            entries: VecDeque::with_capacity(limit.min(Self::DEFAULT_LIMIT)),
            limit,
            overflow,
        }
    }

    /// Returns the number of entries in the mailbox
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the mailbox is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the mailbox capacity limit
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Returns the overflow strategy
    pub fn overflow_strategy(&self) -> OverflowStrategy {
        self.overflow
    }

    /// Pushes an entry into the mailbox
    ///
    /// The behavior when the mailbox is full depends on the overflow strategy:
    /// - `DropOldest`: Removes the oldest entry and adds the new one
    /// - `DropNewest`: Discards the new entry
    /// - `Error`: Returns an error
    ///
    /// # Errors
    ///
    /// Returns `Err` if the overflow strategy is `Error` and the mailbox is full.
    pub fn push(&mut self, entry: MailboxEntry) -> Result<(), MailboxOverflowError> {
        if self.entries.len() >= self.limit {
            match self.overflow {
                OverflowStrategy::DropOldest => {
                    self.entries.pop_front();
                    self.entries.push_back(entry);
                    Ok(())
                }
                OverflowStrategy::DropNewest => {
                    // Silently drop the new entry
                    Ok(())
                }
                OverflowStrategy::Error => Err(MailboxOverflowError::new(self.limit)),
            }
        } else {
            self.entries.push_back(entry);
            Ok(())
        }
    }

    /// Pops the oldest entry from the mailbox
    pub fn pop(&mut self) -> Option<MailboxEntry> {
        self.entries.pop_front()
    }

    /// Returns a reference to the oldest entry without removing it
    pub fn peek(&self) -> Option<&MailboxEntry> {
        self.entries.front()
    }

    /// Clears all entries from the mailbox
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns an iterator over the entries (oldest first)
    pub fn iter(&self) -> impl Iterator<Item = &MailboxEntry> {
        self.entries.iter()
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned when mailbox overflow occurs with `Error` strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailboxOverflowError {
    limit: usize,
}

impl MailboxOverflowError {
    /// Creates a new mailbox overflow error with the specified limit
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    /// Returns the mailbox limit that was exceeded
    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl std::fmt::Display for MailboxOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mailbox overflow: limit of {} entries exceeded",
            self.limit
        )
    }
}

impl std::error::Error for MailboxOverflowError {}

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
    fn test_stream_ref_from_string() {
        let stream = StreamRef::new("kafka".to_string(), "orders".to_string());
        assert_eq!(stream.capability(), "kafka");
        assert_eq!(stream.channel(), "orders");
    }

    #[test]
    fn test_receive_mode_blocking() {
        let mode = ReceiveMode::Blocking(None); // wait forever
        assert!(mode.is_blocking());
        assert!(!mode.is_non_blocking());
        assert_eq!(mode.timeout(), None);
    }

    #[test]
    fn test_receive_mode_timeout() {
        let mode = ReceiveMode::Blocking(Some(Duration::from_secs(30)));
        assert!(mode.is_blocking());
        assert_eq!(mode.timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_receive_mode_non_blocking() {
        let mode = ReceiveMode::NonBlocking;
        assert!(!mode.is_blocking());
        assert!(mode.is_non_blocking());
        assert_eq!(mode.timeout(), None);
    }

    #[test]
    fn test_receive_arm_creation() {
        let arm = ReceiveArm {
            pattern: Pattern::Variable("x".to_string()),
            guard: Some(Expr::Literal(Value::Bool(true))),
            body: Workflow::Done,
        };
        assert_eq!(arm.pattern.bindings(), vec!["x"]);
        assert!(arm.guard.is_some());
    }

    #[test]
    fn test_receive_creation() {
        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![ReceiveArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Workflow::Done,
            }],
            control_arms: None,
        };
        assert!(receive.has_wildcard());
        assert_eq!(receive.arm_count(), 1);
    }

    #[test]
    fn test_receive_without_wildcard() {
        let receive = Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![ReceiveArm {
                pattern: Pattern::Variable("x".to_string()),
                guard: None,
                body: Workflow::Done,
            }],
            control_arms: None,
        };
        assert!(!receive.has_wildcard());
    }

    #[test]
    fn test_mailbox_entry_creation() {
        let entry = MailboxEntry::new("kafka", "orders", Value::Int(42));
        assert_eq!(entry.source(), "kafka");
        assert_eq!(entry.channel(), "orders");
        assert_eq!(entry.value, Value::Int(42));
    }

    #[test]
    fn test_mailbox_default() {
        let mailbox = Mailbox::new();
        assert_eq!(mailbox.limit(), Mailbox::DEFAULT_LIMIT);
        assert_eq!(mailbox.overflow_strategy(), OverflowStrategy::DropOldest);
        assert!(mailbox.is_empty());
    }

    #[test]
    fn test_mailbox_with_limit() {
        let mailbox = Mailbox::with_limit(100);
        assert_eq!(mailbox.limit(), 100);
    }

    #[test]
    fn test_mailbox_with_strategy() {
        let mailbox = Mailbox::with_strategy(50, OverflowStrategy::DropNewest);
        assert_eq!(mailbox.limit(), 50);
        assert_eq!(mailbox.overflow_strategy(), OverflowStrategy::DropNewest);
    }

    #[test]
    fn test_mailbox_push_and_pop() {
        let mut mailbox = Mailbox::with_limit(10);
        let entry = MailboxEntry::new("src", "chan", Value::Int(1));

        assert!(mailbox.push(entry.clone()).is_ok());
        assert_eq!(mailbox.len(), 1);

        let popped = mailbox.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().value, Value::Int(1));
        assert!(mailbox.is_empty());
    }

    #[test]
    fn test_mailbox_peek() {
        let mut mailbox = Mailbox::with_limit(10);
        let entry = MailboxEntry::new("src", "chan", Value::Int(42));

        mailbox.push(entry).unwrap();
        assert_eq!(mailbox.len(), 1);

        let peeked = mailbox.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().value, Value::Int(42));
        assert_eq!(mailbox.len(), 1); // Still there
    }

    #[test]
    fn test_mailbox_respects_limit() {
        let mut mailbox = Mailbox::with_limit(10);
        for i in 0..15 {
            mailbox
                .push(MailboxEntry::new("src", "chan", Value::Int(i)))
                .unwrap();
        }
        assert_eq!(mailbox.len(), 10); // Oldest dropped

        // Verify oldest entries were dropped (values 0-4 should be gone)
        let values: Vec<i64> = mailbox.iter().filter_map(|e| e.value.as_int()).collect();
        assert_eq!(values, vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_mailbox_drop_newest_strategy() {
        let mut mailbox = Mailbox::with_strategy(5, OverflowStrategy::DropNewest);
        for i in 0..10 {
            mailbox
                .push(MailboxEntry::new("src", "chan", Value::Int(i)))
                .unwrap();
        }
        assert_eq!(mailbox.len(), 5); // Only first 5 kept

        let values: Vec<i64> = mailbox.iter().filter_map(|e| e.value.as_int()).collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_mailbox_error_strategy() {
        let mut mailbox = Mailbox::with_strategy(3, OverflowStrategy::Error);

        // Fill to capacity
        for i in 0..3 {
            assert!(
                mailbox
                    .push(MailboxEntry::new("src", "chan", Value::Int(i)))
                    .is_ok()
            );
        }
        assert_eq!(mailbox.len(), 3);

        // Next push should error
        let result = mailbox.push(MailboxEntry::new("src", "chan", Value::Int(99)));
        assert!(result.is_err());
        assert_eq!(mailbox.len(), 3); // Unchanged
    }

    #[test]
    fn test_mailbox_clear() {
        let mut mailbox = Mailbox::with_limit(10);
        for i in 0..5 {
            mailbox
                .push(MailboxEntry::new("src", "chan", Value::Int(i)))
                .unwrap();
        }
        assert_eq!(mailbox.len(), 5);

        mailbox.clear();
        assert!(mailbox.is_empty());
        assert_eq!(mailbox.len(), 0);
    }

    #[test]
    fn test_mailbox_iter() {
        let mut mailbox = Mailbox::with_limit(10);
        for i in 0..3 {
            mailbox
                .push(MailboxEntry::new("src", "chan", Value::Int(i)))
                .unwrap();
        }

        let values: Vec<i64> = mailbox.iter().filter_map(|e| e.value.as_int()).collect();
        assert_eq!(values, vec![0, 1, 2]);
    }

    #[test]
    fn test_mailbox_overflow_error_display() {
        let error = MailboxOverflowError { limit: 100 };
        assert_eq!(
            format!("{}", error),
            "mailbox overflow: limit of 100 entries exceeded"
        );
    }

    #[test]
    fn test_receive_with_control_arms() {
        let receive = Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![ReceiveArm {
                pattern: Pattern::Variable("msg".to_string()),
                guard: None,
                body: Workflow::Done,
            }],
            control_arms: Some(vec![ReceiveArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Workflow::Done,
            }]),
        };
        assert!(receive.has_wildcard()); // From control_arms
        assert_eq!(receive.arm_count(), 2);
    }
}
