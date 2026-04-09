//! Unified capability provider trait and error types
//!
//! This module defines the shared `CapabilityProvider` trait and `CapabilityError`
//! type used across the Ash workspace.

use crate::{Constraint, Effect, Value};

/// Unified error type for all capability operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CapabilityError {
    #[error("Capability '{0}' not available")]
    NotAvailable(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Unified capability provider trait
///
/// Both primitive and user-defined capabilities implement this trait.
#[async_trait::async_trait]
pub trait CapabilityProvider: Send + Sync + std::fmt::Debug {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the effect level of this provider
    fn effect(&self) -> Effect;

    /// Observe/read from this capability
    ///
    /// Uses unevaluated constraints (delayed evaluation).
    /// Constraints are evaluated by the provider as needed.
    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>;

    /// Execute an action on this capability
    ///
    /// Arguments are already evaluated (eager evaluation).
    ///
    /// # Arguments
    /// * `action_name` - The name of the action to execute
    /// * `args` - The evaluated arguments for the action
    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockProvider {
        name: &'static str,
        effect: Effect,
    }

    #[async_trait::async_trait]
    impl CapabilityProvider for MockProvider {
        fn name(&self) -> &str {
            self.name
        }
        fn effect(&self) -> Effect {
            self.effect
        }

        async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
            Ok(Value::Null)
        }

        async fn execute(
            &self,
            action_name: &str,
            _args: &[Value],
        ) -> Result<Value, CapabilityError> {
            Ok(Value::String(format!("executed: {action_name}")))
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider {
            name: "test",
            effect: Effect::Operational,
        };

        assert_eq!(provider.name(), "test");
        assert_eq!(provider.effect(), Effect::Operational);

        let result = provider.execute("do_something", &[]).await.unwrap();
        assert_eq!(result, Value::String("executed: do_something".to_string()));
    }

    #[test]
    fn test_capability_error_display() {
        let err = CapabilityError::NotAvailable("test".to_string());
        assert_eq!(err.to_string(), "Capability 'test' not available");
    }
}
