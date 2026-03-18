//! Behaviour provider trait and registry for sampling observable values
//!
//! This module provides the core abstraction for behaviour providers that can
//! be sampled to observe values from external sources (sensors, databases, etc.).
//!
//! # Settable Providers
//!
//! The [`SettableBehaviourProvider`] trait extends [`BehaviourProvider`] with
//! the ability to set values on output channels. See [`MockSettableProvider`]
//! and [`TypedSettableProvider`] for implementations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use ash_core::{Constraint, Name, Value};
use ash_typeck::Type;

use crate::error::{ExecError, ExecResult, ValidationError};
use crate::typed_provider::TypedBehaviourProvider;

/// Behaviour provider trait for sampling observable values
///
/// Providers implement this trait to expose observable values from external
/// sources such as sensors, databases, or other data streams.
#[async_trait]
pub trait BehaviourProvider: Send + Sync {
    /// Returns the capability name for this provider
    fn capability_name(&self) -> &str;

    /// Returns the channel name for this provider
    fn channel_name(&self) -> &str;

    /// Sample the current value with optional constraints
    ///
    /// # Arguments
    ///
    /// * `constraints` - Optional filtering constraints for the sample
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if sampling fails
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value>;

    /// Check if value has changed since last sample
    ///
    /// Default implementation always returns `true`.
    /// Providers can override this for optimization.
    ///
    /// # Arguments
    ///
    /// * `constraints` - Optional filtering constraints
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if the check fails
    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        Ok(true)
    }
}

/// Registry of behaviour providers indexed by capability and channel names
///
/// The registry stores providers in a HashMap keyed by (capability_name, channel_name)
/// for efficient lookup during execution.
#[derive(Default)]
pub struct BehaviourRegistry {
    providers: HashMap<(Name, Name), TypedBehaviourProvider>,
}

impl BehaviourRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a behaviour provider
    ///
    /// The provider is indexed by its capability_name and channel_name.
    /// If a provider with the same names already exists, it is replaced.
    pub fn register(&mut self, provider: TypedBehaviourProvider) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a provider by capability and channel names
    #[must_use]
    pub fn get(&self, cap: &str, channel: &str) -> Option<&TypedBehaviourProvider> {
        self.providers.get(&(cap.to_string(), channel.to_string()))
    }

    /// Get the type schema for a provider
    #[must_use]
    pub fn get_schema(&self, cap: &str, channel: &str) -> Option<&Type> {
        self.get(cap, channel).map(|p| p.schema())
    }

    /// Check if a provider exists for the given capability and channel
    #[must_use]
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers
            .contains_key(&(cap.to_string(), channel.to_string()))
    }
}

/// Context for behaviour sampling during workflow execution
///
/// The `BehaviourContext` wraps a `BehaviourRegistry` and provides
/// high-level methods for sampling values and checking for changes.
/// It also manages settable providers for output operations.
pub struct BehaviourContext {
    registry: BehaviourRegistry,
    settable_registry: SettableRegistry,
}

impl BehaviourContext {
    /// Create a new behaviour context with empty registries
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: BehaviourRegistry::new(),
            settable_registry: SettableRegistry::new(),
        }
    }

    /// Create a context with an existing registry
    #[must_use]
    pub fn with_registry(registry: BehaviourRegistry) -> Self {
        Self {
            registry,
            settable_registry: SettableRegistry::new(),
        }
    }

    /// Register a behaviour provider
    pub fn register(&mut self, provider: TypedBehaviourProvider) {
        self.registry.register(provider);
    }

    /// Register a settable behaviour provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The typed settable provider to register
    pub fn register_settable(&mut self, provider: TypedSettableProvider) {
        self.settable_registry.register(provider);
    }

    /// Get a settable provider by capability and channel names
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    #[must_use]
    pub fn get_settable(&self, cap: &str, channel: &str) -> Option<&TypedSettableProvider> {
        self.settable_registry.get(cap, channel)
    }

    /// Set a value on the specified settable provider
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    /// * `value` - The value to set
    ///
    /// # Errors
    ///
    /// Returns `ExecError::CapabilityNotAvailable` if no provider is found
    pub async fn set(&self, cap: &str, channel: &str, value: Value) -> ExecResult<()> {
        let provider = self
            .settable_registry
            .get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{cap}:{channel}")))?;
        provider.set(value).await
    }

    /// Sample a value from the specified provider
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    /// * `constraints` - Optional filtering constraints
    ///
    /// # Errors
    ///
    /// Returns `ExecError::CapabilityNotAvailable` if no provider is found
    pub async fn sample(
        &self,
        cap: &str,
        channel: &str,
        constraints: &[Constraint],
    ) -> ExecResult<Value> {
        let provider = self
            .registry
            .get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{cap}:{channel}")))?;
        provider.sample(constraints).await
    }

    /// Check if the value has changed since last sample
    ///
    /// # Arguments
    ///
    /// * `cap` - Capability name
    /// * `channel` - Channel name
    /// * `constraints` - Optional filtering constraints
    ///
    /// # Errors
    ///
    /// Returns `ExecError::CapabilityNotAvailable` if no provider is found
    pub async fn has_changed(
        &self,
        cap: &str,
        channel: &str,
        constraints: &[Constraint],
    ) -> ExecResult<bool> {
        let provider = self
            .registry
            .get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{cap}:{channel}")))?;
        provider.has_changed(constraints).await
    }
}

impl Default for BehaviourContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock behaviour provider for testing
///
/// Stores a value in a `Mutex` and tracks the last sampled value.
/// Useful for testing behaviour-dependent workflows without external dependencies.
#[derive(Clone)]
pub struct MockBehaviourProvider {
    name: (String, String),
    value: std::sync::Arc<Mutex<Value>>,
    last_value: std::sync::Arc<Mutex<Option<Value>>>,
}

impl MockBehaviourProvider {
    /// Create a new mock provider with the given capability and channel names
    #[must_use]
    pub fn new(cap: &str, channel: &str) -> Self {
        Self {
            name: (cap.to_string(), channel.to_string()),
            value: std::sync::Arc::new(Mutex::new(Value::Null)),
            last_value: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    /// Set the initial value and return self (builder pattern)
    #[must_use]
    pub fn with_value(self, value: Value) -> Self {
        *self.value.lock().unwrap() = value;
        self
    }

    /// Update the stored value
    pub fn set_value(&self, value: Value) {
        *self.value.lock().unwrap() = value;
    }
}

#[async_trait]
impl BehaviourProvider for MockBehaviourProvider {
    fn capability_name(&self) -> &str {
        &self.name.0
    }

    fn channel_name(&self) -> &str {
        &self.name.1
    }

    async fn sample(&self, _constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.value.lock().unwrap().clone();
        *self.last_value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }

    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        let current = self.value.lock().unwrap().clone();
        let last = self.last_value.lock().unwrap().clone();

        match last {
            None => Ok(true), // Never sampled before
            Some(last_val) => Ok(current != last_val),
        }
    }
}

/// Settable behaviour provider trait for output channels
///
/// This trait extends [`BehaviourProvider`] with the ability to set values
/// on writable channels (actuators, output devices, etc.). Providers that
/// implement both reading and writing should implement this trait.
///
/// # Type Parameters
///
/// The trait requires `BehaviourProvider` as a supertrait since settable
/// providers must also be readable.
///
/// # Example
///
/// ```
/// use ash_interp::behaviour::{BehaviourProvider, SettableBehaviourProvider, MockSettableProvider};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockSettableProvider::new("actuator", "led");
/// provider.set(Value::Bool(true)).await.unwrap();
/// let value = provider.sample(&[]).await.unwrap();
/// assert_eq!(value, Value::Bool(true));
/// # });
/// ```
#[async_trait]
pub trait SettableBehaviourProvider: BehaviourProvider + Send + Sync {
    /// Set a value on this provider's output channel
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if the set operation fails, including validation errors.
    async fn set(&self, value: Value) -> ExecResult<()>;

    /// Validate a value before setting
    ///
    /// Default implementation accepts all values. Providers can override this
    /// to perform pre-checks on values before they are set.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if the value is invalid for this provider.
    fn validate(&self, _value: &Value) -> Result<(), ValidationError> {
        Ok(()) // Default: accept all
    }
}

/// Typed wrapper for [`SettableBehaviourProvider`] with schema validation
///
/// This struct wraps a settable behaviour provider and carries a [`Type`] schema
/// that describes the expected type of values that can be set. The schema is
/// used for runtime validation before calling the inner provider's `set` method.
///
/// # Example
///
/// ```
/// use ash_interp::behaviour::{SettableBehaviourProvider, MockSettableProvider, TypedSettableProvider};
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let inner = MockSettableProvider::new("actuator", "brightness");
/// let typed = TypedSettableProvider::new(inner, Type::Int);
///
/// // Valid value - accepted
/// typed.set(Value::Int(50)).await.unwrap();
///
/// // Invalid value - rejected with type mismatch
/// let result = typed.set(Value::String("invalid".to_string())).await;
/// assert!(result.is_err());
/// # });
/// ```
pub struct TypedSettableProvider {
    inner: Box<dyn SettableBehaviourProvider>,
    write_schema: Type,
}

impl TypedSettableProvider {
    /// Create a new typed settable provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The settable behaviour provider to wrap
    /// * `write_schema` - The expected type schema for values that can be set
    ///
    /// # Type Parameters
    ///
    /// * `P` - The concrete provider type, must implement [`SettableBehaviourProvider`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::behaviour::{MockSettableProvider, TypedSettableProvider};
    /// use ash_typeck::Type;
    ///
    /// let inner = MockSettableProvider::new("actuator", "brightness");
    /// let typed = TypedSettableProvider::new(inner, Type::Int);
    ///
    /// assert_eq!(typed.write_schema(), &Type::Int);
    /// ```
    pub fn new<P>(provider: P, write_schema: Type) -> Self
    where
        P: SettableBehaviourProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            write_schema,
        }
    }

    /// Get the write type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values that can be set on this provider.
    #[must_use]
    pub fn write_schema(&self) -> &Type {
        &self.write_schema
    }

    /// Set a value with schema validation
    ///
    /// Validates the value against the write schema before delegating
    /// to the inner provider's `set` method.
    ///
    /// # Errors
    ///
    /// Returns `ExecError::TypeMismatch` if the value doesn't match the schema.
    pub async fn set(&self, value: Value) -> ExecResult<()> {
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

        self.inner.set(value).await
    }
}

#[async_trait]
impl BehaviourProvider for TypedSettableProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        self.inner.sample(constraints).await
    }

    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        self.inner.has_changed(constraints).await
    }
}

#[async_trait]
impl SettableBehaviourProvider for TypedSettableProvider {
    async fn set(&self, value: Value) -> ExecResult<()> {
        self.inner.set(value).await
    }

    fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        self.inner.validate(value)
    }
}

/// Mock settable behaviour provider for testing
///
/// Stores a value in a `Mutex` that can be both sampled and set.
/// Useful for testing output-capable workflows without external dependencies.
///
/// # Example
///
/// ```
/// use ash_interp::behaviour::{BehaviourProvider, SettableBehaviourProvider, MockSettableProvider};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockSettableProvider::new("actuator", "led");
///
/// // Set a value
/// provider.set(Value::Bool(true)).await.unwrap();
///
/// // Sample the value back
/// let value = provider.sample(&[]).await.unwrap();
/// assert_eq!(value, Value::Bool(true));
/// # });
/// ```
/// Type alias for the validator function to reduce complexity
pub type ValueValidator = Box<dyn Fn(&Value) -> Result<(), ValidationError> + Send + Sync>;

pub struct MockSettableProvider {
    name: (String, String),
    value: std::sync::Arc<Mutex<Value>>,
    last_value: std::sync::Arc<Mutex<Option<Value>>>,
    validator: Option<ValueValidator>,
}

impl Clone for MockSettableProvider {
    fn clone(&self) -> Self {
        // Clone the current value state, but create new validator (it can't be cloned)
        Self {
            name: self.name.clone(),
            value: std::sync::Arc::new(Mutex::new(self.value.lock().unwrap().clone())),
            last_value: std::sync::Arc::new(Mutex::new(self.last_value.lock().unwrap().clone())),
            validator: None, // Validator can't be cloned, tests can re-add if needed
        }
    }
}

impl MockSettableProvider {
    /// Create a new mock settable provider with the given capability and channel names
    #[must_use]
    pub fn new(cap: &str, channel: &str) -> Self {
        Self {
            name: (cap.to_string(), channel.to_string()),
            value: std::sync::Arc::new(Mutex::new(Value::Null)),
            last_value: std::sync::Arc::new(Mutex::new(None)),
            validator: None,
        }
    }

    /// Set the initial value and return self (builder pattern)
    #[must_use]
    pub fn with_value(self, value: Value) -> Self {
        *self.value.lock().unwrap() = value;
        self
    }

    /// Set a validation function and return self (builder pattern)
    #[must_use]
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }
}

#[async_trait]
impl BehaviourProvider for MockSettableProvider {
    fn capability_name(&self) -> &str {
        &self.name.0
    }

    fn channel_name(&self) -> &str {
        &self.name.1
    }

    async fn sample(&self, _constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.value.lock().unwrap().clone();
        *self.last_value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }

    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        let current = self.value.lock().unwrap().clone();
        let last = self.last_value.lock().unwrap().clone();

        match last {
            None => Ok(true), // Never sampled before
            Some(last_val) => Ok(current != last_val),
        }
    }
}

#[async_trait]
impl SettableBehaviourProvider for MockSettableProvider {
    async fn set(&self, value: Value) -> ExecResult<()> {
        // Run validation first
        self.validate(&value)
            .map_err(|e| ExecError::ValidationFailed(e.to_string()))?;

        *self.value.lock().unwrap() = value;
        Ok(())
    }

    fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        if let Some(ref validator) = self.validator {
            validator(value)
        } else {
            Ok(())
        }
    }
}

/// Registry of settable behaviour providers indexed by capability and channel names
///
/// Similar to [`BehaviourRegistry`] but specifically for settable providers.
#[derive(Default)]
pub struct SettableRegistry {
    providers: HashMap<(Name, Name), TypedSettableProvider>,
}

impl SettableRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a settable provider
    ///
    /// The provider is indexed by its capability_name and channel_name.
    /// If a provider with the same names already exists, it is replaced.
    pub fn register(&mut self, provider: TypedSettableProvider) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a provider by capability and channel names
    #[must_use]
    pub fn get(&self, cap: &str, channel: &str) -> Option<&TypedSettableProvider> {
        self.providers.get(&(cap.to_string(), channel.to_string()))
    }

    /// Check if a provider exists for the given capability and channel
    #[must_use]
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers
            .contains_key(&(cap.to_string(), channel.to_string()))
    }
}

/// Bidirectional behaviour trait for internal implementations
///
/// This trait combines both reading (sample) and writing (set) operations
/// for bidirectional providers. It serves as the underlying implementation
/// for [`BidirectionalBehaviourProvider`].
#[async_trait]
pub trait BidirectionalBehaviour: Send + Sync {
    /// Returns the capability name for this provider
    fn capability_name(&self) -> &str;

    /// Returns the channel name for this provider
    fn channel_name(&self) -> &str;

    /// Sample the current value with optional constraints
    ///
    /// # Arguments
    ///
    /// * `constraints` - Optional filtering constraints for the sample
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if sampling fails
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value>;

    /// Set a value on this provider's output channel
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set
    ///
    /// # Errors
    ///
    /// Returns `ExecError` if the set operation fails
    async fn set(&self, value: Value) -> ExecResult<()>;

    /// Check if value has changed since last sample
    ///
    /// Default implementation always returns `true`.
    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        Ok(true)
    }
}

/// Bidirectional behaviour provider wrapper with separate read/write schemas
///
/// This struct wraps a [`BidirectionalBehaviour`] implementation and provides
/// both [`BehaviourProvider`] and [`SettableBehaviourProvider`] implementations
/// with separate type schemas for reading (sample) and writing (set) operations.
///
/// # Example
///
/// ```
/// use ash_interp::behaviour::{BidirectionalBehaviourProvider, MockBidirectionalProvider};
/// use ash_interp::behaviour::{BehaviourProvider, SettableBehaviourProvider};
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let inner = MockBidirectionalProvider::new("device", "setting");
/// let provider = BidirectionalBehaviourProvider::new(
///     inner,
///     Type::Int,  // read schema
///     Type::Int   // write schema
/// );
///
/// // Can sample (read) and set (write)
/// provider.set(Value::Int(42)).await.unwrap();
/// let value = provider.sample(&[]).await.unwrap();
/// assert_eq!(value, Value::Int(42));
/// # });
/// ```
pub struct BidirectionalBehaviourProvider {
    inner: Box<dyn BidirectionalBehaviour>,
    read_schema: Type,
    write_schema: Type,
}

impl BidirectionalBehaviourProvider {
    /// Create a new bidirectional behaviour provider
    ///
    /// # Arguments
    ///
    /// * `inner` - The bidirectional behaviour implementation to wrap
    /// * `read_schema` - The expected type schema for values returned by sample
    /// * `write_schema` - The expected type schema for values accepted by set
    ///
    /// # Type Parameters
    ///
    /// * `B` - The concrete behaviour type, must implement [`BidirectionalBehaviour`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::behaviour::{BidirectionalBehaviourProvider, MockBidirectionalProvider};
    /// use ash_typeck::Type;
    ///
    /// let inner = MockBidirectionalProvider::new("device", "setting");
    /// let provider = BidirectionalBehaviourProvider::new(
    ///     inner,
    ///     Type::Int,
    ///     Type::Int
    /// );
    ///
    /// assert_eq!(provider.read_schema(), &Type::Int);
    /// assert_eq!(provider.write_schema(), &Type::Int);
    /// ```
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

    /// Get the read type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values returned by sample.
    #[must_use]
    pub fn read_schema(&self) -> &Type {
        &self.read_schema
    }

    /// Get the write type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values accepted by set.
    #[must_use]
    pub fn write_schema(&self) -> &Type {
        &self.write_schema
    }
}

#[async_trait]
impl BehaviourProvider for BidirectionalBehaviourProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.inner.sample(constraints).await?;

        // Validate against read_schema
        if !self.read_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.read_schema.to_string(),
                value.to_string(),
            ));
        }

        Ok(value)
    }

    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        self.inner.has_changed(constraints).await
    }
}

#[async_trait]
impl SettableBehaviourProvider for BidirectionalBehaviourProvider {
    async fn set(&self, value: Value) -> ExecResult<()> {
        // Validate against write_schema
        if !self.write_schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.write_schema.to_string(),
                value.to_string(),
            ));
        }

        self.inner.set(value).await
    }

    fn validate(&self, _value: &Value) -> Result<(), ValidationError> {
        // Validation is done in set() with type checking
        Ok(())
    }
}

/// Mock bidirectional provider for testing
///
/// Stores a value in a `Mutex` that can be both sampled and set.
/// Tracks both read and write operations for verification.
///
/// # Example
///
/// ```
/// use ash_interp::behaviour::{BidirectionalBehaviour, MockBidirectionalProvider};
/// use ash_core::Value;
///
/// # tokio_test::block_on(async {
/// let provider = MockBidirectionalProvider::new("device", "setting");
///
/// // Set a value
/// provider.set(Value::Int(42)).await.unwrap();
///
/// // Sample the value back
/// let value = provider.sample(&[]).await.unwrap();
/// assert_eq!(value, Value::Int(42));
///
/// // Check tracking
/// assert_eq!(provider.read_count(), 1);
/// assert_eq!(provider.write_count(), 1);
/// # });
/// ```
#[derive(Clone)]
pub struct MockBidirectionalProvider {
    name: (String, String),
    value: std::sync::Arc<Mutex<Value>>,
    last_value: std::sync::Arc<Mutex<Option<Value>>>,
    read_count: std::sync::Arc<Mutex<usize>>,
    write_count: std::sync::Arc<Mutex<usize>>,
}

impl MockBidirectionalProvider {
    /// Create a new mock bidirectional provider with the given capability and channel names
    #[must_use]
    pub fn new(cap: &str, channel: &str) -> Self {
        Self {
            name: (cap.to_string(), channel.to_string()),
            value: std::sync::Arc::new(Mutex::new(Value::Null)),
            last_value: std::sync::Arc::new(Mutex::new(None)),
            read_count: std::sync::Arc::new(Mutex::new(0)),
            write_count: std::sync::Arc::new(Mutex::new(0)),
        }
    }

    /// Set the initial value and return self (builder pattern)
    #[must_use]
    pub fn with_value(self, value: Value) -> Self {
        *self.value.lock().unwrap() = value;
        self
    }

    /// Get the number of read (sample) operations performed
    #[must_use]
    pub fn read_count(&self) -> usize {
        *self.read_count.lock().unwrap()
    }

    /// Get the number of write (set) operations performed
    #[must_use]
    pub fn write_count(&self) -> usize {
        *self.write_count.lock().unwrap()
    }

    /// Reset operation counters
    pub fn reset_counts(&self) {
        *self.read_count.lock().unwrap() = 0;
        *self.write_count.lock().unwrap() = 0;
    }
}

#[async_trait]
impl BidirectionalBehaviour for MockBidirectionalProvider {
    fn capability_name(&self) -> &str {
        &self.name.0
    }

    fn channel_name(&self) -> &str {
        &self.name.1
    }

    async fn sample(&self, _constraints: &[Constraint]) -> ExecResult<Value> {
        *self.read_count.lock().unwrap() += 1;
        let value = self.value.lock().unwrap().clone();
        *self.last_value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }

    async fn set(&self, value: Value) -> ExecResult<()> {
        *self.write_count.lock().unwrap() += 1;
        *self.value.lock().unwrap() = value;
        Ok(())
    }

    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        let current = self.value.lock().unwrap().clone();
        let last = self.last_value.lock().unwrap().clone();

        match last {
            None => Ok(true), // Never sampled before
            Some(last_val) => Ok(current != last_val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Expr, Predicate};
    use ash_typeck::Type;

    #[tokio::test]
    async fn test_mock_provider_sample() {
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));

        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn test_provider_with_constraint() {
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(25));

        // Create constraints using the actual Constraint type from ash_core
        let celsius_constraint = Constraint {
            predicate: Predicate {
                name: "unit".to_string(),
                arguments: vec![Expr::Literal(Value::String("celsius".to_string()))],
            },
        };

        let fahrenheit_constraint = Constraint {
            predicate: Predicate {
                name: "unit".to_string(),
                arguments: vec![Expr::Literal(Value::String("fahrenheit".to_string()))],
            },
        };

        // Mock provider ignores constraints, returns same value
        let celsius = provider.sample(&[celsius_constraint]).await.unwrap();
        assert_eq!(celsius, Value::Int(25));

        let fahrenheit = provider.sample(&[fahrenheit_constraint]).await.unwrap();
        assert_eq!(fahrenheit, Value::Int(25));
    }

    #[tokio::test]
    async fn test_has_changed() {
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));

        // First check should report changed (no previous value)
        assert!(provider.has_changed(&[]).await.unwrap());

        // Sample to establish baseline
        let _ = provider.sample(&[]).await;

        // Same value - should report unchanged
        assert!(!provider.has_changed(&[]).await.unwrap());

        // Change value
        provider.set_value(Value::Int(43));

        // Should report changed
        assert!(provider.has_changed(&[]).await.unwrap());
    }

    #[test]
    fn test_behaviour_registry() {
        let mut registry = BehaviourRegistry::new();
        let provider = MockBehaviourProvider::new("sensor", "temp");

        registry.register(TypedBehaviourProvider::new(provider, Type::Int));

        assert!(registry.has("sensor", "temp"));
        assert!(!registry.has("sensor", "pressure"));
    }

    #[test]
    fn test_registry_stores_typed_provider() {
        let mut registry = BehaviourRegistry::new();
        let provider = MockBehaviourProvider::new("sensor", "temp");
        let typed = TypedBehaviourProvider::new(provider, Type::Int);

        registry.register(typed);

        assert!(registry.has("sensor", "temp"));
        let retrieved = registry.get("sensor", "temp");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().schema(), &Type::Int);
    }

    #[test]
    fn test_registry_get_schema() {
        let mut registry = BehaviourRegistry::new();
        let provider = MockBehaviourProvider::new("sensor", "temp");
        let typed = TypedBehaviourProvider::new(
            provider,
            Type::Record(vec![
                (Box::from("value"), Type::Int),
                (Box::from("unit"), Type::String),
            ]),
        );

        registry.register(typed);

        let schema = registry.get_schema("sensor", "temp");
        assert!(schema.is_some());
        assert!(matches!(schema.unwrap(), Type::Record(_)));
    }

    #[tokio::test]
    async fn test_behaviour_context_sample() {
        let mut ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(100));

        ctx.register(TypedBehaviourProvider::new(provider, Type::Int));

        let value = ctx.sample("sensor", "temp", &[]).await.unwrap();
        assert_eq!(value, Value::Int(100));
    }

    #[tokio::test]
    async fn test_behaviour_context_provider_not_found() {
        let ctx = BehaviourContext::new();

        let result = ctx.sample("nonexistent", "channel", &[]).await;
        assert!(matches!(
            result,
            Err(ExecError::CapabilityNotAvailable(name)) if name == "nonexistent:channel"
        ));
    }

    #[tokio::test]
    async fn test_behaviour_context_has_changed() {
        let mut ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(50));

        ctx.register(TypedBehaviourProvider::new(provider, Type::Int));

        // First check should be true (never sampled)
        assert!(ctx.has_changed("sensor", "temp", &[]).await.unwrap());

        // Sample to establish baseline
        let _ = ctx.sample("sensor", "temp", &[]).await;

        // Same value - should report unchanged
        assert!(!ctx.has_changed("sensor", "temp", &[]).await.unwrap());
    }

    // ============================================================
    // Settable Provider Tests (TASK-101)
    // ============================================================

    #[tokio::test]
    async fn test_set_changes_value() {
        let provider = MockSettableProvider::new("actuator", "led");

        // Initially null
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Null);

        // Set a value
        provider.set(Value::Bool(true)).await.unwrap();

        // Sample the new value
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Bool(true));

        // Set another value
        provider.set(Value::Bool(false)).await.unwrap();

        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Bool(false));
    }

    #[tokio::test]
    async fn test_validate_rejects_invalid() {
        let provider = MockSettableProvider::new("actuator", "brightness").with_validator(
            |value| match value {
                Value::Int(n) if (0..=100).contains(n) => Ok(()),
                _ => Err(ValidationError::InvalidValue(
                    "brightness must be 0-100".to_string(),
                )),
            },
        );

        // Valid value - should succeed
        provider.set(Value::Int(50)).await.unwrap();

        // Invalid value - should fail
        let result = provider.set(Value::Int(150)).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("validation failed"));

        // Another invalid value
        let result = provider.set(Value::String("invalid".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_typed_settable_validation() {
        let inner = MockSettableProvider::new("actuator", "brightness");
        let typed = TypedSettableProvider::new(inner, Type::Int);

        // Valid value - should succeed
        typed.set(Value::Int(50)).await.unwrap();

        // Verify the value was set
        let value = typed.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(50));

        // Invalid type - should fail with type mismatch
        let result = typed.set(Value::String("not a number".to_string())).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("actuator:brightness"));
    }

    #[tokio::test]
    async fn test_typed_settable_with_complex_type() {
        use std::collections::HashMap;

        let record_type = Type::Record(vec![
            (Box::from("red"), Type::Int),
            (Box::from("green"), Type::Int),
            (Box::from("blue"), Type::Int),
        ]);

        let inner = MockSettableProvider::new("actuator", "rgb_led");
        let typed = TypedSettableProvider::new(inner, record_type);

        // Valid record - should succeed
        let mut valid_map = HashMap::new();
        valid_map.insert("red".to_string(), Value::Int(255));
        valid_map.insert("green".to_string(), Value::Int(128));
        valid_map.insert("blue".to_string(), Value::Int(0));
        let valid_record = Value::Record(valid_map);
        typed.set(valid_record).await.unwrap();

        // Invalid record - wrong field type
        let mut invalid_map = HashMap::new();
        invalid_map.insert("red".to_string(), Value::String("high".to_string()));
        invalid_map.insert("green".to_string(), Value::Int(128));
        invalid_map.insert("blue".to_string(), Value::Int(0));
        let invalid_record = Value::Record(invalid_map);
        let result = typed.set(invalid_record).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_settable_registry() {
        let mut registry = SettableRegistry::new();
        let provider = MockSettableProvider::new("actuator", "led");

        registry.register(TypedSettableProvider::new(provider, Type::Bool));

        assert!(registry.has("actuator", "led"));
        assert!(!registry.has("actuator", "motor"));
    }

    #[tokio::test]
    async fn test_behaviour_context_settable() {
        let mut ctx = BehaviourContext::new();
        let provider = MockSettableProvider::new("actuator", "led");

        ctx.register_settable(TypedSettableProvider::new(provider, Type::Bool));

        // Set a value
        ctx.set("actuator", "led", Value::Bool(true)).await.unwrap();

        // Get the provider and verify
        let provider = ctx.get_settable("actuator", "led").unwrap();
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[tokio::test]
    async fn test_behaviour_context_settable_not_found() {
        let ctx = BehaviourContext::new();

        let result = ctx.set("nonexistent", "channel", Value::Int(42)).await;
        assert!(matches!(
            result,
            Err(ExecError::CapabilityNotAvailable(name)) if name == "nonexistent:channel"
        ));
    }

    #[test]
    fn test_typed_settable_provider_schema() {
        let inner = MockSettableProvider::new("actuator", "brightness");
        let typed = TypedSettableProvider::new(inner, Type::Int);

        assert_eq!(typed.write_schema(), &Type::Int);
        assert_eq!(typed.capability_name(), "actuator");
        assert_eq!(typed.channel_name(), "brightness");
    }

    // ============================================================
    // Bidirectional Provider Tests (TASK-107)
    // ============================================================

    #[tokio::test]
    async fn test_bidirectional_observe_and_set() {
        let inner = MockBidirectionalProvider::new("device", "setting").with_value(Value::Int(10));
        let provider = BidirectionalBehaviourProvider::new(inner, Type::Int, Type::Int);

        // Initially reads the set value
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(10));

        // Set a new value
        provider.set(Value::Int(42)).await.unwrap();

        // Sample returns the new value
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn test_bidirectional_read_schema_validation() {
        let inner = MockBidirectionalProvider::new("sensor", "temp").with_value(Value::Int(25));
        let provider = BidirectionalBehaviourProvider::new(inner, Type::Int, Type::String);

        // Read should succeed with valid type
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(25));

        // Create a new provider with mismatched read schema
        let inner2 = MockBidirectionalProvider::new("sensor", "temp")
            .with_value(Value::String("hot".to_string()));
        let provider2 = BidirectionalBehaviourProvider::new(inner2, Type::Int, Type::String);

        // Read should fail due to schema mismatch
        let result = provider2.sample(&[]).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("sensor:temp"));
    }

    #[tokio::test]
    async fn test_bidirectional_write_schema_validation() {
        let inner = MockBidirectionalProvider::new("actuator", "brightness");
        let provider = BidirectionalBehaviourProvider::new(inner, Type::Int, Type::Int);

        // Write with valid type should succeed
        provider.set(Value::Int(50)).await.unwrap();

        // Write with invalid type should fail
        let result = provider.set(Value::String("invalid".to_string())).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("actuator:brightness"));
    }

    #[tokio::test]
    async fn test_bidirectional_separate_read_write_schemas() {
        // Different read and write schemas
        let inner = MockBidirectionalProvider::new("converter", "value")
            .with_value(Value::String("read".to_string()));
        let provider = BidirectionalBehaviourProvider::new(
            inner,
            Type::String, // read schema
            Type::Int,    // write schema (different!)
        );

        // Verify schemas are different
        assert_eq!(provider.read_schema(), &Type::String);
        assert_eq!(provider.write_schema(), &Type::Int);

        // Read validates against read_schema
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::String("read".to_string()));

        // Write validates against write_schema
        provider.set(Value::Int(123)).await.unwrap();

        // Write with wrong type for write_schema should fail
        let result = provider.set(Value::String("wrong".to_string())).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("type mismatch"));
    }

    #[tokio::test]
    async fn test_mock_bidirectional_provider_tracking() {
        let provider =
            MockBidirectionalProvider::new("device", "counter").with_value(Value::Int(0));

        // Initially no operations
        assert_eq!(provider.read_count(), 0);
        assert_eq!(provider.write_count(), 0);

        // Perform some reads
        let _ = provider.sample(&[]).await.unwrap();
        let _ = provider.sample(&[]).await.unwrap();
        assert_eq!(provider.read_count(), 2);
        assert_eq!(provider.write_count(), 0);

        // Perform some writes
        provider.set(Value::Int(10)).await.unwrap();
        provider.set(Value::Int(20)).await.unwrap();
        provider.set(Value::Int(30)).await.unwrap();
        assert_eq!(provider.read_count(), 2);
        assert_eq!(provider.write_count(), 3);

        // Reset counters
        provider.reset_counts();
        assert_eq!(provider.read_count(), 0);
        assert_eq!(provider.write_count(), 0);
    }

    #[tokio::test]
    async fn test_bidirectional_provider_implements_both_traits() {
        use crate::behaviour::{BehaviourProvider, SettableBehaviourProvider};

        let inner = MockBidirectionalProvider::new("device", "state");
        let provider = BidirectionalBehaviourProvider::new(inner, Type::Int, Type::Int);

        // Should implement BehaviourProvider
        assert_eq!(provider.capability_name(), "device");
        assert_eq!(provider.channel_name(), "state");

        // Should implement SettableBehaviourProvider
        provider.set(Value::Int(100)).await.unwrap();

        // Should be able to sample
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(100));
    }

    #[tokio::test]
    async fn test_bidirectional_provider_has_changed() {
        let inner = MockBidirectionalProvider::new("sensor", "reading").with_value(Value::Int(10));
        let provider = BidirectionalBehaviourProvider::new(inner, Type::Int, Type::Int);

        // First check should be true (never sampled)
        assert!(provider.has_changed(&[]).await.unwrap());

        // Sample to establish baseline
        let _ = provider.sample(&[]).await;

        // Same value - should report unchanged
        assert!(!provider.has_changed(&[]).await.unwrap());

        // Set new value via inner (simulating external change)
        provider.set(Value::Int(20)).await.unwrap();

        // Value changed - should report changed
        assert!(provider.has_changed(&[]).await.unwrap());

        // Sample to update baseline
        let _ = provider.sample(&[]).await;

        // Now unchanged again
        assert!(!provider.has_changed(&[]).await.unwrap());
    }
}
