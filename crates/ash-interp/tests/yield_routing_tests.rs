//! Integration tests for yield routing by role
//!
//! Tests the YieldRouter's ability to route yields to role handlers
//! and resume workflows with responses.

use ash_core::{Expr, Value, Workflow, WorkflowId};
use ash_interp::context::Context;
use ash_interp::yield_routing::{YieldError, YieldId, YieldRouter};
use std::collections::HashMap;

/// Create a simple test workflow that returns a constant value
fn test_workflow() -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(42)),
    }
}

#[test]
fn test_register_and_route() {
    let mut router = YieldRouter::new();

    // Register a handler for a role
    let handler_id = WorkflowId::new();
    router.register_handler("ai_assistant", handler_id);

    // Route a yield to the handler
    let caller_id = WorkflowId::new();
    let mut record_data = HashMap::new();
    record_data.insert("data".to_string(), Value::Int(42));
    let request = Value::Record(Box::new(record_data));
    let continuation = test_workflow();
    let ctx = Context::new();

    let yield_id = router
        .route_yield(caller_id, "ai_assistant", request, continuation, ctx)
        .expect("routing should succeed");

    // Verify yield is pending
    assert!(router.is_pending(&yield_id), "yield should be pending");
    assert_eq!(router.pending_count(), 1, "should have 1 pending yield");

    // Verify handler association
    assert_eq!(
        router.get_handler("ai_assistant"),
        Some(handler_id),
        "handler should be registered"
    );
}

#[test]
fn test_route_to_unknown_role_fails() {
    let mut router = YieldRouter::new();
    // No handler registered

    let caller_id = WorkflowId::new();
    let result = router.route_yield(
        caller_id,
        "unknown_role",
        Value::Null,
        test_workflow(),
        Context::new(),
    );

    assert!(result.is_err(), "routing to unknown role should fail");
    let err = result.unwrap_err();
    assert!(
        matches!(err, YieldError::NoHandlerForRole(ref role) if role == "unknown_role"),
        "error should be NoHandlerForRole"
    );
}

#[test]
fn test_resume_continues_execution() {
    let mut router = YieldRouter::new();

    // Setup: register handler and route a yield
    let handler_id = WorkflowId::new();
    router.register_handler("ai", handler_id);

    let caller_id = WorkflowId::new();
    let mut query_data = HashMap::new();
    query_data.insert("query".to_string(), Value::String("hello".to_string()));
    let yield_id = router
        .route_yield(
            caller_id,
            "ai",
            Value::Record(Box::new(query_data)),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Resume with response
    let mut response_data = HashMap::new();
    response_data.insert("result".to_string(), Value::String("success".to_string()));
    let response = Value::Record(Box::new(response_data));
    let result = router.resume_with_response(yield_id, response);

    assert!(result.is_ok(), "resume should succeed");
    let resume_result = result.unwrap();
    assert_eq!(
        resume_result.caller, caller_id,
        "resume result should contain correct caller"
    );
    assert!(resume_result.result.is_ok(), "result should be Ok");

    // Verify yield is no longer pending
    assert!(
        !router.is_pending(&yield_id),
        "yield should no longer be pending"
    );
    assert_eq!(router.pending_count(), 0, "should have no pending yields");
}

#[test]
fn test_resume_unknown_yield_fails() {
    let mut router = YieldRouter::new();

    let unknown_yield_id = YieldId::new();
    let result = router.resume_with_response(unknown_yield_id, Value::Null);

    assert!(result.is_err(), "resuming unknown yield should fail");
    let err = result.unwrap_err();
    assert!(
        matches!(err, YieldError::UnknownYield(id) if id == unknown_yield_id),
        "error should be UnknownYield"
    );
}

#[test]
fn test_yield_id_unique() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();
    router.register_handler("test_role", handler_id);

    let mut yield_ids = Vec::new();

    // Create many yields
    for i in 0..100 {
        let caller_id = WorkflowId::new();
        let yield_id = router
            .route_yield(
                caller_id,
                "test_role",
                Value::Int(i),
                test_workflow(),
                Context::new(),
            )
            .unwrap();
        yield_ids.push(yield_id);
    }

    // Verify all IDs are unique
    let unique_count = yield_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(
        unique_count,
        yield_ids.len(),
        "all yield IDs should be unique"
    );
}

#[test]
fn test_multiple_concurrent_yields() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    // Register multiple handlers
    router.register_handler("ai_assistant", handler_id);
    router.register_handler("data_processor", handler_id);
    router.register_handler("validator", handler_id);

    // Create yields from different workflows
    let caller1 = WorkflowId::new();
    let caller2 = WorkflowId::new();
    let caller3 = WorkflowId::new();

    let yield1 = router
        .route_yield(
            caller1,
            "ai_assistant",
            Value::String("request1".to_string()),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    let yield2 = router
        .route_yield(
            caller2,
            "data_processor",
            Value::String("request2".to_string()),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    let yield3 = router
        .route_yield(
            caller3,
            "validator",
            Value::String("request3".to_string()),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Verify all yields are pending
    assert_eq!(router.pending_count(), 3, "should have 3 pending yields");
    assert!(router.is_pending(&yield1));
    assert!(router.is_pending(&yield2));
    assert!(router.is_pending(&yield3));

    // Resume in different order
    let result3 = router.resume_with_response(yield3, Value::String("response3".to_string()));
    assert!(result3.is_ok());
    assert_eq!(router.pending_count(), 2);

    let result1 = router.resume_with_response(yield1, Value::String("response1".to_string()));
    assert!(result1.is_ok());
    assert_eq!(router.pending_count(), 1);

    let result2 = router.resume_with_response(yield2, Value::String("response2".to_string()));
    assert!(result2.is_ok());
    assert_eq!(router.pending_count(), 0);
}

#[test]
fn test_handler_deregistration() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    // Register handler
    router.register_handler("test_role", handler_id);
    assert_eq!(router.get_handler("test_role"), Some(handler_id));

    // Unregister handler
    let removed = router.unregister_handler("test_role");
    assert_eq!(removed, Some(handler_id), "should return removed handler");
    assert_eq!(
        router.get_handler("test_role"),
        None,
        "handler should be gone"
    );

    // Routing should now fail
    let result = router.route_yield(
        WorkflowId::new(),
        "test_role",
        Value::Null,
        test_workflow(),
        Context::new(),
    );
    assert!(result.is_err());
}

#[test]
fn test_get_pending_yield_details() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();
    let caller_id = WorkflowId::new();

    router.register_handler("ai_assistant", handler_id);

    let mut request_data = HashMap::new();
    request_data.insert("query".to_string(), Value::String("test".to_string()));
    request_data.insert("priority".to_string(), Value::Int(1));
    let request = Value::Record(Box::new(request_data));

    let yield_id = router
        .route_yield(
            caller_id,
            "ai_assistant",
            request.clone(),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Get pending details
    let pending = router
        .get_pending(&yield_id)
        .expect("should find pending yield");

    assert_eq!(pending.yield_id, yield_id);
    assert_eq!(pending.caller, caller_id);
    assert_eq!(pending.role, "ai_assistant");
    assert_eq!(pending.request, request);
}

#[test]
fn test_cancel_yield() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    router.register_handler("test_role", handler_id);

    let yield_id = router
        .route_yield(
            WorkflowId::new(),
            "test_role",
            Value::Null,
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    assert!(router.is_pending(&yield_id));

    // Cancel the yield
    let cancelled = router.cancel_yield(yield_id);
    assert!(cancelled.is_some(), "should return cancelled yield");
    assert!(!router.is_pending(&yield_id), "yield should not be pending");

    // Cancel again should return None
    let cancelled_again = router.cancel_yield(yield_id);
    assert!(
        cancelled_again.is_none(),
        "second cancel should return None"
    );
}

#[test]
fn test_get_pending_for_caller() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    router.register_handler("role_a", handler_id);
    router.register_handler("role_b", handler_id);

    let caller_a = WorkflowId::new();
    let caller_b = WorkflowId::new();

    // Create yields for caller A
    let _yield_a1 = router
        .route_yield(
            caller_a,
            "role_a",
            Value::Int(1),
            test_workflow(),
            Context::new(),
        )
        .unwrap();
    let _yield_a2 = router
        .route_yield(
            caller_a,
            "role_b",
            Value::Int(2),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Create yield for caller B
    let _yield_b1 = router
        .route_yield(
            caller_b,
            "role_a",
            Value::Int(3),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Get pending yields for each caller
    let caller_a_yields = router.get_pending_for_caller(caller_a);
    let caller_b_yields = router.get_pending_for_caller(caller_b);

    assert_eq!(
        caller_a_yields.len(),
        2,
        "caller A should have 2 pending yields"
    );
    assert_eq!(
        caller_b_yields.len(),
        1,
        "caller B should have 1 pending yield"
    );
}

#[test]
fn test_get_pending_for_role() {
    let mut router = YieldRouter::new();

    router.register_handler("role_a", WorkflowId::new());
    router.register_handler("role_b", WorkflowId::new());

    // Create multiple yields for role_a
    for _ in 0..3 {
        let _ = router
            .route_yield(
                WorkflowId::new(),
                "role_a",
                Value::Null,
                test_workflow(),
                Context::new(),
            )
            .unwrap();
    }

    // Create yields for role_b
    for _ in 0..2 {
        let _ = router
            .route_yield(
                WorkflowId::new(),
                "role_b",
                Value::Null,
                test_workflow(),
                Context::new(),
            )
            .unwrap();
    }

    let role_a_yields = router.get_pending_for_role("role_a");
    let role_b_yields = router.get_pending_for_role("role_b");

    assert_eq!(
        role_a_yields.len(),
        3,
        "role_a should have 3 pending yields"
    );
    assert_eq!(
        role_b_yields.len(),
        2,
        "role_b should have 2 pending yields"
    );
}

#[test]
fn test_resume_by_correlation_id() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    router.register_handler("test_role", handler_id);

    let caller_id = WorkflowId::new();
    let yield_id = router
        .route_yield(
            caller_id,
            "test_role",
            Value::Int(42),
            test_workflow(),
            Context::new(),
        )
        .unwrap();

    // Get the correlation ID
    let pending = router.get_pending(&yield_id).unwrap();
    let correlation_id = pending.correlation_id;

    // Verify lookup by correlation works
    assert!(router.is_pending_by_correlation(correlation_id));

    let found_by_corr = router.get_pending_by_correlation(correlation_id);
    assert!(found_by_corr.is_some());
    assert_eq!(found_by_corr.unwrap().yield_id, yield_id);

    // Resume by correlation ID
    let result = router.resume_by_correlation(correlation_id, Value::String("done".to_string()));
    assert!(result.is_ok());
    assert!(!router.is_pending(&yield_id));
}

#[test]
fn test_clear_pending() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    router.register_handler("test_role", handler_id);

    // Create several yields
    for _ in 0..5 {
        let _ = router
            .route_yield(
                WorkflowId::new(),
                "test_role",
                Value::Null,
                test_workflow(),
                Context::new(),
            )
            .unwrap();
    }

    assert_eq!(router.pending_count(), 5);

    // Clear pending yields
    router.clear_pending();

    assert_eq!(router.pending_count(), 0);
    assert!(router.get_pending_for_role("test_role").is_empty());
}

#[test]
fn test_clear_handlers() {
    let mut router = YieldRouter::new();

    // Register several handlers
    router.register_handler("role_a", WorkflowId::new());
    router.register_handler("role_b", WorkflowId::new());
    router.register_handler("role_c", WorkflowId::new());

    assert_eq!(router.handlers().len(), 3);

    // Clear handlers
    router.clear_handlers();

    assert!(router.handlers().is_empty());
    assert!(router.get_handler("role_a").is_none());
}

#[test]
fn test_yield_error_display() {
    let yield_id = YieldId::new();

    let err1 = YieldError::NoHandlerForRole("test_role".to_string());
    assert!(err1.to_string().contains("test_role"));

    let err2 = YieldError::UnknownYield(yield_id);
    assert!(err2.to_string().contains("unknown yield ID"));

    let err3 = YieldError::HandlerBusy;
    assert!(err3.to_string().contains("busy"));

    let err4 = YieldError::YieldAlreadyPending;
    assert!(err4.to_string().contains("already pending"));
}

#[test]
fn test_handler_replacement() {
    let mut router = YieldRouter::new();

    let handler1 = WorkflowId::new();
    let handler2 = WorkflowId::new();

    // Register first handler
    router.register_handler("test_role", handler1);
    assert_eq!(router.get_handler("test_role"), Some(handler1));

    // Replace with second handler
    router.register_handler("test_role", handler2);
    assert_eq!(router.get_handler("test_role"), Some(handler2));
    assert_ne!(router.get_handler("test_role"), Some(handler1));
}

#[test]
fn test_saved_bindings_in_pending() {
    let mut router = YieldRouter::new();
    let handler_id = WorkflowId::new();

    router.register_handler("test_role", handler_id);

    // Create context with bindings
    let mut ctx = Context::new();
    ctx.set("var1".to_string(), Value::Int(10));
    ctx.set("var2".to_string(), Value::String("hello".to_string()));

    let yield_id = router
        .route_yield(
            WorkflowId::new(),
            "test_role",
            Value::Null,
            test_workflow(),
            ctx,
        )
        .unwrap();

    // Verify bindings were captured
    let pending = router.get_pending(&yield_id).unwrap();
    assert_eq!(pending.saved_bindings.get("var1"), Some(&Value::Int(10)));
    assert_eq!(
        pending.saved_bindings.get("var2"),
        Some(&Value::String("hello".to_string()))
    );
}
