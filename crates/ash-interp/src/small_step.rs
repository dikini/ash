//! Small-step abstract machine for Ash workflows.
//!
//! Drives a `Config` through small-step transitions until it reaches a
//! terminal configuration.

use ash_core::Value;
use ash_core::small_step::{Config, Frame, Stmt, StmtList, lower_workflow};

use crate::ExecResult;
use crate::behaviour::BehaviourContext;
use crate::capability::CapabilityContext;
use crate::capability_policy::{CapabilityPolicyEvaluator, Role};
use crate::context::Context;
use crate::error::{EvalError, ExecError};
use crate::eval::eval_expr;
use crate::exec_send::execute_send;
use crate::execute::resolve_registered_runtime_call_target;
use crate::execute_set::execute_set;
use crate::guard::eval_guard;
use crate::pattern::match_pattern;
use crate::policy::PolicyEvaluator;
use crate::runtime_state::RuntimeState;
use crate::stream::StreamContext;

use std::collections::HashMap;

/// Returns the active actor role from the context, or "system" as fallback.
fn active_actor(ctx: &Context) -> Role {
    ctx.role_context()
        .map(|role_ctx| Role::new(role_ctx.active_role.name.clone()))
        .unwrap_or_else(|| Role::new("system"))
}

/// Resolve a control link from a variable name in the context.
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

/// Outcome of a single small step.
#[derive(Debug, Clone, PartialEq)]
pub enum StepOutcome {
    /// The machine made progress and can step again.
    Progress,
    /// The machine reached a terminal success state.
    Returned(Value),
    /// The machine reached a terminal failure state.
    Rejected(String),
}

/// Apply the top frame from a running configuration, returning the next
/// statement list to execute, if any.
fn apply_frame(config: &mut Config) -> ExecResult<Option<StmtList>> {
    if let Config::Running { frames, .. } = config {
        if let Some(frame) = frames.pop() {
            match frame {
                Frame::Seq { rest } => Ok(Some(rest)),
                Frame::ForEachIter {
                    pattern,
                    mut items,
                    body,
                } => {
                    if items.is_empty() {
                        return apply_frame(config);
                    }
                    let item = items.remove(0);
                    let bindings = match_pattern(&pattern, &item).map_err(|_| {
                        ExecError::PatternMatchFailed {
                            pattern: format!("{pattern:?}"),
                            value: Box::new(item.clone()),
                        }
                    })?;
                    if let Config::Running { env, frames, .. } = config {
                        env.extend(bindings);
                        if !items.is_empty() {
                            frames.push(Frame::ForEachIter {
                                pattern,
                                items,
                                body: body.clone(),
                            });
                        }
                    }
                    Ok(Some(body))
                }
                Frame::ResumeYield { .. } | Frame::Catch { .. } | Frame::MustGuard => {
                    apply_frame(config)
                }
                Frame::RestoreEnv { saved } => {
                    if let Config::Running { env, .. } = config {
                        *env = saved;
                    }
                    apply_frame(config)
                }
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Unwind the frame stack looking for the nearest `Catch` frame.
/// If a `MustGuard` is encountered first, wrap the error in `MustFailure`.
fn unwind_stack(config: &mut Config, error: ExecError) -> ExecResult<StepOutcome> {
    if let Config::Running { frames, stmts, .. } = config {
        let mut must_failed = false;
        while let Some(frame) = frames.pop() {
            match frame {
                Frame::Catch { fallback } => {
                    *stmts = fallback.stmts;
                    frames.extend(fallback.frames);
                    return Ok(StepOutcome::Progress);
                }
                Frame::MustGuard => {
                    must_failed = true;
                }
                Frame::Seq { .. }
                | Frame::ForEachIter { .. }
                | Frame::ResumeYield { .. }
                | Frame::RestoreEnv { .. } => {}
            }
        }
        if must_failed {
            return Err(ExecError::MustFailure(error.to_string()));
        }
    }
    Err(error)
}

/// Execute one small step.
///
/// # Errors
/// Returns `ExecError` if expression evaluation, guard evaluation, or the
/// capability action fails.
pub async fn step(
    config: &mut Config,
    cap_ctx: &CapabilityContext,
    behaviour_ctx: &BehaviourContext,
    policy_eval: &PolicyEvaluator,
    stream_ctx: Option<&StreamContext>,
    runtime_state: &RuntimeState,
) -> ExecResult<StepOutcome> {
    match step_inner(
        config,
        cap_ctx,
        behaviour_ctx,
        policy_eval,
        stream_ctx,
        runtime_state,
    )
    .await
    {
        Ok(outcome) => Ok(outcome),
        Err(err) => unwind_stack(config, err),
    }
}

async fn step_inner(
    config: &mut Config,
    cap_ctx: &CapabilityContext,
    behaviour_ctx: &BehaviourContext,
    policy_eval: &PolicyEvaluator,
    stream_ctx: Option<&StreamContext>,
    runtime_state: &RuntimeState,
) -> ExecResult<StepOutcome> {
    match config {
        Config::Running { env, stmts, .. } => {
            if stmts.is_empty() {
                loop {
                    match apply_frame(config)? {
                        Some(rest) => {
                            if let Config::Running {
                                stmts: s,
                                frames: f,
                                ..
                            } = config
                            {
                                *s = rest.stmts;
                                f.extend(rest.frames);
                                if !s.is_empty() {
                                    return Ok(StepOutcome::Progress);
                                }
                            }
                        }
                        None => {
                            return Err(ExecError::ExecutionFailed(
                                "stuck: no statements and no frames".to_string(),
                            ));
                        }
                    }
                }
            }

            let head = stmts.remove(0);
            let tail = stmts.clone();
            stmts.clear();

            match head {
                Stmt::Done => finish_with_value(config, Value::Null),
                Stmt::Ret { expr } => {
                    let ctx = Context::with_bindings(env.clone());
                    let value = eval_expr(&expr, &ctx).map_err(ExecError::Eval)?;
                    finish_with_value(config, value)
                }
                Stmt::Let { pattern, expr } => {
                    let ctx = Context::with_bindings(env.clone());
                    let value = eval_expr(&expr, &ctx).map_err(ExecError::Eval)?;
                    let bindings = match_pattern(&pattern, &value).map_err(|_| {
                        ExecError::PatternMatchFailed {
                            pattern: format!("{pattern:?}"),
                            value: Box::new(value.clone()),
                        }
                    })?;
                    env.extend(bindings);
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let cond = eval_expr(&condition, &ctx).map_err(ExecError::Eval)?;
                    match cond {
                        Value::Bool(true) => {
                            if let Config::Running {
                                stmts: s,
                                frames: f,
                                ..
                            } = config
                            {
                                *s = then_branch.stmts;
                                if !tail.is_empty() {
                                    f.push(Frame::Seq {
                                        rest: StmtList {
                                            stmts: tail,
                                            frames: vec![],
                                        },
                                    });
                                }
                                f.extend(then_branch.frames);
                            }
                            Ok(StepOutcome::Progress)
                        }
                        Value::Bool(false) => {
                            if let Config::Running {
                                stmts: s,
                                frames: f,
                                ..
                            } = config
                            {
                                *s = else_branch.stmts;
                                if !tail.is_empty() {
                                    f.push(Frame::Seq {
                                        rest: StmtList {
                                            stmts: tail,
                                            frames: vec![],
                                        },
                                    });
                                }
                                f.extend(else_branch.frames);
                            }
                            Ok(StepOutcome::Progress)
                        }
                        other => Err(ExecError::Eval(EvalError::TypeMismatch {
                            expected: "bool".to_string(),
                            actual: format!("{other:?}"),
                        })),
                    }
                }
                Stmt::Act {
                    provider_name,
                    action_name,
                    arguments,
                    guard,
                    provenance: _,
                    result_name,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let guard_ok = eval_guard(&guard, &ctx).map_err(ExecError::Eval)?;
                    if !guard_ok {
                        return Err(ExecError::GuardFailed {
                            guard: format!("{guard:?}"),
                        });
                    }
                    let args: Vec<Value> = arguments
                        .iter()
                        .map(|arg| eval_expr(arg, &ctx).map_err(ExecError::Eval))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = cap_ctx.execute(&provider_name, &action_name, &args).await?;
                    if let Some(name) = result_name {
                        env.insert(name, result);
                    }
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Call { target, arguments } => {
                    let callable = runtime_state.callable_workflow(&target).await;
                    let child_workflow = resolve_registered_runtime_call_target(
                        runtime_state,
                        &target,
                        arguments.len(),
                    )
                    .await?;

                    // Evaluate arguments in caller's context, build isolated child env
                    let child_env = if let Some(ref callable) = callable {
                        let ctx = Context::with_bindings(env.clone());
                        let mut child_env = HashMap::new();
                        for (param_name, arg_expr) in callable.params.iter().zip(arguments.iter()) {
                            let arg_value = eval_expr(arg_expr, &ctx).map_err(ExecError::Eval)?;
                            child_env.insert(param_name.clone(), arg_value);
                        }
                        child_env
                    } else {
                        HashMap::new()
                    };

                    let child_config = lower_workflow(&child_workflow);
                    let (child_stmts, child_frames) = match child_config {
                        Config::Running { stmts, frames, .. } => (stmts, frames),
                        Config::Returned(value) => {
                            return Err(ExecError::ExecutionFailed(format!(
                                "runtime call target '{target}' lowered to unexpected returned config: {value:?}"
                            )));
                        }
                        Config::Rejected(message) => {
                            return Err(ExecError::ExecutionFailed(format!(
                                "runtime call target '{target}' lowered to unexpected rejected config: {message}"
                            )));
                        }
                    };

                    if let Config::Running {
                        stmts: s,
                        frames: f,
                        env,
                    } = config
                    {
                        *s = child_stmts;
                        if !tail.is_empty() {
                            f.push(Frame::Seq {
                                rest: StmtList {
                                    stmts: tail,
                                    frames: vec![],
                                },
                            });
                        }
                        // Save parent env and install child env for child execution.
                        // RestoreEnv is pushed AFTER Seq so it's popped FIRST,
                        // restoring the parent env before the continuation runs.
                        let parent_env = std::mem::replace(env, child_env);
                        f.push(Frame::RestoreEnv { saved: parent_env });
                        f.extend(child_frames);
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Observe {
                    capability,
                    pattern,
                } => {
                    let value = cap_ctx.observe(&capability).await?;
                    let bindings = match_pattern(&pattern, &value).map_err(|_| {
                        ExecError::PatternMatchFailed {
                            pattern: format!("{pattern:?}"),
                            value: Box::new(value.clone()),
                        }
                    })?;
                    env.extend(bindings);
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Orient { expr } => {
                    let ctx = Context::with_bindings(env.clone());
                    eval_expr(&expr, &ctx).map_err(ExecError::Eval)?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Propose {
                    action_name: _,
                    action_arguments: _,
                } => {
                    // Advisory: no-op in the small-step prototype.
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Decide { expr, policy } => {
                    let ctx = Context::with_bindings(env.clone());
                    let value = eval_expr(&expr, &ctx).map_err(ExecError::Eval)?;
                    let mut decision_ctx = ctx.extend();
                    decision_ctx.set("decision_value".to_string(), value);
                    let decision = policy_eval.evaluate(&policy, &decision_ctx)?;
                    match decision {
                        ash_core::Decision::Permit => {
                            if let Config::Running { stmts: s, .. } = config {
                                *s = tail;
                            }
                            Ok(StepOutcome::Progress)
                        }
                        ash_core::Decision::Deny => Err(ExecError::PolicyDenied {
                            policy: policy.clone(),
                        }),
                        ash_core::Decision::RequireApproval | ash_core::Decision::Escalate => {
                            Err(ExecError::PolicyDenied {
                                policy: policy.clone(),
                            })
                        }
                    }
                }
                Stmt::Check { obligation } => {
                    let ctx = Context::with_bindings(env.clone());
                    match obligation {
                        ash_core::Obligation::Obliged { role: _, condition } => {
                            match eval_expr(&condition, &ctx).map_err(ExecError::Eval)? {
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
                        _ => {
                            return Err(ExecError::ExecutionFailed(
                                "unsupported runtime obligation check".to_string(),
                            ));
                        }
                    }
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::With { capability: _ } => {
                    // Capability scoping is a no-op in the prototype.
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Oblig { role: _ } => {
                    // Role assignment is a no-op in the prototype.
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Maybe { primary } => {
                    if let Config::Running {
                        stmts: s,
                        frames: f,
                        ..
                    } = config
                    {
                        *s = primary.stmts;
                        if !tail.is_empty() {
                            f.push(Frame::Seq {
                                rest: StmtList {
                                    stmts: tail,
                                    frames: vec![],
                                },
                            });
                        }
                        f.extend(primary.frames);
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Must { body } => {
                    if let Config::Running {
                        stmts: s,
                        frames: f,
                        ..
                    } = config
                    {
                        *s = body.stmts;
                        if !tail.is_empty() {
                            f.push(Frame::Seq {
                                rest: StmtList {
                                    stmts: tail,
                                    frames: vec![],
                                },
                            });
                        }
                        f.extend(body.frames);
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::ForEach {
                    pattern,
                    collection,
                    body,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let coll_val = eval_expr(&collection, &ctx).map_err(ExecError::Eval)?;
                    match coll_val {
                        Value::List(items) => {
                            if items.is_empty() {
                                finish_with_value(config, Value::Null)
                            } else {
                                let mut iter = *items;
                                let first = iter.remove(0);
                                let bindings = match_pattern(&pattern, &first).map_err(|_| {
                                    ExecError::PatternMatchFailed {
                                        pattern: format!("{pattern:?}"),
                                        value: Box::new(first.clone()),
                                    }
                                })?;
                                env.extend(bindings);
                                if !iter.is_empty()
                                    && let Config::Running { frames: f, .. } = config
                                {
                                    f.push(Frame::ForEachIter {
                                        pattern,
                                        items: iter,
                                        body: body.clone(),
                                    });
                                }
                                if let Config::Running {
                                    stmts: s,
                                    frames: f,
                                    ..
                                } = config
                                {
                                    *s = body.stmts;
                                    f.extend(body.frames);
                                }
                                Ok(StepOutcome::Progress)
                            }
                        }
                        _ => Err(ExecError::Eval(EvalError::TypeMismatch {
                            expected: "list".to_string(),
                            actual: format!("{coll_val:?}"),
                        })),
                    }
                }
                Stmt::Spawn {
                    workflow_type,
                    init: _,
                    pattern,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let spawn_expr = ash_core::Expr::Spawn {
                        workflow_type: workflow_type.clone(),
                        init: Box::new(ash_core::Expr::Literal(Value::Null)),
                    };
                    let instance_value = eval_expr(&spawn_expr, &ctx).map_err(ExecError::Eval)?;
                    let bindings = match_pattern(&pattern, &instance_value).map_err(|_| {
                        ExecError::PatternMatchFailed {
                            pattern: format!("{pattern:?}"),
                            value: Box::new(instance_value.clone()),
                        }
                    })?;
                    env.extend(bindings);
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Split { expr, pattern } => {
                    let ctx = Context::with_bindings(env.clone());
                    let split_expr = ash_core::Expr::Split(Box::new(expr));
                    let split_value = eval_expr(&split_expr, &ctx).map_err(ExecError::Eval)?;
                    let bindings = match_pattern(&pattern, &split_value).map_err(|_| {
                        ExecError::PatternMatchFailed {
                            pattern: format!("{pattern:?}"),
                            value: Box::new(split_value.clone()),
                        }
                    })?;
                    env.extend(bindings);
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Kill { target } => {
                    let ctx = Context::with_bindings(env.clone());
                    let link = resolve_control_link(&target, &ctx)?;
                    let control_registry = runtime_state.control_registry();
                    control_registry.lock().await.kill(&link).map_err(|error| {
                        ExecError::InvalidRuntimeState(format!(
                            "kill on control target '{target}' failed: {error}"
                        ))
                    })?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Pause { target } => {
                    let ctx = Context::with_bindings(env.clone());
                    let link = resolve_control_link(&target, &ctx)?;
                    let control_registry = runtime_state.control_registry();
                    control_registry
                        .lock()
                        .await
                        .pause(&link)
                        .map_err(|error| {
                            ExecError::InvalidRuntimeState(format!(
                                "pause on control target '{target}' failed: {error}"
                            ))
                        })?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Resume { target } => {
                    let ctx = Context::with_bindings(env.clone());
                    let link = resolve_control_link(&target, &ctx)?;
                    let control_registry = runtime_state.control_registry();
                    control_registry
                        .lock()
                        .await
                        .resume(&link)
                        .map_err(|error| {
                            ExecError::InvalidRuntimeState(format!(
                                "resume on control target '{target}' failed: {error}"
                            ))
                        })?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::CheckHealth { target } => {
                    let ctx = Context::with_bindings(env.clone());
                    let link = resolve_control_link(&target, &ctx)?;
                    let control_registry = runtime_state.control_registry();
                    control_registry
                        .lock()
                        .await
                        .check_health(&link)
                        .map_err(|error| {
                            ExecError::InvalidRuntimeState(format!(
                                "check_health on control target '{target}' failed: {error}"
                            ))
                        })?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Yield {
                    role,
                    request: _,
                    resume_var: _,
                } => Err(ExecError::Blocked(format!(
                    "workflow yielded to role '{role}' and is awaiting response"
                ))),
                Stmt::Set {
                    capability,
                    channel,
                    value,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let val = eval_expr(&value, &ctx).map_err(ExecError::Eval)?;
                    let capability_policy_eval = CapabilityPolicyEvaluator::new();
                    let actor = active_actor(&ctx);
                    execute_set(
                        &capability,
                        &channel,
                        val,
                        behaviour_ctx,
                        &capability_policy_eval,
                        &actor,
                    )
                    .await?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Send {
                    capability,
                    channel,
                    value,
                } => {
                    let ctx = Context::with_bindings(env.clone());
                    let val = eval_expr(&value, &ctx).map_err(ExecError::Eval)?;
                    let stream_ctx = stream_ctx.ok_or_else(|| {
                        ExecError::ExecutionFailed("Send requires StreamContext".to_string())
                    })?;
                    let capability_policy_eval = CapabilityPolicyEvaluator::new();
                    let actor = active_actor(&ctx);
                    execute_send(
                        &capability,
                        &channel,
                        val,
                        stream_ctx,
                        &capability_policy_eval,
                        &actor,
                    )
                    .await?;
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Oblige { name: _ } => Err(ExecError::ExecutionFailed(
                    "Oblige not supported in the small-step prototype".to_string(),
                )),
                Stmt::CheckObligation { name } => {
                    let ctx = Context::with_bindings(env.clone());
                    let check_expr = ash_core::Expr::CheckObligation {
                        obligation: name,
                        span: ash_core::ast::Span::default(),
                    };
                    let result = eval_expr(&check_expr, &ctx).map_err(ExecError::Eval)?;
                    env.insert("check_result".to_string(), result);
                    if let Config::Running { stmts: s, .. } = config {
                        *s = tail;
                    }
                    Ok(StepOutcome::Progress)
                }
                Stmt::Receive {
                    mode: _,
                    arms: _,
                    control: _,
                } => Err(ExecError::ExecutionFailed(
                    "Receive requires StreamContext in the small-step prototype".to_string(),
                )),
            }
        }
        Config::Returned(value) => Ok(StepOutcome::Returned(value.clone())),
        Config::Rejected(msg) => Ok(StepOutcome::Rejected(msg.clone())),
    }
}

/// Finish the current block with a value, applying frames until
/// the machine finds more work or reaches a terminal configuration.
fn finish_with_value(config: &mut Config, value: Value) -> ExecResult<StepOutcome> {
    loop {
        match apply_frame(config)? {
            Some(rest) => {
                if let Config::Running { stmts, frames, .. } = config {
                    *stmts = rest.stmts;
                    frames.extend(rest.frames);
                    if !stmts.is_empty() {
                        return Ok(StepOutcome::Progress);
                    }
                }
            }
            None => {
                *config = Config::Returned(value.clone());
                return Ok(StepOutcome::Returned(value));
            }
        }
    }
}

/// Run the small-step machine to completion.
///
/// Repeatedly calls `step` until the configuration becomes terminal.
///
/// # Errors
/// Returns `ExecError` if any step fails.
pub async fn run(
    config: &mut Config,
    cap_ctx: &CapabilityContext,
    behaviour_ctx: &BehaviourContext,
    policy_eval: &PolicyEvaluator,
    stream_ctx: Option<&StreamContext>,
    runtime_state: &RuntimeState,
) -> ExecResult<Value> {
    loop {
        match step(
            config,
            cap_ctx,
            behaviour_ctx,
            policy_eval,
            stream_ctx,
            runtime_state,
        )
        .await?
        {
            StepOutcome::Progress => continue,
            StepOutcome::Returned(value) => return Ok(value),
            StepOutcome::Rejected(msg) => return Err(ExecError::ExecutionFailed(msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityContext, MockProvider};
    use ash_core::ast::Span;
    use ash_core::small_step::lower_workflow;
    use ash_core::{BinaryOp, Effect, Expr, Guard, Pattern, Provenance, Value, Workflow};

    fn test_contexts() -> (
        BehaviourContext,
        PolicyEvaluator,
        StreamContext,
        RuntimeState,
    ) {
        (
            BehaviourContext::new(),
            PolicyEvaluator::new(),
            StreamContext::new(),
            RuntimeState::new(),
        )
    }

    #[tokio::test]
    async fn test_done() {
        let workflow = Workflow::Done;
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_ret() {
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_let() {
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: Span::default(),
                },
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn test_seq() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Done),
            second: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[tokio::test]
    async fn test_if_true() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_if_false() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(false)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[tokio::test]
    async fn test_if_with_tail() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::If {
                condition: Expr::Literal(Value::Bool(true)),
                then_branch: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                }),
                else_branch: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(2)),
                }),
            }),
            second: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(3)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_act() {
        let mut cap_ctx = CapabilityContext::new();
        cap_ctx.register(Box::new(
            MockProvider::new("test_provider", Effect::Operational)
                .with_execute_result(Ok(Value::Int(123))),
        ));

        let workflow = Workflow::Act {
            provider_name: "test_provider".to_string(),
            action_name: "do_it".to_string(),
            arguments: vec![],
            guard: Guard::Always,
            provenance: Provenance::new(),
            result_name: Some("result".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "result".to_string(),
                    span: Span::default(),
                },
            }),
        };

        let mut config = lower_workflow(&workflow);
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(123));
    }

    #[tokio::test]
    async fn test_act_guard_failure() {
        let mut cap_ctx = CapabilityContext::new();
        cap_ctx.register(Box::new(
            MockProvider::new("test_provider", Effect::Operational)
                .with_execute_result(Ok(Value::Int(123))),
        ));

        let workflow = Workflow::Act {
            provider_name: "test_provider".to_string(),
            action_name: "do_it".to_string(),
            arguments: vec![],
            guard: Guard::Never,
            provenance: Provenance::new(),
            result_name: Some("result".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "result".to_string(),
                    span: Span::default(),
                },
            }),
        };

        let mut config = lower_workflow(&workflow);
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nested_seq() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Done),
                second: Box::new(Workflow::Done),
            }),
            second: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_let_seq() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "x".to_string(),
                    span: Span::default(),
                },
                expr: Expr::Literal(Value::Int(10)),
                continuation: Box::new(Workflow::Done),
            }),
            second: Box::new(Workflow::Ret {
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Value::Int(5))),
                },
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[tokio::test]
    async fn test_foreach_over_list() {
        let workflow = Workflow::ForEach {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            collection: Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]))),
            body: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "sum".to_string(),
                    span: Span::default(),
                },
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "sum".to_string(),
                        span: Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: Span::default(),
                    }),
                },
                continuation: Box::new(Workflow::Done),
            }),
        };

        let mut config = lower_workflow(&workflow);
        // Pre-seed sum so the accumulation works.
        if let Config::Running { env, .. } = &mut config {
            env.insert("sum".to_string(), Value::Int(0));
        }
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Null);
        // Re-run a simpler ForEach that returns the last element.
        let workflow2 = Workflow::ForEach {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            collection: Expr::Literal(Value::List(Box::new(vec![Value::Int(10), Value::Int(20)]))),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: Span::default(),
                },
            }),
        };
        let mut config2 = lower_workflow(&workflow2);
        let result2 = run(
            &mut config2,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result2, Value::Int(20));
    }

    #[tokio::test]
    async fn test_maybe_fallback_on_error() {
        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Act {
                provider_name: "missing".to_string(),
                action_name: "fail".to_string(),
                arguments: vec![],
                guard: Guard::Always,
                provenance: Provenance::new(),
                result_name: None,
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                }),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
        .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_must_propagates_error_as_must_failure() {
        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Act {
                provider_name: "missing".to_string(),
                action_name: "fail".to_string(),
                arguments: vec![],
                guard: Guard::Always,
                provenance: Provenance::new(),
                result_name: None,
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                }),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;
        assert!(matches!(result, Err(ExecError::MustFailure(_))));
    }

    #[tokio::test]
    async fn test_yield_blocked_state() {
        let workflow = Workflow::Yield {
            role: "proxy".to_string(),
            request: Box::new(Expr::Literal(Value::Int(7))),
            expected_response_type: ash_core::workflow_contract::TypeExpr::Named("Int".to_string()),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
            span: Span::default(),
            resume_var: "response".to_string(),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        let result = step(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;
        assert!(matches!(result, Err(ExecError::Blocked(_))));
    }

    #[tokio::test]
    async fn test_workflow_call_executes_registered_runtime_workflow_in_small_step() {
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
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, _) = test_contexts();

        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;

        assert_eq!(result.unwrap(), Value::Int(99));
    }

    #[tokio::test]
    async fn test_workflow_call_rejects_unknown_runtime_target_in_small_step() {
        let workflow = Workflow::Call {
            target: "missing_worker".to_string(),
            arguments: vec![],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();

        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;

        assert!(matches!(
            result,
            Err(ExecError::ExecutionFailed(message)) if message.contains("missing_worker")
        ));
    }

    #[tokio::test]
    async fn test_workflow_call_rejects_non_zero_arity_in_small_step() {
        let runtime_state = RuntimeState::new();
        runtime_state
            .register_child_workflow(
                "worker",
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(7)),
                },
            )
            .await;

        let workflow = Workflow::Call {
            target: "worker".to_string(),
            arguments: vec![Expr::Literal(Value::Int(1))],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };
        let mut config = lower_workflow(&workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, _) = test_contexts();

        let result = run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await;

        assert!(matches!(
            result,
            Err(ExecError::Eval(EvalError::WrongArity {
                expected: 0,
                actual: 1,
                callee: Some(callee),
            })) if callee == "worker"
        ));
    }

    // ---------------------------------------------------------------
    // Parity tests: run the same workflow through both big-step and
    // small-step interpreters, asserting identical results.
    // ---------------------------------------------------------------

    /// Helper: run a workflow through the big-step interpreter.
    async fn big_step(workflow: &Workflow) -> ExecResult<Value> {
        crate::execute::execute_simple(workflow).await
    }

    /// Helper: run a workflow through the big-step interpreter with a
    /// shared RuntimeState (needed for callable-workflow tests).
    async fn big_step_in_state(workflow: &Workflow, state: &RuntimeState) -> ExecResult<Value> {
        crate::execute::execute_simple_in_state(workflow, state).await
    }

    /// Helper: run a workflow through the small-step interpreter.
    async fn small_step_run(workflow: &Workflow) -> ExecResult<Value> {
        let mut config = lower_workflow(workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, runtime_state) = test_contexts();
        run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            &runtime_state,
        )
        .await
    }

    /// Helper: run a workflow through the small-step interpreter with a
    /// shared RuntimeState (needed for callable-workflow tests).
    async fn small_step_run_in_state(
        workflow: &Workflow,
        runtime_state: &RuntimeState,
    ) -> ExecResult<Value> {
        let mut config = lower_workflow(workflow);
        let cap_ctx = CapabilityContext::new();
        let (behaviour_ctx, policy_eval, stream_ctx, _) = test_contexts();
        run(
            &mut config,
            &cap_ctx,
            &behaviour_ctx,
            &policy_eval,
            Some(&stream_ctx),
            runtime_state,
        )
        .await
    }

    #[tokio::test]
    async fn parity_done() {
        let workflow = Workflow::Done;
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Null);
        assert_eq!(ss, Value::Null);
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_ret() {
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(42));
        assert_eq!(ss, Value::Int(42));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_let_then_ret() {
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            expr: Expr::Literal(Value::Int(5)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: Span::default(),
                },
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(5));
        assert_eq!(ss, Value::Int(5));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_seq() {
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Done),
            second: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(42));
        assert_eq!(ss, Value::Int(42));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_if_true() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(1));
        assert_eq!(ss, Value::Int(1));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_if_false() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(false)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(2));
        assert_eq!(ss, Value::Int(2));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_let_seq() {
        // Let x = 10 in (Seq Done (Ret (x + 5)))
        // The Let binds x, then the continuation is Seq(Done, Ret(x+5))
        // which should see x through the Let's extended scope.
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Done),
                second: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: Span::default(),
                        }),
                        right: Box::new(Expr::Literal(Value::Int(5))),
                    },
                }),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(15));
        assert_eq!(ss, Value::Int(15));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_foreach() {
        // ForEach over [10, 20] where body returns x.
        // The last iteration's Ret(x) should produce 20 because the
        // small-step machine finishes the loop body on each iteration
        // and the final iteration's return wins.
        let workflow = Workflow::ForEach {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            collection: Expr::Literal(Value::List(Box::new(vec![Value::Int(10), Value::Int(20)]))),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: Span::default(),
                },
            }),
        };
        let bs = big_step(&workflow).await;
        let ss = small_step_run(&workflow).await;
        // Both interpreters should produce the same result.
        // ForEach returns the result of the last iteration, or Null for empty.
        assert_eq!(bs.unwrap(), ss.unwrap());
    }

    #[tokio::test]
    async fn parity_maybe_success() {
        // Maybe wraps a simple Ret(1) -- primary succeeds, so fallback is
        // never reached.
        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(1));
        assert_eq!(ss, Value::Int(1));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_must_success() {
        // Must wraps a simple Ret(7) -- body succeeds, so no MustFailure.
        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(7)),
            }),
        };
        let bs = big_step(&workflow).await.unwrap();
        let ss = small_step_run(&workflow).await.unwrap();
        assert_eq!(bs, Value::Int(7));
        assert_eq!(ss, Value::Int(7));
        assert_eq!(bs, ss);
    }

    #[tokio::test]
    async fn parity_must_failure() {
        // Must wraps an Act against a missing provider -- both interpreters
        // should report failure (big-step: ExecutionFailed; small-step:
        // MustFailure).  We check that *both* error out.
        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Act {
                provider_name: "missing".to_string(),
                action_name: "fail".to_string(),
                arguments: vec![],
                guard: Guard::Always,
                provenance: Provenance::new(),
                result_name: None,
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                }),
            }),
        };
        let bs = big_step(&workflow).await;
        let ss = small_step_run(&workflow).await;
        assert!(bs.is_err(), "big-step must failure should error");
        assert!(ss.is_err(), "small-step must failure should error");
    }

    #[tokio::test]
    async fn parity_workflow_call_with_params() {
        // Register a callable "add" that takes (a, b) and returns a + b.
        let runtime_state = RuntimeState::new();
        let add_body = Workflow::Ret {
            expr: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "b".to_string(),
                    span: Span::default(),
                }),
            },
        };
        runtime_state
            .register_callable_workflow("add", add_body, 2, vec!["a".into(), "b".into()])
            .await;

        // Call "add" with (3, 4) and return the result.
        let workflow = Workflow::Call {
            target: "add".to_string(),
            arguments: vec![Expr::Literal(Value::Int(3)), Expr::Literal(Value::Int(4))],
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(99)),
            }),
        };

        let bs = big_step_in_state(&workflow, &runtime_state).await;
        let ss = small_step_run_in_state(&workflow, &runtime_state).await;

        // Both should succeed with the same value (99, because the
        // continuation runs after the call completes).
        let bs_val = bs.unwrap();
        let ss_val = ss.unwrap();
        assert_eq!(
            bs_val, ss_val,
            "big-step and small-step should agree on workflow call result"
        );
    }

    #[tokio::test]
    async fn parity_workflow_call_parent_child_overlapping_names() {
        // Register a callable "worker" that takes param x and returns x.
        // Parent sets x = 10, calls worker(5), then checks x is still 10.
        let runtime_state = RuntimeState::new();
        let worker_body = Workflow::Ret {
            expr: Expr::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
        };
        runtime_state
            .register_callable_workflow("worker", worker_body, 1, vec!["x".into()])
            .await;

        // Let x = 10 in Call worker(5) then Ret(x)
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Call {
                target: "worker".to_string(),
                arguments: vec![Expr::Literal(Value::Int(5))],
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable {
                        name: "x".to_string(),
                        span: Span::default(),
                    },
                }),
            }),
        };

        let bs = big_step_in_state(&workflow, &runtime_state).await;
        let ss = small_step_run_in_state(&workflow, &runtime_state).await;

        // Both should return x = 10 (parent env preserved after child call).
        let bs_val = bs.unwrap();
        let ss_val = ss.unwrap();
        assert_eq!(
            bs_val,
            Value::Int(10),
            "big-step: parent x should be 10 after child call"
        );
        assert_eq!(
            ss_val,
            Value::Int(10),
            "small-step: parent x should be 10 after child call"
        );
        assert_eq!(
            bs_val, ss_val,
            "big-step and small-step should agree on env isolation result"
        );
    }
}
