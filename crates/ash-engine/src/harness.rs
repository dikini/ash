//! Agent Harness for LLM agent integration
//!
//! Provides controlled, permission-based AI agent interactions.

use ash_core::Value;
use ash_core::capabilities::{
    AcceptanceMode, AgentHarnessCapability, AgentHarnessConfig, AgentHarnessOperation,
    ProjectionPolicy,
};
use thiserror::Error;

/// Errors that can occur during harness operations
#[derive(Error, Debug, Clone)]
pub enum HarnessError {
    /// Permission denied for operation
    #[error("permission denied: {operation} requires {permission}")]
    PermissionDenied {
        /// The operation that was attempted
        operation: String,
        /// The permission that was required
        permission: String,
    },

    /// No MCP provider configured
    #[error("no MCP provider configured")]
    NoMcpProvider,

    /// Delegation to agent failed
    #[error("delegation failed: {0}")]
    DelegationFailed(String),

    /// Response validation failed
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// Response requires human approval
    #[error("response requires human approval")]
    RequiresHumanApproval,
}

/// Result type for harness operations
pub type HarnessResult<T> = Result<T, HarnessError>;

/// Agent Harness for mediating between Ash runtime and LLM agents
#[derive(Debug, Clone)]
pub struct AgentHarness {
    /// The capability defining permissions
    capability: AgentHarnessCapability,
    /// Configuration for behavior
    config: AgentHarnessConfig,
    /// Optional MCP provider for delegation
    mcp_provider: Option<crate::providers::McpProvider>,
}

impl AgentHarness {
    /// Create a new `AgentHarness` with the given capability and config
    #[must_use]
    pub const fn new(capability: AgentHarnessCapability, config: AgentHarnessConfig) -> Self {
        Self {
            capability,
            config,
            mcp_provider: None,
        }
    }

    /// Builder method to add MCP provider
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Option::insert is not const
    pub fn with_mcp_provider(mut self, provider: crate::providers::McpProvider) -> Self {
        self.mcp_provider = Some(provider);
        self
    }

    /// Project context based on projection policy
    ///
    /// # Errors
    ///
    /// Returns `HarnessError::PermissionDenied` if the harness does not have
    /// permission to project context.
    pub fn project_context(&self, workflow_state: &Value) -> HarnessResult<Value> {
        // Check permission
        if !self.capability.can(AgentHarnessOperation::ProjectContext) {
            return Err(HarnessError::PermissionDenied {
                operation: "project_context".to_string(),
                permission: "project_context".to_string(),
            });
        }

        // Apply projection policy
        let context = match self.config.projection_policy {
            ProjectionPolicy::FullContext => workflow_state.clone(),
            ProjectionPolicy::ObligationsVisible => {
                // Extract only obligations field
                if let Value::Record(map) = workflow_state {
                    let obligations = map.get("obligations").cloned().unwrap_or(Value::Null);
                    let mut result = std::collections::HashMap::new();
                    result.insert("obligations".to_string(), obligations);
                    Value::Record(Box::new(result))
                } else {
                    Value::Null
                }
            }
            ProjectionPolicy::Minimal => Value::Null,
        };

        Ok(context)
    }

    /// Delegate task to LLM agent via MCP
    ///
    /// # Errors
    ///
    /// Returns `HarnessError::PermissionDenied` if the harness does not have
    /// permission to delegate to agents.
    /// Returns `HarnessError::NoMcpProvider` if no MCP provider is configured.
    pub async fn delegate_to_agent(&self, task: &str, context: &Value) -> HarnessResult<Value> {
        // Check permission
        if !self.capability.can(AgentHarnessOperation::DelegateToAgent) {
            return Err(HarnessError::PermissionDenied {
                operation: "delegate_to_agent".to_string(),
                permission: "delegate_to_agent".to_string(),
            });
        }

        let provider = self
            .mcp_provider
            .as_ref()
            .ok_or(HarnessError::NoMcpProvider)?;

        // Call MCP tool for delegation
        let mut args = std::collections::HashMap::new();
        args.insert("task".to_string(), Value::String(task.to_string()));
        args.insert("context".to_string(), context.clone());

        provider
            .call_tool("delegate", args)
            .await
            .map_err(|e| HarnessError::DelegationFailed(e.to_string()))
    }

    /// Validate agent response against expected schema
    ///
    /// # Errors
    ///
    /// Returns `HarnessError::PermissionDenied` if the harness does not have
    /// permission to validate responses.
    pub fn validate_response(&self, response: &Value, _expected_type: &str) -> HarnessResult<bool> {
        // Check permission
        if !self.capability.can(AgentHarnessOperation::ValidateResponse) {
            return Err(HarnessError::PermissionDenied {
                operation: "validate_response".to_string(),
                permission: "validate_response".to_string(),
            });
        }

        // Placeholder validation - check if response is a Record
        let is_valid = matches!(response, Value::Record(_));
        Ok(is_valid)
    }

    /// Accept response into workflow based on acceptance mode
    ///
    /// # Errors
    ///
    /// Returns `HarnessError::PermissionDenied` if the harness does not have
    /// permission to accept responses.
    /// Returns `HarnessError::ValidationFailed` if validation fails in conditional mode.
    /// Returns `HarnessError::RequiresHumanApproval` in human review mode.
    pub fn accept_response(
        &self,
        response: Value,
        validation_result: bool,
    ) -> HarnessResult<Value> {
        // Check permission
        if !self.capability.can(AgentHarnessOperation::AcceptResponse) {
            return Err(HarnessError::PermissionDenied {
                operation: "accept_response".to_string(),
                permission: "accept_response".to_string(),
            });
        }

        match self.config.acceptance_mode {
            AcceptanceMode::Automatic => Ok(response),
            AcceptanceMode::Conditional => {
                if validation_result {
                    Ok(response)
                } else {
                    Err(HarnessError::ValidationFailed(
                        "response validation failed".to_string(),
                    ))
                }
            }
            AcceptanceMode::HumanReview => Err(HarnessError::RequiresHumanApproval),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_error_display() {
        let err = HarnessError::PermissionDenied {
            operation: "test_op".to_string(),
            permission: "test_perm".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));

        let err = HarnessError::NoMcpProvider;
        assert_eq!(err.to_string(), "no MCP provider configured");

        let err = HarnessError::RequiresHumanApproval;
        assert_eq!(err.to_string(), "response requires human approval");
    }
}
