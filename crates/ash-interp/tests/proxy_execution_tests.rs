//! Proxy execution tests for yield/resume semantics
//!
//! Tests the implementation of SPEC-023 Section 6 yield/resume execution.

use ash_core::ast::Span;
use ash_core::workflow_contract::TypeExpr;
use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::{
    behaviour::BehaviourContext,
    capability::CapabilityContext,
    context::Context,
    error::ExecError,
    execute_workflow_with_behaviour_in_state,
    policy::PolicyEvaluator,
    proxy_registry::ProxyRegistry,
    runtime_state::RuntimeState,
    yield_state::{SuspendedYields, YieldState},
};

/// Helper to create a minimal runtime state for testing
fn setup_test_runtime() -> RuntimeState {
    RuntimeState::new()
}

/// Helper to create a context with proxy registry registered
fn setup_test_context() -> Context {
    Context::new()
}

/// Helper to create a simple yield workflow
fn create_yield_workflow(role: &str, request_value: Value, response_type: TypeExpr) -> Workflow {
    Workflow::Yield {
        role: role.to_string(),
        request: Box::new(Expr::Literal(request_value)),
        expected_response_type: response_type,
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::String("resumed".to_string())),
        }),
        span: Span::default(),
        resume_var: "response".to_string(),
    }
}

/// Helper to create a yield workflow with pattern-matching continuation
#[allow(dead_code)]
fn create_yield_with_arms(
    _role: &str,
    _request_value: Value,
    _response_type: TypeExpr,
    arms: Vec<(Pattern, Workflow)>,
) -> Workflow {
    // Create receive-style arms using Let patterns for matching
    let mut current = Workflow::Done;
    for (pattern, body) in arms.into_iter().rev() {
        current = Workflow::Let {
            pattern,
            expr: Expr::Literal(Value::Null),
            continuation: Box::new(body),
        };
    }
    current
}

#[tokio::test]
async fn test_yield_suspends_workflow() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register a proxy for the "test_role"
    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("test_role".to_string(), "proxy://instance-1".to_string());
    }

    let workflow = create_yield_workflow(
        "test_role",
        Value::Int(42),
        TypeExpr::Named("Int".to_string()),
    );

    // Execute the workflow - should suspend, not complete
    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Should return YieldSuspended error with correct details
    match result {
        Err(ExecError::YieldSuspended {
            role,
            request,
            expected_response_type,
            correlation_id,
            proxy_addr,
        }) => {
            assert_eq!(role, "test_role");
            assert_eq!(*request, Value::Int(42));
            assert_eq!(expected_response_type, "Named(\"Int\")");
            assert!(!correlation_id.is_empty());
            assert_eq!(proxy_addr, "proxy://instance-1");
        }
        Err(other) => panic!("Expected YieldSuspended error, got: {:?}", other),
        Ok(_) => panic!("Yield should suspend workflow and not return a value"),
    }

    // Check that a yield was suspended
    let suspended_guard = runtime.suspended_yields();
    let suspended = suspended_guard.lock().await;
    assert_eq!(suspended.len(), 1, "Should have one suspended yield");
}

#[tokio::test]
async fn test_yield_without_registered_proxy_fails() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Do NOT register any proxy

    let workflow = create_yield_workflow(
        "unregistered_role",
        Value::Int(42),
        TypeExpr::Named("Int".to_string()),
    );

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    assert!(
        result.is_err(),
        "Should fail when no proxy registered for role"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("proxy") || err_msg.contains("not found") || err_msg.contains("No proxy"),
        "Error should indicate missing proxy: {}",
        err_msg
    );
}

#[test]
fn test_correlation_id_generation() {
    let mut suspended = SuspendedYields::new();

    let state1 = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Done,
        origin_workflow: "instance-1".to_string(),
        target_role: "admin".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };

    let id1 = suspended.suspend(state1.clone());

    // Suspend another
    let state2 = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Done,
        origin_workflow: "instance-2".to_string(),
        target_role: "user".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };
    let id2 = suspended.suspend(state2);

    // IDs should be different
    assert_ne!(id1, id2, "Correlation IDs should be unique");

    // Both should be in suspended
    assert!(suspended.contains(id1));
    assert!(suspended.contains(id2));
}

#[tokio::test]
async fn test_resume_continues_workflow() {
    // This test verifies that when a proxy response is received,
    // the suspended workflow can be resumed
    let runtime = setup_test_runtime();

    // Create and suspend a yield state manually
    let correlation_id = ash_interp::yield_state::CorrelationId::new();
    let state = YieldState {
        correlation_id,
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Ret {
            expr: Expr::Literal(Value::String("completed".to_string())),
        },
        origin_workflow: "instance-1".to_string(),
        target_role: "admin".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };

    {
        let suspended_ref = runtime.suspended_yields();
        let mut suspended = suspended_ref.lock().await;
        suspended.suspend(state);
    }

    // Verify it's suspended
    {
        let suspended_ref = runtime.suspended_yields();
        let suspended = suspended_ref.lock().await;
        assert!(suspended.contains(correlation_id));
    }

    // Simulate resuming by removing from suspended
    let resumed_state = {
        let suspended_ref = runtime.suspended_yields();
        let mut suspended = suspended_ref.lock().await;
        suspended.resume(correlation_id)
    };

    assert!(resumed_state.is_some());
    assert_eq!(resumed_state.unwrap().correlation_id, correlation_id);

    // Verify it's no longer suspended
    let suspended_ref = runtime.suspended_yields();
    let suspended = suspended_ref.lock().await;
    assert!(!suspended.contains(correlation_id));
}

#[test]
fn test_correlation_id_matching() {
    let mut suspended = SuspendedYields::new();

    // Suspend multiple yields
    let state1 = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Done,
        origin_workflow: "instance-1".to_string(),
        target_role: "admin".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };
    let id1 = suspended.suspend(state1);

    let state2 = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::String,
        continuation: Workflow::Done,
        origin_workflow: "instance-2".to_string(),
        target_role: "user".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };
    let id2 = suspended.suspend(state2);

    // Resume with correct IDs
    let resumed1 = suspended.resume(id1);
    assert!(resumed1.is_some());
    assert_eq!(resumed1.unwrap().origin_workflow, "instance-1");

    let resumed2 = suspended.resume(id2);
    assert!(resumed2.is_some());
    assert_eq!(resumed2.unwrap().origin_workflow, "instance-2");
}

#[test]
fn test_resume_unknown_correlation_id_fails() {
    let mut suspended = SuspendedYields::new();

    // Try to resume a non-existent correlation ID
    let unknown_id = ash_interp::yield_state::CorrelationId(999999);
    let result = suspended.resume(unknown_id);

    assert!(result.is_none(), "Resuming unknown ID should return None");
}

#[test]
fn test_proxy_registry_lookup() {
    let mut registry = ProxyRegistry::new();

    // Register some proxies
    registry.register("admin".to_string(), "proxy://admin-proxy".to_string());
    registry.register("user".to_string(), "proxy://user-proxy".to_string());
    registry.register("moderator".to_string(), "proxy://admin-proxy".to_string());

    // Test lookups
    assert_eq!(
        registry.lookup("admin"),
        Some(&"proxy://admin-proxy".to_string())
    );
    assert_eq!(
        registry.lookup("user"),
        Some(&"proxy://user-proxy".to_string())
    );
    assert_eq!(
        registry.lookup("moderator"),
        Some(&"proxy://admin-proxy".to_string())
    );
    assert_eq!(registry.lookup("unknown"), None);

    // Test reverse lookup
    let admin_roles = registry.get_roles("proxy://admin-proxy").unwrap();
    assert_eq!(admin_roles.len(), 2);
    assert!(admin_roles.contains("admin"));
    assert!(admin_roles.contains("moderator"));
}

#[test]
fn test_yield_state_contains_expected_info() {
    let correlation_id = ash_interp::yield_state::CorrelationId::new();
    let continuation = Workflow::Ret {
        expr: Expr::Literal(Value::Bool(true)),
    };

    let state = YieldState {
        correlation_id,
        expected_response_type: ash_typeck::types::Type::Bool,
        continuation: continuation.clone(),
        origin_workflow: "test-instance".to_string(),
        target_role: "test_role".to_string(),
        request_sent_at: std::time::Instant::now(),
        resume_var: "response".to_string(),
    };

    assert_eq!(state.correlation_id, correlation_id);
    assert_eq!(state.expected_response_type, ash_typeck::types::Type::Bool);
    assert_eq!(state.origin_workflow, "test-instance");
    assert_eq!(state.target_role, "test_role");
    // Continuation should be preserved
    assert!(matches!(state.continuation, Workflow::Ret { .. }));
}

#[test]
fn test_suspended_yields_timeout_handling() {
    use std::time::Duration;

    let mut suspended = SuspendedYields::new();

    // Add an old suspended yield
    let old_state = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Done,
        origin_workflow: "old-instance".to_string(),
        target_role: "admin".to_string(),
        request_sent_at: std::time::Instant::now() - Duration::from_secs(100),
        resume_var: "response".to_string(),
    };
    let old_id = old_state.correlation_id;
    suspended.suspend(old_state);

    // Add a recent suspended yield
    let recent_state = YieldState {
        correlation_id: ash_interp::yield_state::CorrelationId::new(),
        expected_response_type: ash_typeck::types::Type::Int,
        continuation: Workflow::Done,
        origin_workflow: "recent-instance".to_string(),
        target_role: "user".to_string(),
        request_sent_at: std::time::Instant::now() - Duration::from_secs(5),
        resume_var: "response".to_string(),
    };
    let recent_id = recent_state.correlation_id;
    suspended.suspend(recent_state);

    assert_eq!(suspended.len(), 2);

    // Remove expired yields (older than 60 seconds)
    let expired = suspended.remove_expired(Duration::from_secs(60));

    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].correlation_id, old_id);
    assert_eq!(suspended.len(), 1);
    assert!(suspended.contains(recent_id));
    assert!(!suspended.contains(old_id));
}

#[tokio::test]
async fn test_multiple_yields_same_role() {
    let runtime = setup_test_runtime();

    // Register a proxy
    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("test_role".to_string(), "proxy://instance-1".to_string());
    }

    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // First yield
    let workflow1 = create_yield_workflow(
        "test_role",
        Value::Int(1),
        TypeExpr::Named("Int".to_string()),
    );
    let _result1 = execute_workflow_with_behaviour_in_state(
        &workflow1,
        ctx.clone(),
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Second yield to same role
    let workflow2 = create_yield_workflow(
        "test_role",
        Value::Int(2),
        TypeExpr::Named("Int".to_string()),
    );
    let _result2 = execute_workflow_with_behaviour_in_state(
        &workflow2,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Should have two suspended yields
    let suspended_guard = runtime.suspended_yields();
    let suspended = suspended_guard.lock().await;
    assert_eq!(
        suspended.len(),
        2,
        "Should have two suspended yields to the same role"
    );
}
