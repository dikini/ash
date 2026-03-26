//! Tests for Agent Harness (TASK-269)
//!
//! Tests the `AgentHarness` workflow pattern for LLM agent integration.

#![allow(clippy::doc_markdown)]
#![allow(clippy::box_default)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::redundant_clone)]

use ash_core::Value;
use ash_core::capabilities::{
    AcceptanceMode, AgentHarnessCapability, AgentHarnessConfig, ProjectionPolicy,
};
use ash_engine::harness::{AgentHarness, HarnessError};
use ash_engine::providers::McpProvider;

/// Test that default capability denies accept_response
#[test]
fn test_default_permissions_deny_accept() {
    let capability = AgentHarnessCapability::default();
    let config = AgentHarnessConfig::default();
    let harness = AgentHarness::new(capability, config);

    let response = Value::Record(Box::new(std::collections::HashMap::new()));
    let result = harness.accept_response(response, true);

    assert!(
        matches!(result, Err(HarnessError::PermissionDenied { operation, permission })
            if operation == "accept_response" && permission == "accept_response"
        ),
        "Default capability should deny accept_response"
    );
}

/// Test that read_only capability can project_context
#[test]
fn test_read_only_can_project_context() {
    let capability = AgentHarnessCapability::read_only();
    let config = AgentHarnessConfig::default();
    let harness = AgentHarness::new(capability, config);

    let mut state_map = std::collections::HashMap::new();
    state_map.insert("obligations".to_string(), Value::List(Box::new(vec![])));
    let state = Value::Record(Box::new(state_map));

    let result = harness.project_context(&state);
    assert!(
        result.is_ok(),
        "read_only capability should allow project_context"
    );
}

/// Test that read_only capability cannot delegate_to_agent
#[test]
fn test_read_only_cannot_delegate() {
    let capability = AgentHarnessCapability::read_only();
    let config = AgentHarnessConfig::default();
    let harness = AgentHarness::new(capability, config);

    let context = Value::Null;
    // We can't easily test the async function without an executor, but we can
    // verify the permission check by examining the error

    // Create a mock runtime to test the async function
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result: Result<Value, HarnessError> =
        rt.block_on(async { harness.delegate_to_agent("test task", &context).await });

    assert!(
        matches!(result, Err(HarnessError::PermissionDenied { operation, permission })
            if operation == "delegate_to_agent" && permission == "delegate_to_agent"
        ),
        "read_only capability should deny delegate_to_agent"
    );
}

/// Test that full capability allows all operations
#[test]
fn test_full_permissions_allow_all() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    // Set to Automatic so accept_response doesn't require validation
    config.acceptance_mode = AcceptanceMode::Automatic;

    let harness = AgentHarness::new(capability.clone(), config.clone());

    // Test project_context
    let state = Value::Record(Box::new(std::collections::HashMap::new()));
    assert!(
        harness.project_context(&state).is_ok(),
        "Full capability should allow project_context"
    );

    // Test validate_response
    let response = Value::Record(Box::new(std::collections::HashMap::new()));
    assert!(
        harness.validate_response(&response, "test").is_ok(),
        "Full capability should allow validate_response"
    );

    // Test accept_response with Automatic mode
    let response = Value::Record(Box::new(std::collections::HashMap::new()));
    assert!(
        harness.accept_response(response, true).is_ok(),
        "Full capability should allow accept_response"
    );
}

/// Test that FullContext projection policy returns full state
#[test]
fn test_projection_policy_full() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.projection_policy = ProjectionPolicy::FullContext;

    let harness = AgentHarness::new(capability, config);

    let mut state_map = std::collections::HashMap::new();
    state_map.insert("obligations".to_string(), Value::List(Box::new(vec![])));
    state_map.insert(
        "bindings".to_string(),
        Value::Record(Box::new(std::collections::HashMap::new())),
    );
    state_map.insert("secret".to_string(), Value::String("hidden".to_string()));
    let state = Value::Record(Box::new(state_map));

    let result = harness.project_context(&state).unwrap();
    assert_eq!(
        result, state,
        "FullContext should return the full workflow state"
    );
}

/// Test that ObligationsVisible projection policy filters correctly
#[test]
fn test_projection_policy_obligations() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.projection_policy = ProjectionPolicy::ObligationsVisible;

    let harness = AgentHarness::new(capability, config);

    let obligations = Value::List(Box::new(vec![Value::String("obl1".to_string())]));
    let mut state_map = std::collections::HashMap::new();
    state_map.insert("obligations".to_string(), obligations.clone());
    state_map.insert(
        "bindings".to_string(),
        Value::Record(Box::new(std::collections::HashMap::new())),
    );
    state_map.insert("secret".to_string(), Value::String("hidden".to_string()));
    let state = Value::Record(Box::new(state_map));

    let result = harness.project_context(&state).unwrap();
    let mut expected_map = std::collections::HashMap::new();
    expected_map.insert("obligations".to_string(), obligations);
    let expected = Value::Record(Box::new(expected_map));

    assert_eq!(
        result, expected,
        "ObligationsVisible should only return obligations field"
    );
}

/// Test that Minimal projection policy returns Null
#[test]
fn test_projection_policy_minimal() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.projection_policy = ProjectionPolicy::Minimal;

    let harness = AgentHarness::new(capability, config);

    let mut state_map = std::collections::HashMap::new();
    state_map.insert("obligations".to_string(), Value::List(Box::new(vec![])));
    state_map.insert(
        "bindings".to_string(),
        Value::Record(Box::new(std::collections::HashMap::new())),
    );
    let state = Value::Record(Box::new(state_map));

    let result = harness.project_context(&state).unwrap();
    assert_eq!(
        result,
        Value::Null,
        "Minimal should return Null regardless of state"
    );
}

/// Test that Automatic acceptance mode accepts without validation
#[test]
fn test_acceptance_mode_automatic() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.acceptance_mode = AcceptanceMode::Automatic;

    let harness = AgentHarness::new(capability, config);

    let response = Value::String("any response".to_string());
    // Even with validation_result = false, Automatic mode should accept
    let result = harness.accept_response(response.clone(), false);

    assert!(
        result.is_ok(),
        "Automatic mode should accept regardless of validation"
    );
    assert_eq!(result.unwrap(), response);
}

/// Test that Conditional acceptance mode requires validation
#[test]
fn test_acceptance_mode_conditional() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.acceptance_mode = AcceptanceMode::Conditional;

    let harness = AgentHarness::new(capability, config);

    let response = Value::String("response".to_string());

    // With validation_result = true, should accept
    let result = harness.accept_response(response.clone(), true);
    assert!(
        result.is_ok(),
        "Conditional mode should accept when validation passes"
    );

    // With validation_result = false, should reject
    let result = harness.accept_response(response, false);
    assert!(
        matches!(result, Err(HarnessError::ValidationFailed(_))),
        "Conditional mode should reject when validation fails"
    );
}

/// Test that HumanReview acceptance mode requires approval
#[test]
fn test_acceptance_mode_human_review() {
    let capability = AgentHarnessCapability::full();
    let mut config = AgentHarnessConfig::default();
    config.acceptance_mode = AcceptanceMode::HumanReview;

    let harness = AgentHarness::new(capability, config);

    let response = Value::String("response".to_string());
    // Even with validation_result = true, HumanReview mode should require approval
    let result = harness.accept_response(response, true);

    assert!(
        matches!(result, Err(HarnessError::RequiresHumanApproval)),
        "HumanReview mode should always require human approval"
    );
}

/// Test that delegate without provider returns NoMcpProvider error
#[test]
fn test_no_mcp_provider_error() {
    let capability = AgentHarnessCapability::full();
    let config = AgentHarnessConfig::default();
    let harness = AgentHarness::new(capability, config);

    let context = Value::Null;

    // Create a mock runtime to test the async function
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result: Result<Value, HarnessError> =
        rt.block_on(async { harness.delegate_to_agent("test task", &context).await });

    assert!(
        matches!(result, Err(HarnessError::NoMcpProvider)),
        "delegate_to_agent without MCP provider should return NoMcpProvider error"
    );
}

/// Test that delegate with provider fails when server is unavailable
#[tokio::test]
async fn test_delegate_with_provider_no_server() {
    use ash_engine::providers::McpConfig;

    let capability = AgentHarnessCapability::full();
    let config = AgentHarnessConfig::default();

    // Use a port that's unlikely to have a server
    let provider = McpProvider::new(McpConfig {
        base_url: "http://localhost:59999".to_string(),
        timeout_ms: 100,
    })
    .unwrap();

    let harness = AgentHarness::new(capability, config).with_mcp_provider(provider);
    let context = Value::Null;

    // Should fail because there's no server running
    let result = harness.delegate_to_agent("test task", &context).await;
    assert!(
        result.is_err(),
        "delegate_to_agent should fail when MCP server is unavailable"
    );
}
