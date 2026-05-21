use ash_core::capability::CapabilityError;
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Constraint, ControlLink, Effect,
    Expr, Guard, Pattern, Provenance, Value, Workflow, WorkflowId,
};
use ash_interp::RuntimeState;
use ash_interp::act_env::ActEnv;
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::{CapabilityContext, CapabilityProvider, MockProvider};
use ash_interp::context::Context;
use ash_interp::error::{EvalError, ExecError};
use ash_interp::eval::eval_expr;
use ash_interp::execute::{
    execute_workflow_with_behaviour, execute_workflow_with_behaviour_in_state,
};
use ash_interp::policy::PolicyEvaluator;
use ash_interp::{RetainedCompletionKind, RuntimeOutcomeState};
use std::collections::BTreeSet;

fn effect_set(effects: &[Effect]) -> BTreeSet<Effect> {
    effects.iter().copied().collect()
}
use async_trait::async_trait;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, Notify, oneshot};
use tokio::time::{Duration, timeout};

fn assert_retained_provenance_workflow_id(
    completion: &ash_interp::RetainedCompletionRecord,
    expected_workflow_id: WorkflowId,
) {
    let retained = completion
        .conservative_provenance_summary()
        .expect("completed child should retain provenance contents");
    assert_eq!(retained.workflow_id(), expected_workflow_id);
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

fn host_binding(name: &str, provider_name: &str, admitted: Vec<&str>) -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        name,
        CapabilityInterfaceId::new("ActionControl"),
        provider_name,
        admitted.into_iter().map(str::to_string).collect(),
    )
}

async fn admit_host_binding(
    runtime_state: &RuntimeState,
    name: &str,
    provider_name: &str,
    admitted: Vec<&str>,
) -> CapabilityBindingId {
    let binding = host_binding(name, provider_name, admitted);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("test host binding admission should succeed");
    binding_id
}

fn invoke_expr() -> Expr {
    Expr::Call {
        func: "invoke".to_string(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String("sensor".to_string())),
            Expr::Literal(Value::String("read".to_string())),
            Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
        ],
    }
}

fn spawn_with_control(continuation: Workflow) -> Workflow {
    Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: Pattern::Variable {
            name: "worker".to_string(),
            span: ash_core::ast::Span::default(),
        },
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable {
                name: "worker".to_string(),
                span: ash_core::ast::Span::default(),
            },
            pattern: Pattern::Tuple(vec![
                Pattern::Wildcard,
                Pattern::Variable {
                    name: "ctrl".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ]),
            continuation: Box::new(continuation),
        }),
    }
}

fn spawn_and_return_control() -> Workflow {
    spawn_with_control(Workflow::Ret {
        expr: Expr::Variable {
            name: "ctrl".to_string(),
            span: ash_core::ast::Span::default(),
        },
    })
}

async fn runtime_state_with_registered_worker(worker: Workflow) -> RuntimeState {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_child_workflow("worker", worker)
        .await;
    runtime_state
}

#[derive(Debug)]
struct BlockingActionProvider {
    name: String,
    started: Arc<Notify>,
    release_rx: Mutex<Option<oneshot::Receiver<()>>>,
    calls: Arc<AtomicUsize>,
}

impl BlockingActionProvider {
    fn new(
        name: &str,
        started: Arc<Notify>,
        release_rx: oneshot::Receiver<()>,
        calls: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            name: name.to_string(),
            started,
            release_rx: Mutex::new(Some(release_rx)),
            calls,
        }
    }
}

#[async_trait]
impl CapabilityProvider for BlockingActionProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("blocking action test provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_waiters();
        if let Some(release_rx) = self.release_rx.lock().await.take() {
            let _ = release_rx.await;
        }
        Ok(Value::String("released".to_string()))
    }
}

#[derive(Debug)]
struct CountingActionProvider {
    name: String,
    calls: Arc<AtomicUsize>,
    started: Arc<Notify>,
}

impl CountingActionProvider {
    fn new(name: &str, calls: Arc<AtomicUsize>, started: Arc<Notify>) -> Self {
        Self {
            name: name.to_string(),
            calls,
            started,
        }
    }
}

#[async_trait]
impl CapabilityProvider for CountingActionProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("counting action test provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_waiters();
        Ok(Value::String("counted".to_string()))
    }
}

fn blocking_child_action_workflow() -> Workflow {
    Workflow::Act {
        provider_name: "block".to_string(),
        action_name: "block".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    }
}

fn child_two_step_action_workflow() -> Workflow {
    Workflow::Seq {
        first: Box::new(blocking_child_action_workflow()),
        second: Box::new(Workflow::Act {
            provider_name: "mark".to_string(),
            action_name: "mark".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: None,
            continuation: Box::new(Workflow::Done),
        }),
    }
}

async fn runtime_state_with_blocking_worker(
    started: Arc<Notify>,
    release_rx: oneshot::Receiver<()>,
    calls: Arc<AtomicUsize>,
) -> RuntimeState {
    runtime_state_with_registered_worker(blocking_child_action_workflow())
        .await
        .with_provider(
            "block",
            Arc::new(BlockingActionProvider::new(
                "block", started, release_rx, calls,
            )),
        )
}

async fn spawn_real_child_control(runtime_state: &RuntimeState) -> ash_core::ControlLink {
    spawn_real_child_control_with_admitted(runtime_state, Vec::new()).await
}

async fn spawn_real_child_control_with_admitted(
    runtime_state: &RuntimeState,
    admitted_bindings: Vec<CapabilityBindingId>,
) -> ash_core::ControlLink {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(),
        ctx.with_admitted_capability_bindings(admitted_bindings),
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime_state,
    )
    .await
    .expect("spawn should return a control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    link
}

async fn wait_for_retained_completion(
    runtime_state: &RuntimeState,
    link: &ash_core::ControlLink,
) -> ash_interp::RetainedCompletionRecord {
    timeout(
        Duration::from_secs(1),
        runtime_state.wait_for_retained_completion(link),
    )
    .await
    .expect("spawned child should eventually seal retained completion")
    .expect("completion wait should return the sealed retained record")
}

async fn wait_for_runtime_state(
    runtime_state: &RuntimeState,
    link: &ash_core::ControlLink,
    expected: RuntimeOutcomeState,
) {
    timeout(Duration::from_secs(1), async {
        loop {
            if runtime_state.control_link_runtime_outcome_state(link).await == expected {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("spawned child runtime state should eventually converge");
}

#[tokio::test]
async fn act_executes_registered_operational_provider() {
    let (ctx, mut cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    cap_ctx.register(Box::new(
        MockProvider::new("deploy", Effect::Operational)
            .with_execute_result(Ok(Value::String("deployed".to_string()))),
    ));

    let workflow = Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
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
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Never,
        provenance: Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
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
    let runtime_state = runtime_state_with_registered_worker(Workflow::Done).await;
    tokio::task::yield_now().await;
    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should return a control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    runtime_state
        .pause_control_link(&link)
        .await
        .expect("pause should succeed for a live child control link");
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::BlockedOrSuspended
    );

    runtime_state
        .resume_control_link(&link)
        .await
        .expect("resume should succeed for a paused child control link");
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::Active
    );
}

#[tokio::test]
async fn transferred_control_link_remains_valid_across_executions() {
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = runtime_state_with_registered_worker(Workflow::Done).await;
    tokio::task::yield_now().await;

    let control = execute_workflow_with_behaviour_in_state(
        &spawn_and_return_control(),
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("spawn should return a control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    runtime_state
        .pause_control_link(&link)
        .await
        .expect("transferred control link should remain pausable in a later execution");
    runtime_state
        .resume_control_link(&link)
        .await
        .expect("transferred control link should remain resumable in a later execution");

    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::Active
    );
}

#[tokio::test]
async fn spawned_control_link_is_not_eagerly_terminated_before_supervisor_can_use_it() {
    let started_block = Arc::new(Notify::new());
    let (release_tx, release_rx) = oneshot::channel();
    let block_calls = Arc::new(AtomicUsize::new(0));

    let runtime_state =
        runtime_state_with_blocking_worker(started_block.clone(), release_rx, block_calls.clone())
            .await;
    let binding_id = admit_host_binding(
        &runtime_state,
        "workflow-block",
        "block",
        vec!["block.block"],
    )
    .await;

    let link = spawn_real_child_control_with_admitted(&runtime_state, vec![binding_id]).await;

    timeout(Duration::from_millis(250), started_block.notified())
        .await
        .expect("real spawned child should begin executing the blocking action");
    assert_eq!(block_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        ash_interp::RuntimeOutcomeState::Active
    );

    release_tx
        .send(())
        .expect("test should still control the blocking provider");
}

#[tokio::test]
async fn kill_invalidates_future_control_operations() {
    let started_block = Arc::new(Notify::new());
    let (release_tx, release_rx) = oneshot::channel();
    let block_calls = Arc::new(AtomicUsize::new(0));

    let runtime_state =
        runtime_state_with_blocking_worker(started_block.clone(), release_rx, block_calls.clone())
            .await;
    let binding_id = admit_host_binding(
        &runtime_state,
        "workflow-block",
        "block",
        vec!["block.block"],
    )
    .await;
    let link = spawn_real_child_control_with_admitted(&runtime_state, vec![binding_id]).await;

    timeout(Duration::from_millis(250), started_block.notified())
        .await
        .expect("real spawned child should begin executing the blocking action");
    assert_eq!(block_calls.load(Ordering::SeqCst), 1);

    runtime_state
        .kill_control_link(&link)
        .await
        .expect("terminal control should invalidate the link");
    let error = runtime_state
        .pause_control_link(&link)
        .await
        .expect_err("future control operations should fail after kill");

    assert!(
        matches!(error, ash_interp::ControlLinkError::Terminated(id) if id == link.instance_id)
    );

    release_tx
        .send(())
        .expect("test should still control the blocking provider");
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
        matches!(error, ExecError::InvalidRuntimeState(message) if message.contains("not registered"))
    );
}

#[tokio::test]
async fn pause_blocks_real_spawned_child_progress_until_resume() {
    let started_block = Arc::new(Notify::new());
    let started_mark = Arc::new(Notify::new());
    let (release_tx, release_rx) = oneshot::channel();
    let block_calls = Arc::new(AtomicUsize::new(0));
    let mark_calls = Arc::new(AtomicUsize::new(0));

    let runtime_state = runtime_state_with_registered_worker(child_two_step_action_workflow())
        .await
        .with_provider(
            "block",
            Arc::new(BlockingActionProvider::new(
                "block",
                started_block.clone(),
                release_rx,
                block_calls.clone(),
            )),
        )
        .with_provider(
            "mark",
            Arc::new(CountingActionProvider::new(
                "mark",
                mark_calls.clone(),
                started_mark.clone(),
            )),
        );
    let block_binding_id = admit_host_binding(
        &runtime_state,
        "workflow-block",
        "block",
        vec!["block.block"],
    )
    .await;
    let mark_binding_id =
        admit_host_binding(&runtime_state, "workflow-mark", "mark", vec!["mark.mark"]).await;

    let link = spawn_real_child_control_with_admitted(
        &runtime_state,
        vec![block_binding_id, mark_binding_id],
    )
    .await;

    timeout(Duration::from_millis(250), started_block.notified())
        .await
        .expect("real spawned child should begin executing the first action");
    assert_eq!(block_calls.load(Ordering::SeqCst), 1);
    runtime_state
        .pause_control_link(&link)
        .await
        .expect("pause should succeed for live child");

    release_tx
        .send(())
        .expect("test should still control the blocking provider");

    timeout(Duration::from_millis(100), started_mark.notified())
        .await
        .expect_err("paused child must not advance to the next workflow step");
    assert_eq!(mark_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::BlockedOrSuspended
    );

    runtime_state
        .resume_control_link(&link)
        .await
        .expect("resume should release paused child progress");

    timeout(Duration::from_millis(250), started_mark.notified())
        .await
        .expect("resumed child should continue into the next step");

    let completion = wait_for_retained_completion(&runtime_state, &link).await;
    assert_eq!(mark_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::TerminalSuccess
    );
    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.terminal_result(),
        Some(&Ok(Value::String("counted".to_string())))
    );
    let effects = completion
        .conservative_effect_summary()
        .expect("completed child should retain effect summary contents");
    assert_eq!(effects.terminal_upper_bound(), Effect::Operational);
    assert_eq!(
        effects.reached_upper_bound(),
        &effect_set(&[Effect::Operational])
    );
}

#[tokio::test]
async fn kill_stops_real_spawned_child_before_later_steps_and_keeps_kill_seal() {
    let started_block = Arc::new(Notify::new());
    let started_mark = Arc::new(Notify::new());
    let (release_tx, release_rx) = oneshot::channel();
    let block_calls = Arc::new(AtomicUsize::new(0));
    let mark_calls = Arc::new(AtomicUsize::new(0));

    let runtime_state = runtime_state_with_registered_worker(child_two_step_action_workflow())
        .await
        .with_provider(
            "block",
            Arc::new(BlockingActionProvider::new(
                "block",
                started_block.clone(),
                release_rx,
                block_calls.clone(),
            )),
        )
        .with_provider(
            "mark",
            Arc::new(CountingActionProvider::new(
                "mark",
                mark_calls.clone(),
                started_mark.clone(),
            )),
        );
    let block_binding_id = admit_host_binding(
        &runtime_state,
        "workflow-block",
        "block",
        vec!["block.block"],
    )
    .await;
    let mark_binding_id =
        admit_host_binding(&runtime_state, "workflow-mark", "mark", vec!["mark.mark"]).await;

    let link = spawn_real_child_control_with_admitted(
        &runtime_state,
        vec![block_binding_id, mark_binding_id],
    )
    .await;

    timeout(Duration::from_millis(250), started_block.notified())
        .await
        .expect("real spawned child should begin executing the first action");
    assert_eq!(block_calls.load(Ordering::SeqCst), 1);

    runtime_state
        .kill_control_link(&link)
        .await
        .expect("kill should succeed for live child");

    release_tx
        .send(())
        .expect("test should still control the blocking provider");

    timeout(Duration::from_millis(100), started_mark.notified())
        .await
        .expect_err("killed child must not advance to later workflow steps");

    let completion = wait_for_retained_completion(&runtime_state, &link).await;
    assert_eq!(mark_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
    assert_eq!(completion.kind(), RetainedCompletionKind::ControlTerminated);
    assert_eq!(completion.terminal_result(), None);
    assert_eq!(completion.conservative_effect_summary(), None);
    assert_eq!(completion.conservative_obligations_summary(), None);
    assert_eq!(completion.conservative_provenance_summary(), None);
    assert_eq!(
        runtime_state
            .control_link_runtime_outcome_state(&link)
            .await,
        RuntimeOutcomeState::InvalidOrTerminated
    );
}

#[tokio::test]
async fn completion_state_wins_if_child_finishes_before_later_kill_attempt() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Ret {
        expr: Expr::Literal(Value::Int(1)),
    })
    .await;
    let link = spawn_real_child_control(&runtime_state).await;

    let completion = wait_for_retained_completion(&runtime_state, &link).await;
    wait_for_runtime_state(
        &runtime_state,
        &link,
        RuntimeOutcomeState::InvalidOrTerminated,
    )
    .await;

    let error = runtime_state
        .kill_control_link(&link)
        .await
        .expect_err("kill after child completion must lose the terminal race");

    assert!(
        matches!(error, ash_interp::ControlLinkError::Terminated(id) if id == link.instance_id)
    );
    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::TerminalSuccess
    );
    assert_eq!(completion.terminal_result(), Some(&Ok(Value::Int(1))));
    assert_eq!(
        runtime_state.retained_completion(&link).await,
        Some(completion)
    );
}

#[tokio::test]
async fn completion_wait_returns_immediately_for_already_sealed_record() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Ret {
        expr: Expr::Literal(Value::Int(5)),
    })
    .await;
    let link = spawn_real_child_control(&runtime_state).await;
    let sealed = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(
        timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("already-sealed retained completion should return immediately")
        .expect("already-sealed retained completion should be observable"),
        sealed
    );
}

#[tokio::test]
async fn completion_wait_distinguishes_unregistered_targets() {
    let runtime_state = RuntimeState::new();
    let link = ControlLink {
        instance_id: WorkflowId::new(),
    };

    assert_eq!(
        timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("unregistered targets should not hang completion waiting"),
        Err(ash_interp::ControlLinkError::NotFound(link.instance_id))
    );
}

#[tokio::test]
async fn spawned_child_failure_retains_direct_error_payload() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Ret {
        expr: Expr::Variable {
            name: "missing_child_value".to_string(),
            span: ash_core::ast::Span::default(),
        },
    })
    .await;
    let link = spawn_real_child_control(&runtime_state).await;

    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(
        completion.terminal_result(),
        Some(&Err(ExecError::Eval(EvalError::UndefinedVariable(
            "missing_child_value".into()
        ))))
    );
    let effects = completion
        .conservative_effect_summary()
        .expect("failed child should retain effect summary contents");
    assert_eq!(effects.terminal_upper_bound(), Effect::Epistemic);
    assert_eq!(effects.reached_upper_bound(), &effect_set(&[]));
}

#[tokio::test]
async fn spawned_child_success_retains_provenance_contents() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: ash_core::Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    })
    .await
    .with_provider(
        "deploy",
        Arc::new(
            MockProvider::new("deploy", Effect::Operational)
                .with_execute_result(Ok(Value::String("deployed".to_string()))),
        ),
    );
    let binding = host_binding("workflow-deploy", "deploy", vec!["deploy.deploy"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("spawned child authority should be admitted explicitly");
    let link = spawn_real_child_control_with_admitted(&runtime_state, vec![binding_id]).await;

    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::TerminalSuccess
    );
    assert_eq!(
        completion.terminal_result(),
        Some(&Ok(Value::String("deployed".to_string())))
    );
    assert_retained_provenance_workflow_id(&completion, link.instance_id);
    let effects = completion
        .conservative_effect_summary()
        .expect("successful child should retain effect summary contents");
    assert_eq!(effects.terminal_upper_bound(), Effect::Operational);
    assert_eq!(
        effects.reached_upper_bound(),
        &effect_set(&[Effect::Operational])
    );
}

#[tokio::test]
async fn spawned_child_without_inherited_grant_cannot_gain_provider_authority() {
    let calls = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(Notify::new());
    let runtime_state = runtime_state_with_registered_worker(Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: ash_core::Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    })
    .await
    .with_provider(
        "deploy",
        Arc::new(CountingActionProvider::new(
            "deploy",
            calls.clone(),
            started.clone(),
        )),
    );

    let link = spawn_real_child_control(&runtime_state).await;
    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "registered provider existence must not grant spawned child authority"
    );
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    let terminal = completion
        .terminal_result()
        .expect("child should retain a terminal result");
    let Err(error) = terminal else {
        panic!("child should fail at the authority boundary, got {terminal:?}");
    };
    assert!(
        error.to_string().contains("deploy"),
        "child failure should identify the denied provider/action: {error}"
    );
}

#[tokio::test]
async fn spawned_child_failure_retains_provenance_contents() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: ash_core::Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    })
    .await
    .with_provider(
        "deploy",
        Arc::new(
            MockProvider::new("deploy", Effect::Operational).with_execute_result(Err(
                CapabilityError::ExecutionFailed("deploy failed".to_string()),
            )),
        ),
    );
    let binding = host_binding("workflow-deploy", "deploy", vec!["deploy.deploy"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("spawned child authority should be admitted explicitly");
    let link = spawn_real_child_control_with_admitted(&runtime_state, vec![binding_id]).await;

    let completion = wait_for_retained_completion(&runtime_state, &link).await;

    assert_eq!(completion.kind(), RetainedCompletionKind::Completed);
    assert_eq!(
        completion.outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(
        completion.terminal_result(),
        Some(&Err(ExecError::ExecutionFailed(
            "deploy failed".to_string()
        )))
    );
    assert_retained_provenance_workflow_id(&completion, link.instance_id);
    let effects = completion
        .conservative_effect_summary()
        .expect("failed child should retain effect summary contents");
    assert_eq!(effects.terminal_upper_bound(), Effect::Operational);
    assert_eq!(
        effects.reached_upper_bound(),
        &effect_set(&[Effect::Operational])
    );
}

#[tokio::test]
async fn retained_completion_write_once_keeps_original_provenance_contents() {
    let runtime_state = runtime_state_with_registered_worker(Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "deploy".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: ash_core::Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    })
    .await
    .with_provider(
        "deploy",
        Arc::new(
            MockProvider::new("deploy", Effect::Operational)
                .with_execute_result(Ok(Value::String("deployed".to_string()))),
        ),
    );
    let link = spawn_real_child_control(&runtime_state).await;

    let sealed = wait_for_retained_completion(&runtime_state, &link).await;
    assert_retained_provenance_workflow_id(&sealed, link.instance_id);

    let error = runtime_state
        .record_control_completion(
            &link,
            Ok(Value::String("overwritten".to_string())),
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
            Some(ash_interp::ConservativeRetainedProvenanceSummary::new(
                WorkflowId::new(),
                None,
                vec![],
            )),
        )
        .await
        .expect_err("automatic sealing must remain write-once even with retained provenance");

    assert_eq!(
        error,
        ash_interp::ControlLinkError::CompletionAlreadySealed(
            link.instance_id,
            Box::new(sealed.clone())
        )
    );
    assert_eq!(runtime_state.retained_completion(&link).await, Some(sealed));
}

#[tokio::test]
async fn workflow_can_transport_and_reapply_effectful_closures() {
    let workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "compose".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::FnDef {
            params: vec![("act".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "act".to_string(),
                span: ash_core::ast::Span::default(),
            }),
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::FnApply {
                func: Box::new(Expr::FnApply {
                    func: Box::new(Expr::Variable {
                        name: "compose".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    args: vec![invoke_expr()],
                }),
                args: vec![],
            },
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("read-result".to_string()))),
        ),
    );
    let binding_id =
        admit_host_binding(&runtime_state, "sensor", "sensor", vec!["sensor.read"]).await;

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        &runtime_state,
    )
    .await
    .expect("workflow should transport and reapply effectful closures");

    assert!(
        matches!(result, Value::Closure { .. }),
        "workflow should transport the public opaque Act closure until an ActEnv forces it, got {result:?}"
    );

    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        policy_eval,
        Provenance::new(),
    )
    .await
    .expect("act env projection should succeed");
    let forced = eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Literal(result)),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &Context::new()
            .with_runtime_state(runtime_state)
            .with_admitted_capability_bindings(vec![binding_id])
            .with_act_env(act_env),
    )
    .expect("transported effectful closure should force through the hidden runtime ActEnv");

    assert_eq!(
        forced,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("read-result".to_string())
        ]))
    );
}
