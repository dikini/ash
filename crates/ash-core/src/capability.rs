//! Unified capability provider trait and error types
//!
//! This module defines the shared `CapabilityProvider` trait and `CapabilityError`
//! type used across the Ash workspace.

use crate::{Constraint, Effect, Value};

/// One operation exposed by a capability provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderOperationMetadata {
    /// Provider-local operation name.
    pub operation_name: String,
    /// Effect level for this operation.
    pub effect: Effect,
    /// Required operation rows that must be discharged elsewhere.
    pub required_rows: Vec<String>,
    /// Constraint fields this operation understands.
    pub constraints: Vec<String>,
    /// Host/runtime resources touched by this operation.
    pub resources: Vec<String>,
    /// Sandbox policy identity that must be checked before host execution.
    pub sandbox_policy: Option<String>,
    /// Provenance policy identity used to record/redact host-boundary evidence.
    pub provenance_policy: Option<String>,
    /// Whether this operation metadata claims to grant authority directly.
    pub grants_authority: bool,
}

impl ProviderOperationMetadata {
    /// Create provider operation metadata.
    #[must_use]
    pub fn new(operation_name: impl Into<String>, effect: Effect) -> Self {
        Self {
            operation_name: operation_name.into(),
            effect,
            required_rows: Vec::new(),
            constraints: Vec::new(),
            resources: Vec::new(),
            sandbox_policy: None,
            provenance_policy: None,
            grants_authority: false,
        }
    }

    /// Add a required operation row.
    #[must_use]
    pub fn with_required_row(mut self, row: impl Into<String>) -> Self {
        self.required_rows.push(row.into());
        self
    }

    /// Add a declared constraint field.
    #[must_use]
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Add a declared host/runtime resource.
    #[must_use]
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    /// Attach a sandbox policy identity.
    #[must_use]
    pub fn with_sandbox_policy(mut self, policy: impl Into<String>) -> Self {
        self.sandbox_policy = Some(policy.into());
        self
    }

    /// Attach a provenance policy identity.
    #[must_use]
    pub fn with_provenance_policy(mut self, policy: impl Into<String>) -> Self {
        self.provenance_policy = Some(policy.into());
        self
    }

    /// Escape hatch for constructing invalid metadata in validation tests.
    #[must_use]
    pub fn with_authority_grant_for_test(mut self, grants_authority: bool) -> Self {
        self.grants_authority = grants_authority;
        self
    }
}

/// Provider authoring metadata visible at registration/admission boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthoringMetadata {
    /// Provider name.
    pub provider_name: String,
    /// Operations exposed by this provider.
    pub operations: Vec<ProviderOperationMetadata>,
}

impl ProviderAuthoringMetadata {
    /// Create explicit provider metadata.
    #[must_use]
    pub fn new(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            operations: Vec::new(),
        }
    }

    /// Add one operation to the provider surface.
    #[must_use]
    pub fn with_operation(mut self, operation: ProviderOperationMetadata) -> Self {
        self.operations.push(operation);
        self
    }

    /// Look up provider operation metadata by name.
    #[must_use]
    pub fn operation(&self, operation_name: &str) -> Option<&ProviderOperationMetadata> {
        self.operations
            .iter()
            .find(|operation| operation.operation_name == operation_name)
    }
}

/// Provider authoring metadata validation errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderMetadataError {
    /// Provider name was empty.
    #[error("provider metadata is missing provider name")]
    MissingProviderName,
    /// Provider declared no operation surface.
    #[error("provider '{provider_name}' is missing operation surface metadata")]
    MissingOperationSurface {
        /// Provider name.
        provider_name: String,
    },
    /// Operation name was empty.
    #[error("provider '{provider_name}' has an operation with missing name")]
    MissingOperationName {
        /// Provider name.
        provider_name: String,
    },
    /// Provider declared the same operation more than once.
    #[error("provider '{provider_name}' declares duplicate operation '{operation_name}'")]
    DuplicateOperation {
        /// Provider name.
        provider_name: String,
        /// Duplicated operation name.
        operation_name: String,
    },
    /// Operation had no row metadata.
    #[error("provider '{provider_name}' operation '{operation_name}' is missing required rows")]
    MissingRequiredRows {
        /// Provider name.
        provider_name: String,
        /// Operation name.
        operation_name: String,
    },
    /// Operation lacked sandbox policy metadata.
    #[error("provider '{provider_name}' operation '{operation_name}' is missing sandbox policy")]
    MissingSandboxPolicy {
        /// Provider name.
        provider_name: String,
        /// Operation name.
        operation_name: String,
    },
    /// Operation lacked provenance policy metadata.
    #[error("provider '{provider_name}' operation '{operation_name}' is missing provenance policy")]
    MissingProvenancePolicy {
        /// Provider name.
        provider_name: String,
        /// Operation name.
        operation_name: String,
    },
    /// Operation attempted to grant authority directly.
    #[error("provider '{provider_name}' operation '{operation_name}' must not grant authority")]
    AuthorityWideningOperation {
        /// Provider name.
        provider_name: String,
        /// Operation name.
        operation_name: String,
    },
}

/// Validate provider authoring metadata.
///
/// # Errors
///
/// Returns [`ProviderMetadataError`] when required provider or operation metadata is missing,
/// duplicated, malformed, or authority-widening.
pub fn validate_provider_authoring_metadata(
    metadata: &ProviderAuthoringMetadata,
) -> Result<(), ProviderMetadataError> {
    if metadata.provider_name.trim().is_empty() {
        return Err(ProviderMetadataError::MissingProviderName);
    }
    if metadata.operations.is_empty() {
        return Err(ProviderMetadataError::MissingOperationSurface {
            provider_name: metadata.provider_name.clone(),
        });
    }

    let mut names = std::collections::HashSet::new();
    for operation in &metadata.operations {
        if operation.operation_name.trim().is_empty() {
            return Err(ProviderMetadataError::MissingOperationName {
                provider_name: metadata.provider_name.clone(),
            });
        }
        if !names.insert(operation.operation_name.as_str()) {
            return Err(ProviderMetadataError::DuplicateOperation {
                provider_name: metadata.provider_name.clone(),
                operation_name: operation.operation_name.clone(),
            });
        }
        if operation.required_rows.is_empty() {
            return Err(ProviderMetadataError::MissingRequiredRows {
                provider_name: metadata.provider_name.clone(),
                operation_name: operation.operation_name.clone(),
            });
        }
        if operation
            .sandbox_policy
            .as_deref()
            .is_none_or(str::is_empty)
        {
            return Err(ProviderMetadataError::MissingSandboxPolicy {
                provider_name: metadata.provider_name.clone(),
                operation_name: operation.operation_name.clone(),
            });
        }
        if operation
            .provenance_policy
            .as_deref()
            .is_none_or(str::is_empty)
        {
            return Err(ProviderMetadataError::MissingProvenancePolicy {
                provider_name: metadata.provider_name.clone(),
                operation_name: operation.operation_name.clone(),
            });
        }
        if operation.grants_authority {
            return Err(ProviderMetadataError::AuthorityWideningOperation {
                provider_name: metadata.provider_name.clone(),
                operation_name: operation.operation_name.clone(),
            });
        }
    }

    Ok(())
}

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

    /// Return provider authoring metadata.
    ///
    /// Providers should override this with per-operation metadata. The default value is explicit
    /// but incomplete, so metadata validation fails closed until a provider declares its operation
    /// surface.
    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name())
    }

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
