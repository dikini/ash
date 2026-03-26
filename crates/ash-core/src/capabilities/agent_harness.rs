//! Agent Harness Capability for LLM agent integration
//!
//! Provides controlled, permission-based AI agent interactions via MCP.

use serde::{Deserialize, Serialize};

/// Agent harness capability with permission-based security
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentHarnessCapability {
    /// Permission to access project context
    pub project_context: bool,
    /// Permission to delegate tasks to LLM agents
    pub delegate_to_agent: bool,
    /// Permission to validate agent responses
    pub validate_response: bool,
    /// Permission to accept responses into workflow (default: false)
    pub accept_response: bool,
}

impl Default for AgentHarnessCapability {
    /// Default capability - accept_response denied by default
    fn default() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: true,
            validate_response: true,
            accept_response: false,
        }
    }
}

impl AgentHarnessCapability {
    /// Create default capability - accept_response denied by default
    pub fn new_default() -> Self {
        Self::default()
    }

    /// Full permissions - all operations allowed
    pub fn full() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: true,
            validate_response: true,
            accept_response: true,
        }
    }

    /// Read-only - only project_context allowed
    pub fn read_only() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: false,
            validate_response: false,
            accept_response: false,
        }
    }

    /// Check if operation is permitted
    pub fn can(&self, operation: AgentHarnessOperation) -> bool {
        match operation {
            AgentHarnessOperation::ProjectContext => self.project_context,
            AgentHarnessOperation::DelegateToAgent => self.delegate_to_agent,
            AgentHarnessOperation::ValidateResponse => self.validate_response,
            AgentHarnessOperation::AcceptResponse => self.accept_response,
        }
    }
}

/// Operations supported by agent harness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentHarnessOperation {
    /// Access project context
    ProjectContext,
    /// Delegate tasks to LLM agents
    DelegateToAgent,
    /// Validate agent responses
    ValidateResponse,
    /// Accept responses into workflow
    AcceptResponse,
}

impl AgentHarnessOperation {
    /// Get the permission flag required for this operation
    pub fn required_permission(&self) -> &'static str {
        match self {
            AgentHarnessOperation::ProjectContext => "project_context",
            AgentHarnessOperation::DelegateToAgent => "delegate_to_agent",
            AgentHarnessOperation::ValidateResponse => "validate_response",
            AgentHarnessOperation::AcceptResponse => "accept_response",
        }
    }
}

/// Configuration for agent harness behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentHarnessConfig {
    /// How much context to expose to agents
    pub projection_policy: ProjectionPolicy,
    /// How responses are accepted into workflow
    pub acceptance_mode: AcceptanceMode,
    /// Maximum delegation retries
    pub max_retries: u32,
    /// Timeout for delegation in milliseconds
    pub timeout_ms: u64,
}

impl Default for AgentHarnessConfig {
    fn default() -> Self {
        Self {
            projection_policy: ProjectionPolicy::ObligationsVisible,
            acceptance_mode: AcceptanceMode::Conditional,
            max_retries: 3,
            timeout_ms: 30000,
        }
    }
}

/// Context projection policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionPolicy {
    /// Full workflow state including obligations and bindings
    FullContext,
    /// Only obligations and contract requirements (recommended default)
    ObligationsVisible,
    /// No context exposed
    Minimal,
}

/// Response acceptance modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceptanceMode {
    /// Accept immediately (requires accept_response permission)
    Automatic,
    /// Accept after validation (default)
    Conditional,
    /// Requires explicit human approval
    HumanReview,
}
