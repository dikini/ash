use ash_core::{Capability, Effect, Expr, Pattern, Value, Workflow};
use ash_interp::RuntimeState;
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::error::ExecError;
use ash_interp::execute::{
    execute_workflow_with_behaviour, execute_workflow_with_behaviour_in_state,
};
use ash_interp::policy::PolicyEvaluator;

fn execution_contexts() -> (
    Context,
    CapabilityContext,
    PolicyEvaluator,
    BehaviourContext,
) {
    (
        Context::new(),
        CapabilityContext::new(),
        PolicyEvaluator::new(),
        BehaviourContext::new(),
    )
}

#[tokio::test]
async fn stateful_execution_preserves_control_link_authority_across_top_level_runs() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();

    let spawn = Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: Pattern::Variable("worker".to_string()),
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable("worker".to_string()),
            pattern: Pattern::Tuple(vec![
                Pattern::Wildcard,
                Pattern::Variable("ctrl".to_string()),
            ]),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("ctrl".to_string()),
            }),
        }),
    };

    let control = execute_workflow_with_behaviour_in_state(
        &spawn,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should produce a control link");

    let mut next_ctx = Context::new();
    next_ctx.set("ctrl".to_string(), control);

    let pause = Workflow::Pause {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        }),
    };

    let result = execute_workflow_with_behaviour_in_state(
        &pause,
        next_ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("runtime-owned state should preserve control authority");

    assert_eq!(result, Value::Int(1));
}

#[tokio::test]
async fn terminated_control_links_remain_observable_as_tombstones_across_runs() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();

    let spawn = Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: Pattern::Variable("worker".to_string()),
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable("worker".to_string()),
            pattern: Pattern::Tuple(vec![
                Pattern::Wildcard,
                Pattern::Variable("ctrl".to_string()),
            ]),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("ctrl".to_string()),
            }),
        }),
    };

    let control = execute_workflow_with_behaviour_in_state(
        &spawn,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should produce a control link");

    let mut kill_ctx = Context::new();
    kill_ctx.set("ctrl".to_string(), control.clone());

    let kill = Workflow::Kill {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Null),
        }),
    };

    execute_workflow_with_behaviour_in_state(
        &kill,
        kill_ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("kill should terminate the link");

    let mut next_ctx = Context::new();
    next_ctx.set("ctrl".to_string(), control);

    let check = Workflow::CheckHealth {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        }),
    };

    let error = execute_workflow_with_behaviour_in_state(
        &check,
        next_ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect_err("terminated links should remain observable as tombstones");

    assert!(matches!(error, ExecError::ExecutionFailed(message) if message.contains("terminated")));
}

#[tokio::test]
async fn send_without_stream_context_reports_runtime_boundary_failure() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let workflow = Workflow::Send {
        capability: "queue".to_string(),
        channel: "output".to_string(),
        value: Expr::Literal(Value::Int(42)),
    };

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("send should reject when no stream context is present");

    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message.contains("Send requires StreamContext"))
    );
}

#[tokio::test]
async fn receive_without_stream_context_reports_runtime_boundary_failure() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let workflow = Workflow::Receive {
        mode: ash_core::ReceiveMode::NonBlocking,
        arms: vec![],
        control: false,
    };

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("receive should reject when no stream context is present");

    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message.contains("Receive requires StreamContext"))
    );
}

#[tokio::test]
async fn observe_missing_provider_reports_capability_not_available() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let workflow = Workflow::Observe {
        capability: Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Variable("x".to_string()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable("x".to_string()),
        }),
    };

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("observe should reject missing providers explicitly");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "sensor"));
}
