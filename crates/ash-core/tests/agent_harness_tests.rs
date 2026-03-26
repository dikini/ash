//! Tests for Agent Harness Capability (TASK-268)
//!
//! Tests permission-based security model for LLM agent integration.

use ash_core::capabilities::{
    AcceptanceMode, AgentHarnessCapability, AgentHarnessConfig, AgentHarnessOperation,
    ProjectionPolicy,
};

/// Test that default permissions have accept_response denied by default
#[test]
fn test_default_permissions() {
    let capability: AgentHarnessCapability = Default::default();

    assert!(capability.project_context, "project_context should be true");
    assert!(
        capability.delegate_to_agent,
        "delegate_to_agent should be true"
    );
    assert!(
        capability.validate_response,
        "validate_response should be true"
    );
    assert!(
        !capability.accept_response,
        "accept_response should be false by default"
    );
}

/// Test that full permissions have all permissions set to true
#[test]
fn test_full_permissions() {
    let capability = AgentHarnessCapability::full();

    assert!(capability.project_context, "project_context should be true");
    assert!(
        capability.delegate_to_agent,
        "delegate_to_agent should be true"
    );
    assert!(
        capability.validate_response,
        "validate_response should be true"
    );
    assert!(capability.accept_response, "accept_response should be true");
}

/// Test that read-only permissions only allow project_context
#[test]
fn test_read_only_permissions() {
    let capability = AgentHarnessCapability::read_only();

    assert!(capability.project_context, "project_context should be true");
    assert!(
        !capability.delegate_to_agent,
        "delegate_to_agent should be false"
    );
    assert!(
        !capability.validate_response,
        "validate_response should be false"
    );
    assert!(
        !capability.accept_response,
        "accept_response should be false"
    );
}

/// Test that each operation maps to the correct permission name
#[test]
fn test_operation_permission_mapping() {
    assert_eq!(
        AgentHarnessOperation::ProjectContext.required_permission(),
        "project_context"
    );
    assert_eq!(
        AgentHarnessOperation::DelegateToAgent.required_permission(),
        "delegate_to_agent"
    );
    assert_eq!(
        AgentHarnessOperation::ValidateResponse.required_permission(),
        "validate_response"
    );
    assert_eq!(
        AgentHarnessOperation::AcceptResponse.required_permission(),
        "accept_response"
    );
}

/// Test that default config values are as expected
#[test]
fn test_config_defaults() {
    let config = AgentHarnessConfig::default();

    assert_eq!(
        config.projection_policy,
        ProjectionPolicy::ObligationsVisible,
        "projection_policy should be ObligationsVisible"
    );
    assert_eq!(
        config.acceptance_mode,
        AcceptanceMode::Conditional,
        "acceptance_mode should be Conditional"
    );
    assert_eq!(config.max_retries, 3, "max_retries should be 3");
    assert_eq!(config.timeout_ms, 30000, "timeout_ms should be 30000");
}

/// Test that can() returns correct values for all operations
#[test]
fn test_can_method() {
    // Test with default capability
    let default_cap = AgentHarnessCapability::default();
    assert!(default_cap.can(AgentHarnessOperation::ProjectContext));
    assert!(default_cap.can(AgentHarnessOperation::DelegateToAgent));
    assert!(default_cap.can(AgentHarnessOperation::ValidateResponse));
    assert!(!default_cap.can(AgentHarnessOperation::AcceptResponse));

    // Test with full capability
    let full_cap = AgentHarnessCapability::full();
    assert!(full_cap.can(AgentHarnessOperation::ProjectContext));
    assert!(full_cap.can(AgentHarnessOperation::DelegateToAgent));
    assert!(full_cap.can(AgentHarnessOperation::ValidateResponse));
    assert!(full_cap.can(AgentHarnessOperation::AcceptResponse));

    // Test with read-only capability
    let read_only_cap = AgentHarnessCapability::read_only();
    assert!(read_only_cap.can(AgentHarnessOperation::ProjectContext));
    assert!(!read_only_cap.can(AgentHarnessOperation::DelegateToAgent));
    assert!(!read_only_cap.can(AgentHarnessOperation::ValidateResponse));
    assert!(!read_only_cap.can(AgentHarnessOperation::AcceptResponse));
}
