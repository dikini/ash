//! Stream provider trait and registry for event sources
//!
//! Provides the interface for integrating external event streams (like Kafka, sensors,
//! message queues) into Ash workflow execution.
//!
//! # Sendable Stream Providers
//!
//! The [`SendableStreamProvider`] trait extends [`StreamProvider`] with the ability
//! to send values to output streams. See [`MockSendableProvider`] and
//! [`TypedSendableProvider`] for implementations.

use crate::error::{ExecError, ExecResult};
use crate::typed_provider::TypedStreamProvider;
use ash_core::{Name, Value};
use ash_typeck::Type;
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
    providers: HashMap<(Name, Name), TypedStreamProvider>,
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
    pub fn register(&mut self, provider: TypedStreamProvider) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a reference to a registered provider
    ///
    /// Returns `None` if no provider is registered for the given capability and channel.
    pub fn get(&self, cap: &str, channel: &str) -> Option<&TypedStreamProvider> {
        self.providers.get(&(cap.to_string(), channel.to_string()))
    }

    /// Get the type schema for a registered provider
    ///
    /// Returns `None` if no provider is registered for the given capability and channel.
    pub fn get_schema(&self, cap: &str, channel: &str) -> Option<&Type> {
        self.get(cap, channel).map(|p| p.schema())
    }

    /// Check if a provider is registered for the given capability and channel
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers
            .contains_key(&(cap.to_string(), channel.to_string()))
    }

    /// Iterate over all registered providers
    ///
    /// Returns an iterator over ((capability_name, channel_name), provider) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&(Name, Name), &TypedStreamProvider)> {
        self.providers.iter()
    }
}

/// Context for stream operations during execution
///
/// The StreamContext wraps a StreamRegistry and provides convenient methods
/// for receiving values from registered streams. It also manages sendable
/// providers for output operations.
pub struct StreamContext {
    registry: StreamRegistry,
    sendable_registry: SendableRegistry,
    control_messages: Mutex<VecDeque<Value>>,
}

impl StreamContext {
    /// Create a new empty stream context
    pub fn new() -> Self {
        Self {
            registry: StreamRegistry::new(),
            sendable_registry: SendableRegistry::new(),
            control_messages: Mutex::new(VecDeque::new()),
        }
    }

    /// Create a stream context with an existing registry
    pub fn with_registry(registry: StreamRegistry) -> Self {
        Self {
            registry,
            sendable_registry: SendableRegistry::new(),
            control_messages: Mutex::new(VecDeque::new()),
        }
    }

    /// Register a stream provider
    pub fn register(&mut self, provider: TypedStreamProvider) {
        self.registry.register(provider);
    }

    /// Register a sendable stream provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The typed sendable provider to register
    pub fn register_sendable(&mut self, provider: TypedSendableProvider) {
        self.sendable_registry.register(provider);
    }

    /// Get a sendable provider by capability and channel names
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    #[must_use]
    pub fn get_sendable(&self, cap: &str, channel: &str) -> Option<&TypedSendableProvider> {
        self.sendable_registry.get(cap, channel)
    }

    /// Send a value to the specified sendable provider
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    /// * `value` - The value to send
    ///
    /// # Errors
    ///
    /// Returns `ExecError::CapabilityNotAvailable` if no provider is found
    pub async fn send(&self, cap: &str, channel: &str, value: Value) -> ExecResult<()> {
        let provider = self
            .sendable_registry
            .get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{cap}:{channel}")))?;
        provider.send(value).await
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
        self.registry
            .get(cap, channel)
            .map(|p| p as &dyn StreamProvider)
    }

    /// Get the type schema for a registered provider
    ///
    /// Returns `None` if no provider is registered for the given capability and channel.
    pub fn get_schema(&self, cap: &str, channel: &str) -> Option<&Type> {
        self.registry.get_schema(cap, channel)
    }

    /// Iterate over all registered stream providers
    ///
    /// Returns an iterator over references to all registered `TypedStreamProvider`s.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::stream::{StreamContext, MockStreamProvider};
    /// use ash_interp::TypedStreamProvider;
    /// use ash_interp::stream::StreamProvider;
    /// use ash_typeck::Type;
    ///
    /// let mut ctx = StreamContext::new();
    /// ctx.register(TypedStreamProvider::new(
    ///     MockStreamProvider::new("kafka", "orders"),
    ///     Type::Int
    /// ));
    ///
    /// for provider in ctx.iter_providers() {
    ///     println!("Provider: {}:{}", provider.capability_name(), provider.channel_name());
    /// }
    /// ```
    pub fn iter_providers(&self) -> impl Iterator<Item = &TypedStreamProvider> {
        self.registry.iter().map(|(_, provider)| provider)
    }

    /// Try to receive from any available stream (non-blocking)
    ///
    /// Polls all registered providers and returns the first available message.
    /// Returns `Some((capability_name, channel_name, result))` if a message is available,
    /// or `None` if no provider has a message ready.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::stream::{StreamContext, MockStreamProvider, StreamProvider};
    /// use ash_interp::TypedStreamProvider;
    /// use ash_core::Value;
    /// use ash_typeck::Type;
    ///
    /// let mut ctx = StreamContext::new();
    /// let provider = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]);
    /// ctx.register(TypedStreamProvider::new(provider, Type::Int));
    ///
    /// // Receive from any available stream
    /// if let Some((cap, chan, result)) = ctx.try_recv_any() {
    ///     println!("Received from {}:{}: {:?}", cap, chan, result.unwrap());
    /// }
    /// ```
    pub fn try_recv_any(&self) -> Option<(String, String, ExecResult<Value>)> {
        for ((cap, chan), provider) in self.registry.iter() {
            if let Some(result) = provider.try_recv() {
                return Some((cap.clone(), chan.clone(), result));
            }
        }
        None
    }

    /// Push a control message into the implicit control mailbox.
    pub fn push_control(&self, value: Value) {
        let mut control_messages = self.control_messages.lock().unwrap();
        control_messages.push_back(value);
    }

    /// Try to receive a control message without blocking.
    pub fn try_recv_control(&self) -> Option<Value> {
        let mut control_messages = self.control_messages.lock().unwrap();
        control_messages.pop_front()
    }
}

impl Default for StreamContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Sendable stream provider trait for output streams
///
/// This trait extends [`StreamProvider`] with the ability to send values
/// to writable streams (message queues, output channels, etc.). Providers
/// that support both reading and writing should implement this trait.
///
/// # Type Parameters
///
/// The trait requires `StreamProvider` as a supertrait since sendable
/// providers must also be readable.
///
/// # Example
///
/// ```
/// use ash_interp::stream::{StreamProvider, SendableStreamProvider, MockSendableProvider};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockSendableProvider::new("queue", "output");
/// provider.send(Value::Int(42)).await.unwrap();
/// # });
/// ```
#[async_trait]
pub trait SendableStreamProvider: StreamProvider + Send + Sync {
    /// Send a value to this provider's output stream
    ///
    /// # Arguments
    ///
    /// * `value` - The value to send
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if the send operation fails.
    async fn send(&self, value: Value) -> ExecResult<()>;

    /// Check if sending would block (backpressure detection)
    ///
    /// Default implementation returns `false` (never blocks).
    /// Providers can override this to signal backpressure.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::stream::{SendableStreamProvider, MockSendableProvider};
    ///
    /// let provider = MockSendableProvider::new("queue", "output");
    /// assert!(!provider.would_block()); // Default is false
    /// ```
    fn would_block(&self) -> bool {
        false // Default: never blocks
    }

    /// Flush any buffered sends
    ///
    /// Default implementation is a no-op.
    /// Providers with internal buffering should override this
    /// to ensure all buffered values are sent.
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if the flush operation fails.
    async fn flush(&self) -> ExecResult<()> {
        Ok(()) // Default: no-op
    }
}

/// Typed wrapper for [`SendableStreamProvider`] with schema validation
///
/// This struct wraps a sendable stream provider and carries a [`Type`] schema
/// that describes the expected type of values that can be sent. The schema is
/// used for runtime validation before calling the inner provider's `send` method.
///
/// # Example
///
/// ```
/// use ash_interp::stream::{SendableStreamProvider, MockSendableProvider, TypedSendableProvider};
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let inner = MockSendableProvider::new("queue", "output");
/// let typed = TypedSendableProvider::new(inner, Type::Int);
///
/// // Valid value - accepted
/// typed.send(Value::Int(50)).await.unwrap();
///
/// // Invalid value - rejected with type mismatch
/// let result = typed.send(Value::String("invalid".to_string())).await;
/// assert!(result.is_err());
/// # });
/// ```
pub struct TypedSendableProvider {
    inner: Box<dyn SendableStreamProvider>,
    write_schema: Type,
}

impl TypedSendableProvider {
    /// Create a new typed sendable provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The sendable stream provider to wrap
    /// * `write_schema` - The expected type schema for values that can be sent
    ///
    /// # Type Parameters
    ///
    /// * `P` - The concrete provider type, must implement [`SendableStreamProvider`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::stream::{MockSendableProvider, TypedSendableProvider};
    /// use ash_typeck::Type;
    ///
    /// let inner = MockSendableProvider::new("queue", "output");
    /// let typed = TypedSendableProvider::new(inner, Type::Int);
    ///
    /// assert_eq!(typed.write_schema(), &Type::Int);
    /// ```
    pub fn new<P>(provider: P, write_schema: Type) -> Self
    where
        P: SendableStreamProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            write_schema,
        }
    }

    /// Get the write type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values that can be sent on this provider.
    #[must_use]
    pub fn write_schema(&self) -> &Type {
        &self.write_schema
    }

    /// Send a value with schema validation
    ///
    /// Validates the value against the write schema before delegating
    /// to the inner provider's `send` method.
    ///
    /// # Errors
    ///
    /// Returns `ExecError::TypeMismatch` if the value doesn't match the schema.
    pub async fn send(&self, value: Value) -> ExecResult<()> {
        if !self.write_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!(
                    "{}:{}",
                    self.inner.capability_name(),
                    self.inner.channel_name()
                ),
                self.write_schema.to_string(),
                value.to_string(),
            ));
        }

        self.inner.send(value).await
    }
}

#[async_trait]
impl StreamProvider for TypedSendableProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    fn try_recv(&self) -> Option<ExecResult<Value>> {
        self.inner.try_recv()
    }

    async fn recv(&self) -> ExecResult<Value> {
        self.inner.recv().await
    }

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

#[async_trait]
impl SendableStreamProvider for TypedSendableProvider {
    async fn send(&self, value: Value) -> ExecResult<()> {
        // Validate against write_schema before sending
        if !self.write_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.write_schema.to_string(),
                value.to_string(),
            ));
        }
        self.inner.send(value).await
    }

    fn would_block(&self) -> bool {
        self.inner.would_block()
    }

    async fn flush(&self) -> ExecResult<()> {
        self.inner.flush().await
    }
}

/// Mock sendable stream provider for testing
///
/// Stores sent values in a queue that can be inspected for verification.
/// Useful for testing output-capable stream workflows without external dependencies.
///
/// # Example
///
/// ```
/// use ash_interp::stream::{SendableStreamProvider, MockSendableProvider};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockSendableProvider::new("queue", "output");
///
/// // Send values
/// provider.send(Value::Int(1)).await.unwrap();
/// provider.send(Value::Int(2)).await.unwrap();
///
/// // Verify sent values
/// let sent = provider.sent_values();
/// assert_eq!(sent.len(), 2);
/// assert_eq!(sent[0], Value::Int(1));
/// assert_eq!(sent[1], Value::Int(2));
/// # });
/// ```
#[derive(Clone)]
pub struct MockSendableProvider {
    capability: String,
    channel: String,
    sent: std::sync::Arc<Mutex<VecDeque<Value>>>,
    would_block: std::sync::Arc<AtomicBool>,
}

impl MockSendableProvider {
    /// Create a new mock sendable provider for the given capability and channel
    #[must_use]
    pub fn new(capability: &str, channel: &str) -> Self {
        Self {
            capability: capability.to_string(),
            channel: channel.to_string(),
            sent: std::sync::Arc::new(Mutex::new(VecDeque::new())),
            would_block: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a copy of all sent values
    #[must_use]
    pub fn sent_values(&self) -> Vec<Value> {
        let sent = self.sent.lock().unwrap();
        sent.iter().cloned().collect()
    }

    /// Get the number of sent values
    #[must_use]
    pub fn sent_count(&self) -> usize {
        let sent = self.sent.lock().unwrap();
        sent.len()
    }

    /// Clear all sent values
    pub fn clear_sent(&self) {
        let mut sent = self.sent.lock().unwrap();
        sent.clear();
    }

    /// Set whether the provider would block
    pub fn set_would_block(&self, would_block: bool) {
        self.would_block.store(would_block, Ordering::SeqCst);
    }
}

#[async_trait]
impl StreamProvider for MockSendableProvider {
    fn capability_name(&self) -> &str {
        &self.capability
    }

    fn channel_name(&self) -> &str {
        &self.channel
    }

    fn try_recv(&self) -> Option<ExecResult<Value>> {
        // Mock sendable provider doesn't support receiving
        None
    }

    async fn recv(&self) -> ExecResult<Value> {
        // Mock sendable provider doesn't support receiving
        Err(ExecError::ExecutionFailed(
            "mock sendable provider does not support recv".to_string(),
        ))
    }

    fn is_closed(&self) -> bool {
        false
    }
}

#[async_trait]
impl SendableStreamProvider for MockSendableProvider {
    async fn send(&self, value: Value) -> ExecResult<()> {
        let mut sent = self.sent.lock().unwrap();
        sent.push_back(value);
        Ok(())
    }

    fn would_block(&self) -> bool {
        self.would_block.load(Ordering::SeqCst)
    }
}

/// Registry of sendable stream providers indexed by capability and channel names
///
/// Similar to [`StreamRegistry`] but specifically for sendable providers.
#[derive(Default)]
pub struct SendableRegistry {
    providers: HashMap<(Name, Name), TypedSendableProvider>,
}

impl SendableRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a sendable provider
    ///
    /// The provider is indexed by its capability_name and channel_name.
    /// If a provider with the same names already exists, it is replaced.
    pub fn register(&mut self, provider: TypedSendableProvider) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a provider by capability and channel names
    #[must_use]
    pub fn get(&self, cap: &str, channel: &str) -> Option<&TypedSendableProvider> {
        self.providers.get(&(cap.to_string(), channel.to_string()))
    }

    /// Check if a provider exists for the given capability and channel
    #[must_use]
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers
            .contains_key(&(cap.to_string(), channel.to_string()))
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
            // In a real implementation, this would wait for a value.
            // For the mock, classify the empty-open-stream case as blocked rather than failed.
            Err(crate::error::ExecError::Blocked(
                "no value available".to_string(),
            ))
        }
    }

    fn is_closed(&self) -> bool {
        // Check if explicitly closed, or if queue is empty and closed flag is set
        self.closed.load(Ordering::SeqCst)
    }
}

/// Bidirectional stream trait for internal implementations
///
/// This trait combines both receiving and sending operations for
/// bidirectional stream providers. It serves as the underlying implementation
/// for [`BidirectionalStreamProvider`].
#[async_trait]
pub trait BidirectionalStream: Send + Sync {
    /// Returns the capability name for this provider
    fn capability_name(&self) -> &str;

    /// Returns the channel name for this provider
    fn channel_name(&self) -> &str;

    /// Try to receive a value without blocking
    fn try_recv(&self) -> Option<ExecResult<Value>>;

    /// Block until a value is available
    async fn recv(&self) -> ExecResult<Value>;

    /// Check if the stream is closed
    fn is_closed(&self) -> bool;

    /// Send a value to this provider's output stream
    async fn send(&self, value: Value) -> ExecResult<()>;

    /// Check if sending would block
    fn would_block(&self) -> bool {
        false
    }

    /// Flush any buffered sends
    async fn flush(&self) -> ExecResult<()> {
        Ok(())
    }
}

/// Bidirectional stream provider wrapper with separate read/write schemas
///
/// This struct wraps a [`BidirectionalStream`] implementation and provides
/// both [`StreamProvider`] and [`SendableStreamProvider`] implementations
/// with separate type schemas for receiving and sending operations.
///
/// # Example
///
/// ```
/// use ash_interp::stream::{BidirectionalStreamProvider, MockBidirectionalStream};
/// use ash_interp::stream::{StreamProvider, SendableStreamProvider};
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let inner = MockBidirectionalStream::new("queue", "events");
/// inner.push(Value::Int(1));
/// inner.push(Value::Int(2));
/// let provider = BidirectionalStreamProvider::new(
///     inner,
///     Type::Int,  // read schema
///     Type::Int   // write schema
/// );
///
/// // Can receive and send
/// let value = provider.recv().await.unwrap();
/// assert_eq!(value, Value::Int(1));
/// provider.send(Value::Int(100)).await.unwrap();
/// # });
/// ```
pub struct BidirectionalStreamProvider {
    inner: Box<dyn BidirectionalStream>,
    read_schema: Type,
    write_schema: Type,
}

impl BidirectionalStreamProvider {
    /// Create a new bidirectional stream provider
    ///
    /// # Arguments
    ///
    /// * `inner` - The bidirectional stream implementation to wrap
    /// * `read_schema` - The expected type schema for values received
    /// * `write_schema` - The expected type schema for values sent
    ///
    /// # Type Parameters
    ///
    /// * `B` - The concrete stream type, must implement [`BidirectionalStream`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::stream::{BidirectionalStreamProvider, MockBidirectionalStream};
    /// use ash_typeck::Type;
    ///
    /// let inner = MockBidirectionalStream::new("queue", "messages");
    /// let provider = BidirectionalStreamProvider::new(
    ///     inner,
    ///     Type::String,
    ///     Type::String
    /// );
    ///
    /// assert_eq!(provider.read_schema(), &Type::String);
    /// assert_eq!(provider.write_schema(), &Type::String);
    /// ```
    pub fn new<B>(inner: B, read_schema: Type, write_schema: Type) -> Self
    where
        B: BidirectionalStream + 'static,
    {
        Self {
            inner: Box::new(inner),
            read_schema,
            write_schema,
        }
    }

    /// Get the read type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values received from this provider.
    #[must_use]
    pub fn read_schema(&self) -> &Type {
        &self.read_schema
    }

    /// Get the write type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values sent to this provider.
    #[must_use]
    pub fn write_schema(&self) -> &Type {
        &self.write_schema
    }

    /// Send a value with schema validation
    ///
    /// Validates the value against the write schema before delegating
    /// to the inner provider's `send` method.
    ///
    /// # Errors
    ///
    /// Returns `ExecError::TypeMismatch` if the value doesn't match the schema.
    pub async fn send(&self, value: Value) -> ExecResult<()> {
        if !self.write_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.write_schema.to_string(),
                value.to_string(),
            ));
        }

        self.inner.send(value).await
    }
}

#[async_trait]
impl StreamProvider for BidirectionalStreamProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    fn try_recv(&self) -> Option<ExecResult<Value>> {
        self.inner.try_recv().map(|result| {
            result.and_then(|value| {
                if self.read_schema.matches(&value) {
                    Ok(value)
                } else {
                    Err(ExecError::type_mismatch(
                        format!("{}:{}", self.capability_name(), self.channel_name()),
                        self.read_schema.to_string(),
                        value.to_string(),
                    ))
                }
            })
        })
    }

    async fn recv(&self) -> ExecResult<Value> {
        let value = self.inner.recv().await?;

        if !self.read_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.read_schema.to_string(),
                value.to_string(),
            ));
        }

        Ok(value)
    }

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

#[async_trait]
impl SendableStreamProvider for BidirectionalStreamProvider {
    async fn send(&self, value: Value) -> ExecResult<()> {
        // Validate against write_schema before sending
        if !self.write_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.write_schema.to_string(),
                value.to_string(),
            ));
        }
        self.inner.send(value).await
    }

    fn would_block(&self) -> bool {
        self.inner.would_block()
    }

    async fn flush(&self) -> ExecResult<()> {
        self.inner.flush().await
    }
}

/// Mock bidirectional stream provider for testing
///
/// Stores values in a queue for receiving and tracks sent values.
/// Supports both read and write operations for bidirectional testing.
///
/// # Example
///
/// ```
/// use ash_interp::stream::{BidirectionalStream, MockBidirectionalStream};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockBidirectionalStream::new("queue", "messages");
///
/// // Push values for receiving
/// provider.push(Value::Int(1));
/// provider.push(Value::Int(2));
///
/// // Receive values
/// let v1 = provider.recv().await.unwrap();
/// assert_eq!(v1, Value::Int(1));
///
/// // Send values
/// provider.send(Value::Int(100)).await.unwrap();
///
/// // Check sent values
/// assert_eq!(provider.sent_count(), 1);
/// # });
/// ```
#[derive(Clone)]
pub struct MockBidirectionalStream {
    capability: String,
    channel: String,
    values: std::sync::Arc<Mutex<VecDeque<Value>>>,
    sent: std::sync::Arc<Mutex<VecDeque<Value>>>,
    closed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    would_block: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl MockBidirectionalStream {
    /// Create a new mock bidirectional stream provider
    #[must_use]
    pub fn new(capability: &str, channel: &str) -> Self {
        Self {
            capability: capability.to_string(),
            channel: channel.to_string(),
            values: std::sync::Arc::new(Mutex::new(VecDeque::new())),
            sent: std::sync::Arc::new(Mutex::new(VecDeque::new())),
            closed: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            would_block: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Create a mock provider with initial receive values
    #[must_use]
    pub fn with_values(capability: &str, channel: &str, values: Vec<Value>) -> Self {
        Self {
            capability: capability.to_string(),
            channel: channel.to_string(),
            values: std::sync::Arc::new(Mutex::new(values.into_iter().collect())),
            sent: std::sync::Arc::new(Mutex::new(VecDeque::new())),
            closed: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            would_block: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Add a value to the receive queue
    pub fn push(&self, value: Value) {
        let mut values = self.values.lock().unwrap();
        values.push_back(value);
    }

    /// Close the stream
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    /// Get a copy of all sent values
    #[must_use]
    pub fn sent_values(&self) -> Vec<Value> {
        let sent = self.sent.lock().unwrap();
        sent.iter().cloned().collect()
    }

    /// Get the number of sent values
    #[must_use]
    pub fn sent_count(&self) -> usize {
        let sent = self.sent.lock().unwrap();
        sent.len()
    }

    /// Clear all sent values
    pub fn clear_sent(&self) {
        let mut sent = self.sent.lock().unwrap();
        sent.clear();
    }

    /// Set whether the provider would block
    pub fn set_would_block(&self, would_block: bool) {
        self.would_block.store(would_block, Ordering::SeqCst);
    }
}

#[async_trait]
impl BidirectionalStream for MockBidirectionalStream {
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
        let mut values = self.values.lock().unwrap();
        if let Some(value) = values.pop_front() {
            Ok(value)
        } else if self.is_closed() {
            Err(ExecError::ExecutionFailed("stream is closed".to_string()))
        } else {
            Err(ExecError::ExecutionFailed("no value available".to_string()))
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    async fn send(&self, value: Value) -> ExecResult<()> {
        let mut sent = self.sent.lock().unwrap();
        sent.push_back(value);
        Ok(())
    }

    fn would_block(&self) -> bool {
        self.would_block.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_typeck::Type;

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

        registry.register(TypedStreamProvider::new(provider, Type::Int));

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
    fn test_registry_stores_typed_provider() {
        let mut registry = StreamRegistry::new();
        let provider = MockStreamProvider::new("kafka", "orders");
        let typed = TypedStreamProvider::new(provider, Type::Int);

        registry.register(typed);

        assert!(registry.has("kafka", "orders"));
        let retrieved = registry.get("kafka", "orders");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().schema(), &Type::Int);
    }

    #[test]
    fn test_registry_get_schema() {
        let mut registry = StreamRegistry::new();
        let provider = MockStreamProvider::new("kafka", "events");
        let typed = TypedStreamProvider::new(
            provider,
            Type::Record(vec![
                (Box::from("event_id"), Type::String),
                (Box::from("timestamp"), Type::Time),
            ]),
        );

        registry.register(typed);

        let schema = registry.get_schema("kafka", "events");
        assert!(schema.is_some());
        assert!(matches!(schema.unwrap(), Type::Record(_)));
    }

    #[test]
    fn test_stream_context_try_recv() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]);
        ctx.register(TypedStreamProvider::new(provider, Type::Int));

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
        ctx.register(TypedStreamProvider::new(provider, Type::Int));

        // Can receive values through context
        let v1 = ctx.recv("sensor", "temp").await.unwrap().unwrap();
        assert_eq!(v1, Value::Int(100));

        let v2 = ctx.recv("sensor", "temp").await.unwrap().unwrap();
        assert_eq!(v2, Value::Int(200));

        // Non-existent stream returns None
        assert!(ctx.recv("nonexistent", "channel").await.is_none());
    }

    // ============================================================
    // Sendable Stream Provider Tests (TASK-102)
    // ============================================================

    #[tokio::test]
    async fn test_send_accepts_value() {
        let provider = MockSendableProvider::new("queue", "output");

        // Send values
        provider.send(Value::Int(42)).await.unwrap();
        provider
            .send(Value::String("hello".to_string()))
            .await
            .unwrap();

        // Verify sent values
        let sent = provider.sent_values();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0], Value::Int(42));
        assert_eq!(sent[1], Value::String("hello".to_string()));
    }

    #[test]
    fn test_would_block_default() {
        let provider = MockSendableProvider::new("queue", "output");

        // Default should_block is false
        assert!(!provider.would_block());
    }

    #[test]
    fn test_would_block_can_be_set() {
        let provider = MockSendableProvider::new("queue", "output");

        // Initially false
        assert!(!provider.would_block());

        // Set to true
        provider.set_would_block(true);
        assert!(provider.would_block());

        // Set back to false
        provider.set_would_block(false);
        assert!(!provider.would_block());
    }

    #[tokio::test]
    async fn test_mock_sendable_sent_count() {
        let provider = MockSendableProvider::new("queue", "output");

        // Initially empty
        assert_eq!(provider.sent_count(), 0);

        // Send values
        provider.send(Value::Int(1)).await.unwrap();
        assert_eq!(provider.sent_count(), 1);

        provider.send(Value::Int(2)).await.unwrap();
        assert_eq!(provider.sent_count(), 2);

        // Clear
        provider.clear_sent();
        assert_eq!(provider.sent_count(), 0);
    }

    #[tokio::test]
    async fn test_typed_sendable_validation() {
        let inner = MockSendableProvider::new("queue", "output");
        let typed = TypedSendableProvider::new(inner, Type::Int);

        // Valid value - should succeed
        typed.send(Value::Int(50)).await.unwrap();

        // Invalid type - should fail with type mismatch
        let result = typed.send(Value::String("not a number".to_string())).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("queue:output"));
    }

    #[tokio::test]
    async fn test_typed_sendable_with_complex_type() {
        use std::collections::HashMap;

        let record_type = Type::Record(vec![
            (Box::from("id"), Type::Int),
            (Box::from("name"), Type::String),
        ]);

        let inner = MockSendableProvider::new("queue", "events");
        let typed = TypedSendableProvider::new(inner, record_type);

        // Valid record - should succeed
        let mut valid_map = HashMap::new();
        valid_map.insert("id".to_string(), Value::Int(1));
        valid_map.insert("name".to_string(), Value::String("test".to_string()));
        let valid_record = Value::Record(Box::new(valid_map));
        typed.send(valid_record).await.unwrap();

        // Invalid record - wrong field type
        let mut invalid_map = HashMap::new();
        invalid_map.insert("id".to_string(), Value::String("not an int".to_string()));
        invalid_map.insert("name".to_string(), Value::String("test".to_string()));
        let invalid_record = Value::Record(Box::new(invalid_map));
        let result = typed.send(invalid_record).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sendable_registry() {
        let mut registry = SendableRegistry::new();
        let provider = MockSendableProvider::new("queue", "output");

        registry.register(TypedSendableProvider::new(provider, Type::Int));

        assert!(registry.has("queue", "output"));
        assert!(!registry.has("queue", "other"));
        assert!(!registry.has("other", "output"));
    }

    #[tokio::test]
    async fn test_stream_context_sendable() {
        let mut ctx = StreamContext::new();
        let mock = MockSendableProvider::new("queue", "output");

        ctx.register_sendable(TypedSendableProvider::new(mock.clone(), Type::Int));

        // Send a value
        ctx.send("queue", "output", Value::Int(42)).await.unwrap();

        // Verify through the mock directly
        assert_eq!(mock.sent_count(), 1);
        assert_eq!(mock.sent_values()[0], Value::Int(42));
    }

    #[tokio::test]
    async fn test_stream_context_sendable_not_found() {
        let ctx = StreamContext::new();

        let result = ctx.send("nonexistent", "channel", Value::Int(42)).await;
        assert!(matches!(
            result,
            Err(ExecError::CapabilityNotAvailable(name)) if name == "nonexistent:channel"
        ));
    }

    #[test]
    fn test_typed_sendable_provider_schema() {
        let inner = MockSendableProvider::new("queue", "output");
        let typed = TypedSendableProvider::new(inner, Type::Int);

        assert_eq!(typed.write_schema(), &Type::Int);
        assert_eq!(typed.capability_name(), "queue");
        assert_eq!(typed.channel_name(), "output");
    }

    #[tokio::test]
    async fn test_typed_sendable_flush_delegates() {
        let inner = MockSendableProvider::new("queue", "output");
        let typed = TypedSendableProvider::new(inner, Type::Int);

        // Flush should succeed (default no-op)
        typed.flush().await.unwrap();
    }

    #[tokio::test]
    async fn test_sendable_provider_stream_trait_methods() {
        // MockSendableProvider implements StreamProvider with minimal support
        let provider = MockSendableProvider::new("queue", "output");

        // These methods are available through StreamProvider supertrait
        assert_eq!(provider.capability_name(), "queue");
        assert_eq!(provider.channel_name(), "output");
        assert!(!provider.is_closed());

        // try_recv returns None (not supported)
        assert!(provider.try_recv().is_none());

        // recv returns error (not supported)
        let result = provider.recv().await;
        assert!(result.is_err());
    }

    // ============================================================
    // Bidirectional Stream Provider Tests (TASK-107)
    // ============================================================

    #[tokio::test]
    async fn test_bidirectional_stream_recv_and_send() {
        let inner = MockBidirectionalStream::with_values(
            "queue",
            "events",
            vec![Value::Int(1), Value::Int(2)],
        );
        let provider = BidirectionalStreamProvider::new(inner.clone(), Type::Int, Type::Int);

        // Receive values
        let v1 = provider.recv().await.unwrap();
        assert_eq!(v1, Value::Int(1));

        let v2 = provider.recv().await.unwrap();
        assert_eq!(v2, Value::Int(2));

        // Send values
        provider.send(Value::Int(100)).await.unwrap();
        provider.send(Value::Int(200)).await.unwrap();

        // Verify sent values through mock directly
        assert_eq!(inner.sent_count(), 2);
    }

    #[tokio::test]
    async fn test_bidirectional_stream_read_schema_validation() {
        let inner = MockBidirectionalStream::with_values("queue", "events", vec![Value::Int(42)]);
        let provider = BidirectionalStreamProvider::new(inner, Type::Int, Type::String);

        // Read with valid type should succeed
        let value = provider.recv().await.unwrap();
        assert_eq!(value, Value::Int(42));

        // Create provider with mismatched read schema
        let inner2 = MockBidirectionalStream::with_values(
            "queue",
            "events",
            vec![Value::String("test".to_string())],
        );
        let provider2 = BidirectionalStreamProvider::new(inner2, Type::Int, Type::String);

        // Read should fail due to schema mismatch
        let result = provider2.recv().await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("queue:events"));
    }

    #[tokio::test]
    async fn test_bidirectional_stream_write_schema_validation() {
        let inner = MockBidirectionalStream::new("queue", "output");
        let provider = BidirectionalStreamProvider::new(inner, Type::String, Type::Int);

        // Write with valid type should succeed
        provider.send(Value::Int(50)).await.unwrap();

        // Write with invalid type should fail
        let result = provider.send(Value::String("invalid".to_string())).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("queue:output"));
    }

    #[tokio::test]
    async fn test_bidirectional_stream_try_recv_validation() {
        let inner = MockBidirectionalStream::with_values("queue", "events", vec![Value::Int(42)]);
        let provider = BidirectionalStreamProvider::new(inner, Type::Int, Type::String);

        // Valid value
        let result = provider.try_recv().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));

        // No more values
        assert!(provider.try_recv().is_none());
    }

    #[tokio::test]
    async fn test_bidirectional_stream_try_recv_validation_fails() {
        let inner = MockBidirectionalStream::with_values(
            "queue",
            "events",
            vec![Value::String("not an int".to_string())],
        );
        let provider = BidirectionalStreamProvider::new(inner, Type::Int, Type::String);

        // Invalid value should return error (not None)
        let result = provider.try_recv().unwrap();
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
    }

    #[tokio::test]
    async fn test_bidirectional_stream_separate_schemas() {
        // Different read and write schemas
        let inner = MockBidirectionalStream::with_values(
            "queue",
            "events",
            vec![Value::String("received".to_string())],
        );
        let provider = BidirectionalStreamProvider::new(
            inner,
            Type::String, // read schema
            Type::Int,    // write schema (different!)
        );

        // Verify schemas are different
        assert_eq!(provider.read_schema(), &Type::String);
        assert_eq!(provider.write_schema(), &Type::Int);

        // Read validates against read_schema
        let value = provider.recv().await.unwrap();
        assert_eq!(value, Value::String("received".to_string()));

        // Write validates against write_schema
        provider.send(Value::Int(123)).await.unwrap();

        // Write with wrong type for write_schema should fail
        let result = provider.send(Value::String("wrong".to_string())).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("type mismatch"));
    }

    #[tokio::test]
    async fn test_mock_bidirectional_stream_tracking() {
        let provider = MockBidirectionalStream::with_values("queue", "events", vec![Value::Int(1)]);

        // Initially has one receive value, no sent values
        assert_eq!(provider.sent_count(), 0);

        // Send values
        provider.send(Value::Int(10)).await.unwrap();
        provider.send(Value::Int(20)).await.unwrap();
        assert_eq!(provider.sent_count(), 2);

        // Verify sent values
        let sent = provider.sent_values();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0], Value::Int(10));
        assert_eq!(sent[1], Value::Int(20));

        // Clear sent
        provider.clear_sent();
        assert_eq!(provider.sent_count(), 0);
    }

    #[tokio::test]
    async fn test_bidirectional_stream_provider_implements_both_traits() {
        use crate::stream::StreamProvider;

        let inner = MockBidirectionalStream::with_values("queue", "events", vec![Value::Int(1)]);
        let provider = BidirectionalStreamProvider::new(inner, Type::Int, Type::Int);

        // Should implement StreamProvider
        assert_eq!(provider.capability_name(), "queue");
        assert_eq!(provider.channel_name(), "events");

        // Should be able to recv
        let value = provider.recv().await.unwrap();
        assert_eq!(value, Value::Int(1));

        // Should implement SendableStreamProvider
        provider.send(Value::Int(100)).await.unwrap();
    }

    #[tokio::test]
    async fn test_bidirectional_stream_flush_delegates() {
        let inner = MockBidirectionalStream::new("queue", "output");
        let provider = BidirectionalStreamProvider::new(inner, Type::Int, Type::Int);

        // Flush should succeed (default no-op)
        provider.flush().await.unwrap();
    }

    #[test]
    fn test_bidirectional_stream_is_closed() {
        let inner = MockBidirectionalStream::with_values("queue", "events", vec![Value::Int(1)]);
        let provider = BidirectionalStreamProvider::new(inner.clone(), Type::Int, Type::Int);

        // Initially not closed
        assert!(!provider.is_closed());

        // Close the stream (through inner mock directly)
        inner.close();

        // Now closed
        assert!(provider.is_closed());

        // try_recv returns None when closed
        assert!(provider.try_recv().is_none());
    }

    #[test]
    fn test_bidirectional_stream_would_block() {
        let inner = MockBidirectionalStream::new("queue", "output");
        let provider = BidirectionalStreamProvider::new(inner.clone(), Type::Int, Type::Int);

        // Initially not blocking
        assert!(!provider.would_block());

        // Set blocking through inner mock directly
        inner.set_would_block(true);
        assert!(provider.would_block());
    }

    #[tokio::test]
    async fn test_mock_bidirectional_stream_push_and_recv() {
        let provider = MockBidirectionalStream::new("queue", "messages");

        // Push values
        provider.push(Value::Int(1));
        provider.push(Value::Int(2));
        provider.push(Value::Int(3));

        // Receive in order
        assert_eq!(provider.recv().await.unwrap(), Value::Int(1));
        assert_eq!(provider.recv().await.unwrap(), Value::Int(2));
        assert_eq!(provider.try_recv().unwrap().unwrap(), Value::Int(3));

        // No more values
        assert!(provider.try_recv().is_none());
    }

    // ============================================================
    // Stream Iteration Tests (Stream Iteration Task)
    // ============================================================

    #[test]
    fn test_stream_registry_iter() {
        let mut registry = StreamRegistry::new();
        let provider1 = MockStreamProvider::new("kafka", "orders");
        let provider2 = MockStreamProvider::new("sensor", "temp");

        registry.register(TypedStreamProvider::new(provider1, Type::Int));
        registry.register(TypedStreamProvider::new(provider2, Type::String));

        // Collect all providers from iterator
        let providers: Vec<_> = registry.iter().collect();
        assert_eq!(providers.len(), 2);

        // Verify we can access both providers
        let caps: std::collections::HashSet<_> =
            providers.iter().map(|((cap, _), _)| cap.as_str()).collect();
        assert!(caps.contains("kafka"));
        assert!(caps.contains("sensor"));
    }

    #[test]
    fn test_stream_context_iter_providers() {
        let mut ctx = StreamContext::new();

        // Register multiple providers
        let provider1 = MockStreamProvider::new("kafka", "orders");
        let provider2 = MockStreamProvider::new("sensor", "temp");
        let provider3 = MockStreamProvider::new("api", "events");

        ctx.register(TypedStreamProvider::new(provider1, Type::Int));
        ctx.register(TypedStreamProvider::new(provider2, Type::Bool));
        ctx.register(TypedStreamProvider::new(provider3, Type::String));

        // Iterate over all providers
        let providers: Vec<_> = ctx.iter_providers().collect();
        assert_eq!(providers.len(), 3);

        // Collect capability names
        let caps: std::collections::HashSet<_> =
            providers.iter().map(|p| p.capability_name()).collect();
        assert!(caps.contains("kafka"));
        assert!(caps.contains("sensor"));
        assert!(caps.contains("api"));
    }

    #[test]
    fn test_stream_context_try_recv_any_single_provider() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]);
        ctx.register(TypedStreamProvider::new(provider, Type::Int));

        // Should receive from the only provider
        let result = ctx.try_recv_any();
        assert!(result.is_some());

        let (cap, chan, value_result) = result.unwrap();
        assert_eq!(cap, "sensor");
        assert_eq!(chan, "temp");
        assert_eq!(value_result.unwrap(), Value::Int(42));

        // No more messages
        assert!(ctx.try_recv_any().is_none());
    }

    #[test]
    fn test_stream_context_try_recv_any_multiple_providers() {
        let mut ctx = StreamContext::new();

        // Register multiple providers with values
        let provider1 = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1)]);
        let provider2 = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(2)]);

        ctx.register(TypedStreamProvider::new(provider1, Type::Int));
        ctx.register(TypedStreamProvider::new(provider2, Type::Int));

        // Collect results from both providers (order is non-deterministic)
        let mut received_values = Vec::new();
        let mut received_providers = std::collections::HashSet::new();

        for _ in 0..2 {
            let result = ctx.try_recv_any();
            assert!(result.is_some());
            let (cap, chan, value) = result.unwrap();
            received_values.push(value.unwrap());
            received_providers.insert((cap, chan));
        }

        // Verify we got both values
        assert!(received_values.contains(&Value::Int(1)));
        assert!(received_values.contains(&Value::Int(2)));

        // Verify we got both different providers
        assert_eq!(received_providers.len(), 2);

        // No more messages
        assert!(ctx.try_recv_any().is_none());
    }

    #[test]
    fn test_stream_context_try_recv_any_empty() {
        let ctx = StreamContext::new();

        // No providers registered - should return None
        assert!(ctx.try_recv_any().is_none());
    }

    #[test]
    fn test_stream_context_try_recv_all_closed() {
        let mut ctx = StreamContext::new();
        let provider = MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]);
        provider.close();
        ctx.register(TypedStreamProvider::new(provider, Type::Int));

        // Closed provider should return None
        assert!(ctx.try_recv_any().is_none());
    }

    #[test]
    fn test_stream_context_iter_providers_empty() {
        let ctx = StreamContext::new();

        // No providers registered - iterator should be empty
        let count = ctx.iter_providers().count();
        assert_eq!(count, 0);
    }
}
