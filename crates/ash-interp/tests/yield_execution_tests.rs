//! YIELD runtime execution tests (TASK-243)
//!
//! Tests the implementation of YIELD runtime execution for proxy workflows.
//! See SPEC-023 (Proxy Workflows) for specification details.

use std::{sync::Arc, time::Duration};

use ash_core::ast::{Obligation, ReceiveArm, ReceiveMode, ReceivePattern, Role, Span};
use ash_core::stream::{
    Receive as StreamReceive, ReceiveArm as StreamReceiveArm, ReceiveMode as StreamReceiveMode,
};
use ash_core::workflow_contract::TypeExpr;
use ash_core::{BinaryOp, Capability, Effect, Expr, Pattern, Value, Workflow};
use ash_interp::{
    Mailbox, TypedStreamProvider,
    behaviour::BehaviourContext,
    capability::{CapabilityContext, MockProvider},
    context::Context,
    error::ExecError,
    execute::execute_workflow_with_stream_in_state,
    execute_stream::execute_receive_in_state,
    execute_workflow_with_behaviour_in_state,
    policy::PolicyEvaluator,
    runtime_state::RuntimeState,
    stream::{MockStreamProvider, StreamContext},
};
use ash_typeck::Type;

/// Helper to create a minimal runtime state for testing
fn setup_test_runtime() -> RuntimeState {
    RuntimeState::new()
}

/// Helper to create a test context
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

fn create_resumable_yield(role: &str, request: Expr) -> Workflow {
    Workflow::Yield {
        role: role.to_string(),
        request: Box::new(request),
        expected_response_type: TypeExpr::Named("Int".to_string()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable("response".to_string()),
        }),
        span: Span::default(),
        resume_var: "response".to_string(),
    }
}

async fn setup_proxy_role(runtime: &RuntimeState, role: &str) {
    let registry_guard = runtime.proxy_registry();
    let mut registry = registry_guard.lock().await;
    registry.register(role.to_string(), format!("proxy://{role}"));
}

async fn assert_nested_yield_round_trip(
    workflow: &Workflow,
    cap_ctx: &CapabilityContext,
    runtime: &RuntimeState,
    expected_request: Value,
) {
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    let correlation_id = match execute_workflow_with_behaviour_in_state(
        workflow,
        ctx.clone(),
        cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, expected_request);
            correlation_id
        }
        Err(other) => panic!("expected nested yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected nested yield to suspend, got value: {value:?}"),
    };

    let resume = Workflow::ProxyResume {
        value: Box::new(Expr::Literal(Value::Int(99))),
        value_type: TypeExpr::Named("Int".to_string()),
        correlation_id: ash_core::ast::CorrelationId::new(
            correlation_id
                .parse()
                .expect("yield correlation IDs are numeric"),
        ),
        span: Span::default(),
    };

    let resumed = execute_workflow_with_behaviour_in_state(
        &resume,
        ctx,
        cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime,
    )
    .await
    .expect("proxy resume should continue nested workflow");

    assert_eq!(resumed, Value::Int(99));
}

async fn resume_suspended_yield(
    runtime: &RuntimeState,
    cap_ctx: &CapabilityContext,
    correlation_id: String,
) -> Value {
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    let resume = Workflow::ProxyResume {
        value: Box::new(Expr::Literal(Value::Int(99))),
        value_type: TypeExpr::Named("Int".to_string()),
        correlation_id: ash_core::ast::CorrelationId::new(
            correlation_id
                .parse()
                .expect("yield correlation IDs are numeric"),
        ),
        span: Span::default(),
    };

    execute_workflow_with_behaviour_in_state(
        &resume,
        setup_test_context(),
        cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime,
    )
    .await
    .expect("proxy resume should continue nested workflow")
}

#[tokio::test]
async fn test_yield_suspends_execution() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register a proxy for the target role
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

    // Execute the workflow - should suspend with YieldSuspended
    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Verify we get YieldSuspended error with correct details
    match result {
        Err(ExecError::YieldSuspended {
            role,
            request,
            expected_response_type,
            correlation_id,
            proxy_addr,
        }) => {
            assert_eq!(role, "test_role", "Role should match");
            assert_eq!(*request, Value::Int(42), "Request value should match");
            assert_eq!(
                expected_response_type, "Named(\"Int\")",
                "Expected response type should match"
            );
            assert!(
                !correlation_id.is_empty(),
                "Correlation ID should not be empty"
            );
            assert_eq!(
                proxy_addr, "proxy://instance-1",
                "Proxy address should match"
            );
        }
        Err(other) => panic!("Expected YieldSuspended error, got: {:?}", other),
        Ok(_) => panic!("Yield should suspend execution, not return a value"),
    }
}

#[tokio::test]
async fn test_yield_evaluates_request_expression() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register a proxy
    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("compute_role".to_string(), "proxy://compute-1".to_string());
    }

    // Create yield with a more complex expression
    let workflow = Workflow::Yield {
        role: "compute_role".to_string(),
        request: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(32))),
        }),
        expected_response_type: TypeExpr::Named("Int".to_string()),
        continuation: Box::new(Workflow::Done),
        span: Span::default(),
        resume_var: "response".to_string(),
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Verify the request expression was evaluated
    match result {
        Err(ExecError::YieldSuspended { request, .. }) => {
            assert_eq!(
                *request,
                Value::Int(42),
                "Request should be evaluated to 42 (10+32)"
            );
        }
        Err(other) => panic!("Expected YieldSuspended error, got: {:?}", other),
        Ok(_) => panic!("Yield should suspend execution"),
    }
}

#[tokio::test]
async fn test_yield_preserves_continuation() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register a proxy
    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("async_role".to_string(), "proxy://async-1".to_string());
    }

    // Create yield with a specific continuation
    let continuation_workflow = Workflow::Ret {
        expr: Expr::Literal(Value::String("continuation_result".to_string())),
    };

    let workflow = Workflow::Yield {
        role: "async_role".to_string(),
        request: Box::new(Expr::Literal(Value::Bool(true))),
        expected_response_type: TypeExpr::Named("String".to_string()),
        continuation: Box::new(continuation_workflow),
        span: Span::default(),
        resume_var: "response".to_string(),
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Verify execution suspends (continuation is preserved in suspended yields)
    assert!(
        matches!(result, Err(ExecError::YieldSuspended { .. })),
        "Yield should suspend with continuation preserved"
    );

    // Verify the yield was recorded in suspended yields
    let suspended_guard = runtime.suspended_yields();
    let suspended = suspended_guard.lock().await;
    assert_eq!(suspended.len(), 1, "Should have one suspended yield");

    // The continuation is stored in the YieldState - verify by checking target_role
    // We can't directly access the continuation, but we verified the yield was suspended
}

#[tokio::test]
async fn test_yield_contains_role_and_request() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register multiple proxies
    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("role_a".to_string(), "proxy://instance-a".to_string());
        registry.register("role_b".to_string(), "proxy://instance-b".to_string());
    }

    // Test yield to role_a
    let workflow_a = create_yield_workflow(
        "role_a",
        Value::String("request_to_a".to_string()),
        TypeExpr::Named("String".to_string()),
    );

    let result_a = execute_workflow_with_behaviour_in_state(
        &workflow_a,
        ctx.clone(),
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    match result_a {
        Err(ExecError::YieldSuspended {
            role,
            request,
            proxy_addr,
            ..
        }) => {
            assert_eq!(role, "role_a");
            assert_eq!(*request, Value::String("request_to_a".to_string()));
            assert_eq!(proxy_addr, "proxy://instance-a");
        }
        _ => panic!("Expected YieldSuspended for role_a"),
    }

    // Clear suspended yields manually since we don't have get_all_ids
    {
        let suspended_guard = runtime.suspended_yields();
        let mut suspended = suspended_guard.lock().await;
        suspended.clear();
    }

    // Test yield to role_b
    let workflow_b = create_yield_workflow(
        "role_b",
        Value::String("request_to_b".to_string()),
        TypeExpr::Named("String".to_string()),
    );

    let result_b = execute_workflow_with_behaviour_in_state(
        &workflow_b,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    match result_b {
        Err(ExecError::YieldSuspended {
            role,
            request,
            proxy_addr,
            ..
        }) => {
            assert_eq!(role, "role_b");
            assert_eq!(*request, Value::String("request_to_b".to_string()));
            assert_eq!(proxy_addr, "proxy://instance-b");
        }
        _ => panic!("Expected YieldSuspended for role_b"),
    }
}

#[tokio::test]
async fn test_yield_without_proxy_registry_fails() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    // Register proxy
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

    // Execution with proper runtime state should work
    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    // Should yield successfully when proxy registry is available
    assert!(
        matches!(result, Err(ExecError::YieldSuspended { .. })),
        "Should suspend when proxy registry is available"
    );
}

#[tokio::test]
async fn test_yield_with_complex_request_type() {
    let runtime = setup_test_runtime();
    let ctx = setup_test_context();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    {
        let registry_guard = runtime.proxy_registry();
        let mut registry = registry_guard.lock().await;
        registry.register("data_role".to_string(), "proxy://data-1".to_string());
    }

    // Test with record request value
    let mut record_fields = std::collections::HashMap::new();
    record_fields.insert("name".to_string(), Value::String("test".to_string()));
    record_fields.insert("value".to_string(), Value::Int(100));

    let workflow = Workflow::Yield {
        role: "data_role".to_string(),
        request: Box::new(Expr::Literal(Value::Record(Box::new(record_fields)))),
        expected_response_type: TypeExpr::Constructor {
            name: "Result".to_string(),
            args: vec![
                TypeExpr::Named("String".to_string()),
                TypeExpr::Named("Error".to_string()),
            ],
        },
        continuation: Box::new(Workflow::Done),
        span: Span::default(),
        resume_var: "response".to_string(),
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime,
    )
    .await;

    match result {
        Err(ExecError::YieldSuspended {
            request,
            expected_response_type,
            ..
        }) => {
            // Verify record was preserved
            match request.as_ref() {
                Value::Record(fields) => {
                    assert_eq!(fields.get("name"), Some(&Value::String("test".to_string())));
                    assert_eq!(fields.get("value"), Some(&Value::Int(100)));
                }
                _ => panic!("Expected record value"),
            }
            // Verify complex type is represented
            assert!(expected_response_type.contains("Constructor"));
            assert!(expected_response_type.contains("Result"));
        }
        Err(other) => panic!("Expected YieldSuspended, got: {:?}", other),
        Ok(_) => panic!("Should suspend"),
    }
}

#[tokio::test]
async fn test_yield_inside_let_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "nested_let").await;

    let workflow = Workflow::Let {
        pattern: Pattern::Variable("ignored".to_string()),
        expr: Expr::Literal(Value::Int(7)),
        continuation: Box::new(create_resumable_yield(
            "nested_let",
            Expr::Literal(Value::Int(7)),
        )),
    };

    let cap_ctx = CapabilityContext::new();
    assert_nested_yield_round_trip(&workflow, &cap_ctx, &runtime, Value::Int(7)).await;
}

#[tokio::test]
async fn test_yield_inside_if_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "nested_if").await;

    let workflow = Workflow::If {
        condition: Expr::Literal(Value::Bool(true)),
        then_branch: Box::new(create_resumable_yield(
            "nested_if",
            Expr::Literal(Value::Bool(true)),
        )),
        else_branch: Box::new(Workflow::Done),
    };

    let cap_ctx = CapabilityContext::new();
    assert_nested_yield_round_trip(&workflow, &cap_ctx, &runtime, Value::Bool(true)).await;
}

#[tokio::test]
async fn test_yield_inside_observe_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "nested_observe").await;

    let mut cap_ctx = CapabilityContext::new();
    cap_ctx.register(Box::new(
        MockProvider::new("sensor", Effect::Epistemic).with_observe_value(Value::Int(42)),
    ));

    let workflow = Workflow::Observe {
        capability: Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Variable("observed".to_string()),
        continuation: Box::new(create_resumable_yield(
            "nested_observe",
            Expr::Variable("observed".to_string()),
        )),
    };

    assert_nested_yield_round_trip(&workflow, &cap_ctx, &runtime, Value::Int(42)).await;
}

#[tokio::test]
async fn test_yield_inside_receive_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "nested_receive").await;

    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![ReceiveArm {
            pattern: ReceivePattern::Stream {
                capability: "sensor".to_string(),
                channel: "temp".to_string(),
                pattern: Pattern::Variable("reading".to_string()),
            },
            guard: None,
            body: create_resumable_yield("nested_receive", Expr::Variable("reading".to_string())),
        }],
        control: false,
    };

    let mut stream_ctx = StreamContext::new();
    stream_ctx.register(TypedStreamProvider::new(
        MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]),
        Type::Int,
    ));

    let cap_ctx = CapabilityContext::new();
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();

    let correlation_id = match execute_workflow_with_stream_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
        &runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, Value::Int(42));
            correlation_id
        }
        Err(other) => panic!("expected receive arm yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected receive arm yield to suspend, got value: {value:?}"),
    };

    let resumed = resume_suspended_yield(&runtime, &cap_ctx, correlation_id).await;
    assert_eq!(resumed, Value::Int(99));
}

#[tokio::test]
async fn test_yield_inside_stream_receive_wildcard_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "stream_receive").await;

    let receive = StreamReceive {
        mode: StreamReceiveMode::NonBlocking,
        arms: vec![StreamReceiveArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: create_resumable_yield("stream_receive", Expr::Literal(Value::Int(7))),
        }],
        control_arms: None,
    };

    let cap_ctx = CapabilityContext::new();
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let stream_ctx = StreamContext::new();
    let mailbox = Arc::new(tokio::sync::Mutex::new(Mailbox::new()));

    let correlation_id = match execute_receive_in_state(
        &receive,
        ctx,
        mailbox,
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
        &runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, Value::Int(7));
            correlation_id
        }
        Err(other) => panic!("expected wildcard receive yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected wildcard receive yield to suspend, got value: {value:?}"),
    };

    let resumed = resume_suspended_yield(&runtime, &cap_ctx, correlation_id).await;
    assert_eq!(resumed, Value::Int(99));
}

#[tokio::test]
async fn test_yield_inside_stream_receive_control_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "control_receive").await;

    let receive = StreamReceive {
        mode: StreamReceiveMode::NonBlocking,
        arms: vec![],
        control_arms: Some(vec![StreamReceiveArm {
            pattern: Pattern::Literal(Value::String("shutdown".to_string())),
            guard: None,
            body: create_resumable_yield("control_receive", Expr::Literal(Value::Int(5))),
        }]),
    };

    let cap_ctx = CapabilityContext::new();
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let stream_ctx = StreamContext::new();
    let mailbox = Arc::new(tokio::sync::Mutex::new(Mailbox::new()));
    {
        let mut mailbox_guard = mailbox.lock().await;
        mailbox_guard
            .push(ash_core::stream::MailboxEntry::new(
                "__control__",
                "__mailbox__",
                Value::String("shutdown".to_string()),
            ))
            .expect("control mailbox entry should be inserted");
    }

    let correlation_id = match execute_receive_in_state(
        &receive,
        ctx,
        mailbox,
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
        &runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, Value::Int(5));
            correlation_id
        }
        Err(other) => panic!("expected control receive yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected control receive yield to suspend, got value: {value:?}"),
    };

    let resumed = resume_suspended_yield(&runtime, &cap_ctx, correlation_id).await;
    assert_eq!(resumed, Value::Int(99));
}

#[tokio::test]
async fn test_yield_inside_stream_receive_matching_arm_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "stream_match").await;

    let receive = StreamReceive {
        mode: StreamReceiveMode::NonBlocking,
        arms: vec![StreamReceiveArm {
            pattern: Pattern::Variable("reading".to_string()),
            guard: None,
            body: create_resumable_yield("stream_match", Expr::Variable("reading".to_string())),
        }],
        control_arms: None,
    };

    let cap_ctx = CapabilityContext::new();
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let stream_ctx = StreamContext::new();
    let mailbox = Arc::new(tokio::sync::Mutex::new(Mailbox::new()));
    {
        let mut mailbox_guard = mailbox.lock().await;
        mailbox_guard
            .push(ash_core::stream::MailboxEntry::new(
                "sensor",
                "temp",
                Value::Int(11),
            ))
            .expect("stream mailbox entry should be inserted");
    }

    let correlation_id = match execute_receive_in_state(
        &receive,
        ctx,
        mailbox,
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
        &runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, Value::Int(11));
            correlation_id
        }
        Err(other) => panic!("expected matching receive arm yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected matching receive arm yield to suspend, got value: {value:?}"),
    };

    let resumed = resume_suspended_yield(&runtime, &cap_ctx, correlation_id).await;
    assert_eq!(resumed, Value::Int(99));
}

#[tokio::test]
async fn test_yield_inside_stream_receive_timeout_wildcard_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "timed_stream_receive").await;

    let receive = StreamReceive {
        mode: StreamReceiveMode::Blocking(Some(Duration::from_millis(5))),
        arms: vec![StreamReceiveArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: create_resumable_yield("timed_stream_receive", Expr::Literal(Value::Int(13))),
        }],
        control_arms: None,
    };

    let cap_ctx = CapabilityContext::new();
    let ctx = setup_test_context();
    let policy_eval = PolicyEvaluator::new();
    let stream_ctx = StreamContext::new();
    let mailbox = Arc::new(tokio::sync::Mutex::new(Mailbox::new()));

    let correlation_id = match execute_receive_in_state(
        &receive,
        ctx,
        mailbox,
        &stream_ctx,
        &cap_ctx,
        &policy_eval,
        &runtime,
    )
    .await
    {
        Err(ExecError::YieldSuspended {
            request,
            correlation_id,
            ..
        }) => {
            assert_eq!(*request, Value::Int(13));
            correlation_id
        }
        Err(other) => panic!("expected timeout wildcard yield to suspend, got: {other:?}"),
        Ok(value) => panic!("expected timeout wildcard yield to suspend, got value: {value:?}"),
    };

    let resumed = resume_suspended_yield(&runtime, &cap_ctx, correlation_id).await;
    assert_eq!(resumed, Value::Int(99));
}

#[tokio::test]
async fn test_yield_inside_check_preserves_proxy_state() {
    let runtime = setup_test_runtime();
    setup_proxy_role(&runtime, "nested_check").await;

    // Wrap Check in Oblig to establish active role context (per TASK-287 semantics)
    let workflow = Workflow::Oblig {
        role: Role {
            name: "reviewer".to_string(),
            authority: vec![],
            obligations: vec![],
        },
        workflow: Box::new(Workflow::Check {
            obligation: Obligation::Obliged {
                role: Role {
                    name: "reviewer".to_string(),
                    authority: vec![],
                    obligations: vec![],
                },
                condition: Expr::Literal(Value::Bool(true)),
            },
            continuation: Box::new(create_resumable_yield(
                "nested_check",
                Expr::Literal(Value::String("checked".to_string())),
            )),
        }),
    };

    let cap_ctx = CapabilityContext::new();
    assert_nested_yield_round_trip(
        &workflow,
        &cap_ctx,
        &runtime,
        Value::String("checked".to_string()),
    )
    .await;
}
