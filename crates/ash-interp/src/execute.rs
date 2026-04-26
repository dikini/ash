//! Workflow execution engine
//!
//! Executes workflows in a runtime context, handling all workflow variants.

use ash_core::runtime::{
    FailureEntity, LexicalFrameId, OperationalFailure, ProcessId, ProcessTerminalState, TowerLevel,
};
use ash_core::{Capability, Effect, Expr, Provenance, Value, Workflow, WorkflowId};

use crate::act_env::ActEnv;

use crate::ExecResult;
use crate::behaviour::BehaviourContext;
use crate::capability::CapabilityContext;
use crate::capability_policy::{CapabilityPolicyEvaluator, Role};
use crate::context::Context;
use crate::control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkError, ControlLinkRegistry,
    RetainedCompletionKind,
};
use crate::error::{EvalError, ExecError};
use crate::eval::eval_expr_async;
use crate::exec_send::execute_send;
use crate::execute_set::execute_set;
use crate::execute_stream::{CoreReceiveRuntime, execute_core_receive};
use crate::execution_record::{ExecutionRecord, ExecutionRecorder};
use crate::guard::eval_guard;
use crate::mailbox::{Mailbox, SharedMailbox};
use crate::pattern::match_pattern;
use crate::policy::PolicyEvaluator;
use crate::process_env::{ChildEnvProjection, derive_child_env};
use crate::proxy_registry::ProxyRegistry;
use crate::runtime_outcome_state::RuntimeOutcomeState;
use crate::runtime_state::{RuntimeState, SPAWNED_CHILD_CONTROL_BINDING};
use crate::stream::StreamContext;
use crate::yield_state::{CorrelationId, SuspendedYields, YieldState};

use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Boxed future type for recursive async execution
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
type SharedControlRegistry = Arc<Mutex<ControlLinkRegistry>>;
type SharedProxyRegistry = Arc<Mutex<ProxyRegistry>>;
type SharedSuspendedYields = Arc<Mutex<SuspendedYields>>;

/// Execute a workflow, returning the final value (legacy signature without BehaviourContext)
///
/// This is kept for backward compatibility. For workflows that use Set statements,
/// use [`execute_workflow_with_behaviour`] instead.
pub fn execute_workflow<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        // Create an empty behaviour context for backward compatibility
        let behaviour_ctx = BehaviourContext::new();
        let runtime_state = RuntimeState::new();
        execute_workflow_with_behaviour_in_state(
            workflow,
            ctx,
            cap_ctx,
            policy_eval,
            &behaviour_ctx,
            &runtime_state,
        )
        .await
    })
}

/// Execute a workflow with behaviour context, returning the final value
///
/// This is the main entry point for workflow execution when using settable providers.
///
/// # Arguments
/// * `workflow` - The workflow to execute
/// * `ctx` - The runtime context with variable bindings
/// * `cap_ctx` - The capability context for external operations
/// * `policy_eval` - The policy evaluator for permission checks
/// * `behaviour_ctx` - The behaviour context for settable providers
///
/// # Examples
/// ```
/// use ash_core::{Workflow, Value};
/// use ash_interp::behaviour::BehaviourContext;
/// use ash_interp::context::Context;
/// use ash_interp::capability::CapabilityContext;
/// use ash_interp::policy::PolicyEvaluator;
/// use ash_interp::execute::execute_workflow_with_behaviour;
///
/// # tokio_test::block_on(async {
/// let ctx = Context::new();
/// let cap_ctx = CapabilityContext::new();
/// let policy_eval = PolicyEvaluator::new();
/// let behaviour_ctx = BehaviourContext::new();
/// let workflow = Workflow::Done;
/// let result = execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx).await.unwrap();
/// assert_eq!(result, Value::Null);
/// # });
/// ```
pub fn execute_workflow_with_behaviour<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        let runtime_state = RuntimeState::new();
        execute_workflow_with_behaviour_in_state(
            workflow,
            ctx,
            cap_ctx,
            policy_eval,
            behaviour_ctx,
            &runtime_state,
        )
        .await
    })
}

fn shared_mailbox() -> SharedMailbox {
    Arc::new(Mutex::new(Mailbox::new()))
}

pub(crate) fn shared_control_registry(runtime_state: &RuntimeState) -> SharedControlRegistry {
    runtime_state.control_registry()
}

pub(crate) fn shared_proxy_registry(runtime_state: &RuntimeState) -> SharedProxyRegistry {
    runtime_state.proxy_registry()
}

pub(crate) fn shared_suspended_yields(runtime_state: &RuntimeState) -> SharedSuspendedYields {
    runtime_state.suspended_yields()
}

async fn build_workflow_act_env(
    runtime_state: &RuntimeState,
    policy_eval: &PolicyEvaluator,
    provenance: Provenance,
) -> ActEnv {
    ActEnv::from_runtime_state(runtime_state, policy_eval.clone(), provenance).await
}

fn active_actor(ctx: &Context) -> Role {
    ctx.role_context()
        .map(|role_ctx| Role::new(role_ctx.active_role.name.clone()))
        .unwrap_or_else(|| Role::new("system"))
}

fn require_active_role(ctx: &Context, expected_role: &ash_core::Role) -> ExecResult<()> {
    let role_ctx = ctx.role_context().ok_or_else(|| {
        ExecError::ExecutionFailed(format!(
            "obligation check requires active role '{}'",
            expected_role.name
        ))
    })?;

    if role_ctx.active_role.name == expected_role.name {
        Ok(())
    } else {
        Err(ExecError::ExecutionFailed(format!(
            "active role '{}' does not match obligation role '{}'",
            role_ctx.active_role.name, expected_role.name
        )))
    }
}

fn require_role_authority(ctx: &Context, capability: &Capability) -> ExecResult<()> {
    if let Some(role_ctx) = ctx.role_context() {
        if role_ctx.can_access(capability) {
            return Ok(());
        }

        return Err(ExecError::ExecutionFailed(format!(
            "active role '{}' does not have authority for capability '{}'",
            role_ctx.active_role.name, capability.name
        )));
    }

    Ok(())
}

pub fn execute_workflow_with_behaviour_in_state<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    runtime_state: &'a RuntimeState,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        let ctx = ctx
            .with_policy_evaluator(policy_eval.clone())
            .with_runtime_state(runtime_state.clone());
        let mailbox = shared_mailbox();
        let control_registry = shared_control_registry(runtime_state);
        let proxy_registry = shared_proxy_registry(runtime_state);
        let suspended_yields = shared_suspended_yields(runtime_state);
        let execution_recorder = ExecutionRecorder::new(Provenance::new());
        let ctx = if ctx.act_env().is_some() {
            ctx
        } else {
            let act_env = build_workflow_act_env(
                runtime_state,
                policy_eval,
                execution_recorder.snapshot().provenance().clone(),
            )
            .await;
            ctx.with_act_env(act_env)
        };
        let result = execute_workflow_inner_observed(
            workflow,
            ctx,
            cap_ctx,
            policy_eval,
            behaviour_ctx,
            None,
            mailbox,
            control_registry,
            Some(proxy_registry),
            Some(suspended_yields),
            runtime_state,
            None,
            Some(&execution_recorder),
        )
        .await;
        execution_recorder.set_phase_from_result(&result);
        runtime_state
            .set_last_execution_record(execution_recorder.snapshot())
            .await;
        result
    })
}

fn resolve_control_link(target: &str, ctx: &Context) -> ExecResult<ash_core::ControlLink> {
    match ctx.get(target) {
        Some(Value::ControlLink(link)) => Ok(link.clone()),
        Some(value) => Err(ExecError::ExecutionFailed(format!(
            "control target '{target}' is not a ControlLink: {value}"
        ))),
        None => Err(ExecError::ExecutionFailed(format!(
            "control target '{target}' is undefined"
        ))),
    }
}

fn spawned_child_control_link(ctx: &Context) -> ExecResult<Option<ash_core::ControlLink>> {
    match ctx.get(SPAWNED_CHILD_CONTROL_BINDING) {
        Some(Value::ControlLink(link)) => Ok(Some(link.clone())),
        Some(value) => Err(ExecError::InvalidRuntimeState(format!(
            "spawned child control binding '{SPAWNED_CHILD_CONTROL_BINDING}' is not a ControlLink: {value}"
        ))),
        None => Ok(None),
    }
}

pub(crate) async fn resolve_registered_runtime_call_target(
    runtime_state: &RuntimeState,
    target: &str,
    arity: usize,
) -> ExecResult<Workflow> {
    if let Some(callable) = runtime_state.callable_workflow(target).await {
        if arity != callable.arity {
            return Err(ExecError::Eval(EvalError::WrongArity {
                expected: callable.arity,
                actual: arity,
                callee: Some(target.to_string()),
            }));
        }
        return Ok(callable.workflow);
    }

    let workflow = runtime_state.child_workflow(target).await.ok_or_else(|| {
        ExecError::ExecutionFailed(format!(
            "workflow call target '{target}' is not registered in runtime state"
        ))
    })?;

    if arity != 0 {
        return Err(ExecError::Eval(EvalError::WrongArity {
            expected: 0,
            actual: arity,
            callee: Some(target.to_string()),
        }));
    }

    Ok(workflow)
}

#[derive(Debug, Clone)]
struct TerminalObservationRecorder {
    obligations: Arc<std::sync::Mutex<Option<ConservativeRetainedObligationsSummary>>>,
}

impl TerminalObservationRecorder {
    fn new() -> Self {
        Self {
            obligations: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn record_terminal_result(&self, ctx: &Context, result: &ExecResult<Value>) {
        if RuntimeOutcomeState::from_exec_result(result).is_terminal() {
            let mut slot = self
                .obligations
                .lock()
                .expect("terminal observation recorder mutex should not be poisoned");
            if slot.is_none() {
                *slot = Some(conservative_obligations_summary_from_context(ctx));
            }
        }
    }

    fn observed_obligations(&self) -> Option<ConservativeRetainedObligationsSummary> {
        self.obligations
            .lock()
            .expect("terminal observation recorder mutex should not be poisoned")
            .clone()
    }
}

fn conservative_obligations_summary_from_context(
    ctx: &Context,
) -> ConservativeRetainedObligationsSummary {
    let (active_role, role_pending, role_discharged) = match ctx.role_context() {
        Some(role_ctx) => (
            Some(role_ctx.active_role.name.clone()),
            role_ctx.pending_obligations_set(),
            role_ctx.discharged_obligations_set(),
        ),
        None => (None, BTreeSet::new(), BTreeSet::new()),
    };

    ConservativeRetainedObligationsSummary::new(
        ctx.local_pending_obligations(),
        active_role,
        role_pending,
        role_discharged,
    )
}

fn record_terminal_result_if_observed(
    terminal_observer: Option<&TerminalObservationRecorder>,
    ctx: &Context,
    result: &ExecResult<Value>,
) {
    if let Some(observer) = terminal_observer {
        observer.record_terminal_result(ctx, result);
    }
}

fn sync_execution_context(execution_recorder: Option<&ExecutionRecorder>, ctx: &Context) {
    if let Some(recorder) = execution_recorder {
        recorder.set_running();
        recorder.sync_context(ctx);
    }
}

fn conservative_spawn_provenance_summary(
    workflow_id: ash_core::WorkflowId,
    parent_workflow_id: Option<ash_core::WorkflowId>,
    lineage: Vec<ash_core::WorkflowId>,
) -> ConservativeRetainedProvenanceSummary {
    ConservativeRetainedProvenanceSummary::new(workflow_id, parent_workflow_id, lineage)
}

fn finish_with_terminal_observation(
    terminal_observer: Option<&TerminalObservationRecorder>,
    ctx: &Context,
    result: ExecResult<Value>,
) -> ExecResult<Value> {
    record_terminal_result_if_observed(terminal_observer, ctx, &result);
    result
}

fn process_terminal_state_from_exec_result(
    process_id: ProcessId,
    result: &ExecResult<Value>,
) -> Option<ProcessTerminalState> {
    match RuntimeOutcomeState::from_exec_result(result) {
        RuntimeOutcomeState::TerminalSuccess => match result {
            Ok(value) => Some(ProcessTerminalState::Succeeded {
                value: value.clone(),
            }),
            Err(_) => None,
        },
        RuntimeOutcomeState::ExecutionFailure => match result {
            Err(error) => Some(ProcessTerminalState::Failed {
                process_id,
                failure: Box::new(operational_failure_from_exec_error(process_id, error)),
            }),
            Ok(_) => None,
        },
        RuntimeOutcomeState::InvalidOrTerminated => match result {
            Err(error) => Some(ProcessTerminalState::Cancelled {
                process_id,
                failure: Box::new(operational_failure_from_exec_error(process_id, error)),
            }),
            Ok(_) => None,
        },
        RuntimeOutcomeState::BlockedOrSuspended | RuntimeOutcomeState::Active => None,
    }
}

fn operational_failure_from_exec_error(
    process_id: ProcessId,
    error: &ExecError,
) -> OperationalFailure {
    match error {
        ExecError::Eval(EvalError::OperationalFailure(failure)) => failure.as_ref().clone(),
        ExecError::Eval(eval_error) => OperationalFailure::new(
            TowerLevel::Proc,
            FailureEntity::Process(process_id),
            Value::String(eval_error.to_string()),
            "String",
        )
        .with_cause(operational_failure_from_eval_error(eval_error)),
        _ => OperationalFailure::new(
            TowerLevel::Proc,
            FailureEntity::Process(process_id),
            Value::String(error.to_string()),
            "String",
        )
        .with_cause(OperationalFailure::new(
            TowerLevel::Workflow,
            FailureEntity::Workflow(WorkflowId::new()),
            Value::String(error.to_string()),
            "String",
        )),
    }
}

fn operational_failure_from_eval_error(error: &EvalError) -> OperationalFailure {
    match error {
        EvalError::OperationalFailure(failure) => failure.as_ref().clone(),
        _ => OperationalFailure::new(
            TowerLevel::Pure,
            FailureEntity::LexicalFrame(LexicalFrameId::new()),
            Value::String(error.to_string()),
            "String",
        ),
    }
}

async fn run_spawned_child_workflow(
    runtime_state: RuntimeState,
    child_workflow: Workflow,
    child_ctx: Context,
    link: ash_core::ControlLink,
    process_id: ProcessId,
    provenance: ConservativeRetainedProvenanceSummary,
    execution_provenance: Provenance,
) -> ExecResult<()> {
    tokio::task::yield_now().await;

    let terminal_observer = TerminalObservationRecorder::new();
    let (child_result, child_execution_record) =
        execute_with_context_with_terminal_observation_in_state(
            &child_workflow,
            &runtime_state,
            child_ctx,
            &terminal_observer,
            execution_provenance,
            false,
        )
        .await;

    if let Some(terminal_state) = process_terminal_state_from_exec_result(process_id, &child_result)
    {
        runtime_state
            .record_process_terminal(process_id, terminal_state)
            .await
            .map_err(|error| {
                ExecError::ExecutionFailed(format!(
                    "spawned child terminal process state recording failed unexpectedly for process {process_id:?}: {error}"
                ))
            })?;
    }

    let outcome_state = RuntimeOutcomeState::from_exec_result(&child_result);
    if !outcome_state.is_terminal() {
        return Ok(());
    }

    let completion_payload = child_execution_record
        .project_completion()
        .expect("terminal child executions should project retained completion payloads");
    let effects = ConservativeRetainedEffectSummary::from_semantic(completion_payload.effects());

    let obligations = terminal_observer.observed_obligations().unwrap_or_else(|| {
        ConservativeRetainedObligationsSummary::new(
            BTreeSet::new(),
            None,
            BTreeSet::new(),
            BTreeSet::new(),
        )
    });

    match runtime_state
        .record_control_completion(&link, child_result, effects, obligations, Some(provenance))
        .await
    {
        Ok(_) => Ok(()),
        Err(ControlLinkError::CompletionAlreadySealed(_, record))
            if record.kind() == RetainedCompletionKind::ControlTerminated =>
        {
            Ok(())
        }
        Err(ControlLinkError::Terminated(..)) => Ok(()),
        Err(error) => Err(ExecError::ExecutionFailed(format!(
            "spawned child completion sealing failed unexpectedly for instance {:?}: {error}",
            link.instance_id
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_workflow_inner<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    stream_ctx: Option<&'a StreamContext>,
    mailbox: SharedMailbox,
    control_registry: SharedControlRegistry,
    proxy_registry: Option<SharedProxyRegistry>,
    suspended_yields: Option<SharedSuspendedYields>,
    runtime_state: &'a RuntimeState,
    execution_recorder: Option<&'a ExecutionRecorder>,
) -> BoxFuture<'a, ExecResult<Value>> {
    execute_workflow_inner_observed(
        workflow,
        ctx,
        cap_ctx,
        policy_eval,
        behaviour_ctx,
        stream_ctx,
        mailbox,
        control_registry,
        proxy_registry,
        suspended_yields,
        runtime_state,
        None,
        execution_recorder,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_workflow_inner_observed<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    stream_ctx: Option<&'a StreamContext>,
    mailbox: SharedMailbox,
    control_registry: SharedControlRegistry,
    proxy_registry: Option<SharedProxyRegistry>,
    suspended_yields: Option<SharedSuspendedYields>,
    runtime_state: &'a RuntimeState,
    terminal_observer: Option<&'a TerminalObservationRecorder>,
    execution_recorder: Option<&'a ExecutionRecorder>,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        sync_execution_context(execution_recorder, &ctx);
        if let Some(link) = spawned_child_control_link(&ctx)? {
            runtime_state.wait_for_control_authority(&link).await?;
        }

        let terminal_ctx_snapshot = ctx.clone();
        let result = match workflow {
            // Terminal workflow - returns null
            Workflow::Done => Ok(Value::Null),

            // Return with value
            Workflow::Ret { expr } => eval_expr_async(expr, &ctx).await.map_err(ExecError::Eval),

            // Variable binding
            Workflow::Let {
                pattern,
                expr,
                continuation,
            } => {
                let value = eval_expr_async(expr, &ctx).await.map_err(ExecError::Eval)?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: Box::new(value.clone()),
                    })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow_inner_observed(
                    continuation,
                    new_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Conditional execution
            Workflow::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = eval_expr_async(condition, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;
                match cond_val {
                    Value::Bool(true) => {
                        execute_workflow_inner_observed(
                            then_branch,
                            ctx,
                            cap_ctx,
                            policy_eval,
                            behaviour_ctx,
                            stream_ctx,
                            mailbox,
                            control_registry,
                            proxy_registry.clone(),
                            suspended_yields.clone(),
                            runtime_state,
                            terminal_observer,
                            execution_recorder,
                        )
                        .await
                    }
                    Value::Bool(false) => {
                        execute_workflow_inner_observed(
                            else_branch,
                            ctx,
                            cap_ctx,
                            policy_eval,
                            behaviour_ctx,
                            stream_ctx,
                            mailbox,
                            control_registry,
                            proxy_registry.clone(),
                            suspended_yields.clone(),
                            runtime_state,
                            terminal_observer,
                            execution_recorder,
                        )
                        .await
                    }
                    _ => Err(ExecError::Eval(EvalError::TypeMismatch {
                        expected: "bool".to_string(),
                        actual: format!("{:?}", cond_val),
                    })),
                }
            }

            // Sequential composition
            Workflow::Seq { first, second } => {
                let _ = execute_workflow_inner_observed(
                    first,
                    ctx.clone(),
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox.clone(),
                    control_registry.clone(),
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await?;
                if let Some(recorder) = execution_recorder {
                    ctx.replace_local_obligations(
                        recorder.snapshot().obligations().pending().iter().cloned(),
                    );
                }
                execute_workflow_inner_observed(
                    second,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Observe from capability
            Workflow::Observe {
                capability,
                pattern,
                continuation,
            } => {
                if let Some(recorder) = execution_recorder {
                    recorder.record_observe(&capability.name, capability.effect);
                }
                require_role_authority(&ctx, capability)?;
                let value = cap_ctx.observe(capability).await?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: Box::new(value.clone()),
                    })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow_inner_observed(
                    continuation,
                    new_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Orient - evaluate expression and continue
            Workflow::Orient { expr, continuation } => {
                let _ = eval_expr_async(expr, &ctx).await.map_err(ExecError::Eval)?;
                if let Some(recorder) = execution_recorder {
                    recorder.record_orient(&format!("{expr:?}"));
                }
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Execute action with guard
            Workflow::Act {
                provider_name,
                action_name,
                arguments,
                guard,
                provenance: _,
                result_name,
                continuation,
            } => {
                // Evaluate guard
                let guard_result = eval_guard(guard, &ctx).map_err(|_| ExecError::GuardFailed {
                    guard: format!("{guard:?}"),
                })?;

                if !guard_result {
                    return Err(ExecError::GuardFailed {
                        guard: format!("{guard:?}"),
                    });
                }

                // Evaluate action arguments
                let evaluated_args = {
                    let mut values = Vec::with_capacity(arguments.len());
                    for expr in arguments {
                        values.push(eval_expr_async(expr, &ctx).await.map_err(ExecError::Eval)?);
                    }
                    values
                };

                if let Some(recorder) = execution_recorder {
                    recorder.record_act(action_name, &format!("{guard:?}"));
                }

                let capability = Capability {
                    name: provider_name.clone(),
                    effect: Effect::Operational,
                    constraints: vec![],
                };
                require_role_authority(&ctx, &capability)?;

                // Lookup provider by provider_name and dispatch action
                let result = cap_ctx
                    .execute(provider_name, action_name, &evaluated_args)
                    .await?;

                // If continuation is Done, return the action result directly (bare act semantics)
                if matches!(**continuation, Workflow::Done) {
                    return Ok(result);
                }

                // Bind result and execute continuation
                let exec_ctx = if let Some(name) = result_name {
                    let mut child_ctx = ctx.extend();
                    child_ctx.set(name.clone(), result);
                    child_ctx
                } else {
                    ctx.clone()
                };

                execute_workflow_inner_observed(
                    continuation,
                    exec_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            Workflow::Call {
                target,
                arguments,
                continuation,
            } => {
                let callable = runtime_state.callable_workflow(target).await;
                let child_workflow =
                    resolve_registered_runtime_call_target(runtime_state, target, arguments.len())
                        .await?;

                // Build child context with arguments bound to parameter names.
                // Arguments are evaluated in the caller's context.
                let child_ctx = if let Some(ref callable) = callable {
                    let mut child = Context::new();
                    if let Some(policy_evaluator) = ctx.policy_evaluator() {
                        child = child.with_policy_evaluator_arc(policy_evaluator);
                    }
                    if let Some(act_env) = ctx.act_env() {
                        child = child.with_act_env_arc(act_env);
                    }
                    for (param_name, arg_expr) in callable.params.iter().zip(arguments.iter()) {
                        let arg_value = eval_expr_async(arg_expr, &ctx)
                            .await
                            .map_err(ExecError::Eval)?;
                        child.set(param_name.clone(), arg_value);
                    }
                    child
                } else {
                    let mut child = Context::new();
                    if let Some(policy_evaluator) = ctx.policy_evaluator() {
                        child = child.with_policy_evaluator_arc(policy_evaluator);
                    }
                    if let Some(act_env) = ctx.act_env() {
                        child = child.with_act_env_arc(act_env);
                    }
                    child
                };

                execute_workflow_inner_observed(
                    &child_workflow,
                    child_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox.clone(),
                    control_registry.clone(),
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await?;

                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Propose action (advisory - just continue)
            Workflow::Propose {
                action_name: _,
                action_arguments: _,
                continuation,
            } => {
                // Proposal is advisory - just continue
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Decide under policy
            Workflow::Decide {
                expr,
                policy,
                continuation,
            } => {
                let value = eval_expr_async(expr, &ctx).await.map_err(ExecError::Eval)?;

                // Create a temporary context with the decision value
                let mut decision_ctx = ctx.extend();
                decision_ctx.set("decision_value".to_string(), value);

                let decision = policy_eval.evaluate(policy, &decision_ctx)?;
                if let Some(recorder) = execution_recorder {
                    recorder.record_decide(policy, decision);
                }

                match decision {
                    ash_core::Decision::Permit => {
                        execute_workflow_inner_observed(
                            continuation,
                            ctx,
                            cap_ctx,
                            policy_eval,
                            behaviour_ctx,
                            stream_ctx,
                            mailbox,
                            control_registry,
                            proxy_registry.clone(),
                            suspended_yields.clone(),
                            runtime_state,
                            terminal_observer,
                            execution_recorder,
                        )
                        .await
                    }
                    ash_core::Decision::Deny => Err(ExecError::PolicyDenied {
                        policy: policy.clone(),
                    }),
                    ash_core::Decision::RequireApproval | ash_core::Decision::Escalate => {
                        // For now, escalate is treated as deny
                        Err(ExecError::PolicyDenied {
                            policy: policy.clone(),
                        })
                    }
                }
            }

            // For each iteration
            Workflow::ForEach {
                pattern,
                collection,
                body,
            } => {
                let coll_val = eval_expr_async(collection, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;

                match coll_val {
                    Value::List(items) => {
                        let mut last_result = Value::Null;

                        for item in items.iter() {
                            let bindings = match_pattern(pattern, item).map_err(|_| {
                                ExecError::PatternMatchFailed {
                                    pattern: format!("{:?}", pattern),
                                    value: Box::new(item.clone()),
                                }
                            })?;

                            let mut iter_ctx = ctx.extend();
                            iter_ctx.set_many(bindings);

                            last_result = execute_workflow_inner_observed(
                                body,
                                iter_ctx,
                                cap_ctx,
                                policy_eval,
                                behaviour_ctx,
                                stream_ctx,
                                mailbox.clone(),
                                control_registry.clone(),
                                proxy_registry.clone(),
                                suspended_yields.clone(),
                                runtime_state,
                                terminal_observer,
                                execution_recorder,
                            )
                            .await?;
                        }

                        Ok(last_result)
                    }
                    _ => Err(ExecError::Eval(EvalError::TypeMismatch {
                        expected: "list".to_string(),
                        actual: format!("{:?}", coll_val),
                    })),
                }
            }

            // With capability scope
            Workflow::With {
                capability: _,
                workflow,
            } => {
                // For now, just execute the workflow
                // In a full implementation, this would set up capability context
                execute_workflow_inner_observed(
                    workflow,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Maybe - try primary, fallback on failure
            Workflow::Maybe { primary, fallback } => {
                match execute_workflow_inner_observed(
                    primary,
                    ctx.clone(),
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox.clone(),
                    control_registry.clone(),
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
                {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        execute_workflow_inner_observed(
                            fallback,
                            ctx,
                            cap_ctx,
                            policy_eval,
                            behaviour_ctx,
                            stream_ctx,
                            mailbox,
                            control_registry,
                            proxy_registry.clone(),
                            suspended_yields.clone(),
                            runtime_state,
                            terminal_observer,
                            execution_recorder,
                        )
                        .await
                    }
                }
            }

            // Must - fail if workflow fails
            Workflow::Must { workflow: inner } => {
                execute_workflow_inner_observed(
                    inner,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Check obligation (simplified - just continue)
            Workflow::Check {
                obligation,
                continuation,
            } => {
                match obligation {
                    ash_core::Obligation::Obliged { role, condition } => {
                        require_active_role(&ctx, role)?;

                        match eval_expr_async(condition, &ctx)
                            .await
                            .map_err(ExecError::Eval)?
                        {
                            Value::Bool(true) => {
                                if let Some(recorder) = execution_recorder {
                                    recorder.record_obligation_check(&role.name, true);
                                }
                            }
                            Value::Bool(false) => {
                                if let Some(recorder) = execution_recorder {
                                    recorder.record_obligation_check(&role.name, false);
                                }
                                return Err(ExecError::ExecutionFailed(
                                    "obligation check failed".to_string(),
                                ));
                            }
                            value => {
                                return Err(ExecError::ExecutionFailed(format!(
                                    "obligation condition did not evaluate to Bool: {value}"
                                )));
                            }
                        }
                    }
                    other => {
                        return Err(ExecError::ExecutionFailed(format!(
                            "unsupported runtime obligation check: {other:?}"
                        )));
                    }
                }

                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Obligate a role (simplified - just execute workflow)
            Workflow::Oblig {
                role,
                workflow: inner,
            } => {
                let ctx =
                    ctx.with_role_context(crate::role_context::RoleContext::new(role.clone()));
                execute_workflow_inner_observed(
                    inner,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Set a value on a writable channel
            Workflow::Set {
                capability,
                channel,
                value,
            } => {
                let val = eval_expr_async(value, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;
                let capability_policy_eval = CapabilityPolicyEvaluator::new();
                let actor = active_actor(&ctx);
                execute_set(
                    capability,
                    channel,
                    val,
                    behaviour_ctx,
                    &capability_policy_eval,
                    &actor,
                )
                .await?;
                Ok(Value::Null)
            }

            Workflow::Send {
                capability,
                channel,
                value,
            } => {
                let stream_ctx = stream_ctx.ok_or_else(|| {
                    ExecError::ExecutionFailed(
                        "Send requires StreamContext - use execute_workflow_with_stream"
                            .to_string(),
                    )
                })?;
                let val = eval_expr_async(value, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;
                let capability_policy_eval = CapabilityPolicyEvaluator::new();
                let actor = active_actor(&ctx);
                execute_send(
                    capability,
                    channel,
                    val,
                    stream_ctx,
                    &capability_policy_eval,
                    &actor,
                )
                .await?;
                Ok(Value::Null)
            }

            Workflow::Receive {
                mode,
                arms,
                control,
            } => {
                let capability_policy_eval = CapabilityPolicyEvaluator::new();
                let actor = Role::new("system");
                let stream_ctx = stream_ctx.ok_or_else(|| {
                    ExecError::ExecutionFailed(
                        "Receive requires StreamContext - use execute_workflow_with_stream"
                            .to_string(),
                    )
                })?;
                execute_core_receive(
                    mode,
                    arms,
                    *control,
                    ctx,
                    CoreReceiveRuntime {
                        mailbox,
                        control_registry,
                        proxy_registry,
                        suspended_yields,
                        stream_ctx,
                        cap_ctx,
                        policy_eval,
                        capability_policy_eval: &capability_policy_eval,
                        actor: &actor,
                        behaviour_ctx,
                        runtime_state,
                        execution_recorder,
                    },
                )
                .await
            }

            // Spawn a workflow instance
            Workflow::Spawn {
                workflow_type,
                init,
                pattern,
                continuation,
            } => {
                let init_value = eval_expr_async(init, &ctx).await.map_err(ExecError::Eval)?;
                let child_workflow = runtime_state.child_workflow(workflow_type).await;
                let instance_id = ash_core::WorkflowId::new();
                let control = child_workflow
                    .as_ref()
                    .map(|_| ash_core::ControlLink { instance_id });
                let instance_value = Value::Instance(Box::new(ash_core::Instance {
                    addr: ash_core::InstanceAddr {
                        workflow_type: workflow_type.clone(),
                        instance_id,
                    },
                    control: control.clone(),
                }));

                if let (Some(control), Some(child_workflow)) = (control, child_workflow) {
                    let child_process_id = ProcessId::new();
                    let parent_process_id =
                        ctx.process_identity().map(|identity| identity.process_id);
                    let child_index = if let Some(parent_process_id) = parent_process_id {
                        runtime_state
                            .process_children(parent_process_id)
                            .await
                            .len()
                    } else {
                        0
                    };
                    match parent_process_id {
                        Some(parent_process_id) => runtime_state
                            .register_child_process(
                                parent_process_id,
                                child_process_id,
                                child_index,
                            )
                            .await
                            .map_err(|error| {
                                ExecError::ExecutionFailed(format!(
                                    "failed to register child process {child_process_id:?}: {error}"
                                ))
                            })?,
                        None => runtime_state
                            .register_root_process(child_process_id)
                            .await
                            .map_err(|error| {
                                ExecError::ExecutionFailed(format!(
                                    "failed to register root process {child_process_id:?}: {error}"
                                ))
                            })?,
                    }
                    runtime_state
                        .mark_process_running(child_process_id)
                        .await
                        .map_err(|error| {
                            ExecError::ExecutionFailed(format!(
                                "failed to mark process {child_process_id:?} running: {error}"
                            ))
                        })?;

                    let child_projection = parent_process_id
                        .map(|parent_process_id| {
                            ChildEnvProjection::new(child_process_id, child_index)
                                .with_parent_process_id(parent_process_id)
                        })
                        .unwrap_or_else(|| ChildEnvProjection::new(child_process_id, child_index));
                    let mut child_ctx = derive_child_env(&ctx, child_projection).map_err(|error| {
                        ExecError::ExecutionFailed(format!(
                            "failed to project spawned child process environment for {child_process_id:?}: {error}"
                        ))
                    })?;
                    child_ctx.set_many(RuntimeState::spawned_child_init_bindings(
                        init_value.clone(),
                        control.clone(),
                    ));
                    let parent_workflow_id = None;
                    let parent_lineage = vec![];
                    let provenance = conservative_spawn_provenance_summary(
                        control.instance_id,
                        parent_workflow_id,
                        parent_lineage,
                    );
                    runtime_state
                        .register_spawned_control_link_with_provenance(provenance.clone())
                        .await;
                    let child_execution_provenance = execution_recorder
                        .map(|recorder| recorder.child_provenance(control.instance_id))
                        .unwrap_or_else(|| Provenance {
                            workflow_id: control.instance_id,
                            parent: None,
                            lineage: vec![],
                        });
                    let spawned_runtime_state = (*runtime_state).clone();
                    let spawned_control = control.clone();
                    tokio::spawn(async move {
                        if let Err(error) = run_spawned_child_workflow(
                            spawned_runtime_state,
                            child_workflow,
                            child_ctx,
                            spawned_control.clone(),
                            child_process_id,
                            provenance,
                            child_execution_provenance,
                        )
                        .await
                        {
                            eprintln!(
                                "spawned child workflow failed for instance {:?}: {error}",
                                spawned_control.instance_id
                            );
                        }
                    });
                }

                // Match pattern and bind
                let bindings = match_pattern(pattern, &instance_value).map_err(|_| {
                    ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: Box::new(instance_value.clone()),
                    }
                })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow_inner_observed(
                    continuation,
                    new_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Split an instance into (addr, control)
            Workflow::Split {
                expr,
                pattern,
                continuation,
            } => {
                // Evaluate the split expression
                let split_value = eval_expr_async(&Expr::Split(Box::new(expr.clone())), &ctx)
                    .await
                    .map_err(ExecError::Eval)?;

                // Match pattern and bind
                let bindings = match_pattern(pattern, &split_value).map_err(|_| {
                    ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: Box::new(split_value.clone()),
                    }
                })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow_inner_observed(
                    continuation,
                    new_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Kill a workflow instance using control link
            Workflow::Kill {
                target,
                continuation,
            } => {
                let link = resolve_control_link(target, &ctx)?;
                control_registry.lock().await.kill(&link).map_err(|error| {
                    ExecError::InvalidRuntimeState(format!(
                        "kill on control target '{target}' failed: {error}"
                    ))
                })?;
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Pause a workflow instance using control link
            Workflow::Pause {
                target,
                continuation,
            } => {
                let link = resolve_control_link(target, &ctx)?;
                control_registry
                    .lock()
                    .await
                    .pause(&link)
                    .map_err(|error| {
                        ExecError::InvalidRuntimeState(format!(
                            "pause on control target '{target}' failed: {error}"
                        ))
                    })?;
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Resume a workflow instance using control link
            Workflow::Resume {
                target,
                continuation,
            } => {
                let link = resolve_control_link(target, &ctx)?;
                control_registry
                    .lock()
                    .await
                    .resume(&link)
                    .map_err(|error| {
                        ExecError::InvalidRuntimeState(format!(
                            "resume on control target '{target}' failed: {error}"
                        ))
                    })?;
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // Check health of a workflow instance using control link
            Workflow::CheckHealth {
                target,
                continuation,
            } => {
                let link = resolve_control_link(target, &ctx)?;
                control_registry
                    .lock()
                    .await
                    .check_health(&link)
                    .map_err(|error| {
                        ExecError::InvalidRuntimeState(format!(
                            "check_health on control target '{target}' failed: {error}"
                        ))
                    })?;
                execute_workflow_inner_observed(
                    continuation,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry.clone(),
                    suspended_yields.clone(),
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }

            // OBLIGE - Introduce a linear obligation (contract tracking)
            Workflow::Oblige { name, span: _ } => {
                // Check for linearity violation (duplicate obligation)
                if ctx.has_obligation(name) {
                    return Err(ExecError::ExecutionFailed(format!(
                        "Linear obligation violation: obligation '{name}' already exists"
                    )));
                }

                // Add the obligation to the context
                ctx.add_obligation(name.clone());
                if let Some(recorder) = execution_recorder {
                    recorder.sync_context(&ctx);
                }

                // Return null as per spec
                Ok(Value::Null)
            }

            // CHECK - Check/discharge a linear obligation (contract tracking)
            Workflow::CheckObligation { name, span: _ } => {
                // Attempt to discharge the obligation
                let discharged = ctx.discharge_obligation(name);
                if let Some(recorder) = execution_recorder {
                    recorder.sync_context(&ctx);
                }

                // Return true if obligation was found and discharged, false otherwise
                Ok(Value::Bool(discharged))
            }

            // YIELD - Yield control to proxy (awaiting resume)
            Workflow::Yield {
                role,
                request,
                expected_response_type,
                continuation,
                span: _,
                resume_var,
            } => {
                // Check if proxy registry is available
                let proxy_reg = match proxy_registry {
                    Some(reg) => reg,
                    None => {
                        return Err(ExecError::ExecutionFailed(
                            "YIELD requires proxy registry - use execute_workflow_with_behaviour_in_state".to_string()
                        ));
                    }
                };

                // Look up the proxy for this role
                let proxy_addr = {
                    let registry = proxy_reg.lock().await;
                    match registry.lookup(role) {
                        Some(addr) => addr.clone(),
                        None => {
                            return Err(ExecError::ExecutionFailed(format!(
                                "No proxy registered for role '{}'",
                                role
                            )));
                        }
                    }
                };

                // Check if suspended yields registry is available
                let suspended = match suspended_yields {
                    Some(s) => s,
                    None => {
                        return Err(ExecError::ExecutionFailed(
                            "YIELD requires suspended yields registry".to_string(),
                        ));
                    }
                };

                // Evaluate the request expression
                let request_value = eval_expr_async(request, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;

                // Generate correlation ID and create yield state
                let correlation_id = CorrelationId::new();
                let yield_state = YieldState {
                    correlation_id,
                    expected_response_type: convert_type_expr(expected_response_type),
                    continuation: (**continuation).clone(),
                    origin_workflow: "workflow-instance".to_string(),
                    target_role: role.clone(),
                    request_sent_at: Instant::now(),
                    resume_var: resume_var.clone(),
                };

                // Suspend the workflow
                {
                    let mut suspended = suspended.lock().await;
                    suspended.suspend(yield_state);
                }

                // Return YieldSuspended to signal the runtime that the workflow yielded
                // The runtime can then route the request to the appropriate proxy
                Err(ExecError::YieldSuspended {
                    role: role.clone(),
                    request: Box::new(request_value),
                    expected_response_type: format!("{:?}", expected_response_type),
                    correlation_id: correlation_id.0.to_string(),
                    proxy_addr: proxy_addr.clone(),
                })
            }

            // PROXY_RESUME - Resume after proxy yields
            Workflow::ProxyResume {
                value,
                value_type: _,
                correlation_id,
                span: _,
            } => {
                // Check if suspended yields registry is available
                let suspended = match suspended_yields {
                    Some(ref s) => s,
                    None => {
                        return Err(ExecError::ExecutionFailed(
                            "PROXY_RESUME requires suspended yields registry".to_string(),
                        ));
                    }
                };

                // Convert ash_core::ast::CorrelationId to ash_interp::yield_state::CorrelationId
                let correlation_id = CorrelationId(correlation_id.0);

                // Look up and remove the suspended yield
                let yield_state = {
                    let mut suspended = suspended.lock().await;
                    suspended.resume(correlation_id)
                };

                let yield_state = match yield_state {
                    Some(state) => state,
                    None => {
                        return Err(ExecError::ExecutionFailed(format!(
                            "No suspended yield found for correlation_id {}",
                            correlation_id.0
                        )));
                    }
                };

                // Evaluate the response value expression
                let response_value = eval_expr_async(value, &ctx)
                    .await
                    .map_err(ExecError::Eval)?;

                // TODO: Type-check the response value against expected_response_type
                // For now, we skip type checking but the infrastructure is in place
                let _expected_type = &yield_state.expected_response_type;

                // Create a new context with the response value bound to the resume variable
                let mut new_ctx = ctx.extend();
                new_ctx.set(yield_state.resume_var.clone(), response_value);

                // Execute the continuation workflow with the new context
                execute_workflow_inner_observed(
                    &yield_state.continuation,
                    new_ctx,
                    cap_ctx,
                    policy_eval,
                    behaviour_ctx,
                    stream_ctx,
                    mailbox,
                    control_registry,
                    proxy_registry,
                    suspended_yields,
                    runtime_state,
                    terminal_observer,
                    execution_recorder,
                )
                .await
            }
        };
        let result =
            finish_with_terminal_observation(terminal_observer, &terminal_ctx_snapshot, result);
        if let Some(recorder) = execution_recorder {
            recorder.set_phase_from_result(&result);
        }
        result
    })
}
/// Convert a workflow_contract TypeExpr to a typeck Type
fn convert_type_expr(type_expr: &ash_core::workflow_contract::TypeExpr) -> ash_typeck::types::Type {
    use ash_core::workflow_contract::TypeExpr;

    match type_expr {
        TypeExpr::Named(name) => {
            match name.as_str() {
                "Int" => ash_typeck::types::Type::Int,
                "Bool" => ash_typeck::types::Type::Bool,
                "String" => ash_typeck::types::Type::String,
                _ => ash_typeck::types::Type::Var(ash_typeck::types::TypeVar(0)), // Fallback
            }
        }
        TypeExpr::Constructor { name, args } => {
            // For now, treat constructors as lists or special types
            if name == "List" && args.len() == 1 {
                ash_typeck::types::Type::List(Box::new(convert_type_expr(&args[0])))
            } else {
                ash_typeck::types::Type::Var(ash_typeck::types::TypeVar(0))
            }
        }
        TypeExpr::Tuple(types) => {
            // Build a record type from tuple elements
            let converted: Vec<(Box<str>, ash_typeck::types::Type)> = types
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("_{}", i).into_boxed_str(), convert_type_expr(t)))
                .collect();
            ash_typeck::types::Type::Record(converted)
        }
    }
}

/// Execute a workflow with stream context, returning the final value
///
/// This is the main entry point for workflow execution when using sendable stream providers.
///
/// # Arguments
/// * `workflow` - The workflow to execute
/// * `ctx` - The runtime context with variable bindings
/// * `cap_ctx` - The capability context for external operations
/// * `policy_eval` - The policy evaluator for permission checks
/// * `behaviour_ctx` - The behaviour context for settable providers
/// * `stream_ctx` - The stream context for sendable providers
///
/// # Examples
/// ```
/// use ash_core::{Workflow, Expr, Value};
/// use ash_interp::behaviour::BehaviourContext;
/// use ash_interp::stream::{StreamContext, MockSendableProvider, TypedSendableProvider};
/// use ash_interp::context::Context;
/// use ash_interp::capability::CapabilityContext;
/// use ash_interp::policy::PolicyEvaluator;
/// use ash_interp::execute::execute_workflow_with_stream;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let ctx = Context::new();
/// let cap_ctx = CapabilityContext::new();
/// let policy_eval = PolicyEvaluator::new();
/// let behaviour_ctx = BehaviourContext::new();
/// let mut stream_ctx = StreamContext::new();
/// let provider = MockSendableProvider::new("queue", "output");
/// stream_ctx.register_sendable(TypedSendableProvider::new(provider, Type::Int));
/// let workflow = Workflow::Done;
/// let result = execute_workflow_with_stream(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx, &stream_ctx).await.unwrap();
/// assert_eq!(result, Value::Null);
/// # });
/// ```
pub fn execute_workflow_with_stream<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    stream_ctx: &'a StreamContext,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        let runtime_state = RuntimeState::new();
        execute_workflow_with_stream_in_state(
            workflow,
            ctx,
            cap_ctx,
            policy_eval,
            behaviour_ctx,
            stream_ctx,
            &runtime_state,
        )
        .await
    })
}

/// Execute a workflow with default contexts (convenience function)
pub async fn execute_simple(workflow: &Workflow) -> ExecResult<Value> {
    let runtime_state = RuntimeState::new();
    execute_simple_in_state(workflow, &runtime_state).await
}

pub fn execute_workflow_with_stream_in_state<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    stream_ctx: &'a StreamContext,
    runtime_state: &'a RuntimeState,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        let ctx = ctx
            .with_policy_evaluator(policy_eval.clone())
            .with_runtime_state(runtime_state.clone());
        let mailbox = shared_mailbox();
        let control_registry = shared_control_registry(runtime_state);
        let proxy_registry = shared_proxy_registry(runtime_state);
        let suspended_yields = shared_suspended_yields(runtime_state);
        let execution_recorder = ExecutionRecorder::new(Provenance::new());
        let ctx = if ctx.act_env().is_some() {
            ctx
        } else {
            let act_env = build_workflow_act_env(
                runtime_state,
                policy_eval,
                execution_recorder.snapshot().provenance().clone(),
            )
            .await;
            ctx.with_act_env(act_env)
        };
        let result = execute_workflow_inner_observed(
            workflow,
            ctx,
            cap_ctx,
            policy_eval,
            behaviour_ctx,
            Some(stream_ctx),
            mailbox,
            control_registry,
            Some(proxy_registry),
            Some(suspended_yields),
            runtime_state,
            None,
            Some(&execution_recorder),
        )
        .await;
        execution_recorder.set_phase_from_result(&result);
        runtime_state
            .set_last_execution_record(execution_recorder.snapshot())
            .await;
        result
    })
}

/// Execute a workflow with default contexts using explicit runtime-owned state.
pub async fn execute_simple_in_state(
    workflow: &Workflow,
    runtime_state: &RuntimeState,
) -> ExecResult<Value> {
    let ctx = Context::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();
    execute_workflow_with_behaviour_in_state(
        workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime_state,
    )
    .await
}

/// Execute a workflow with initial input bindings using explicit runtime-owned state.
///
/// This is similar to `execute_simple_in_state` but allows passing initial variable
/// bindings that will be available in the workflow's execution context.
///
/// # Arguments
/// * `workflow` - The workflow to execute
/// * `runtime_state` - The runtime state with configured providers
/// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
///
/// # Errors
///
/// Returns execution errors from the interpreter.
pub async fn execute_with_bindings_in_state(
    workflow: &Workflow,
    runtime_state: &RuntimeState,
    input_bindings: std::collections::HashMap<String, Value>,
) -> ExecResult<Value> {
    let ctx = Context::with_bindings(input_bindings);
    // Use capability providers from RuntimeState instead of creating an empty context
    let cap_ctx = runtime_state.create_capability_context().await;
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();
    execute_workflow_with_behaviour_in_state(
        workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime_state,
    )
    .await
}

async fn execute_with_context_with_terminal_observation_in_state(
    workflow: &Workflow,
    runtime_state: &RuntimeState,
    ctx: Context,
    terminal_observer: &TerminalObservationRecorder,
    execution_provenance: Provenance,
    persist_last_execution_record: bool,
) -> (ExecResult<Value>, ExecutionRecord) {
    let cap_ctx = runtime_state.create_capability_context().await;
    let policy_eval = PolicyEvaluator::new();
    let act_env =
        build_workflow_act_env(runtime_state, &policy_eval, execution_provenance.clone()).await;
    let ctx = ctx
        .with_policy_evaluator(policy_eval.clone())
        .with_runtime_state(runtime_state.clone())
        .with_act_env(act_env);
    let behaviour_ctx = BehaviourContext::new();
    let mailbox = shared_mailbox();
    let control_registry = shared_control_registry(runtime_state);
    let proxy_registry = shared_proxy_registry(runtime_state);
    let suspended_yields = shared_suspended_yields(runtime_state);
    let execution_recorder = ExecutionRecorder::new(execution_provenance);
    let result = execute_workflow_inner_observed(
        workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        None,
        mailbox,
        control_registry,
        Some(proxy_registry),
        Some(suspended_yields),
        runtime_state,
        Some(terminal_observer),
        Some(&execution_recorder),
    )
    .await;
    execution_recorder.set_phase_from_result(&result);
    let execution_record = execution_recorder.snapshot();
    if persist_last_execution_record {
        runtime_state
            .set_last_execution_record(execution_record.clone())
            .await;
    }
    (result, execution_record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{
        BinaryOp, Capability, ControlLink, Effect, Expr, Guard, Obligation, Pattern, Provenance,
        RoleObligationRef,
    };
    use std::sync::Arc;

    fn test_role(name: &str) -> ash_core::Role {
        ash_core::Role {
            name: name.to_string(),
            authority: vec![],
            obligations: vec![],
        }
    }

    #[allow(dead_code)]
    fn test_role_with_obligation(name: &str, obligation: &str) -> ash_core::Role {
        ash_core::Role {
            name: name.to_string(),
            authority: vec![],
            obligations: vec![RoleObligationRef {
                name: obligation.to_string(),
            }],
        }
    }

    fn spawn_and_return_control(init: Expr) -> Workflow {
        Workflow::Spawn {
            workflow_type: "worker".to_string(),
            init,
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
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable {
                        name: "ctrl".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                }),
            }),
        }
    }

    async fn wait_for_child_completion(
        runtime_state: &RuntimeState,
        link: &ControlLink,
    ) -> crate::control_link::RetainedCompletionRecord {
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            runtime_state.wait_for_retained_completion(link),
        )
        .await
        .expect("spawned child should eventually seal retained completion")
        .expect("completion wait should return the sealed retained record")
    }

    #[tokio::test]
    async fn test_execute_done() {
        let workflow = Workflow::Done;
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_workflow_call_executes_registered_runtime_workflow_in_big_step() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_callable_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
                0,
                vec![],
            )
            .await;

        let workflow = Workflow::Call {
            target: "worker".to_string(),
            arguments: vec![],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };

        let result = execute_simple_in_state(&workflow, &runtime_state).await;
        assert_eq!(result.unwrap(), Value::Int(99));
    }

    #[tokio::test]
    async fn test_workflow_call_rejects_unknown_runtime_target_in_big_step() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Call {
            target: "missing_worker".to_string(),
            arguments: vec![],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };

        let result = execute_simple_in_state(&workflow, &runtime_state).await;
        assert!(matches!(
            result,
            Err(ExecError::ExecutionFailed(message)) if message.contains("missing_worker")
        ));
    }

    #[tokio::test]
    async fn test_workflow_call_rejects_non_zero_arity_in_big_step() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_callable_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
                0,
                vec![],
            )
            .await;

        let workflow = Workflow::Call {
            target: "worker".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1))],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };

        let result = execute_simple_in_state(&workflow, &runtime_state).await;
        assert!(matches!(
            result,
            Err(ExecError::Eval(EvalError::WrongArity {
                expected: 0,
                actual: 1,
                callee: Some(callee),
            })) if callee == "worker"
        ));
    }

    #[tokio::test]
    async fn test_build_workflow_act_env_uses_runtime_state_capability_context() {
        let runtime_state = RuntimeState::new().with_provider(
            "sensor",
            Arc::new(
                crate::capability::MockProvider::new("sensor", Effect::Epistemic)
                    .with_observe_value(Value::Int(7)),
            ),
        );
        let provenance = Provenance::new();

        let act_env =
            build_workflow_act_env(&runtime_state, &PolicyEvaluator::new(), provenance.clone())
                .await;

        let observed = act_env
            .capability_ctx
            .observe(&Capability {
                name: "sensor".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            })
            .await
            .expect("capability context should be wired from runtime state");

        assert_eq!(observed, Value::Int(7));
        assert_eq!(act_env.provenance, provenance);
        assert!(act_env.effects.is_empty());
    }

    #[tokio::test]
    async fn test_last_execution_record_projects_terminal_success() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(7)),
        };

        let result = execute_simple_in_state(&workflow, &runtime_state)
            .await
            .unwrap();
        assert_eq!(result, Value::Int(7));

        let record = runtime_state
            .last_execution_record()
            .await
            .expect("top-level execution should store an execution record");
        assert_eq!(
            record.phase(),
            &crate::execution_record::ExecutionPhase::Terminal(
                crate::execution_record::ExecutionTerminal::Return(Value::Int(7)),
            )
        );

        match record
            .project_workflow_outcome()
            .expect("terminal success should project")
        {
            crate::execution_record::SemanticWorkflowOutcome::Return { value, effect, .. } => {
                assert_eq!(value, Value::Int(7));
                assert_eq!(effect, Effect::Epistemic);
            }
            other => panic!("expected return projection, got {other:?}"),
        }

        let completion = record
            .project_completion()
            .expect("terminal success should project a completion payload");
        assert_eq!(completion.result(), &Ok(Value::Int(7)));
    }

    #[tokio::test]
    async fn test_last_execution_record_projects_terminal_rejection() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Ret {
            expr: Expr::Variable {
                name: "missing".to_string(),
                span: ash_core::ast::Span::default(),
            },
        };

        let result = execute_simple_in_state(&workflow, &runtime_state).await;
        assert!(matches!(
            result,
            Err(ExecError::Eval(EvalError::UndefinedVariable(_)))
        ));

        let record = runtime_state
            .last_execution_record()
            .await
            .expect("failing top-level execution should still store an execution record");

        match record.phase() {
            crate::execution_record::ExecutionPhase::Terminal(
                crate::execution_record::ExecutionTerminal::Reject(ExecError::Eval(
                    EvalError::UndefinedVariable(name),
                )),
            ) => assert_eq!(name, "missing"),
            other => panic!("expected terminal rejection record, got {other:?}"),
        }

        let completion = record
            .project_completion()
            .expect("terminal rejection should project a completion payload");
        assert!(matches!(
            completion.result(),
            Err(ExecError::Eval(EvalError::UndefinedVariable(name))) if name == "missing"
        ));
    }

    #[tokio::test]
    async fn test_last_execution_record_carries_orient_trace_and_effect() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Orient {
            expr: Expr::Literal(Value::Int(1)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };

        let result = execute_simple_in_state(&workflow, &runtime_state)
            .await
            .unwrap();
        assert_eq!(result, Value::Int(2));

        let record = runtime_state
            .last_execution_record()
            .await
            .expect("orient execution should store an execution record");
        let outcome = record
            .project_workflow_outcome()
            .expect("orient success should project");

        assert_eq!(outcome.effect(), Effect::Deliberative);
        assert!(matches!(
            outcome.trace(),
            [ash_core::TraceEvent::Orient { .. }]
        ));
    }

    #[tokio::test]
    async fn test_spawned_child_does_not_overwrite_top_level_last_execution_record() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
            )
            .await;

        let result = execute_simple_in_state(
            &spawn_and_return_control(Expr::Literal(Value::Null)),
            &runtime_state,
        )
        .await
        .expect("spawn should return a control link");

        let Value::ControlLink(link) = result.clone() else {
            panic!("expected control link, got {result:?}");
        };

        let _ = wait_for_child_completion(&runtime_state, &link).await;

        let record = runtime_state
            .last_execution_record()
            .await
            .expect("top-level execution should keep its authoritative record");
        assert_eq!(
            record.phase(),
            &crate::execution_record::ExecutionPhase::Terminal(
                crate::execution_record::ExecutionTerminal::Return(Value::ControlLink(
                    link.clone(),
                )),
            )
        );
        assert!(record.provenance().parent.is_none());
    }

    #[tokio::test]
    async fn test_spawned_child_registers_terminal_process_state_under_parent_process() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
            )
            .await;

        let parent_process_id = ProcessId::new();
        runtime_state
            .register_root_process(parent_process_id)
            .await
            .expect("parent root process should register");

        let ctx = Context::new()
            .with_runtime_state(runtime_state.clone())
            .project_process_child(
                crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
                None,
            );
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result = execute_workflow_with_behaviour_in_state(
            &spawn_and_return_control(Expr::Literal(Value::Null)),
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &runtime_state,
        )
        .await
        .expect("spawn should return a control link");

        let Value::ControlLink(link) = result else {
            panic!("expected control link, got {result:?}");
        };

        let _ = wait_for_child_completion(&runtime_state, &link).await;

        let children = runtime_state.process_children(parent_process_id).await;
        assert_eq!(
            children.len(),
            1,
            "spawn should register exactly one child process"
        );
        let child_process_id = children[0];
        assert_eq!(
            runtime_state.process_terminal_state(child_process_id).await,
            Some(ash_core::runtime::ProcessTerminalState::Succeeded {
                value: Value::Int(7),
            })
        );
    }

    #[tokio::test]
    async fn test_spawned_child_failure_records_failed_terminal_state_with_preserved_lower_cause() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Variable {
                        name: "missing_child_value".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                },
            )
            .await;

        let parent_process_id = ProcessId::new();
        runtime_state
            .register_root_process(parent_process_id)
            .await
            .expect("parent root process should register");

        let ctx = Context::new()
            .with_runtime_state(runtime_state.clone())
            .project_process_child(
                crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
                None,
            );
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result = execute_workflow_with_behaviour_in_state(
            &spawn_and_return_control(Expr::Literal(Value::Null)),
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &runtime_state,
        )
        .await
        .expect("spawn should return a control link");

        let Value::ControlLink(link) = result else {
            panic!("expected control link, got {result:?}");
        };

        let _ = wait_for_child_completion(&runtime_state, &link).await;

        let children = runtime_state.process_children(parent_process_id).await;
        assert_eq!(
            children.len(),
            1,
            "spawn should register exactly one child process"
        );
        let child_process_id = children[0];

        let Some(ProcessTerminalState::Failed {
            process_id,
            failure,
        }) = runtime_state.process_terminal_state(child_process_id).await
        else {
            panic!("expected failed terminal state recorded for child process");
        };

        assert_eq!(process_id, child_process_id);
        assert_eq!(failure.tower, TowerLevel::Proc);
        assert_eq!(failure.entity, FailureEntity::Process(child_process_id));

        let cause = failure
            .cause
            .as_deref()
            .expect("failed terminal observation should preserve lower cause");
        assert_eq!(
            cause.payload,
            Value::String("undefined variable: missing_child_value".to_string())
        );
        assert_eq!(cause.payload_type, "String");
    }

    #[test]
    fn test_process_terminal_state_from_exec_result_skips_blocked_children() {
        let process_id = ProcessId::new();
        let result: ExecResult<Value> = Err(ExecError::Blocked("waiting on input".to_string()));

        assert_eq!(
            process_terminal_state_from_exec_result(process_id, &result),
            None
        );
    }

    #[test]
    fn test_process_terminal_state_from_exec_result_records_cancelled_terminal_state_for_invalid_runtime()
     {
        let process_id = ProcessId::new();
        let result: ExecResult<Value> = Err(ExecError::InvalidRuntimeState("killed".to_string()));

        let Some(ProcessTerminalState::Cancelled {
            process_id: observed_process_id,
            failure,
        }) = process_terminal_state_from_exec_result(process_id, &result)
        else {
            panic!("expected invalid runtime state to map to cancelled terminal process state");
        };

        assert_eq!(observed_process_id, process_id);
        assert_eq!(failure.tower, TowerLevel::Proc);
        assert_eq!(failure.entity, FailureEntity::Process(process_id));
        assert!(failure.cause.is_some());
    }

    #[tokio::test]
    async fn test_spawned_blocked_child_transitions_to_running_without_terminal_state() {
        let runtime_state = RuntimeState::new();
        {
            let registry = runtime_state.proxy_registry();
            registry
                .lock()
                .await
                .register("test_role".to_string(), "proxy://instance-1".to_string());
        }
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Yield {
                    role: "test_role".to_string(),
                    request: Box::new(Expr::Literal(Value::Int(42))),
                    expected_response_type: ash_core::workflow_contract::TypeExpr::Named(
                        "Int".to_string(),
                    ),
                    continuation: Box::new(Workflow::Ret {
                        expr: Expr::Literal(Value::Int(0)),
                    }),
                    span: ash_core::ast::Span::default(),
                    resume_var: "response".to_string(),
                },
            )
            .await;

        let parent_process_id = ProcessId::new();
        runtime_state
            .register_root_process(parent_process_id)
            .await
            .expect("parent root process should register");

        let ctx = Context::new()
            .with_runtime_state(runtime_state.clone())
            .project_process_child(
                crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
                None,
            );
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result = execute_workflow_with_behaviour_in_state(
            &spawn_and_return_control(Expr::Literal(Value::Null)),
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &runtime_state,
        )
        .await
        .expect("spawn should return a control link");

        let Value::ControlLink(_) = result else {
            panic!("expected control link from spawn");
        };

        let mut observed_running_child = None;
        for _ in 0..20 {
            let children = runtime_state.process_children(parent_process_id).await;
            if let Some(child_process_id) = children.first().copied()
                && let Some(record) = runtime_state.process_record(child_process_id).await
                && record.lifecycle_state == ash_core::runtime::ProcessLifecycleState::Running
            {
                observed_running_child = Some(child_process_id);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let child_process_id = observed_running_child
            .expect("spawned blocked child should transition to Running before suspension");
        let child_record = runtime_state
            .process_record(child_process_id)
            .await
            .expect("child process record should exist");
        assert_eq!(
            child_record.lifecycle_state,
            ash_core::runtime::ProcessLifecycleState::Running
        );
        assert_eq!(
            runtime_state.process_terminal_state(child_process_id).await,
            None,
            "blocked child should not record a terminal process state"
        );
    }

    #[tokio::test]
    async fn test_spawned_child_does_not_overwrite_stream_top_level_last_execution_record() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
            )
            .await;

        let ctx = Context::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();
        let stream_ctx = StreamContext::new();

        let result = execute_workflow_with_stream_in_state(
            &spawn_and_return_control(Expr::Literal(Value::Null)),
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &stream_ctx,
            &runtime_state,
        )
        .await
        .expect("spawn should return a control link");

        let Value::ControlLink(link) = result.clone() else {
            panic!("expected control link, got {result:?}");
        };

        let _ = wait_for_child_completion(&runtime_state, &link).await;

        let record = runtime_state
            .last_execution_record()
            .await
            .expect("stream top-level execution should keep its authoritative record");
        assert_eq!(
            record.phase(),
            &crate::execution_record::ExecutionPhase::Terminal(
                crate::execution_record::ExecutionTerminal::Return(Value::ControlLink(
                    link.clone(),
                )),
            )
        );
        assert!(record.provenance().parent.is_none());
    }

    #[tokio::test]
    async fn test_oblig_provides_active_role_context_to_check() {
        let workflow = Workflow::Oblig {
            role: test_role("reviewer"),
            workflow: Box::new(Workflow::Check {
                obligation: Obligation::Obliged {
                    role: test_role("reviewer"),
                    condition: Expr::Literal(Value::Bool(true)),
                },
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::String("ok".to_string())),
                }),
            }),
        };

        let result = execute_simple(&workflow).await;

        assert_eq!(result, Ok(Value::String("ok".to_string())));
    }

    #[tokio::test]
    async fn test_check_fails_when_active_role_does_not_match_obligation_role() {
        let workflow = Workflow::Oblig {
            role: test_role("reviewer"),
            workflow: Box::new(Workflow::Check {
                obligation: Obligation::Obliged {
                    role: test_role("approver"),
                    condition: Expr::Literal(Value::Bool(true)),
                },
                continuation: Box::new(Workflow::Done),
            }),
        };

        let result = execute_simple(&workflow).await;

        assert!(matches!(
            result,
            Err(ExecError::ExecutionFailed(message))
                if message.contains("active role")
                    && message.contains("reviewer")
                    && message.contains("approver")
        ));
    }

    #[tokio::test]
    async fn test_check_fails_when_obligation_condition_is_false() {
        let workflow = Workflow::Oblig {
            role: test_role("reviewer"),
            workflow: Box::new(Workflow::Check {
                obligation: Obligation::Obliged {
                    role: test_role("reviewer"),
                    condition: Expr::Literal(Value::Bool(false)),
                },
                continuation: Box::new(Workflow::Done),
            }),
        };

        let result = execute_simple(&workflow).await;

        assert!(matches!(
            result,
            Err(ExecError::ExecutionFailed(message))
                if message.contains("obligation check failed")
        ));
    }

    #[test]
    fn test_active_actor_uses_role_context_before_system_fallback() {
        let ctx = Context::new()
            .with_role_context(crate::role_context::RoleContext::new(test_role("operator")));

        assert_eq!(active_actor(&ctx), Role::new("operator"));
        assert_eq!(active_actor(&Context::new()), Role::new("system"));
    }

    #[tokio::test]
    async fn test_execute_ret() {
        use ash_core::Expr;

        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_let() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(42)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_let_tuple() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable {
                    name: "a".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "b".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ]),
            expr: Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "a".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "b".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                },
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_execute_if_true() {
        use ash_core::Expr;

        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_execute_if_false() {
        use ash_core::Expr;

        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(false)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[tokio::test]
    async fn test_execute_seq_proper() {
        use ash_core::{Expr, Pattern};

        // Proper seq where first binds and second uses
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Done),
                second: Box::new(Workflow::Ret {
                    expr: Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                }),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn test_execute_foreach() {
        use ash_core::{Expr, Pattern};

        // ForEach iterates over a collection, executing body for each element
        // Each iteration gets its own context extended from the parent
        let workflow = Workflow::ForEach {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            collection: Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]))),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        // Returns the result of the last iteration
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_execute_orient() {
        use ash_core::Expr;

        let workflow = Workflow::Orient {
            expr: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Value::Int(1))),
                right: Box::new(Expr::Literal(Value::Int(2))),
            },
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_propose() {
        use ash_core::Expr;

        // Propose is advisory - just continues
        let workflow = Workflow::Propose {
            action_name: "test".to_string(),
            action_arguments: vec![],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_maybe_success() {
        use ash_core::Expr;

        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_execute_maybe_fallback() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                expr: Expr::Literal(Value::Int(1)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable {
                        name: "undefined_var".to_string(),
                        span: ash_core::ast::Span::default(),
                    }, // Will fail
                }),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[tokio::test]
    async fn test_execute_must_success() {
        use ash_core::Expr;

        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_must_failure() {
        use ash_core::Expr;

        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "undefined".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        };
        let result = execute_simple(&workflow).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with() {
        use ash_core::Expr;

        let workflow = Workflow::With {
            capability: Capability {
                name: "test".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_act_guard_fails() {
        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "test".to_string(),
            arguments: vec![],
            guard: Guard::Never,
            provenance: Provenance::new(),
            result_name: None,
            continuation: Box::new(Workflow::Done),
        };
        let result = execute_simple(&workflow).await;
        assert!(matches!(result, Err(ExecError::GuardFailed { .. })));
    }

    #[tokio::test]
    async fn test_complex_workflow() {
        use ash_core::{Expr, Pattern};

        // let (x, y) = (10, 20) in
        //   if x < y then
        //     x + y
        //   else
        //     0
        let workflow = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "y".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ]),
            expr: Expr::Literal(Value::List(Box::new(vec![Value::Int(10), Value::Int(20)]))),
            continuation: Box::new(Workflow::If {
                condition: Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "y".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                },
                then_branch: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                        right: Box::new(Expr::Variable {
                            name: "y".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                    },
                }),
                else_branch: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(0)),
                }),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(30));
    }

    /// Helper: build a CapabilityContext with a single MockProvider for Act tests.
    fn cap_ctx_with_mock(mock: crate::capability::MockProvider) -> CapabilityContext {
        let mut cap_ctx = CapabilityContext::new();
        cap_ctx.register(Box::new(mock));
        cap_ctx
    }

    // ------------------------------------------------------------------
    // TASK-490: Integration tests for Act continuation forms
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_execute_act_with_then_continuation() {
        // Act with result_name: None and an Orient continuation.
        // The action result (99) should be discarded and the continuation
        // should run, returning its own result (42).
        let mock = crate::capability::MockProvider::new("test", Effect::Operational)
            .with_execute_result(Ok(Value::Int(99)));
        let cap_ctx = cap_ctx_with_mock(mock);

        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "do_thing".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: None,
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };

        let ctx = Context::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result =
            execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
                .await
                .unwrap();

        // Continuation result is the final value, not the action result.
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_act_with_result_binding() {
        // Act with result_name: Some("result"), MockProvider returns Int(42).
        // The continuation reads "result" and returns it.
        let mock = crate::capability::MockProvider::new("test", Effect::Operational)
            .with_execute_result(Ok(Value::Int(42)));
        let cap_ctx = cap_ctx_with_mock(mock);

        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "fetch_value".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: Some("result".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "result".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        };

        let ctx = Context::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result =
            execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
                .await
                .unwrap();

        // The bound value should be accessible in the continuation.
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_act_result_binding_and_continuation() {
        // Combined: result_name binds the action result, and the continuation
        // uses it in a computation (Let + arithmetic).
        // Action returns Int(10), continuation doubles it to Int(20).
        let mock = crate::capability::MockProvider::new("test", Effect::Operational)
            .with_execute_result(Ok(Value::Int(10)));
        let cap_ctx = cap_ctx_with_mock(mock);

        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "get_number".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: Some("result".to_string()),
            continuation: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "doubled".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "result".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "result".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                },
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable {
                        name: "doubled".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                }),
            }),
        };

        let ctx = Context::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result =
            execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
                .await
                .unwrap();

        assert_eq!(result, Value::Int(20));
    }

    #[tokio::test]
    async fn test_execute_act_bare_regression() {
        // Bare Act with result_name: None and continuation: Done.
        // Per DESIGN-019: returns the action result directly.
        let mock = crate::capability::MockProvider::new("test", Effect::Operational)
            .with_execute_result(Ok(Value::Int(123)));
        let cap_ctx = cap_ctx_with_mock(mock);

        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "fire".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: None,
            continuation: Box::new(Workflow::Done),
        };

        let ctx = Context::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result =
            execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
                .await
                .unwrap();

        // Bare act returns the action result directly (per DESIGN-019 Decision 5)
        assert_eq!(result, Value::Int(123));
    }

    #[tokio::test]
    async fn test_execute_act_continuation_error_propagation() {
        use ash_core::capability::CapabilityError;

        // Act where the action returns an error. The error should propagate
        // and the continuation must NOT run.
        let mock = crate::capability::MockProvider::new("test", Effect::Operational)
            .with_execute_result(Err(CapabilityError::ExecutionFailed("boom".to_string())));
        let cap_ctx = cap_ctx_with_mock(mock);

        let workflow = Workflow::Act {
            provider_name: "test".to_string(),
            action_name: "fail_action".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: Some("result".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };

        let ctx = Context::new();
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();

        let result =
            execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
                .await;

        assert!(result.is_err(), "action error should propagate");
        match result {
            Err(ExecError::ExecutionFailed(msg)) => {
                assert!(
                    msg.contains("boom"),
                    "error message should contain 'boom', got: {msg}"
                );
            }
            other => panic!("expected ExecutionFailed error, got: {other:?}"),
        }
    }
}
