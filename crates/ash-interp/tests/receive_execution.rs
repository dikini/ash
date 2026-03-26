use std::time::Duration;

use ash_core::{Effect, Value, Workflow};
use ash_interp::TypedStreamProvider;
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::error::ExecError;
use ash_interp::execute::execute_workflow_with_stream;
use ash_interp::stream::{
    BidirectionalStreamProvider, MockBidirectionalStream, MockStreamProvider, StreamContext,
};
use ash_parser::input::new_input;
use ash_parser::lower::lower_workflow;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::Workflow as SurfaceWorkflow;
use ash_typeck::Type;
use ash_typeck::capability_check::CapabilityChecker;
use ash_typeck::runtime_verification::{
    AggregateVerificationInputs, CapabilitySchema, CapabilitySchemaRegistry,
    ObligationRequirements, RuntimeContext, VerificationAggregator, WorkflowCapabilities,
};

fn parse_surface_and_lower(source: &str) -> (SurfaceWorkflow, Workflow) {
    let mut input = new_input(source);
    let parsed = workflow_def(&mut input).expect("workflow should parse");
    let surface = parsed.body.clone();
    let lowered = lower_workflow(&parsed).expect("lowering should succeed");
    (surface, lowered)
}

fn execution_contexts() -> (
    Context,
    CapabilityContext,
    ash_interp::policy::PolicyEvaluator,
    BehaviourContext,
    StreamContext,
) {
    (
        Context::new(),
        CapabilityContext::new(),
        ash_interp::policy::PolicyEvaluator::new(),
        BehaviourContext::new(),
        StreamContext::new(),
    )
}

#[tokio::test]
async fn parsed_non_blocking_receive_executes_matching_stream_arm() {
    let (_surface, lowered) = parse_surface_and_lower(
        "workflow main { receive { sensor:temp as reading => ret reading, _ => ret 0 } }",
    );
    let (ctx, cap_ctx, policy_eval, behaviour_ctx, mut stream_ctx) = execution_contexts();
    stream_ctx.register(TypedStreamProvider::new(
        MockStreamProvider::with_values("sensor", "temp", vec![Value::Int(42)]),
        Type::Int,
    ));

    let result = execute_workflow_with_stream(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
    )
    .await
    .expect("receive should execute");

    assert_eq!(result, Value::Int(42));
}

#[tokio::test]
async fn parsed_blocking_receive_waits_until_stream_value_arrives() {
    let (_surface, lowered) = parse_surface_and_lower(
        "workflow main { receive wait { sensor:temp as reading => ret reading } }",
    );
    let (ctx, cap_ctx, policy_eval, behaviour_ctx, mut stream_ctx) = execution_contexts();
    let provider = MockBidirectionalStream::new("sensor", "temp");
    let delayed_push = provider.clone();
    stream_ctx.register(TypedStreamProvider::new(
        BidirectionalStreamProvider::new(provider, Type::Int, Type::Int),
        Type::Int,
    ));

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        delayed_push.push(Value::Int(7));
    });

    let result = tokio::time::timeout(
        Duration::from_millis(200),
        execute_workflow_with_stream(
            &lowered,
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &stream_ctx,
        ),
    )
    .await
    .expect("receive wait should complete before the test timeout")
    .expect("receive wait should succeed");

    assert_eq!(result, Value::Int(7));
}

#[tokio::test]
async fn parsed_timed_receive_falls_back_to_wildcard_on_timeout() {
    let (_surface, lowered) = parse_surface_and_lower(
        "workflow main { receive wait 1s { sensor:temp as reading => ret reading, _ => ret 0 } }",
    );
    let (ctx, cap_ctx, policy_eval, behaviour_ctx, mut stream_ctx) = execution_contexts();
    stream_ctx.register(TypedStreamProvider::new(
        MockStreamProvider::new("sensor", "temp"),
        Type::Int,
    ));

    let result = execute_workflow_with_stream(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
    )
    .await
    .expect("timed receive should fall through to wildcard");

    assert_eq!(result, Value::Int(0));
}

#[tokio::test]
async fn canonical_receive_ignores_unrelated_stream_providers() {
    let (_surface, lowered) = parse_surface_and_lower(
        "workflow main { receive { sensor:temp as reading => ret reading, _ => ret 0 } }",
    );
    let (ctx, cap_ctx, policy_eval, behaviour_ctx, mut stream_ctx) = execution_contexts();
    stream_ctx.register(TypedStreamProvider::new(
        MockStreamProvider::with_values("other", "noise", vec![Value::Int(99)]),
        Type::Int,
    ));
    stream_ctx.register(TypedStreamProvider::new(
        MockStreamProvider::new("sensor", "temp"),
        Type::Int,
    ));

    let result = execute_workflow_with_stream(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
    )
    .await
    .expect("receive should ignore unrelated providers and fall back");

    assert_eq!(result, Value::Int(0));
    assert_eq!(
        stream_ctx
            .try_recv("other", "noise")
            .expect("unrelated provider should still hold its value")
            .expect("unrelated provider read should succeed"),
        Value::Int(99)
    );
}

#[tokio::test]
async fn parsed_control_receive_consumes_the_implicit_control_mailbox() {
    let (_surface, lowered) = parse_surface_and_lower(
        "workflow main { receive control { \"shutdown\" => ret 1, _ => ret 0 } }",
    );
    let (ctx, cap_ctx, policy_eval, behaviour_ctx, stream_ctx) = execution_contexts();
    stream_ctx.push_control(Value::String("shutdown".to_string()));

    let result = execute_workflow_with_stream(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
    )
    .await
    .expect("control receive should succeed");

    assert_eq!(result, Value::Int(1));
}

#[tokio::test]
async fn canonical_receive_reports_missing_stream_provider() {
    let (surface, lowered) = parse_surface_and_lower(
        "workflow main { receive { sensor:temp as reading => ret reading } }",
    );

    assert!(
        CapabilityChecker::new()
            .receive("sensor", "temp")
            .verify(&surface)
            .is_ok(),
        "compile-time declaration checking should accept the declared receive binding"
    );

    let (ctx, cap_ctx, policy_eval, behaviour_ctx, stream_ctx) = execution_contexts();
    let error = execute_workflow_with_stream(
        &lowered,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &stream_ctx,
    )
    .await
    .expect_err("missing runtime stream provider should fail");

    assert_eq!(
        error,
        ExecError::CapabilityNotAvailable("sensor:temp".to_string())
    );
}

#[test]
fn runtime_verification_keeps_receive_capabilities_separate_from_obligation_requirements() {
    let (surface, _lowered) = parse_surface_and_lower(
        "workflow main { receive { sensor:temp as reading => ret reading } }",
    );

    assert!(
        CapabilityChecker::new()
            .receive("sensor", "temp")
            .verify(&surface)
            .is_ok(),
        "surface receive should still require a declared stream binding"
    );

    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities {
            receives: vec![("sensor".into(), "temp".into())],
            ..WorkflowCapabilities::default()
        },
        ObligationRequirements::new(),
    );
    let runtime = RuntimeContext::new(Effect::Operational).with_capabilities({
        let mut registry = CapabilitySchemaRegistry::new();
        registry.register(
            "sensor".into(),
            "temp".into(),
            CapabilitySchema::read_only(Type::Int),
        );
        registry
    });

    let result = VerificationAggregator::new().aggregate(&surface, &inputs, &runtime);
    assert!(
        result.can_execute(),
        "receive capability declarations must not imply separate obligation requirements"
    );
}
