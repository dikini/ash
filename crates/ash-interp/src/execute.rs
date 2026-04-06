//! Workflow execution engine
//!
//! Executes workflows in a runtime context, handling all workflow variants.

use ash_core::{Effect, Expr, Value, Workflow};

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
use crate::eval::eval_expr;
use crate::exec_send::execute_send;
use crate::execute_set::execute_set;
use crate::execute_stream::{CoreReceiveRuntime, execute_core_receive};
use crate::guard::eval_guard;
use crate::mailbox::{Mailbox, SharedMailbox};
use crate::pattern::match_pattern;
use crate::policy::PolicyEvaluator;
use crate::proxy_registry::ProxyRegistry;
use crate::runtime_outcome_state::RuntimeOutcomeState;
use crate::runtime_state::{RuntimeState, SPAWNED_CHILD_CONTROL_BINDING};
use crate::stream::StreamContext;
use crate::yield_state::{CorrelationId, SuspendedYields, YieldState};

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::BTreeSet, iter};
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

pub fn execute_workflow_with_behaviour_in_state<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
    behaviour_ctx: &'a BehaviourContext,
    runtime_state: &'a RuntimeState,
) -> BoxFuture<'a, ExecResult<Value>> {
    let mailbox = shared_mailbox();
    let control_registry = shared_control_registry(runtime_state);
    let proxy_registry = shared_proxy_registry(runtime_state);
    let suspended_yields = shared_suspended_yields(runtime_state);
    execute_workflow_inner_observed(
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
    )
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

fn conservative_effect_upper_bound(workflow: &Workflow) -> ConservativeRetainedEffectSummary {
    let mut reached = conservative_reached_effect_upper_bound(workflow);
    let terminal = reached.iter().copied().max().unwrap_or(Effect::Epistemic);
    reached.insert(terminal);
    ConservativeRetainedEffectSummary::new(terminal, reached)
}

fn conservative_reached_effect_upper_bound(workflow: &Workflow) -> BTreeSet<Effect> {
    fn singleton(effect: Effect) -> BTreeSet<Effect> {
        iter::once(effect).collect()
    }

    fn union_many<'a>(workflows: impl IntoIterator<Item = &'a Workflow>) -> BTreeSet<Effect> {
        workflows
            .into_iter()
            .flat_map(conservative_reached_effect_upper_bound)
            .collect()
    }

    match workflow {
        Workflow::Observe { continuation, .. } => {
            let mut reached = conservative_reached_effect_upper_bound(continuation);
            reached.insert(Effect::Epistemic);
            reached
        }
        Workflow::Receive { arms, .. } => {
            let mut reached: BTreeSet<_> = arms
                .iter()
                .flat_map(|arm| conservative_reached_effect_upper_bound(&arm.body))
                .collect();
            reached.insert(Effect::Epistemic);
            reached
        }
        Workflow::Orient { continuation, .. } | Workflow::Propose { continuation, .. } => {
            let mut reached = conservative_reached_effect_upper_bound(continuation);
            reached.insert(Effect::Deliberative);
            reached
        }
        Workflow::Decide { continuation, .. }
        | Workflow::Check { continuation, .. }
        | Workflow::Yield { continuation, .. } => {
            let mut reached = conservative_reached_effect_upper_bound(continuation);
            reached.insert(Effect::Evaluative);
            reached
        }
        Workflow::CheckObligation { .. } | Workflow::Oblige { .. } => singleton(Effect::Evaluative),
        Workflow::Act { .. }
        | Workflow::Set { .. }
        | Workflow::Send { .. }
        | Workflow::Spawn { .. }
        | Workflow::Kill { .. }
        | Workflow::Pause { .. }
        | Workflow::Resume { .. }
        | Workflow::ProxyResume { .. } => singleton(Effect::Operational),
        Workflow::Oblig { workflow, .. }
        | Workflow::ForEach { body: workflow, .. }
        | Workflow::With { workflow, .. }
        | Workflow::Must { workflow } => conservative_reached_effect_upper_bound(workflow),
        Workflow::Let { continuation, .. } | Workflow::Split { continuation, .. } => {
            conservative_reached_effect_upper_bound(continuation)
        }
        Workflow::CheckHealth { continuation, .. } => {
            let mut reached = conservative_reached_effect_upper_bound(continuation);
            reached.insert(Effect::Epistemic);
            reached
        }
        Workflow::If {
            then_branch,
            else_branch,
            ..
        } => union_many([then_branch.as_ref(), else_branch.as_ref()]),
        Workflow::Seq { first, second } => union_many([first.as_ref(), second.as_ref()]),
        Workflow::Par { workflows } => union_many(workflows.iter()),
        Workflow::Maybe { primary, fallback } => union_many([primary.as_ref(), fallback.as_ref()]),
        Workflow::Ret { .. } | Workflow::Done => BTreeSet::new(),
    }
}

async fn run_spawned_child_workflow(
    runtime_state: RuntimeState,
    child_workflow: Workflow,
    init_value: Value,
    link: ash_core::ControlLink,
    provenance: ConservativeRetainedProvenanceSummary,
) {
    tokio::task::yield_now().await;

    let effects = conservative_effect_upper_bound(&child_workflow);
    let terminal_observer = TerminalObservationRecorder::new();
    let child_result = execute_with_bindings_with_terminal_observation_in_state(
        &child_workflow,
        &runtime_state,
        RuntimeState::spawned_child_init_bindings(init_value, link.clone()),
        &terminal_observer,
    )
    .await;

    let outcome_state = RuntimeOutcomeState::from_exec_result(&child_result);
    if !outcome_state.is_terminal() {
        return;
    }

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
        Ok(_) => {}
        Err(ControlLinkError::CompletionAlreadySealed(_, record))
            if record.kind() == RetainedCompletionKind::ControlTerminated => {}
        Err(ControlLinkError::Terminated(..)) => {}
        Err(error) => panic!(
            "spawned child completion sealing failed unexpectedly for instance {:?}: {error}",
            link.instance_id
        ),
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
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        if let Some(link) = spawned_child_control_link(&ctx)? {
            runtime_state.wait_for_control_authority(&link).await?;
        }

        let terminal_ctx_snapshot = ctx.clone();
        let result = match workflow {
            // Terminal workflow - returns null
            Workflow::Done => Ok(Value::Null),

            // Return with value
            Workflow::Ret { expr } => eval_expr(expr, &ctx).map_err(ExecError::Eval),

            // Variable binding
            Workflow::Let {
                pattern,
                expr,
                continuation,
            } => {
                let value = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: value.clone(),
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
                )
                .await
            }

            // Conditional execution
            Workflow::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = eval_expr(condition, &ctx).map_err(ExecError::Eval)?;
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
                )
                .await?;
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
                )
                .await
            }

            // Parallel composition
            Workflow::Par { workflows } => {
                if workflows.is_empty() {
                    return Ok(Value::Null);
                }

                // Execute all workflows in parallel and collect results
                let futures: Vec<_> = workflows
                    .iter()
                    .map(|wf| {
                        execute_workflow_inner_observed(
                            wf,
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
                        )
                    })
                    .collect();

                let results = futures::future::join_all(futures).await;

                // Check for errors
                let values: Vec<Value> = results.into_iter().collect::<Result<Vec<_>, _>>()?;

                Ok(Value::List(Box::new(values)))
            }

            // Observe from capability
            Workflow::Observe {
                capability,
                pattern,
                continuation,
            } => {
                let value = cap_ctx.observe(capability).await?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: value.clone(),
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
                )
                .await
            }

            // Orient - evaluate expression and continue
            Workflow::Orient { expr, continuation } => {
                let _ = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;
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
                )
                .await
            }

            // Execute action with guard
            Workflow::Act {
                action,
                guard,
                provenance: _,
            } => {
                // Evaluate guard
                let guard_result = eval_guard(guard, &ctx).map_err(|_| ExecError::GuardFailed {
                    guard: format!("{:?}", guard),
                })?;

                if !guard_result {
                    return Err(ExecError::GuardFailed {
                        guard: format!("{:?}", guard),
                    });
                }

                cap_ctx.execute(action, &action.name).await
            }

            // Propose action (advisory - just continue)
            Workflow::Propose {
                action: _,
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
                )
                .await
            }

            // Decide under policy
            Workflow::Decide {
                expr,
                policy,
                continuation,
            } => {
                let value = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;

                // Create a temporary context with the decision value
                let mut decision_ctx = ctx.extend();
                decision_ctx.set("decision_value".to_string(), value);

                let decision = policy_eval.evaluate(policy, &decision_ctx)?;

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
                let coll_val = eval_expr(collection, &ctx).map_err(ExecError::Eval)?;

                match coll_val {
                    Value::List(items) => {
                        let mut last_result = Value::Null;

                        for item in items.iter() {
                            let bindings = match_pattern(pattern, item).map_err(|_| {
                                ExecError::PatternMatchFailed {
                                    pattern: format!("{:?}", pattern),
                                    value: item.clone(),
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

                        match eval_expr(condition, &ctx).map_err(ExecError::Eval)? {
                            Value::Bool(true) => {}
                            Value::Bool(false) => {
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
                )
                .await
            }

            // Set a value on a writable channel
            Workflow::Set {
                capability,
                channel,
                value,
            } => {
                let val = eval_expr(value, &ctx).map_err(ExecError::Eval)?;
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
                let val = eval_expr(value, &ctx).map_err(ExecError::Eval)?;
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
                let init_value = eval_expr(init, &ctx).map_err(ExecError::Eval)?;
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
                    tokio::spawn(run_spawned_child_workflow(
                        runtime_state.clone(),
                        child_workflow,
                        init_value.clone(),
                        control,
                        provenance,
                    ));
                }

                // Match pattern and bind
                let bindings = match_pattern(pattern, &instance_value).map_err(|_| {
                    ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: instance_value.clone(),
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
                let split_value = eval_expr(&Expr::Split(Box::new(expr.clone())), &ctx)
                    .map_err(ExecError::Eval)?;

                // Match pattern and bind
                let bindings = match_pattern(pattern, &split_value).map_err(|_| {
                    ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: split_value.clone(),
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

                // Return null as per spec
                Ok(Value::Null)
            }

            // CHECK - Check/discharge a linear obligation (contract tracking)
            Workflow::CheckObligation { name, span: _ } => {
                // Attempt to discharge the obligation
                let discharged = ctx.discharge_obligation(name);

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
                let request_value = eval_expr(request, &ctx).map_err(ExecError::Eval)?;

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
                let response_value = eval_expr(value, &ctx).map_err(ExecError::Eval)?;

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
                )
                .await
            }
        };
        finish_with_terminal_observation(terminal_observer, &terminal_ctx_snapshot, result)
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
    let mailbox = shared_mailbox();
    let control_registry = shared_control_registry(runtime_state);
    let proxy_registry = shared_proxy_registry(runtime_state);
    let suspended_yields = shared_suspended_yields(runtime_state);
    execute_workflow_inner_observed(
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
    )
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

async fn execute_with_bindings_with_terminal_observation_in_state(
    workflow: &Workflow,
    runtime_state: &RuntimeState,
    input_bindings: std::collections::HashMap<String, Value>,
    terminal_observer: &TerminalObservationRecorder,
) -> ExecResult<Value> {
    let ctx = Context::with_bindings(input_bindings);
    let cap_ctx = runtime_state.create_capability_context().await;
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();
    let mailbox = shared_mailbox();
    let control_registry = shared_control_registry(runtime_state);
    let proxy_registry = shared_proxy_registry(runtime_state);
    let suspended_yields = shared_suspended_yields(runtime_state);
    execute_workflow_inner_observed(
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
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Action, BinaryOp, Capability, Effect, Expr, Guard, Obligation, Provenance};

    fn test_role(name: &str) -> ash_core::Role {
        ash_core::Role {
            name: name.to_string(),
            authority: vec![],
            obligations: vec![],
        }
    }

    #[tokio::test]
    async fn test_execute_done() {
        let workflow = Workflow::Done;
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Null);
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
            pattern: Pattern::Variable("x".to_string()),
            expr: Expr::Literal(Value::Int(42)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("x".to_string()),
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
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ]),
            expr: Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("a".to_string())),
                    right: Box::new(Expr::Variable("b".to_string())),
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
            pattern: Pattern::Variable("x".to_string()),
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Done),
                second: Box::new(Workflow::Ret {
                    expr: Expr::Variable("x".to_string()),
                }),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn test_execute_par() {
        use ash_core::Expr;

        let workflow = Workflow::Par {
            workflows: vec![
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                },
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(2)),
                },
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(3)),
                },
            ],
        };
        let result = execute_simple(&workflow).await.unwrap();

        // Result is a list of all workflow results
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(items.contains(&Value::Int(1)));
                assert!(items.contains(&Value::Int(2)));
                assert!(items.contains(&Value::Int(3)));
            }
            _ => panic!("Expected list, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_execute_par_empty() {
        let workflow = Workflow::Par { workflows: vec![] };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_execute_foreach() {
        use ash_core::{Expr, Pattern};

        // ForEach iterates over a collection, executing body for each element
        // Each iteration gets its own context extended from the parent
        let workflow = Workflow::ForEach {
            pattern: Pattern::Variable("x".to_string()),
            collection: Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]))),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable("x".to_string()),
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
            action: Action {
                name: "test".to_string(),
                arguments: vec![],
            },
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
                pattern: Pattern::Variable("x".to_string()),
                expr: Expr::Literal(Value::Int(1)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable("undefined_var".to_string()), // Will fail
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
                expr: Expr::Variable("undefined".to_string()),
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
            action: Action {
                name: "test".to_string(),
                arguments: vec![],
            },
            guard: Guard::Never,
            provenance: Provenance::new(),
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
                Pattern::Variable("x".to_string()),
                Pattern::Variable("y".to_string()),
            ]),
            expr: Expr::Literal(Value::List(Box::new(vec![Value::Int(10), Value::Int(20)]))),
            continuation: Box::new(Workflow::If {
                condition: Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Variable("y".to_string())),
                },
                then_branch: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Variable("y".to_string())),
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
}
