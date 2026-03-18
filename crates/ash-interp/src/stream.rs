//! Stream provider trait and registry for event sources
//!
//! Provides the interface for integrating external event streams (like Kafka, sensors,
//! message queues) into Ash workflow execution.

use crate::error::ExecResult;
use ash_core::{Name, Value};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Stream provider trait for event sources
///
/// Implement this trait to provide a stream of values from an external source.
/// Each provider is identified by a capability name and channel name pair.
#[async_trait]
pub trait StreamProvider: Send + Sync {
    /// Get the capability name for this provider
    fn capability_name(&self) -> &str;

    /// Get the channel name for this provider
    fn channel_name(&self) -> &str;

    /// Try to receive a value without blocking
    ///
    /// Returns `None` if no value is available or the stream is closed.
    fn try_recv(&self) -> Option<ExecResult<Value>>;

    /// Block until a value is available
    ///
    /// This is an async operation that waits until a value can be received.
    /// For the mock implementation, this immediately returns if a value is available.
    async fn recv(&self) -> ExecResult<Value>;

    /// Check if the stream is closed
    ///
    /// A closed stream will not produce any more values.
    fn is_closed(&self) -> bool;
}

/// Registry of stream providers indexed by capability and channel names
///
/// The registry stores providers in a HashMap keyed by (capability_name, channel_name)
/// pairs, allowing efficient lookup during workflow execution.
#[derive(Default)]
pub struct StreamRegistry {
    providers: HashMap<(Name, Name), Box<dyn StreamProvider>>,
}

impl StreamRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a stream provider
    ///
    /// The provider is indexed by its (capability_name, channel_name) pair.
    pub fn register(&mut self, provider: Box<dyn StreamProvider>) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a reference to a registered provider
    ///
    /// Returns `None` if no provider is registered for the given capability and channel.
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn StreamProvider> {
        self.providers
            .get(&(cap.to_string(), channel.to_string()))
            .map(|p| p.as_ref())
    }

    /// Check if a provider is registered for the given capability and channel
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers
            .contains_key(&(cap.to_string(), channel.to_string()))
    }
}

/// Context for stream operations during execution
///
/// The StreamContext wraps a StreamRegistry and provides convenient methods
/// for receiving values from registered streams.
pub struct StreamContext {
    registry: StreamRegistry,
}

impl StreamContext {
    /// Create a new empty stream context
    pub fn new() -> Self {
        Self {
            registry: StreamRegistry::new(),
        }
    }

    /// Create a stream context with an existing registry
    pub fn with_registry(registry: StreamRegistry) -> Self {
        Self { registry }
    }

    /// Register a stream provider
    pub fn register(&mut self, provider: Box<dyn StreamProvider>) {
        self.registry.register(provider);
    }

    /// Try to receive a value from a stream without blocking
    ///
    /// Returns `None` if the stream doesn't exist, is closed, or has no values available.
    pub fn try_recv(&self, cap: &str, channel: &str) -> Option<ExecResult<Value>> {
        self.registry.get(cap, channel)?.try_recv()
    }

    /// Receive a value from a stream, blocking until one is available
    ///
    /// Returns an error if the stream doesn't exist or is closed with no pending values.
    pub async fn recv(&self, cap: &str, channel: &str) -> Option<ExecResult<Value>> {
        self.registry.get(cap, channel)?.recv().await.into()
    }

    /// Check if a stream is closed
    ///
    /// Returns `None` if the stream doesn't exist, `Some(true)` if closed,
    /// and `Some(false)` if open.
    pub fn is_closed(&self, cap: &str, channel: &str) -> Option<bool> {
        self.registry.get(cap, channel).map(|p| p.is_closed())
    }

    /// Get a reference to a registered provider
    ///
    /// Returns `None` if no provider is registered for the given capability and channel.
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn StreamProvider> {
        self.registry.get(cap, channel)
    }
}

impl Default for StreamContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock stream provider for testing
///
/// Stores values in a queue and returns them in order. Can be closed to
/// simulate stream termination.
pub struct MockStreamProvider {
    capability: String,
    channel: String,
    values: Mutex<VecDeque<Value>>,
    closed: AtomicBool,
}

impl MockStreamProvider {
    /// Create a new mock provider for the given capability and channel
    pub fn new(capability: &str, channel: &str) -> Self {
        Self {
            capability: capability.to_string(),
            channel: channel.to_string(),
            values: Mutex::new(VecDeque::new()),
            closed: AtomicBool::new(false),
        }
    }

    /// Create a mock provider with initial values
    pub fn with_values(capability: &str, channel: &str, values: Vec<Value>) -> Self {
        Self {
            capability: capability.to_string(),
            channel: channel.to_string(),
            values: Mutex::new(values.into_iter().collect()),
            closed: AtomicBool::new(false),
        }
    }

    /// Add a value to the queue
    pub fn push(&self, value: Value) {
        let mut values = self.values.lock().unwrap();
        values.push_back(value);
    }

    /// Close the stream
    ///
    /// Once closed, the stream will return `None` from `try_recv` and
    /// will error from `recv` when empty.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
impl StreamProvider for MockStreamProvider {
    fn capability_name(&self) -> &str {
        &self.capability
    }

    fn channel_name(&self) -> &str {
        &self.channel
    }

    fn try_recv(&self) -> Option<ExecResult<Value>> {
        if self.is_closed() {
            return None;
        }

        let mut values = self.values.lock().unwrap();
        values.pop_front().map(Ok)
    }

    async fn recv(&self) -> ExecResult<Value> {
        // For the mock, we just return the next value if available
        // In a real implementation, this would wait for values
        let mut values = self.values.lock().unwrap();
        if let Some(value) = values.pop_front() {
            Ok(value)
        } else if self.is_closed() {
            Err(crate::error::ExecError::ExecutionFailed(
                "stream is closed".to_string(),
            ))
        } else {
            // In a real implementation, this would wait for a value
            // For the mock, we return an error indicating no value available
            Err(crate::error::ExecError::ExecutionFailed(
                "no value available".to_string(),
            ))
        }
    }

    fn is_closed(&self) -> bool {
        // Check if explicitly closed, or if queue is empty and closed flag is set
        self.closed.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_recv() {
        let provider =
            MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1), Value::Int(2)]);

        let v1 = provider.recv().await.unwrap();
        assert_eq!(v1, Value::Int(1));

        let v2 = provider.recv().await.unwrap();
        assert_eq!(v2, Value::Int(2));
    }

    #[test]
    fn test_mock_provider_try_recv() {
        let provider =
            MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1), Value::Int(2)]);

        // First value should be available
        let v1 = provider.try_recv().unwrap().unwrap();
        assert_eq!(v1, Value::Int(1));

        // Second value should be available
        let v2 = provider.try_recv().unwrap().unwrap();
        assert_eq!(v2, Value::Int(2));

        // No more values - should return None
        assert!(provider.try_recv().is_none());
    }

    #[test]
    fn test_mock_provider_close() {
        let provider = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1)]);

        // Initially not closed
        assert!(!provider.is_closed());

        // Can still receive values
        assert!(provider.try_recv().is_some());

        // Close the stream
        provider.close();
        assert!(provider.is_closed());

        // After close, try_recv returns None even if we add values
        provider.push(Value::Int(2));
        assert!(provider.try_recv().is_none());
    }

    #[test]
    fn test_stream_registry() {
        let mut registry = StreamRegistry::new();
        let provider = MockStreamProvider::new("kafka", "orders");

        registry.register(Box::new(provider));

        assert!(registry.has("kafka", "orders"));
        assert!(!registry.has("kafka", "metrics"));
        assert!(!registry.has("redis", "orders"));

        // Can retrieve the provider
        let retrieved = registry.get("kafka", "orders");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().capability_name(), "kafka");
        assert_eq!(retrieved.unwrap().channel_name(), "orders");
    }

    #[test]
    fn test_stream_context_try_recv() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]);
        ctx.register(Box::new(provider));

        // Can receive value through context
        let value = ctx.try_recv("sensor", "temp").unwrap().unwrap();
        assert_eq!(value, Value::Int(42));

        // No more values
        assert!(ctx.try_recv("sensor", "temp").is_none());

        // Non-existent stream
        assert!(ctx.try_recv("sensor", "pressure").is_none());
    }

    #[tokio::test]
    async fn test_stream_context_recv() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::with_values(
            "sensor",
            "temp",
            vec![Value::Int(100), Value::Int(200)],
        );
        ctx.register(Box::new(provider));

        // Can receive values through context
        let v1 = ctx.recv("sensor", "temp").await.unwrap().unwrap();
        assert_eq!(v1, Value::Int(100));

        let v2 = ctx.recv("sensor", "temp").await.unwrap().unwrap();
        assert_eq!(v2, Value::Int(200));

        // Non-existent stream returns None
        assert!(ctx.recv("nonexistent", "channel").await.is_none());
    }
}
