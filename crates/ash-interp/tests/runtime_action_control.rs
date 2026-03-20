use ash_core::{
    Action, ControlLink, Effect, Expr, Guard, Pattern, Provenance, Value, Workflow, WorkflowId,
};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::{CapabilityContext, MockProvider};
use ash_interp::context::Context;
use ash_interp::error::ExecError;
use ash_interp::execute::execute_workflow_with_behaviour;
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

fn spawn_with_control(continuation: Workflow) -> Workflow {
    Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: Pattern::Variable("worker".to_string()),
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable("worker".to_string()),
            pattern: Pattern::Tuple(vec![
                Pattern::Wildcard,
                Pattern::Variable("ctrl".to_string()),
            ]),
            continuation: Box::new(continuation),
        }),
    }
}

fn spawn_and_return_control() -> Workflow {
    spawn_with_control(Workflow::Ret {
        expr: Expr::Variable("ctrl".to_string()),
    })
}

#[tokio::test]
async fn act_executes_registered_operational_provider() {
    let (ctx, mut cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    cap_ctx.register(Box::new(
        MockProvider::new("deploy", Effect::Operational)
            .with_execute_result(Ok(Value::String("deployed".to_string()))),
    ));

    let workflow = Workflow::Act {
        action: Action {
            name: "deploy".to_string(),
            arguments: vec![],
        },
        guard: Guard::Always,
        provenance: Provenance::new(),
    };

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("action execution should use the provider");

    assert_eq!(result, Value::String("deployed".to_string()));
}

#[tokio::test]
async fn act_guard_failure_still_rejects_execution() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let workflow = Workflow::Act {
        action: Action {
            name: "deploy".to_string(),
            arguments: vec![],
        },
        guard: Guard::Never,
        provenance: Provenance::new(),
    };

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("guard failure should stop action execution");

    assert!(matches!(error, ExecError::GuardFailed { .. }));
}

#[tokio::test]
async fn spawned_control_link_supports_pause_health_and_resume() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let workflow = spawn_with_control(Workflow::Pause {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::CheckHealth {
            target: "ctrl".to_string(),
            continuation: Box::new(Workflow::Resume {
                target: "ctrl".to_string(),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                }),
            }),
        }),
    });

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("reusable control link should remain valid across non-terminal operations");

    assert_eq!(result, Value::Int(1));
}

#[tokio::test]
async fn transferred_control_link_remains_valid_across_executions() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let control = execute_workflow_with_behaviour(
        &spawn_and_return_control(),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
    )
    .await
    .expect("spawn should return a control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let mut next_ctx = Context::new();
    next_ctx.set("ctrl".to_string(), Value::ControlLink(link));

    let workflow = Workflow::Pause {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::CheckHealth {
            target: "ctrl".to_string(),
            continuation: Box::new(Workflow::Resume {
                target: "ctrl".to_string(),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(2)),
                }),
            }),
        }),
    };

    let result = execute_workflow_with_behaviour(
        &workflow,
        next_ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
    )
    .await
    .expect("transferred control link should remain usable in a later execution");

    assert_eq!(result, Value::Int(2));
}

#[tokio::test]
async fn kill_invalidates_future_control_operations() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let workflow = spawn_with_control(Workflow::Kill {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::CheckHealth {
            target: "ctrl".to_string(),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
        }),
    });

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("terminal control should invalidate the link");

    assert!(matches!(error, ExecError::ExecutionFailed(message) if message.contains("terminated")));
}

#[tokio::test]
async fn unregistered_control_link_does_not_fall_through() {
    let (mut ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    ctx.set(
        "ctrl".to_string(),
        Value::ControlLink(ControlLink {
            instance_id: WorkflowId::new(),
        }),
    );

    let workflow = Workflow::Pause {
        target: "ctrl".to_string(),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        }),
    };

    let error =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect_err("unregistered control links should be rejected");

    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message.contains("not registered"))
    );
}
