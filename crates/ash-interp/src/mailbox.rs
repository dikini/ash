//! Runtime mailbox implementation for stream message buffering
//!
//! This module provides a thread-safe mailbox for buffering stream messages
//! during workflow execution, with configurable size limits and overflow strategies.

pub use ash_core::stream::{MailboxEntry, MailboxOverflowError, OverflowStrategy};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Runtime mailbox for buffering stream messages
///
/// A bounded buffer that stores messages with configurable overflow
/// behavior when the limit is reached. This is the runtime counterpart
/// to the static [`ash_core::stream::Mailbox`] type.
///
/// # Examples
///
/// ```
/// use ash_interp::mailbox::{Mailbox, OverflowStrategy};
/// use ash_core::stream::MailboxEntry;
/// use ash_core::Value;
///
/// // Create a mailbox with default limit and strategy
/// let mut mailbox = Mailbox::new();
///
/// // Push entries
/// mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1))).unwrap();
///
/// // Pop entries (FIFO order)
/// let entry = mailbox.pop().unwrap();
/// assert_eq!(entry.value, Value::Int(1));
/// ```
#[derive(Debug, Clone)]
pub struct Mailbox {
    entries: VecDeque<MailboxEntry>,
    limit: usize,
    overflow: OverflowStrategy,
}

impl Mailbox {
    /// Default mailbox size limit
    pub const DEFAULT_LIMIT: usize = 1000;

    /// Creates a new mailbox with default limit (1000) and DropOldest strategy
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    ///
    /// let mailbox = Mailbox::new();
    /// assert!(mailbox.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::with_limit(Self::DEFAULT_LIMIT, OverflowStrategy::DropOldest)
    }

    /// Creates a new mailbox with specified limit and overflow strategy
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of entries to store
    /// * `overflow` - Strategy when limit is exceeded
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::{Mailbox, OverflowStrategy};
    ///
    /// let mailbox = Mailbox::with_limit(100, OverflowStrategy::DropNewest);
    /// assert!(mailbox.is_empty());
    /// ```
    pub fn with_limit(limit: usize, overflow: OverflowStrategy) -> Self {
        Self {
            entries: VecDeque::with_capacity(limit.min(1024)),
            limit,
            overflow,
        }
    }

    /// Pushes an entry into the mailbox
    ///
    /// The behavior when the mailbox is full depends on the overflow strategy:
    /// - `DropOldest`: Removes the oldest entry and adds the new one
    /// - `DropNewest`: Discards the new entry (returns Ok)
    /// - `Error`: Returns an error
    ///
    /// # Errors
    ///
    /// Returns `Err(MailboxOverflowError)` if the overflow strategy is `Error`
    /// and the mailbox is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// let entry = MailboxEntry::new("kafka", "orders", Value::Int(42));
    ///
    /// mailbox.push(entry).unwrap();
    /// assert_eq!(mailbox.len(), 1);
    /// ```
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

    /// Pops the oldest entry from the mailbox (FIFO order)
    ///
    /// Returns `None` if the mailbox is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(1))).unwrap();
    ///
    /// let entry = mailbox.pop().unwrap();
    /// assert_eq!(entry.value, Value::Int(1));
    /// ```
    pub fn pop(&mut self) -> Option<MailboxEntry> {
        self.entries.pop_front()
    }

    /// Returns a reference to the oldest entry without removing it
    ///
    /// Returns `None` if the mailbox is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(1))).unwrap();
    ///
    /// let entry = mailbox.peek().unwrap();
    /// assert_eq!(entry.value, Value::Int(1));
    /// assert_eq!(mailbox.len(), 1); // Still there
    /// ```
    pub fn peek(&self) -> Option<&MailboxEntry> {
        self.entries.front()
    }

    /// Returns the number of entries in the mailbox
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// assert_eq!(mailbox.len(), 0);
    ///
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(1))).unwrap();
    /// assert_eq!(mailbox.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the mailbox is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    ///
    /// let mailbox = Mailbox::new();
    /// assert!(mailbox.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears all entries from the mailbox
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(1))).unwrap();
    /// mailbox.clear();
    ///
    /// assert!(mailbox.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Finds the first entry matching the given predicate
    ///
    /// Returns `Some(&MailboxEntry)` if a match is found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1))).unwrap();
    /// mailbox.push(MailboxEntry::new("sensor", "temp", Value::Int(2))).unwrap();
    ///
    /// let found = mailbox.find_matching(|e| e.source() == "kafka");
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().value, Value::Int(1));
    /// ```
    pub fn find_matching<F>(&self, predicate: F) -> Option<&MailboxEntry>
    where
        F: Fn(&MailboxEntry) -> bool,
    {
        self.entries.iter().find(|e| predicate(e))
    }

    /// Removes and returns the first entry matching the given predicate
    ///
    /// Returns `Some(MailboxEntry)` if a match is found and removed, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("kafka", "orders", Value::Int(1))).unwrap();
    /// mailbox.push(MailboxEntry::new("sensor", "temp", Value::Int(2))).unwrap();
    ///
    /// let removed = mailbox.remove_matching(|e| e.source() == "kafka");
    /// assert!(removed.is_some());
    /// assert_eq!(mailbox.len(), 1);
    /// ```
    pub fn remove_matching<F>(&mut self, predicate: F) -> Option<MailboxEntry>
    where
        F: Fn(&MailboxEntry) -> bool,
    {
        if let Some(pos) = self.entries.iter().position(predicate) {
            return self.entries.remove(pos);
        }
        None
    }

    /// Returns an iterator over the entries (oldest first)
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_interp::mailbox::Mailbox;
    /// use ash_core::stream::MailboxEntry;
    /// use ash_core::Value;
    ///
    /// let mut mailbox = Mailbox::new();
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(1))).unwrap();
    /// mailbox.push(MailboxEntry::new("a", "b", Value::Int(2))).unwrap();
    ///
    /// let values: Vec<i64> = mailbox.iter().filter_map(|e| e.value.as_int()).collect();
    /// assert_eq!(values, vec![1, 2]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &MailboxEntry> {
        self.entries.iter()
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared mailbox type
///
/// This type alias provides a convenient way to share a mailbox across
/// async tasks. Use [`tokio::sync::Mutex`] to ensure safe concurrent access.
///
/// # Examples
///
/// ```
/// use ash_interp::mailbox::{Mailbox, SharedMailbox};
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// let mailbox: SharedMailbox = Arc::new(Mutex::new(Mailbox::new()));
/// ```
pub type SharedMailbox = Arc<Mutex<Mailbox>>;

/// Errors that can occur during mailbox operations
///
/// This enum represents runtime errors that can occur when working
/// with mailboxes in stream operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MailboxError {
    /// The mailbox has reached its capacity limit
    #[error("mailbox overflow")]
    Overflow,

    /// The mailbox has been closed and cannot accept new entries
    #[error("mailbox closed")]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Value;
    use ash_core::stream::MailboxEntry;

    #[test]
    fn test_mailbox_push_and_pop() {
        let mut mailbox = Mailbox::new();
        let entry = MailboxEntry::new("kafka", "orders", Value::Int(1));

        mailbox.push(entry.clone()).unwrap();
        assert_eq!(mailbox.len(), 1);

        let popped = mailbox.pop().unwrap();
        assert_eq!(popped.value, Value::Int(1));
        assert_eq!(mailbox.len(), 0);
    }

    #[test]
    fn test_mailbox_drop_oldest() {
        let mut mailbox = Mailbox::with_limit(3, OverflowStrategy::DropOldest);

        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(1)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(2)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(3)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(4)))
            .unwrap();

        assert_eq!(mailbox.len(), 3);
        // Oldest (1) should be dropped
        assert_eq!(mailbox.pop().unwrap().value, Value::Int(2));
    }

    #[test]
    fn test_mailbox_drop_newest() {
        let mut mailbox = Mailbox::with_limit(3, OverflowStrategy::DropNewest);

        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(1)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(2)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(3)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(4)))
            .unwrap();

        assert_eq!(mailbox.len(), 3);
        // Newest (4) should be dropped, so we get 1, 2, 3
        assert_eq!(mailbox.pop().unwrap().value, Value::Int(1));
        assert_eq!(mailbox.pop().unwrap().value, Value::Int(2));
        assert_eq!(mailbox.pop().unwrap().value, Value::Int(3));
        assert!(mailbox.pop().is_none());
    }

    #[test]
    fn test_mailbox_error_strategy() {
        let mut mailbox = Mailbox::with_limit(3, OverflowStrategy::Error);

        // Fill to capacity
        for i in 0..3 {
            assert!(
                mailbox
                    .push(MailboxEntry::new("a", "b", Value::Int(i)))
                    .is_ok()
            );
        }
        assert_eq!(mailbox.len(), 3);

        // Next push should error
        let result = mailbox.push(MailboxEntry::new("a", "b", Value::Int(99)));
        assert!(result.is_err());
        assert_eq!(mailbox.len(), 3); // Unchanged
    }

    #[test]
    fn test_mailbox_find_matching() {
        let mut mailbox = Mailbox::new();
        mailbox
            .push(MailboxEntry::new("kafka", "orders", Value::Int(1)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("sensor", "temp", Value::Int(2)))
            .unwrap();

        let found = mailbox.find_matching(|e| e.source() == "kafka");
        assert!(found.is_some());
        assert_eq!(found.unwrap().value, Value::Int(1));

        let not_found = mailbox.find_matching(|e| e.source() == "rabbitmq");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_mailbox_remove_matching() {
        let mut mailbox = Mailbox::new();
        mailbox
            .push(MailboxEntry::new("kafka", "orders", Value::Int(1)))
            .unwrap();
        mailbox
            .push(MailboxEntry::new("sensor", "temp", Value::Int(2)))
            .unwrap();

        let removed = mailbox.remove_matching(|e| e.source() == "kafka");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, Value::Int(1));
        assert_eq!(mailbox.len(), 1);

        // Verify remaining entry
        let remaining = mailbox.pop().unwrap();
        assert_eq!(remaining.value, Value::Int(2));
    }

    #[tokio::test]
    async fn test_mailbox_thread_safe() {
        let mailbox: SharedMailbox = Arc::new(Mutex::new(Mailbox::new()));

        let m1 = mailbox.clone();
        let t1 = tokio::spawn(async move {
            for i in 0..100 {
                m1.lock()
                    .await
                    .push(MailboxEntry::new("a", "b", Value::Int(i)))
                    .unwrap();
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

    #[test]
    fn test_mailbox_peek() {
        let mut mailbox = Mailbox::new();
        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(42)))
            .unwrap();

        let peeked = mailbox.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().value, Value::Int(42));
        assert_eq!(mailbox.len(), 1); // Still there
    }

    #[test]
    fn test_mailbox_clear() {
        let mut mailbox = Mailbox::new();
        for i in 0..5 {
            mailbox
                .push(MailboxEntry::new("a", "b", Value::Int(i)))
                .unwrap();
        }
        assert_eq!(mailbox.len(), 5);

        mailbox.clear();
        assert!(mailbox.is_empty());
        assert_eq!(mailbox.len(), 0);
    }

    #[test]
    fn test_mailbox_iter() {
        let mut mailbox = Mailbox::new();
        for i in 0..3 {
            mailbox
                .push(MailboxEntry::new("a", "b", Value::Int(i)))
                .unwrap();
        }

        let values: Vec<i64> = mailbox.iter().filter_map(|e| e.value.as_int()).collect();
        assert_eq!(values, vec![0, 1, 2]);
    }

    #[test]
    fn test_mailbox_is_empty() {
        let mut mailbox = Mailbox::new();
        assert!(mailbox.is_empty());

        mailbox
            .push(MailboxEntry::new("a", "b", Value::Int(1)))
            .unwrap();
        assert!(!mailbox.is_empty());

        mailbox.pop();
        assert!(mailbox.is_empty());
    }

    #[test]
    fn test_mailbox_default() {
        let mailbox: Mailbox = Default::default();
        assert!(mailbox.is_empty());
        assert_eq!(mailbox.len(), 0);
    }
}
