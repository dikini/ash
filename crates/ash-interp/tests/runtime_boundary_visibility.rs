use ash_core::{Capability, ControlLink, Effect, Expr, Pattern, Value, Workflow, WorkflowId};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::error::{EvalError, ExecError};
use ash_interp::execute::{
    execute_workflow_with_behaviour, execute_workflow_with_behaviour_in_state,
};
use ash_interp::policy::PolicyEvaluator;
use ash_interp::{ControlLinkError, RetainedCompletionKind, RuntimeOutcomeState, RuntimeState};
use std::collections::BTreeSet;
use tokio::time::{Duration, timeout};

fn effect_set(effects: &[Effect]) -> BTreeSet<Effect> {
    effects.iter().copied().collect()
}

fn obligation_set(obligations: &[&str]) -> BTreeSet<String> {
    obligations.iter().map(|name| (*name).to_string()).collect()
}

fn assert_retained_child_success(
    completion: &ash_interp::RetainedCompletionRecord,
    expected_value: Value,
) {
    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::TerminalSuccess
    );
    assert_eq!(completion.terminal_result(), Some(&Ok(expected_value)));
}

fn assert_retained_child_failure(
    completion: &ash_interp::RetainedCompletionRecord,
    expected_error: ExecError,
) {
    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(completion.terminal_result(), Some(&Err(expected_error)));
}

fn assert_conservative_effect_summary(
    completion: &ash_interp::RetainedCompletionRecord,
    expected_terminal_upper_bound: Effect,
    expected_reached_upper_bound: &[Effect],
) {
    let effects = completion
        .conservative_effect_summary()
        .expect("completed child should retain conservative effect summary contents");
    assert_eq!(
        effects.terminal_upper_bound(),
        expected_terminal_upper_bound
    );
    assert_eq!(
        effects.reached_upper_bound(),
        &effect_set(expected_reached_upper_bound)
    );
}

fn assert_conservative_obligations_summary(
    completion: &ash_interp::RetainedCompletionRecord,
    expected_local_pending: &[&str],
    expected_active_role: Option<&str>,
    expected_role_pending: &[&str],
    expected_role_discharged: &[&str],
) {
    let obligations = completion
        .conservative_obligations_summary()
        .expect("completed child should retain conservative obligations summary contents");
    assert_eq!(
        obligations.local_pending_visible_at_terminal(),
        &obligation_set(expected_local_pending)
    );
    assert_eq!(
        obligations.active_role_visible_at_terminal(),
        expected_active_role
    );
    assert_eq!(
        obligations.role_pending_visible_at_terminal(),
        &obligation_set(expected_role_pending)
    );
    assert_eq!(
        obligations.role_discharged_visible_at_terminal(),
        &obligation_set(expected_role_discharged)
    );
}

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

async fn wait_for_retained_completion(
    runtime_state: &RuntimeState,
    link: &ControlLink,
) -> ash_interp::RetainedCompletionRecord {
    timeout(
        Duration::from_secs(1),
        runtime_state.wait_for_retained_completion(link),
    )
    .await
    .expect("spawned child should eventually seal retained completion")
    .expect("completion wait should return the sealed retained record")
}

fn spawn_and_return_control(init: Expr) -> Workflow {
    Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init,
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
    }
}

#[tokio::test]
async fn stateful_execution_preserves_control_link_authority_across_top_level_runs() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow("worker", Workflow::Done)
        .await;

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
    runtime_state
        .register_child_workflow("worker", Workflow::Done)
        .await;

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

    assert!(
        matches!(error, ExecError::InvalidRuntimeState(message) if message.contains("terminated"))
    );
}

#[tokio::test]
async fn spawn_without_registered_workflow_type_returns_honest_instance_without_control() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();

    let instance = execute_workflow_with_behaviour_in_state(
        &Workflow::Spawn {
            workflow_type: "worker".to_string(),
            init: Expr::Literal(Value::Int(7)),
            pattern: Pattern::Variable("worker".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("worker".to_string()),
            }),
        },
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn without a registered child workflow should still return an instance value");

    let Value::Instance(instance) = instance else {
        panic!("expected returned instance, got {instance:?}");
    };

    assert!(instance.control.is_none());
}

#[tokio::test]
async fn spawn_preserves_live_control_authority_before_any_retained_completion_is_recorded() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Variable("init".to_string()),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Int(7))),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should produce a control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    assert!(runtime_state.retained_completion(&link).await.is_none());
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::Active
    );
}

#[tokio::test]
async fn retained_completion_is_automatically_sealed_from_real_spawned_child_lifecycle() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Variable("init".to_string()),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Int(7))),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(completion.instance_id(), link.instance_id);
    assert_retained_child_success(&completion, Value::Int(7));
    assert_conservative_effect_summary(&completion, Effect::Epistemic, &[Effect::Epistemic]);
    assert_conservative_obligations_summary(&completion, &[], None, &[], &[]);
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::InvalidOrTerminated
    );
}

#[tokio::test]
async fn retained_completion_is_write_once_after_automatic_child_sealing() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Variable("init".to_string()),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Int(9))),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let sealed = loop {
        if let Some(record) = runtime_state.retained_completion(&link).await {
            break record;
        }
        tokio::task::yield_now().await;
    };

    assert_retained_child_success(&sealed, Value::Int(9));

    let error = runtime_state
        .record_control_completion(
            &link,
            Ok(Value::Int(10)),
            ash_interp::ConservativeRetainedEffectSummary::new(
                Effect::Operational,
                effect_set(&[Effect::Operational]),
            ),
            ash_interp::ConservativeRetainedObligationsSummary::new(
                BTreeSet::new(),
                None,
                BTreeSet::new(),
                BTreeSet::new(),
            ),
            None,
        )
        .await
        .expect_err("automatic sealing must remain write-once/stable");

    assert_eq!(
        error,
        ControlLinkError::CompletionAlreadySealed(link.instance_id, Box::new(sealed.clone()))
    );
    assert_eq!(runtime_state.retained_completion(&link).await, Some(sealed));
}

#[tokio::test]
async fn spawned_child_runtime_path_can_execute_failure_and_seal_it() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Variable("missing_child_value".to_string()),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Null)),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let completion = loop {
        if let Some(record) = runtime_state.retained_completion(&link).await {
            break record;
        }
        tokio::task::yield_now().await;
    };

    assert_eq!(completion.instance_id(), link.instance_id);
    assert_retained_child_failure(
        &completion,
        ExecError::Eval(EvalError::UndefinedVariable("missing_child_value".into())),
    );
    assert_conservative_effect_summary(&completion, Effect::Epistemic, &[Effect::Epistemic]);
    assert_conservative_obligations_summary(&completion, &[], None, &[], &[]);
}

#[tokio::test]
async fn retained_completion_preserves_terminal_visible_obligations_contents() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Oblig {
                role: ash_core::Role {
                    name: "reviewer".to_string(),
                    authority: vec![],
                    obligations: vec![
                        ash_core::RoleObligationRef {
                            name: "audit".to_string(),
                        },
                        ash_core::RoleObligationRef {
                            name: "log".to_string(),
                        },
                    ],
                },
                workflow: Box::new(Workflow::Seq {
                    first: Box::new(Workflow::Oblige {
                        name: "local-audit".to_string(),
                        span: ash_core::workflow_contract::Span { start: 0, end: 1 },
                    }),
                    second: Box::new(Workflow::Seq {
                        first: Box::new(Workflow::Check {
                            obligation: ash_core::Obligation::Obliged {
                                role: ash_core::Role {
                                    name: "reviewer".to_string(),
                                    authority: vec![],
                                    obligations: vec![],
                                },
                                condition: Expr::Literal(Value::Bool(true)),
                            },
                            continuation: Box::new(Workflow::Done),
                        }),
                        second: Box::new(Workflow::Ret {
                            expr: Expr::Literal(Value::String("done".to_string())),
                        }),
                    }),
                }),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Null)),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let completion = loop {
        if let Some(record) = runtime_state.retained_completion(&link).await {
            break record;
        }
        tokio::task::yield_now().await;
    };

    assert_retained_child_success(&completion, Value::String("done".to_string()));
    assert_conservative_obligations_summary(
        &completion,
        &[],
        Some("reviewer"),
        &["audit", "log"],
        &[],
    );
}

#[tokio::test]
async fn retained_completion_preserves_conservative_multi_effect_summary_contents() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new()
        .with_provider(
            "sensor",
            std::sync::Arc::new(
                ash_interp::MockProvider::new("sensor", Effect::Epistemic)
                    .with_observe_value(Value::Int(1)),
            ),
        )
        .with_provider(
            "deploy",
            std::sync::Arc::new(
                ash_interp::MockProvider::new("deploy", Effect::Operational)
                    .with_execute_result(Ok(Value::String("done".to_string()))),
            ),
        );
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Seq {
                first: Box::new(Workflow::Observe {
                    capability: Capability {
                        name: "sensor".to_string(),
                        effect: Effect::Epistemic,
                        constraints: vec![],
                    },
                    pattern: Pattern::Variable("seen".to_string()),
                    continuation: Box::new(Workflow::Done),
                }),
                second: Box::new(Workflow::Act {
                    action: ash_core::Action {
                        name: "deploy".to_string(),
                        arguments: vec![],
                    },
                    guard: ash_core::Guard::Always,
                    provenance: ash_core::Provenance::new(),
                }),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Null)),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let completion = loop {
        if let Some(record) = runtime_state.retained_completion(&link).await {
            break record;
        }
        tokio::task::yield_now().await;
    };

    assert_retained_child_success(&completion, Value::String("done".to_string()));
    assert_conservative_effect_summary(
        &completion,
        Effect::Operational,
        &[Effect::Epistemic, Effect::Operational],
    );
}

#[tokio::test]
async fn conservative_effect_summary_can_overapproximate_untaken_higher_effect_paths() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Seq {
                first: Box::new(Workflow::Ret {
                    expr: Expr::Variable("missing_before_operational".to_string()),
                }),
                second: Box::new(Workflow::Act {
                    action: ash_core::Action {
                        name: "deploy".to_string(),
                        arguments: vec![],
                    },
                    guard: ash_core::Guard::Always,
                    provenance: ash_core::Provenance::new(),
                }),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Null)),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should succeed");

    let link = match control {
        Value::ControlLink(link) => link,
        value => panic!("expected retained control link, got {value:?}"),
    };

    let completion = loop {
        if let Some(record) = runtime_state.retained_completion(&link).await {
            break record;
        }
        tokio::task::yield_now().await;
    };

    assert_retained_child_failure(
        &completion,
        ExecError::Eval(EvalError::UndefinedVariable(
            "missing_before_operational".into(),
        )),
    );
    let effects = completion
        .conservative_effect_summary()
        .expect("completed child should retain conservative effect summary contents");
    assert_eq!(effects.terminal_upper_bound(), Effect::Operational);
    assert_eq!(
        effects.reached_upper_bound(),
        &effect_set(&[Effect::Operational])
    );
    assert_conservative_obligations_summary(&completion, &[], None, &[], &[]);
}

#[tokio::test]
async fn retained_control_tombstones_remain_distinct_from_child_payloads() {
    let runtime_state = RuntimeState::new();
    let instance_id = WorkflowId::new();
    let link = ControlLink { instance_id };

    runtime_state
        .register_spawned_control_link(instance_id)
        .await;
    runtime_state
        .kill_control_link(&link)
        .await
        .expect("kill should preserve a control tombstone");

    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(completion.instance_id(), link.instance_id);
    assert_eq!(completion.kind(), RetainedCompletionKind::ControlTerminated);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
    assert_eq!(completion.terminal_result(), None);
    assert_eq!(completion.conservative_effect_summary(), None);
    assert_eq!(completion.conservative_obligations_summary(), None);
}

#[tokio::test]
async fn unregistered_control_link_has_no_retained_completion_record() {
    let runtime_state = RuntimeState::new();
    let link = ControlLink {
        instance_id: WorkflowId::new(),
    };

    assert!(runtime_state.retained_completion(&link).await.is_none());
}

#[tokio::test]
async fn completion_wait_returns_immediately_for_already_sealed_records() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Variable("init".to_string()),
            },
        )
        .await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(Expr::Literal(Value::Int(11))),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should hand back control authority without forcing inline termination");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let sealed = wait_for_retained_completion(&runtime_state, &link).await;
    let waited = timeout(
        Duration::from_millis(50),
        runtime_state.wait_for_retained_completion(&link),
    )
    .await
    .expect("already-sealed retained completion should return immediately")
    .expect("already-sealed retained completion should still be readable");

    assert_eq!(waited, sealed);
}

#[tokio::test]
async fn completion_wait_rejects_unregistered_targets_without_hanging_or_synthesizing_completion() {
    let runtime_state = RuntimeState::new();
    let link = ControlLink {
        instance_id: WorkflowId::new(),
    };

    let error = timeout(
        Duration::from_millis(50),
        runtime_state.wait_for_retained_completion(&link),
    )
    .await
    .expect("unregistered completion wait should not hang")
    .expect_err("unregistered completion wait should not synthesize a retained record");

    assert_eq!(error, ControlLinkError::NotFound(link.instance_id));
    assert!(runtime_state.retained_completion(&link).await.is_none());
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
