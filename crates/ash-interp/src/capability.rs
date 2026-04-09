//! Capability provider trait and registry
//!
//! Capabilities represent external resources that workflows can observe or act upon.
//! This module re-exports the unified `CapabilityProvider` trait from `ash_core`
//! and provides the `CapabilityContext` for runtime execution.

use ash_core::{Capability, Effect, Name, Value};
use std::collections::HashMap;

use crate::ExecResult;
use crate::error::ExecError;

// Re-export the unified capability types from ash_core
pub use ash_core::capability::{CapabilityError, CapabilityProvider};

/// Registry of capability providers
#[derive(Default)]
pub struct CapabilityRegistry {
    providers: HashMap<Name, Box<dyn CapabilityProvider>>,
}

impl CapabilityRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a capability provider
    pub fn register(&mut self, provider: Box<dyn CapabilityProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<&dyn CapabilityProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Check if a capability is registered
    pub fn has(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Remove a provider
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn CapabilityProvider>> {
        self.providers.remove(name)
    }

    /// Get all registered capability names
    pub fn names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

/// Context for capability operations during execution
pub struct CapabilityContext {
    registry: CapabilityRegistry,
}

impl CapabilityContext {
    /// Create a new capability context
    pub fn new() -> Self {
        Self {
            registry: CapabilityRegistry::new(),
        }
    }

    /// Create with a pre-configured registry
    pub fn with_registry(registry: CapabilityRegistry) -> Self {
        Self { registry }
    }

    /// Register a capability provider
    pub fn register(&mut self, provider: Box<dyn CapabilityProvider>) {
        self.registry.register(provider);
    }

    /// Convert CapabilityError to ExecError
    fn convert_error(err: CapabilityError) -> ExecError {
        match err {
            CapabilityError::NotAvailable(name) => ExecError::CapabilityNotAvailable(name),
            CapabilityError::ExecutionFailed(msg) => ExecError::ExecutionFailed(msg),
            CapabilityError::ValidationFailed(msg) => ExecError::ExecutionFailed(msg),
            CapabilityError::InvalidArgument(msg) => ExecError::ExecutionFailed(msg),
            CapabilityError::PermissionDenied(msg) => ExecError::ExecutionFailed(msg),
        }
    }

    /// Observe a capability
    pub async fn observe(&self, capability: &Capability) -> ExecResult<Value> {
        let provider = self
            .registry
            .get(&capability.name)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(capability.name.clone()))?;

        if !provider.effect().at_least(Effect::Epistemic) {
            return Err(ExecError::ExecutionFailed(format!(
                "capability '{}' does not support observation",
                capability.name
            )));
        }

        provider
            .observe(&capability.constraints)
            .await
            .map_err(Self::convert_error)
    }

    /// Execute an action on a capability
    pub async fn execute(
        &self,
        action: &ash_core::Action,
        capability_name: &str,
    ) -> ExecResult<Value> {
        let provider = self
            .registry
            .get(capability_name)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(capability_name.to_string()))?;

        if !provider.effect().at_least(Effect::Operational) {
            return Err(ExecError::ExecutionFailed(format!(
                "capability '{}' does not support actions",
                capability_name
            )));
        }

        provider.execute(action).await.map_err(Self::convert_error)
    }
}

impl Default for CapabilityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A mock capability provider for testing
#[derive(Debug)]
pub struct MockProvider {
    name: String,
    effect: Effect,
    observe_value: Value,
    execute_result: Result<Value, CapabilityError>,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new(name: &str, effect: Effect) -> Self {
        Self {
            name: name.to_string(),
            effect,
            observe_value: Value::Null,
            execute_result: Ok(Value::Null),
        }
    }

    /// Set the value to return from observe
    pub fn with_observe_value(mut self, value: Value) -> Self {
        self.observe_value = value;
        self
    }

    /// Set the result to return from execute
    pub fn with_execute_result(mut self, result: Result<Value, CapabilityError>) -> Self {
        self.execute_result = result;
        self
    }
}

#[async_trait::async_trait]
impl CapabilityProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        self.effect
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> Result<Value, CapabilityError> {
        Ok(self.observe_value.clone())
    }

    async fn execute(&self, _action: &ash_core::Action) -> Result<Value, CapabilityError> {
        self.execute_result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_observe() {
        let provider =
            MockProvider::new("test", Effect::Epistemic).with_observe_value(Value::Int(42));

        let result = provider.observe(&[]).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_mock_provider_execute() {
        let provider = MockProvider::new("test", Effect::Operational)
            .with_execute_result(Ok(Value::String("done".to_string())));

        let action = ash_core::Action {
            name: "do_it".to_string(),
            arguments: vec![],
        };
        let result = provider.execute(&action).await.unwrap();
        assert_eq!(result, Value::String("done".to_string()));
    }

    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();

        assert!(!registry.has("test"));

        let provider = Box::new(MockProvider::new("test", Effect::Epistemic));
        registry.register(provider);

        assert!(registry.has("test"));
        assert!(registry.get("test").is_some());
        assert!(registry.get("missing").is_none());

        let names = registry.names();
        assert_eq!(names, vec!["test"]);
    }

    #[tokio::test]
    async fn test_capability_context_observe() {
        let mut ctx = CapabilityContext::new();
        let provider = Box::new(
            MockProvider::new("sensor", Effect::Epistemic).with_observe_value(Value::Int(100)),
        );
        ctx.register(provider);

        let cap = Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        };

        let result = ctx.observe(&cap).await.unwrap();
        assert_eq!(result, Value::Int(100));
    }

    #[tokio::test]
    async fn test_capability_context_missing() {
        let ctx = CapabilityContext::new();

        let cap = Capability {
            name: "missing".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        };

        assert!(ctx.observe(&cap).await.is_err());
    }
}
