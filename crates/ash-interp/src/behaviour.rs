//! Behaviour provider trait and registry for sampling observable values
//!
//! This module provides the core abstraction for behaviour providers that can
//! be sampled to observe values from external sources (sensors, databases, etc.).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use ash_core::{Constraint, Name, Value};

use crate::error::{ExecError, ExecResult};

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
    providers: HashMap<(Name, Name), Box<dyn BehaviourProvider>>,
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
    pub fn register(&mut self, provider: Box<dyn BehaviourProvider>) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }

    /// Get a provider by capability and channel names
    #[must_use]
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn BehaviourProvider> {
        self.providers
            .get(&(cap.to_string(), channel.to_string()))
            .map(|p| p.as_ref())
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
pub struct BehaviourContext {
    registry: BehaviourRegistry,
}

impl BehaviourContext {
    /// Create a new behaviour context with an empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: BehaviourRegistry::new(),
        }
    }

    /// Create a context with an existing registry
    #[must_use]
    pub fn with_registry(registry: BehaviourRegistry) -> Self {
        Self { registry }
    }

    /// Register a behaviour provider
    pub fn register(&mut self, provider: Box<dyn BehaviourProvider>) {
        self.registry.register(provider);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Expr, Predicate};

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

        registry.register(Box::new(provider));

        assert!(registry.has("sensor", "temp"));
        assert!(!registry.has("sensor", "pressure"));
    }

    #[tokio::test]
    async fn test_behaviour_context_sample() {
        let mut ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(100));

        ctx.register(Box::new(provider));

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

        ctx.register(Box::new(provider));

        // First check should be true (never sampled)
        assert!(ctx.has_changed("sensor", "temp", &[]).await.unwrap());

        // Sample to establish baseline
        let _ = ctx.sample("sensor", "temp", &[]).await;

        // Same value - should report unchanged
        assert!(!ctx.has_changed("sensor", "temp", &[]).await.unwrap());
    }
}
