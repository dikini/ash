//! Expression evaluation
//!
//! Evaluates expressions in a runtime context, producing values.

use ash_core::runtime::{
    CapabilityBindingKind, EffectScopeId, OperationalFailure, ProcessId, ProcessTerminalState,
};
use ash_core::{BinaryOp, Expr, Value, ast::Pattern};
use futures::future::join_all;
use std::collections::HashMap;

use crate::EvalResult;
use crate::context::Context;
use crate::error::EvalError;

mod failure;
use failure::{
    operational_eval_error_for_message, operational_eval_error_for_message_with_attribution,
    operational_eval_error_for_resource_policy, operational_failure_for_payload, value_type_name,
};

mod operators;
use operators::{eval_binary_op, eval_unary_op};

fn call_context_from_env(
    env: std::sync::Arc<ash_core::env_frame::EnvFrame>,
    runtime_ctx: &Context,
    enters_effect_scope: bool,
) -> Context {
    let mut call_ctx = Context::from_env_frame(&env).inherit_runtime_metadata_from(runtime_ctx);
    call_ctx = call_ctx
        .with_admitted_capability_bindings(runtime_ctx.admitted_capability_bindings().to_vec());
    if let Some(policy_evaluator) = runtime_ctx.policy_evaluator() {
        call_ctx = call_ctx.with_policy_evaluator_arc(policy_evaluator);
    }
    if let Some(act_env) = runtime_ctx.act_env() {
        call_ctx = call_ctx.with_act_env_arc(act_env);
    }
    if let Some(runtime_state) = runtime_ctx.runtime_state() {
        call_ctx = call_ctx.with_runtime_state_arc(runtime_state);
    }
    if enters_effect_scope {
        call_ctx = call_ctx.with_effect_scope(EffectScopeId::new());
    }
    call_ctx
}

fn observe_terminal_process_sync(
    handle: &ash_core::ProcessHandle,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    if !handle.try_consume() {
        return Err(EvalError::ProcessHandleConsumed {
            process_id: handle.process_id,
        });
    }

    let runtime_state =
        runtime_ctx
            .runtime_state()
            .ok_or_else(|| EvalError::ProcessObservationUnavailable {
                process_id: handle.process_id,
                reason: "missing hidden runtime state".to_string(),
            })?;

    let terminal_state = runtime_state.blocking_process_terminal_state(handle.process_id);
    observe_terminal_state(handle.process_id, terminal_state)
}

async fn observe_terminal_process_async(
    handle: &ash_core::ProcessHandle,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    if !handle.try_consume() {
        return Err(EvalError::ProcessHandleConsumed {
            process_id: handle.process_id,
        });
    }

    let runtime_state =
        runtime_ctx
            .runtime_state()
            .ok_or_else(|| EvalError::ProcessObservationUnavailable {
                process_id: handle.process_id,
                reason: "missing hidden runtime state".to_string(),
            })?;

    let terminal_state = runtime_state
        .process_terminal_state(handle.process_id)
        .await;
    observe_terminal_state(handle.process_id, terminal_state)
}

async fn wait_for_terminal_process_async(
    process_id: ash_core::ProcessId,
    runtime_ctx: &Context,
) -> EvalResult<ProcessTerminalState> {
    let runtime_state =
        runtime_ctx
            .runtime_state()
            .ok_or_else(|| EvalError::ProcessObservationUnavailable {
                process_id,
                reason: "missing hidden runtime state".to_string(),
            })?;

    runtime_state
        .wait_for_process_terminal_state(process_id)
        .await
        .ok_or_else(|| EvalError::ProcessObservationUnavailable {
            process_id,
            reason: "process is not in a retained terminal state".to_string(),
        })
}

fn aggregate_wait_all_failure(
    runtime_ctx: &Context,
    observer_name: &str,
    failures: Vec<(ProcessId, Box<OperationalFailure>)>,
) -> EvalError {
    debug_assert!(!failures.is_empty());
    if failures.len() == 1 {
        let (_process_id, failure) = failures.into_iter().next().expect("single failure");
        return EvalError::OperationalFailure(failure);
    }

    let mut chained_failure: Option<Box<OperationalFailure>> = None;
    for (_process_id, failure) in failures.into_iter().rev() {
        chained_failure = Some(match chained_failure {
            Some(next_failure) => {
                preserve_caught_failure_as_tail_cause(failure, next_failure.as_ref())
            }
            None => failure,
        });
    }

    let mut aggregate = operational_failure_for_payload(
        Value::String(format!(
            "proc::{observer_name} observed one or more child failures"
        )),
        runtime_ctx,
    );
    aggregate.cause = chained_failure;
    EvalError::OperationalFailure(Box::new(aggregate))
}

async fn observe_terminal_processes_wait_all_async(
    handles: &[ash_core::ProcessHandle],
    runtime_ctx: &Context,
    observer_name: &'static str,
) -> EvalResult<Vec<Value>> {
    for handle in handles {
        if !handle.try_consume() {
            return Err(EvalError::ProcessHandleConsumed {
                process_id: handle.process_id,
            });
        }
    }

    let terminal_states = join_all(
        handles
            .iter()
            .map(|handle| wait_for_terminal_process_async(handle.process_id, runtime_ctx)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    let mut successes = Vec::with_capacity(terminal_states.len());
    let mut failures = Vec::new();
    for (handle, terminal_state) in handles.iter().zip(terminal_states) {
        match terminal_state {
            ProcessTerminalState::Succeeded { value } => successes.push(value),
            ProcessTerminalState::Failed { failure, .. }
            | ProcessTerminalState::Cancelled { failure, .. } => {
                failures.push((handle.process_id, failure));
            }
        }
    }

    if failures.is_empty() {
        if let Some(parent_identity) = runtime_ctx.process_identity() {
            let child_process_ids = handles
                .iter()
                .map(|handle| handle.process_id)
                .collect::<Vec<_>>();
            if let Some(runtime_state) = runtime_ctx.runtime_state() {
                runtime_state
                    .apply_process_resource_join(
                        parent_identity.process_id,
                        &child_process_ids,
                        observer_name,
                    )
                    .await
                    .map_err(|violation| {
                        operational_eval_error_for_resource_policy(violation, runtime_ctx)
                    })?;
            }
        }
        Ok(successes)
    } else {
        Err(aggregate_wait_all_failure(
            runtime_ctx,
            observer_name,
            failures,
        ))
    }
}

fn observe_terminal_state(
    process_id: ash_core::ProcessId,
    terminal_state: Option<ash_core::runtime::ProcessTerminalState>,
) -> EvalResult<Value> {
    match terminal_state {
        Some(ash_core::runtime::ProcessTerminalState::Succeeded { value }) => Ok(value),
        Some(ash_core::runtime::ProcessTerminalState::Failed { failure, .. })
        | Some(ash_core::runtime::ProcessTerminalState::Cancelled { failure, .. }) => {
            Err(EvalError::OperationalFailure(failure))
        }
        None => Err(EvalError::ProcessObservationUnavailable {
            process_id,
            reason: "process is not in a retained terminal state".to_string(),
        }),
    }
}

fn maybe_execute_proc_await_capture(value: Value, runtime_ctx: &Context) -> EvalResult<Value> {
    match value {
        Value::ProcAwaitCapture(handle) => observe_terminal_process_sync(&handle, runtime_ctx),
        other => Ok(other),
    }
}

fn maybe_execute_proc_yield_capture(value: Value, _runtime_ctx: &Context) -> EvalResult<Value> {
    match value {
        Value::ProcYieldCapture => Err(EvalError::ProcYieldRequiresAsyncRuntime),
        Value::ProcParCapture { .. }
        | Value::ProcScatterCapture { .. }
        | Value::ProcJoinCapture { .. }
        | Value::ProcGatherCapture { .. } => Err(EvalError::ProcAdmissionRequiresAsyncRuntime),
        other => Ok(other),
    }
}

async fn maybe_execute_proc_await_capture_async(
    value: Value,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    match value {
        Value::ProcAwaitCapture(handle) => {
            observe_terminal_process_async(&handle, runtime_ctx).await
        }
        other => Ok(other),
    }
}

async fn maybe_execute_proc_yield_capture_async(
    value: Value,
    _runtime_ctx: &Context,
) -> EvalResult<Value> {
    match value {
        Value::ProcYieldCapture => {
            tokio::task::yield_now().await;
            Ok(Value::Null)
        }
        other => Ok(other),
    }
}

fn ensure_proc_closure(value: &Value) -> EvalResult<()> {
    match value {
        Value::Closure { params, .. } if params.len() == 1 && params[0].0 == "__proc_env" => Ok(()),
        other => Err(EvalError::TypeMismatch {
            expected: "Proc<A>".to_string(),
            actual: value_type_name(other).to_string(),
        }),
    }
}

async fn spawn_proc_child_runner(
    runtime_state: std::sync::Arc<crate::runtime_state::RuntimeState>,
    proc_value: Value,
    child_ctx: Context,
    process_id: ProcessId,
) {
    let result = apply_closure_async_value(proc_value, vec![Value::Null], &child_ctx).await;
    let terminal_state = match result {
        Ok(value) => ProcessTerminalState::Succeeded { value },
        Err(EvalError::OperationalFailure(failure)) => ProcessTerminalState::Failed {
            process_id,
            failure,
        },
        Err(error) => ProcessTerminalState::Failed {
            process_id,
            failure: Box::new(operational_failure_for_payload(
                Value::String(error.to_string()),
                &child_ctx,
            )),
        },
    };

    if let Err(error) = runtime_state
        .record_process_terminal(process_id, terminal_state)
        .await
    {
        eprintln!(
            "proc child terminal process state recording failed unexpectedly for process {process_id:?}: {error}"
        );
    }
}

async fn admit_proc_children_async(
    parent_ctx: &Context,
    child_procs: Vec<Value>,
    operation_name: &'static str,
) -> EvalResult<Vec<ash_core::ProcessHandle>> {
    let runtime_state = parent_ctx
        .runtime_state()
        .ok_or_else(|| EvalError::ExecutionFailed("missing hidden runtime state".to_string()))?;
    let parent_identity = parent_ctx.process_identity().ok_or_else(|| {
        EvalError::ExecutionFailed(
            "proc child admission requires process identity context".to_string(),
        )
    })?;

    for proc_value in &child_procs {
        ensure_proc_closure(proc_value)?;
    }

    struct ChildPlan {
        process_id: ProcessId,
        child_index: usize,
        proc_value: Value,
        child_ctx: Context,
    }

    let child_count = child_procs.len();
    let base_child_index = runtime_state
        .process_children(parent_identity.process_id)
        .await
        .len();
    let mut plans = Vec::with_capacity(child_procs.len());
    for (offset, proc_value) in child_procs.into_iter().enumerate() {
        let process_id = ProcessId::new();
        let child_index = base_child_index + offset;
        let projection = crate::process_env::ChildEnvProjection::new(process_id, child_index)
            .with_parent_process_id(parent_identity.process_id);
        let child_ctx = crate::derive_child_env(parent_ctx, projection).map_err(|error| {
            EvalError::ExecutionFailed(format!(
                "failed to project child process environment for {process_id:?}: {error}"
            ))
        })?;
        plans.push(ChildPlan {
            process_id,
            child_index,
            proc_value,
            child_ctx,
        });
    }

    runtime_state
        .apply_process_resource_split(parent_identity.process_id, child_count, operation_name)
        .await
        .map_err(|violation| operational_eval_error_for_resource_policy(violation, parent_ctx))?;

    runtime_state
        .register_child_processes_batch(
            parent_identity.process_id,
            plans
                .iter()
                .map(|plan| (plan.process_id, plan.child_index))
                .collect(),
        )
        .await
        .map_err(|error| {
            EvalError::ExecutionFailed(format!(
                "failed to register child processes below {:?}: {error}",
                parent_identity.process_id
            ))
        })?;

    let handles = plans
        .iter()
        .map(|plan| ash_core::ProcessHandle::new(plan.process_id, None))
        .collect::<Vec<_>>();

    for plan in plans {
        runtime_state
            .mark_process_running(plan.process_id)
            .await
            .map_err(|error| {
                EvalError::ExecutionFailed(format!(
                    "failed to mark process {:?} running: {error}",
                    plan.process_id
                ))
            })?;

        let runtime_state = runtime_state.clone();
        tokio::spawn(async move {
            spawn_proc_child_runner(
                runtime_state,
                plan.proc_value,
                plan.child_ctx,
                plan.process_id,
            )
            .await;
        });
    }

    Ok(handles)
}

async fn maybe_execute_proc_admission_capture_async(
    value: Value,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    match value {
        Value::ProcParCapture { left, right } => {
            let handles =
                admit_proc_children_async(runtime_ctx, vec![*left, *right], "par").await?;
            Ok(Value::List(Box::new(
                handles.into_iter().map(Value::ProcessHandle).collect(),
            )))
        }
        Value::ProcScatterCapture { items, mapper } => {
            let mut child_procs = Vec::with_capacity(items.len());
            for item in *items {
                let mapped =
                    apply_closure_async_value((*mapper).clone(), vec![item], runtime_ctx).await?;
                ensure_proc_closure(&mapped)?;
                child_procs.push(mapped);
            }
            let handles = admit_proc_children_async(runtime_ctx, child_procs, "scatter").await?;
            Ok(Value::List(Box::new(
                handles.into_iter().map(Value::ProcessHandle).collect(),
            )))
        }
        other => Ok(other),
    }
}

async fn maybe_execute_proc_wait_all_capture_async(
    value: Value,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    match value {
        Value::ProcJoinCapture { left, right } => {
            let values =
                observe_terminal_processes_wait_all_async(&[left, right], runtime_ctx, "join")
                    .await?;
            let mut fields = HashMap::with_capacity(2);
            fields.insert("_0".to_string(), values[0].clone());
            fields.insert("_1".to_string(), values[1].clone());
            Ok(Value::Record(Box::new(fields)))
        }
        Value::ProcGatherCapture { handles } => {
            let values =
                observe_terminal_processes_wait_all_async(&handles, runtime_ctx, "gather").await?;
            Ok(Value::List(Box::new(values)))
        }
        other => Ok(other),
    }
}

fn preserve_caught_failure_as_tail_cause(
    mut raised: Box<OperationalFailure>,
    caught: &OperationalFailure,
) -> Box<OperationalFailure> {
    let mut cursor = &mut raised;
    while let Some(ref mut cause) = cursor.cause {
        cursor = cause;
    }
    cursor.cause = Some(Box::new(caught.clone()));
    raised
}

// ── Builtin dispatch table (TASK-621) ────────────────────────────

/// Metadata for a builtin function entry in the dispatch table.
mod builtins;
pub use builtins::{BuiltinEntry, builtin_dispatch_table, is_known_builtin};

/// Build a qualified builtin name from its components.
fn qualified_builtin_name(func: &str, module: Option<&str>) -> String {
    match module {
        Some(m) => format!("{m}::{func}"),
        None => func.to_string(),
    }
}

/// Dispatch a builtin call by qualified name.
///
/// Returns `Some(result)` if the qualified name exists in the dispatch table
/// (either as implemented or forward-declared), `None` otherwise.
/// Forward-declared but unimplemented builtins produce
/// [`EvalError::UnimplementedBuiltin`].
pub fn dispatch_builtin(
    qualified_name: &str,
    args: &[Value],
    ctx: &Context,
) -> Option<EvalResult<Value>> {
    let table = builtin_dispatch_table();
    let entry = table.get(qualified_name)?;

    if !entry.implemented {
        return Some(Err(EvalError::UnimplementedBuiltin {
            name: qualified_name.to_string(),
        }));
    }

    // Arity enforcement: reject if argument count doesn't match unless variadic.
    if !entry.variadic && args.len() != entry.arity {
        return Some(Err(EvalError::WrongArity {
            expected: entry.arity,
            actual: args.len(),
            callee: Some(qualified_name.to_string()),
        }));
    }

    // Split qualified name into (module, func) for eval_function_call
    let (module, func) = match qualified_name.rsplit_once("::") {
        Some((m, f)) => (Some(m), f),
        None => (None, qualified_name),
    };

    Some(eval_function_call(func, module, args, ctx))
}

// ── End builtin dispatch table ───────────────────────────────────

/// Runtime primitive for expression-level `invoke(...)`.
///
/// Phase 97 keeps this as a closure-shaped value so the runtime can thread an
/// internal Act-style environment without adding a new AST node.
fn runtime_invoke(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 3 {
        return Err(EvalError::WrongArity {
            expected: 3,
            actual: args.len(),
            callee: Some("invoke".to_string()),
        });
    }

    let provider = match &args[0] {
        Value::String(s) => s.clone(),
        other => {
            return Err(EvalError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{other:?}"),
            });
        }
    };

    let action = match &args[1] {
        Value::String(s) => s.clone(),
        other => {
            return Err(EvalError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{other:?}"),
            });
        }
    };

    let invoke_args = match &args[2] {
        Value::List(items) => (*items).clone(),
        other => {
            return Err(EvalError::TypeMismatch {
                expected: "list".to_string(),
                actual: format!("{other:?}"),
            });
        }
    };

    let body = Expr::Literal(act_result(Value::Variant {
        name: "__InvokeCapture".to_string(),
        fields: Box::new(vec![
            ("provider".to_string(), Value::String(provider)),
            ("action".to_string(), Value::String(action)),
            ("args".to_string(), Value::List(invoke_args)),
        ]),
    }));

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(body),
        env: ctx.to_env_frame(),
    })
}

fn act_result(value: Value) -> Value {
    Value::List(Box::new(vec![Value::ActEnvToken, value]))
}

fn matches_normalized_act_result(value: &Value) -> bool {
    let Value::List(items) = value else {
        return false;
    };
    items.len() == 2 && matches!(items[0], Value::ActEnvToken)
}

fn runtime_proc_unit(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("proc::unit".to_string()),
        });
    }

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(args[0].clone())),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_from_act(args: &[Value], _ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("proc::from_act".to_string()),
        });
    }

    let span = ash_core::ast::Span::default();
    let body = Expr::Let {
        pattern: Pattern::Variable {
            name: "__proc_from_act_result".to_string(),
            span,
        },
        expr: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__proc_from_act".to_string(),
                span,
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        }),
        body: Box::new(Expr::IndexAccess {
            expr: Box::new(Expr::Variable {
                name: "__proc_from_act_result".to_string(),
                span,
            }),
            index: Box::new(Expr::Literal(Value::Int(1))),
        }),
        span,
    };

    let mut frame = ash_core::env_frame::EnvFrame::new();
    frame.insert("__proc_from_act".to_string(), args[0].clone());

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_proc_bind(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("proc::bind".to_string()),
        });
    }

    let mut frame = ash_core::env_frame::EnvFrame::with_parent(ctx.to_env_frame());
    frame.insert("__proc_bind_left".to_string(), args[0].clone());
    frame.insert("__proc_bind_cont".to_string(), args[1].clone());

    let span = ash_core::ast::Span::default();
    let proc_env = Expr::Variable {
        name: "__proc_env".to_string(),
        span,
    };
    let bind_value = Expr::Variable {
        name: "__proc_bind_value".to_string(),
        span,
    };
    let next_proc = Expr::Variable {
        name: "__proc_bind_next".to_string(),
        span,
    };

    let body = Expr::Let {
        pattern: Pattern::Variable {
            name: "__proc_bind_value".to_string(),
            span,
        },
        expr: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__proc_bind_left".to_string(),
                span,
            }),
            args: vec![proc_env.clone()],
        }),
        body: Box::new(Expr::Let {
            pattern: Pattern::Variable {
                name: "__proc_bind_next".to_string(),
                span,
            },
            expr: Box::new(Expr::FnApply {
                func: Box::new(Expr::Variable {
                    name: "__proc_bind_cont".to_string(),
                    span,
                }),
                args: vec![bind_value],
            }),
            body: Box::new(Expr::FnApply {
                func: Box::new(next_proc),
                args: vec![proc_env],
            }),
            span,
        }),
        span,
    };

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_proc_then(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("proc::then".to_string()),
        });
    }

    let mut frame = ash_core::env_frame::EnvFrame::with_parent(ctx.to_env_frame());
    frame.insert("__proc_then_left".to_string(), args[0].clone());
    frame.insert("__proc_then_right".to_string(), args[1].clone());

    let span = ash_core::ast::Span::default();
    let proc_env = Expr::Variable {
        name: "__proc_env".to_string(),
        span,
    };

    let body = Expr::Let {
        pattern: Pattern::Variable {
            name: "__proc_then_ignored".to_string(),
            span,
        },
        expr: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__proc_then_left".to_string(),
                span,
            }),
            args: vec![proc_env.clone()],
        }),
        body: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__proc_then_right".to_string(),
                span,
            }),
            args: vec![proc_env],
        }),
        span,
    };

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_proc_await(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("proc::await".to_string()),
        });
    }

    let Value::ProcessHandle(handle) = &args[0] else {
        return Err(EvalError::TypeMismatch {
            expected: "P<A>".to_string(),
            actual: value_type_name(&args[0]).to_string(),
        });
    };

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcAwaitCapture(handle.clone()))),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_yield(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if !args.is_empty() {
        return Err(EvalError::WrongArity {
            expected: 0,
            actual: args.len(),
            callee: Some("proc::yield".to_string()),
        });
    }

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcYieldCapture)),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_par(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("proc::par".to_string()),
        });
    }
    ensure_proc_closure(&args[0])?;
    ensure_proc_closure(&args[1])?;

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcParCapture {
            left: Box::new(args[0].clone()),
            right: Box::new(args[1].clone()),
        })),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_scatter(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("proc::scatter".to_string()),
        });
    }
    let items = match &args[0] {
        Value::List(items) => (**items).clone(),
        other => {
            // Try to convert Cons/Nil variant to a Vec
            match crate::list_helpers::list_to_vec(other) {
                Some(items) => items,
                None => {
                    return Err(EvalError::TypeMismatch {
                        expected: "List<A>".to_string(),
                        actual: value_type_name(other).to_string(),
                    });
                }
            }
        }
    };
    if !matches!(&args[1], Value::Closure { .. }) {
        return Err(EvalError::TypeMismatch {
            expected: "A -> Proc<B>".to_string(),
            actual: value_type_name(&args[1]).to_string(),
        });
    }

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcScatterCapture {
            items: Box::new(items),
            mapper: Box::new(args[1].clone()),
        })),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_join(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("proc::join".to_string()),
        });
    }
    let Value::ProcessHandle(left) = &args[0] else {
        return Err(EvalError::TypeMismatch {
            expected: "P<A>".to_string(),
            actual: value_type_name(&args[0]).to_string(),
        });
    };
    let Value::ProcessHandle(right) = &args[1] else {
        return Err(EvalError::TypeMismatch {
            expected: "P<B>".to_string(),
            actual: value_type_name(&args[1]).to_string(),
        });
    };

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcJoinCapture {
            left: left.clone(),
            right: right.clone(),
        })),
        env: ctx.to_env_frame(),
    })
}

fn runtime_proc_gather(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("proc::gather".to_string()),
        });
    }
    let Value::List(handles) = &args[0] else {
        return Err(EvalError::TypeMismatch {
            expected: "List<P<A>>".to_string(),
            actual: value_type_name(&args[0]).to_string(),
        });
    };
    let mut gathered_handles = Vec::with_capacity(handles.len());
    for handle in handles.iter() {
        let Value::ProcessHandle(handle) = handle else {
            return Err(EvalError::TypeMismatch {
                expected: "List<P<A>>".to_string(),
                actual: value_type_name(handle).to_string(),
            });
        };
        gathered_handles.push(handle.clone());
    }

    Ok(Value::Closure {
        params: vec![("__proc_env".to_string(), None)],
        body: Box::new(Expr::Literal(Value::ProcGatherCapture {
            handles: Box::new(gathered_handles),
        })),
        env: ctx.to_env_frame(),
    })
}

fn maybe_execute_invoke_capture(value: Value, runtime_ctx: &Context) -> EvalResult<Value> {
    let Value::List(items) = value else {
        return Ok(value);
    };
    if items.len() != 2 || !matches!(items[0], Value::ActEnvToken) {
        return Ok(Value::List(items));
    }

    let Value::Variant { name, fields } = &items[1] else {
        return Ok(Value::List(items));
    };
    if name != "__InvokeCapture" {
        return Ok(Value::List(items));
    }

    let provider = fields
        .iter()
        .find(|(field, _)| field == "provider")
        .and_then(|(_, value)| match value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing string provider".to_string())
        })?;
    let action = fields
        .iter()
        .find(|(field, _)| field == "action")
        .and_then(|(_, value)| match value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing string action".to_string())
        })?;
    let args = fields
        .iter()
        .find(|(field, _)| field == "args")
        .and_then(|(_, value)| match value {
            Value::List(items) => Some((**items).clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing list args".to_string())
        })?;

    let act_env = runtime_ctx.act_env().ok_or_else(|| {
        EvalError::ExecutionFailed("invoke capture missing hidden runtime ActEnv".to_string())
    })?;
    let runtime_state = runtime_ctx.runtime_state();
    let admitted_bindings = runtime_ctx.admitted_capability_bindings().to_vec();
    let (failure_tower, failure_entity) = runtime_ctx.current_failure_attribution();

    let invoked = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                EvalError::ExecutionFailed(format!("invoke helper runtime build failed: {err}"))
            })?;
        runtime.block_on(async move {
            let binding = if let Some(runtime_state) = runtime_state.as_ref() {
                runtime_state.capability_binding_by_name(&provider).await
            } else {
                None
            };
            let binding_admitted = binding
                .as_ref()
                .map(|binding| admitted_bindings.contains(&binding.id));
            let binding_exists = binding.is_some();
            let provider_registered = runtime_state.as_ref().is_some_and(|runtime_state| {
                runtime_state.has_provider(&provider)
                    || matches!(binding_admitted, Some(false))
                    || binding_exists
            });
            if binding_admitted == Some(true)
                && let Some(runtime_state) = runtime_state.as_ref()
                && let Some(CapabilityBindingKind::HostProvider { .. }) =
                    binding.as_ref().map(|binding| &binding.kind)
            {
                let projected_ctx = runtime_state
                    .create_capability_context_for_bindings(&[binding.as_ref().expect("binding checked").id])
                    .await
                    .map_err(|err| EvalError::ExecutionFailed(err.to_string()))?;
                return projected_ctx
                    .execute(&provider, &action, &args)
                    .await
                    .map_err(|err| {
                        operational_eval_error_for_message_with_attribution(
                            err.to_string(),
                            failure_tower,
                            failure_entity,
                        )
                    });
            }
            let ambient_runtime_authorized = act_env.has_runtime_state_ambient_authority()
                && admitted_bindings.is_empty()
                && binding_admitted.is_none();
            let fallback_authorized = binding_admitted == Some(true) || ambient_runtime_authorized;
            let invoke_result = act_env.capability_ctx.execute(&provider, &action, &args).await;
            match invoke_result {
                Ok(value) if fallback_authorized => Ok(value),
                Ok(_) => Err(operational_eval_error_for_message_with_attribution(
                    format!(
                        "authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
                    ),
                    failure_tower,
                    failure_entity,
                )),
                Err(err) if !fallback_authorized && binding_admitted.is_none() && !provider_registered => {
                    Err(operational_eval_error_for_message_with_attribution(
                        err.to_string(),
                        failure_tower,
                        failure_entity,
                    ))
                }
                Err(err) if fallback_authorized => Err(operational_eval_error_for_message_with_attribution(
                    err.to_string(),
                    failure_tower,
                    failure_entity,
                )),
                Err(_) => Err(operational_eval_error_for_message_with_attribution(
                    format!(
                        "authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
                    ),
                    failure_tower,
                    failure_entity,
                )),
            }
        })
    })
    .join()
    .map_err(|_| EvalError::ExecutionFailed("invoke dispatch thread panicked".to_string()))??;

    Ok(Value::List(Box::new(vec![Value::ActEnvToken, invoked])))
}

fn runtime_unit(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("unit".to_string()),
        });
    }

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(Expr::Literal(act_result(args[0].clone()))),
        env: ctx.to_env_frame(),
    })
}

/// Sequence two Act-shaped closures left-to-right.
fn runtime_bind(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("bind".to_string()),
        });
    }

    let mut frame = ash_core::env_frame::EnvFrame::with_parent(ctx.to_env_frame());
    frame.insert("__bind_act".to_string(), args[0].clone());
    frame.insert("__bind_cont".to_string(), args[1].clone());

    let span = ash_core::ast::Span::default();
    let act_env = Expr::Variable {
        name: "__act_env".to_string(),
        span,
    };
    let bind_pair = Expr::Variable {
        name: "__bind_pair".to_string(),
        span,
    };
    let bind_value = Expr::Variable {
        name: "__bind_value".to_string(),
        span,
    };
    let next_act = Expr::Variable {
        name: "__bind_next_act".to_string(),
        span,
    };

    let body = Expr::Let {
        pattern: ash_core::ast::Pattern::Variable {
            name: "__bind_pair".to_string(),
            span,
        },
        expr: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__bind_act".to_string(),
                span,
            }),
            args: vec![act_env.clone()],
        }),
        body: Box::new(Expr::Let {
            pattern: ash_core::ast::Pattern::Variable {
                name: "__bind_value".to_string(),
                span,
            },
            expr: Box::new(Expr::IndexAccess {
                expr: Box::new(bind_pair.clone()),
                index: Box::new(Expr::Literal(Value::Int(1))),
            }),
            body: Box::new(Expr::Let {
                pattern: ash_core::ast::Pattern::Variable {
                    name: "__bind_next_act".to_string(),
                    span,
                },
                expr: Box::new(Expr::FnApply {
                    func: Box::new(Expr::Variable {
                        name: "__bind_cont".to_string(),
                        span,
                    }),
                    args: vec![bind_value],
                }),
                body: Box::new(Expr::FnApply {
                    func: Box::new(next_act),
                    args: vec![Expr::IndexAccess {
                        expr: Box::new(bind_pair),
                        index: Box::new(Expr::Literal(Value::Int(0))),
                    }],
                }),
                span,
            }),
            span,
        }),
        span,
    };

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_then(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("then".to_string()),
        });
    }

    let mut frame = ash_core::env_frame::EnvFrame::with_parent(ctx.to_env_frame());
    frame.insert("__then_left".to_string(), args[0].clone());
    frame.insert("__then_right".to_string(), args[1].clone());

    let span = ash_core::ast::Span::default();
    let act_env = Expr::Variable {
        name: "__act_env".to_string(),
        span,
    };
    let then_pair = Expr::Variable {
        name: "__then_pair".to_string(),
        span,
    };

    let body = Expr::Let {
        pattern: ash_core::ast::Pattern::Variable {
            name: "__then_pair".to_string(),
            span,
        },
        expr: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__then_left".to_string(),
                span,
            }),
            args: vec![act_env],
        }),
        body: Box::new(Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "__then_right".to_string(),
                span,
            }),
            args: vec![Expr::IndexAccess {
                expr: Box::new(then_pair),
                index: Box::new(Expr::Literal(Value::Int(0))),
            }],
        }),
        span,
    };

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_fail(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("fail".to_string()),
        });
    }

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(Expr::Literal(act_result(args[0].clone()))),
        env: ctx.to_env_frame(),
    })
}

fn runtime_guard(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("guard".to_string()),
        });
    }

    let mut frame = ash_core::env_frame::EnvFrame::with_parent(ctx.to_env_frame());
    frame.insert("__guard_policy".to_string(), args[0].clone());
    frame.insert("__guard_act".to_string(), args[1].clone());

    let span = ash_core::ast::Span::default();
    let act_env = Expr::Variable {
        name: "__act_env".to_string(),
        span,
    };
    let body = Expr::Match {
        scrutinee: Box::new(Expr::Call {
            func: "policy_check".to_string(),
            module: Some("act".to_string()),
            arguments: vec![Expr::Variable {
                name: "__guard_policy".to_string(),
                span,
            }],
        }),
        arms: vec![
            ash_core::MatchArm {
                pattern: ash_core::ast::Pattern::Literal(Value::Bool(true)),
                body: Expr::FnApply {
                    func: Box::new(Expr::Variable {
                        name: "__guard_act".to_string(),
                        span,
                    }),
                    args: vec![act_env],
                },
            },
            ash_core::MatchArm {
                pattern: ash_core::ast::Pattern::Literal(Value::Bool(false)),
                body: Expr::Literal(act_result(Value::String("policy denied".to_string()))),
            },
        ],
    };

    Ok(Value::Closure {
        params: vec![("__act_env".to_string(), None)],
        body: Box::new(body),
        env: std::sync::Arc::new(frame),
    })
}

fn runtime_policy_check(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("policy_check".to_string()),
        });
    }

    let policy_name = match &args[0] {
        Value::String(s) => s.clone(),
        other => {
            return Err(EvalError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{other:?}"),
            });
        }
    };

    let Some(policy_evaluator) = ctx.policy_evaluator() else {
        return Ok(Value::Bool(false));
    };

    let permitted = policy_evaluator
        .evaluate(&policy_name, ctx)
        .map(crate::policy::PolicyResult::from)
        .map(|result| matches!(result, crate::policy::PolicyResult::Allow))
        .unwrap_or(false);

    Ok(Value::Bool(permitted))
}

fn runtime_result_and_then(args: &[Value], ctx: &Context) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArity {
            expected: 2,
            actual: args.len(),
            callee: Some("result::and_then".to_string()),
        });
    }

    match &args[0] {
        Value::Variant { name, fields } if name == "Ok" => {
            let value = fields
                .iter()
                .find(|(field, _)| field == "value")
                .map(|(_, value)| value.clone())
                .ok_or_else(|| EvalError::FieldNotFound {
                    field: "value".to_string(),
                    value: Box::new(args[0].clone()),
                })?;
            match &args[1] {
                Value::Closure { params, body, env } => {
                    apply_closure(params, body, env, vec![value], ctx)
                }
                other => Err(EvalError::NotCallable {
                    value: Box::new(other.clone()),
                }),
            }
        }
        Value::Variant { name, .. } if name == "Err" => Ok(args[0].clone()),
        other => Err(EvalError::TypeMismatch {
            expected: "Result".to_string(),
            actual: format!("{other:?}"),
        }),
    }
}

fn runtime_result_ok(args: &[Value]) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("Ok".to_string()),
        });
    }

    Ok(Value::variant("Ok", vec![("value", args[0].clone())]))
}

fn runtime_result_err(args: &[Value]) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArity {
            expected: 1,
            actual: args.len(),
            callee: Some("Err".to_string()),
        });
    }

    Ok(Value::variant("Err", vec![("error", args[0].clone())]))
}

/// Evaluate an expression in the given context
///
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `ctx` - The runtime context with variable bindings
///
/// # Returns
/// The evaluated value or an error
///
/// # Examples
/// ```
/// use ash_core::{Expr, Value};
/// use ash_interp::context::Context;
/// use ash_interp::eval::eval_expr;
///
/// let ctx = Context::new();
/// let expr = Expr::Literal(Value::Int(42));
/// let value = eval_expr(&expr, &ctx).unwrap();
/// assert_eq!(value, Value::Int(42));
/// ```
type EvalBoxFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = EvalResult<Value>> + Send + 'a>>;

fn eval_expr_force_async<'a>(expr: &'a Expr, ctx: &'a Context) -> EvalBoxFuture<'a> {
    Box::pin(async move {
        match expr {
            Expr::Literal(value) => {
                let value = maybe_execute_invoke_capture_async(value.clone(), ctx).await?;
                let value = maybe_execute_proc_await_capture_async(value, ctx).await?;
                let value = maybe_execute_proc_yield_capture_async(value, ctx).await?;
                let value = maybe_execute_proc_admission_capture_async(value, ctx).await?;
                maybe_execute_proc_wait_all_capture_async(value, ctx).await
            }
            Expr::Variable { name, .. } => ctx
                .get(name)
                .cloned()
                .or_else(|| {
                    if name == "()" {
                        Some(Value::Null)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),
            Expr::FieldAccess { expr, field } => {
                let value = eval_expr_force_async(expr, ctx).await?;
                match value {
                    Value::Record(mut fields) => {
                        fields
                            .remove(field)
                            .ok_or_else(|| EvalError::FieldNotFound {
                                field: field.clone(),
                                value: Box::new(Value::Record(fields)),
                            })
                    }
                    Value::List(items) => {
                        let idx = field
                            .parse::<usize>()
                            .map_err(|_| EvalError::TypeMismatch {
                                expected: "record".to_string(),
                                actual: format!("{:?}", Value::List(items.clone())),
                            })?;
                        items.get(idx).cloned().ok_or(EvalError::IndexOutOfBounds {
                            index: idx as i64,
                            len: items.len(),
                        })
                    }
                    _ => Err(EvalError::TypeMismatch {
                        expected: "record".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
            Expr::IndexAccess { expr, index } => {
                let collection = eval_expr_force_async(expr, ctx).await?;
                let idx_val = eval_expr_force_async(index, ctx).await?;
                match idx_val {
                    Value::Int(i) => {
                        let idx = i as usize;
                        match collection {
                            Value::List(list) => {
                                list.get(idx).cloned().ok_or(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: list.len(),
                                })
                            }
                            Value::String(s) => s
                                .chars()
                                .nth(idx)
                                .map(|c| Value::String(c.to_string()))
                                .ok_or(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: s.len(),
                                }),
                            _ => Err(EvalError::TypeMismatch {
                                expected: "list or string".to_string(),
                                actual: format!("{:?}", collection),
                            }),
                        }
                    }
                    _ => Err(EvalError::InvalidIndexType(format!("{:?}", idx_val))),
                }
            }
            Expr::Unary { op, expr } => {
                let value = eval_expr_force_async(expr, ctx).await?;
                eval_unary_op(*op, value)
            }
            Expr::Binary { op, left, right } => match op {
                BinaryOp::And => {
                    let left_val = eval_expr_force_async(left, ctx).await?;
                    match left_val {
                        Value::Bool(false) => Ok(Value::Bool(false)),
                        Value::Bool(true) => {
                            let right_val = eval_expr_force_async(right, ctx).await?;
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(EvalError::InvalidBinaryOp {
                                    op: "and".to_string(),
                                    left: format!("{:?}", left_val),
                                    right: format!("{:?}", right_val),
                                }),
                            }
                        }
                        _ => Err(EvalError::InvalidBinaryOp {
                            op: "and".to_string(),
                            left: format!("{:?}", left_val),
                            right: "<unevaluated>".to_string(),
                        }),
                    }
                }
                BinaryOp::Or => {
                    let left_val = eval_expr_force_async(left, ctx).await?;
                    match left_val {
                        Value::Bool(true) => Ok(Value::Bool(true)),
                        Value::Bool(false) => {
                            let right_val = eval_expr_force_async(right, ctx).await?;
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(EvalError::InvalidBinaryOp {
                                    op: "or".to_string(),
                                    left: format!("{:?}", left_val),
                                    right: format!("{:?}", right_val),
                                }),
                            }
                        }
                        _ => Err(EvalError::InvalidBinaryOp {
                            op: "or".to_string(),
                            left: format!("{:?}", left_val),
                            right: "<unevaluated>".to_string(),
                        }),
                    }
                }
                _ => {
                    let left_val = eval_expr_force_async(left, ctx).await?;
                    let right_val = eval_expr_force_async(right, ctx).await?;
                    eval_binary_op(*op, left_val, right_val)
                }
            },
            Expr::Let {
                pattern,
                expr,
                body,
                ..
            } => {
                let value = eval_expr_force_async(expr, ctx).await?;
                let bindings = crate::pattern::match_pattern(pattern, &value).map_err(|_| {
                    EvalError::LetPatternBindFailed {
                        pattern: format!("{pattern:?}"),
                        value: format!("{value:?}"),
                    }
                })?;
                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);
                eval_expr_force_async(body, &new_ctx).await
            }
            Expr::FnApply { func, args } => {
                let func_val = eval_expr_force_async(func, ctx).await?;
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(eval_expr_force_async(arg, ctx).await?);
                }
                apply_closure_async_value(func_val, arg_vals, ctx).await
            }
            Expr::Call {
                func,
                module,
                arguments,
            } => {
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args.push(eval_expr_force_async(arg, ctx).await?);
                }

                if module.is_none() && func == "__unit" {
                    return runtime_unit(&args, ctx);
                }
                if module.is_none() && func == "__bind" {
                    return runtime_bind(&args, ctx);
                }
                if module.is_none() && func == "__then" {
                    return runtime_then(&args, ctx);
                }
                if module.is_none() && func == "__fail" {
                    return runtime_fail(&args, ctx);
                }
                if module.is_none() && func == "__guard" {
                    return runtime_guard(&args, ctx);
                }
                match (module.as_deref(), func.as_str()) {
                    (Some("proc"), "unit") => return runtime_proc_unit(&args, ctx),
                    (Some("proc"), "from_act") => return runtime_proc_from_act(&args, ctx),
                    (Some("proc"), "bind") => return runtime_proc_bind(&args, ctx),
                    (Some("proc"), "then") => return runtime_proc_then(&args, ctx),
                    (Some("proc"), "await") => return runtime_proc_await(&args, ctx),
                    (Some("proc"), "yield") => return runtime_proc_yield(&args, ctx),
                    (Some("proc"), "par") => return runtime_proc_par(&args, ctx),
                    (Some("proc"), "scatter") => return runtime_proc_scatter(&args, ctx),
                    (Some("proc"), "join") => return runtime_proc_join(&args, ctx),
                    (Some("proc"), "gather") => return runtime_proc_gather(&args, ctx),
                    (Some("workflow"), "unit") => return runtime_proc_unit(&args, ctx),
                    (Some("workflow"), "from_act") => return runtime_proc_from_act(&args, ctx),
                    (Some("workflow"), "from_proc") => return runtime_proc_unit(&args, ctx),
                    (Some("workflow"), "bind") => return runtime_proc_bind(&args, ctx),
                    (Some("workflow"), "then") => return runtime_proc_then(&args, ctx),
                    _ => {}
                }
                if let (true, Some(Value::Closure { params, body, env })) =
                    (module.is_none(), ctx.get(func))
                {
                    return apply_closure_async(params, body, env, args, ctx).await;
                }
                if module.is_none() && func == "unit" {
                    return runtime_unit(&args, ctx);
                }
                if module.is_none() && func == "bind" {
                    return runtime_bind(&args, ctx);
                }
                if module.is_none() && func == "invoke" {
                    return runtime_invoke(&args, ctx);
                }
                if module.is_none() && func == "policy_check" {
                    return runtime_policy_check(&args, ctx);
                }

                let qname = qualified_builtin_name(func, module.as_deref());
                if let Some(result) = dispatch_builtin(&qname, &args, ctx) {
                    return result;
                }

                match eval_function_call(func, module.as_deref(), &args, ctx) {
                    Ok(value) => Ok(value),
                    Err(EvalError::UnknownFunction(_)) => match ctx.get(func) {
                        Some(Value::Closure { params, body, env }) => {
                            apply_closure_async(params, body, env, args, ctx).await
                        }
                        Some(other) => Err(EvalError::TypeMismatch {
                            expected: "callable".to_string(),
                            actual: format!("{other:?}"),
                        }),
                        None => Err(EvalError::UnknownFunction(func.clone())),
                    },
                    Err(e) => Err(e),
                }
            }
            Expr::Constructor { name, fields } => {
                let mut evaluated_fields = Vec::with_capacity(fields.len());
                for (field_name, field_expr) in fields {
                    let value = eval_expr_force_async(field_expr, ctx).await?;
                    evaluated_fields.push((field_name.clone(), value));
                }
                Ok(Value::Variant {
                    name: name.clone(),
                    fields: Box::new(evaluated_fields),
                })
            }
            Expr::Match { scrutinee, arms } => {
                let value = eval_expr_force_async(scrutinee, ctx).await?;
                for arm in arms {
                    match crate::pattern::match_pattern(&arm.pattern, &value) {
                        Ok(bindings) => {
                            let mut new_ctx = ctx.extend();
                            for (name, val) in bindings {
                                new_ctx.set(name, val);
                            }
                            return eval_expr_force_async(&arm.body, &new_ctx).await;
                        }
                        Err(_) => continue,
                    }
                }
                Err(EvalError::NonExhaustiveMatch {
                    value: format!("{:?}", value),
                })
            }
            Expr::Fail { payload } => {
                let payload = eval_expr_force_async(payload, ctx).await?;
                Err(EvalError::OperationalFailure(Box::new(
                    operational_failure_for_payload(payload, ctx),
                )))
            }
            Expr::WithError { body, arms } => match eval_expr_force_async(body, ctx).await {
                Ok(value) => Ok(value),
                Err(EvalError::OperationalFailure(original)) => {
                    for arm in arms {
                        match crate::pattern::match_pattern(&arm.pattern, &original.payload) {
                            Ok(bindings) => {
                                let mut new_ctx = ctx.extend();
                                for (name, val) in bindings {
                                    new_ctx.set(name, val);
                                }
                                return match eval_expr_force_async(&arm.body, &new_ctx).await {
                                    Err(EvalError::OperationalFailure(raised)) => {
                                        Err(EvalError::OperationalFailure(
                                            preserve_caught_failure_as_tail_cause(
                                                raised,
                                                original.as_ref(),
                                            ),
                                        ))
                                    }
                                    other => other,
                                };
                            }
                            Err(_) => continue,
                        }
                    }
                    Err(EvalError::OperationalFailure(original))
                }
                Err(error) => Err(error),
            },
            Expr::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
            } => {
                let value = eval_expr_force_async(expr, ctx).await?;
                match crate::pattern::match_pattern(pattern, &value) {
                    Ok(bindings) => {
                        let mut new_ctx = ctx.extend();
                        for (name, val) in bindings {
                            new_ctx.set(name, val);
                        }
                        eval_expr_force_async(then_branch, &new_ctx).await
                    }
                    Err(_) => eval_expr_force_async(else_branch, ctx).await,
                }
            }
            Expr::Spawn { workflow_type, .. } => eval_spawn(workflow_type),
            Expr::Split(expr) => {
                let value = eval_expr_force_async(expr, ctx).await?;
                match value {
                    Value::Instance(instance) => {
                        let addr = Value::InstanceAddr(instance.addr);
                        let control = instance.control.map(Value::ControlLink);
                        Ok(Value::List(Box::new(vec![
                            addr,
                            control.unwrap_or(Value::Null),
                        ])))
                    }
                    _ => Err(EvalError::TypeMismatch {
                        expected: "Instance".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
            Expr::CheckObligation { obligation, .. } => {
                Ok(Value::Bool(ctx.discharge_obligation(obligation)))
            }
            Expr::FnDef {
                params,
                return_type: _,
                body,
            } => {
                let env_frame = ctx.to_env_frame();
                // SPEC-088: Capture-based effect rule replaces blanket ban.
                if ctx.is_pure() {
                    for (name, value) in env_frame.all_bindings() {
                        if !value.is_pure() {
                            return Err(EvalError::CaptureEffectViolation {
                                var: name.to_string(),
                                var_effect: value.effect_level(),
                                context_effect: "Pure".to_string(),
                                context: "closure created inside pure-function boundary".into(),
                            });
                        }
                    }
                }
                let closure = Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    env: env_frame,
                };
                Ok(closure)
            }
        }
    })
}

fn apply_closure_async_value<'a>(
    callee: Value,
    args: Vec<Value>,
    runtime_ctx: &'a Context,
) -> EvalBoxFuture<'a> {
    Box::pin(async move {
        match callee {
            Value::Closure { params, body, env } => {
                apply_closure_async(&params, &body, &env, args, runtime_ctx).await
            }
            other => Err(EvalError::NotCallable {
                value: Box::new(other),
            }),
        }
    })
}

fn apply_closure_async<'a>(
    params: &'a [(String, Option<String>)],
    body: &'a Expr,
    env: &'a std::sync::Arc<ash_core::env_frame::EnvFrame>,
    args: Vec<Value>,
    runtime_ctx: &'a Context,
) -> EvalBoxFuture<'a> {
    Box::pin(async move {
        let validates_hidden_act_env = |params: &[(String, Option<String>)], args: &[Value]| {
            for ((name, _ty), val) in params.iter().zip(args.iter()) {
                if name == "__act_env" {
                    if !matches!(val, Value::ActEnvToken) {
                        return Err(EvalError::TypeMismatch {
                            expected: "hidden ActEnv token".to_string(),
                            actual: format!("{val:?}"),
                        });
                    }
                    if runtime_ctx.act_env().is_none() {
                        return Err(EvalError::TypeMismatch {
                            expected: "hidden runtime ActEnv".to_string(),
                            actual: "missing runtime ActEnv".to_string(),
                        });
                    }
                }
            }
            Ok(())
        };

        if args.len() == params.len() {
            validates_hidden_act_env(params, &args)?;
            let enters_effect_scope = params
                .iter()
                .take(args.len())
                .any(|(name, _ty)| name == "__act_env");
            let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
            for ((name, _ty), val) in params.iter().zip(args) {
                call_env.insert(name.clone(), val);
            }
            let call_ctx = call_context_from_env(
                std::sync::Arc::new(call_env),
                runtime_ctx,
                enters_effect_scope,
            );
            let result = eval_expr_force_async(body, &call_ctx).await?;
            let result = maybe_execute_invoke_capture_async(result, &call_ctx).await?;
            let result = maybe_execute_proc_await_capture_async(result, &call_ctx).await?;
            let result = maybe_execute_proc_yield_capture_async(result, &call_ctx).await?;
            let result = maybe_execute_proc_admission_capture_async(result, &call_ctx).await?;
            maybe_execute_proc_wait_all_capture_async(result, &call_ctx).await
        } else {
            Err(EvalError::WrongArity {
                expected: params.len(),
                actual: args.len(),
                callee: None,
            })
        }
    })
}

async fn maybe_execute_invoke_capture_async(
    value: Value,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    let Value::List(items) = value else {
        return Ok(value);
    };
    if items.len() != 2 || !matches!(items[0], Value::ActEnvToken) {
        return Ok(Value::List(items));
    }
    let Value::Variant { name, fields } = &items[1] else {
        return Ok(Value::List(items));
    };
    if name != "__InvokeCapture" {
        return Ok(Value::List(items));
    }

    let provider = fields
        .iter()
        .find(|(field, _)| field == "provider")
        .and_then(|(_, value)| match value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing string provider".to_string())
        })?;
    let action = fields
        .iter()
        .find(|(field, _)| field == "action")
        .and_then(|(_, value)| match value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing string action".to_string())
        })?;
    let args = fields
        .iter()
        .find(|(field, _)| field == "args")
        .and_then(|(_, value)| match value {
            Value::List(items) => Some((**items).clone()),
            _ => None,
        })
        .ok_or_else(|| {
            EvalError::ExecutionFailed("invoke capture missing list args".to_string())
        })?;

    let provider_is_local_dependency = runtime_ctx
        .get(&format!("__ash_capability_dependency_local:{provider}"))
        .is_some();
    let provider = if let Some(Value::String(source_binding_name)) = runtime_ctx
        .get(&format!("__ash_capability_dependency_alias:{provider}"))
        .cloned()
    {
        source_binding_name
    } else {
        provider
    };

    let act_env = runtime_ctx.act_env().ok_or_else(|| {
        EvalError::ExecutionFailed("invoke capture missing hidden runtime ActEnv".to_string())
    })?;

    if provider_is_local_dependency {
        let invoked = act_env
            .capability_ctx
            .execute(&provider, &action, &args)
            .await
            .map_err(|err| operational_eval_error_for_message(err.to_string(), runtime_ctx))?;
        return Ok(Value::List(Box::new(vec![Value::ActEnvToken, invoked])));
    }

    if let Some(runtime_state) = runtime_ctx.runtime_state()
        && let Some(binding) = runtime_state.capability_binding_by_name(&provider).await
        && runtime_ctx
            .admitted_capability_bindings()
            .contains(&binding.id)
    {
        match &binding.kind {
            CapabilityBindingKind::Implementation { implementation } => {
                return execute_implementation_operation_body_async(
                    runtime_state.as_ref(),
                    &binding,
                    implementation.clone(),
                    &action,
                    args,
                    runtime_ctx,
                    act_env,
                )
                .await;
            }
            CapabilityBindingKind::HostProvider { .. } => {
                let projected_ctx = runtime_state
                    .create_capability_context_for_bindings(&[binding.id])
                    .await
                    .map_err(|err| EvalError::ExecutionFailed(err.to_string()))?;
                let invoked = projected_ctx
                    .execute(&provider, &action, &args)
                    .await
                    .map_err(|err| {
                        operational_eval_error_for_message(err.to_string(), runtime_ctx)
                    })?;
                return Ok(Value::List(Box::new(vec![Value::ActEnvToken, invoked])));
            }
        }
    }

    let binding = if let Some(runtime_state) = runtime_ctx.runtime_state() {
        runtime_state.capability_binding_by_name(&provider).await
    } else {
        None
    };
    let binding_admitted = binding.as_ref().map(|binding| {
        runtime_ctx
            .admitted_capability_bindings()
            .contains(&binding.id)
    });
    let binding_exists = binding.is_some();
    let provider_registered = runtime_ctx.runtime_state().is_some_and(|runtime_state| {
        runtime_state.has_provider(&provider)
            || matches!(binding_admitted, Some(false))
            || binding_exists
    });
    if binding_admitted == Some(false) {
        let has_unadmitted_implementation = binding.as_ref().is_some_and(|binding| {
            matches!(binding.kind, CapabilityBindingKind::Implementation { .. })
        });
        let has_unadmitted_host_provider = binding.as_ref().is_some_and(|binding| {
            matches!(binding.kind, CapabilityBindingKind::HostProvider { .. })
        });
        let message = if has_unadmitted_implementation {
            format!(
                "capability {provider} not available: authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
            )
        } else if has_unadmitted_host_provider {
            format!(
                "authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
            )
        } else {
            format!("capability {provider} not available")
        };
        return Err(operational_eval_error_for_message(message, runtime_ctx));
    }
    let ambient_runtime_authorized = act_env.has_runtime_state_ambient_authority()
        && runtime_ctx.admitted_capability_bindings().is_empty()
        && binding_admitted.is_none();
    let fallback_authorized = binding_admitted == Some(true) || ambient_runtime_authorized;
    if let Some(runtime_state) = runtime_ctx.runtime_state()
        && let Some(binding) = runtime_state.capability_binding_by_name(&provider).await
        && matches!(binding.kind, CapabilityBindingKind::Implementation { .. })
        && !fallback_authorized
    {
        return Err(operational_eval_error_for_message(
            format!("capability {provider} not available"),
            runtime_ctx,
        ));
    }
    let invoke_result = act_env
        .capability_ctx
        .execute(&provider, &action, &args)
        .await;
    let invoked = match invoke_result {
        Ok(value) if fallback_authorized => value,
        Ok(_) => {
            return Err(operational_eval_error_for_message(
                format!(
                    "authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
                ),
                runtime_ctx,
            ));
        }
        Err(err) if !fallback_authorized && binding_admitted.is_none() && !provider_registered => {
            return Err(operational_eval_error_for_message(
                err.to_string(),
                runtime_ctx,
            ));
        }
        Err(err) if fallback_authorized => {
            return Err(operational_eval_error_for_message(
                err.to_string(),
                runtime_ctx,
            ));
        }
        Err(_) => {
            return Err(operational_eval_error_for_message(
                format!(
                    "authority boundary failure: provider {provider} lacks RuntimeKernel admission for invoke fallback dispatch"
                ),
                runtime_ctx,
            ));
        }
    };
    Ok(Value::List(Box::new(vec![Value::ActEnvToken, invoked])))
}

async fn execute_implementation_operation_body_async(
    runtime_state: &crate::runtime_state::RuntimeState,
    binding: &ash_core::CapabilityBinding,
    implementation: ash_core::CapabilityImplementationId,
    action: &str,
    args: Vec<Value>,
    runtime_ctx: &Context,
    outer_act_env: std::sync::Arc<crate::act_env::ActEnv>,
) -> EvalResult<Value> {
    let body = runtime_state
        .implementation_operation_body(&implementation, action)
        .await
        .ok_or_else(|| {
            operational_eval_error_for_message(
                format!(
                    "no Ash-defined operation body registered for implementation {} operation {action}",
                    implementation.as_str()
                ),
                runtime_ctx,
            )
        })?;

    if body.params.len() != args.len() {
        return Err(operational_eval_error_for_message(
            format!(
                "Ash-defined operation body {}.{} expected {} arguments but received {}",
                binding.name,
                action,
                body.params.len(),
                args.len()
            ),
            runtime_ctx,
        ));
    }

    let (capability_ctx, mut dependency_values, mut dependency_bindings) = runtime_state
        .implementation_binding_dependency_context(binding)
        .await
        .map_err(|err| operational_eval_error_for_message(err.to_string(), runtime_ctx))?;

    let mut admitted_bindings = runtime_ctx.admitted_capability_bindings().to_vec();
    admitted_bindings.append(&mut dependency_bindings);
    admitted_bindings.sort_unstable_by_key(|binding_id| binding_id.0);
    admitted_bindings.dedup();

    let mut body_ctx = Context::new()
        .inherit_runtime_metadata_from(runtime_ctx)
        .with_runtime_state_arc(std::sync::Arc::new(runtime_state.clone()))
        .with_admitted_capability_bindings(admitted_bindings)
        .with_act_env(crate::act_env::ActEnv::new(
            capability_ctx,
            outer_act_env.policies.clone(),
            outer_act_env.provenance.clone(),
        ));
    for (name, value) in dependency_values.drain() {
        body_ctx.set(name, value);
    }
    for (name, value) in body.params.iter().cloned().zip(args) {
        body_ctx.set(name, value);
    }

    let value = eval_expr_force_async(&body.body, &body_ctx)
        .await
        .map_err(|err| {
            operational_eval_error_for_message(
                format!(
                    "Ash-defined operation body {}.{} failed: {err}",
                    binding.name, action
                ),
                runtime_ctx,
            )
        })?;
    let value = if body.returns_act {
        let mut call_ctx = body_ctx.clone();
        call_ctx.set("__ash_operation_body_act".to_string(), value);
        eval_expr_force_async(
            &Expr::Call {
                func: "__ash_operation_body_act".to_string(),
                module: None,
                arguments: vec![Expr::Literal(Value::ActEnvToken)],
            },
            &call_ctx,
        )
        .await?
    } else {
        value
    };
    if matches_normalized_act_result(&value) {
        Ok(value)
    } else {
        Ok(Value::List(Box::new(vec![Value::ActEnvToken, value])))
    }
}

pub async fn eval_expr_async(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    eval_expr_force_async(expr, ctx).await
}

pub fn eval_expr(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    match expr {
        Expr::Literal(value) => {
            let value = maybe_execute_invoke_capture(value.clone(), ctx)?;
            let value = maybe_execute_proc_await_capture(value, ctx)?;
            maybe_execute_proc_yield_capture(value, ctx)
        }

        Expr::Variable { name, .. } => ctx
            .get(name)
            .cloned()
            .or_else(|| {
                if name == "()" {
                    Some(Value::Null)
                } else {
                    None
                }
            })
            .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

        Expr::FieldAccess { expr, field } => {
            let value = eval_expr(expr, ctx)?;
            match value {
                Value::Record(mut fields) => {
                    let removed = fields.remove(field);
                    if removed.is_none() {
                        return Err(EvalError::FieldNotFound {
                            field: field.clone(),
                            value: Box::new(Value::Record(fields)),
                        });
                    }
                    Ok(removed.unwrap())
                }
                Value::List(items) => {
                    let idx = field
                        .parse::<usize>()
                        .map_err(|_| EvalError::TypeMismatch {
                            expected: "record".to_string(),
                            actual: format!("{:?}", Value::List(items.clone())),
                        })?;
                    items.get(idx).cloned().ok_or(EvalError::IndexOutOfBounds {
                        index: idx as i64,
                        len: items.len(),
                    })
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", value),
                }),
            }
        }

        Expr::IndexAccess { expr, index } => {
            let collection = eval_expr(expr, ctx)?;
            let idx_val = eval_expr(index, ctx)?;

            match idx_val {
                Value::Int(i) => {
                    let idx = i as usize;
                    match collection {
                        Value::List(list) => {
                            if idx < list.len() {
                                Ok(list[idx].clone())
                            } else {
                                Err(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: list.len(),
                                })
                            }
                        }
                        Value::String(s) => {
                            if let Some(c) = s.chars().nth(idx) {
                                Ok(Value::String(c.to_string()))
                            } else {
                                Err(EvalError::IndexOutOfBounds {
                                    index: i,
                                    len: s.len(),
                                })
                            }
                        }
                        _ => Err(EvalError::TypeMismatch {
                            expected: "list or string".to_string(),
                            actual: format!("{:?}", collection),
                        }),
                    }
                }
                _ => Err(EvalError::InvalidIndexType(format!("{:?}", idx_val))),
            }
        }

        Expr::Unary { op, expr } => {
            let value = eval_expr(expr, ctx)?;
            eval_unary_op(*op, value)
        }

        Expr::Binary { op, left, right } => {
            // Short-circuit evaluation for and/or (SPEC-004 EXPR-AND-FALSE, EXPR-OR-TRUE)
            match op {
                BinaryOp::And => {
                    let left_val = eval_expr(left, ctx)?;
                    match left_val {
                        Value::Bool(false) => Ok(Value::Bool(false)),
                        Value::Bool(true) => {
                            let right_val = eval_expr(right, ctx)?;
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(EvalError::InvalidBinaryOp {
                                    op: "and".to_string(),
                                    left: format!("{:?}", left_val),
                                    right: format!("{:?}", right_val),
                                }),
                            }
                        }
                        _ => Err(EvalError::InvalidBinaryOp {
                            op: "and".to_string(),
                            left: format!("{:?}", left_val),
                            right: "<unevaluated>".to_string(),
                        }),
                    }
                }
                BinaryOp::Or => {
                    let left_val = eval_expr(left, ctx)?;
                    match left_val {
                        Value::Bool(true) => Ok(Value::Bool(true)),
                        Value::Bool(false) => {
                            let right_val = eval_expr(right, ctx)?;
                            match right_val {
                                Value::Bool(b) => Ok(Value::Bool(b)),
                                _ => Err(EvalError::InvalidBinaryOp {
                                    op: "or".to_string(),
                                    left: format!("{:?}", left_val),
                                    right: format!("{:?}", right_val),
                                }),
                            }
                        }
                        _ => Err(EvalError::InvalidBinaryOp {
                            op: "or".to_string(),
                            left: format!("{:?}", left_val),
                            right: "<unevaluated>".to_string(),
                        }),
                    }
                }
                _ => {
                    let left_val = eval_expr(left, ctx)?;
                    let right_val = eval_expr(right, ctx)?;
                    eval_binary_op(*op, left_val, right_val)
                }
            }
        }

        Expr::Call {
            func,
            module,
            arguments,
        } => {
            let args: Vec<Value> = arguments
                .iter()
                .map(|arg| eval_expr(arg, ctx))
                .collect::<Result<Vec<_>, _>>()?;

            // For unqualified calls (module: None), context closures take priority over
            // builtin dispatch so that `use string::{concat}` shadows the unqualified
            // list-concat builtin. Qualified calls (module: Some) always skip this path
            // and go directly to builtin dispatch — the context is not consulted.
            //
            if module.is_none() && func == "__unit" {
                return runtime_unit(&args, ctx);
            }

            if module.is_none() && func == "__bind" {
                return runtime_bind(&args, ctx);
            }

            if module.is_none() && func == "__then" {
                return runtime_then(&args, ctx);
            }

            if module.is_none() && func == "__fail" {
                return runtime_fail(&args, ctx);
            }

            if module.is_none() && func == "__guard" {
                return runtime_guard(&args, ctx);
            }

            match (module.as_deref(), func.as_str()) {
                (Some("proc"), "unit") => return runtime_proc_unit(&args, ctx),
                (Some("proc"), "from_act") => return runtime_proc_from_act(&args, ctx),
                (Some("proc"), "bind") => return runtime_proc_bind(&args, ctx),
                (Some("proc"), "then") => return runtime_proc_then(&args, ctx),
                (Some("proc"), "await") => return runtime_proc_await(&args, ctx),
                (Some("proc"), "yield") => return runtime_proc_yield(&args, ctx),
                (Some("proc"), "par") => return runtime_proc_par(&args, ctx),
                (Some("proc"), "scatter") => return runtime_proc_scatter(&args, ctx),
                (Some("proc"), "join") => return runtime_proc_join(&args, ctx),
                (Some("proc"), "gather") => return runtime_proc_gather(&args, ctx),
                (Some("workflow"), "unit") => return runtime_proc_unit(&args, ctx),
                (Some("workflow"), "from_act") => return runtime_proc_from_act(&args, ctx),
                (Some("workflow"), "from_proc") => return runtime_proc_unit(&args, ctx),
                (Some("workflow"), "bind") => return runtime_proc_bind(&args, ctx),
                (Some("workflow"), "then") => return runtime_proc_then(&args, ctx),
                _ => {}
            }

            // User-defined closures and builtins both use exact arity after SPEC-072;
            // wrong-arity closure calls produce WrongArity { callee: None }.
            if let (true, Some(Value::Closure { params, body, env })) =
                (module.is_none(), ctx.get(func))
            {
                return apply_closure(params, body, env, args, ctx);
            }

            if module.is_none() && func == "unit" {
                return runtime_unit(&args, ctx);
            }

            if module.is_none() && func == "bind" {
                return runtime_bind(&args, ctx);
            }

            if module.is_none() && func == "invoke" {
                return runtime_invoke(&args, ctx);
            }

            if module.is_none() && func == "policy_check" {
                return runtime_policy_check(&args, ctx);
            }

            // Try builtin dispatch table first (O(1) qualified lookup).
            // dispatch_builtin returns Some(result) when the name is in the table.
            let qname = qualified_builtin_name(func, module.as_deref());
            if let Some(result) = dispatch_builtin(&qname, &args, ctx) {
                return result;
            }

            // Not in dispatch table: try legacy eval_function_call (covers
            // unqualified builtins like "len", "head" matched via pattern).
            match eval_function_call(func, module.as_deref(), &args, ctx) {
                Ok(value) => Ok(value),
                Err(EvalError::UnknownFunction(_)) => {
                    // Not a built-in: try looking up a closure in the context
                    match ctx.get(func) {
                        Some(Value::Closure { params, body, env }) => {
                            apply_closure(params, body, env, args, ctx)
                        }
                        Some(other) => Err(EvalError::TypeMismatch {
                            expected: "callable".to_string(),
                            actual: format!("{other:?}"),
                        }),
                        None => Err(EvalError::UnknownFunction(func.clone())),
                    }
                }
                Err(e) => Err(e),
            }
        }

        Expr::Constructor { name, fields } => {
            // Evaluate each field expression and collect into a vector of (name, value) pairs
            let evaluated_fields: Vec<(String, Value)> = fields
                .iter()
                .map(|(field_name, expr)| {
                    eval_expr(expr, ctx).map(|value| (field_name.clone(), value))
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Value::Variant {
                name: name.clone(),
                fields: Box::new(evaluated_fields),
            })
        }

        Expr::Match { scrutinee, arms } => eval_match(scrutinee, arms, ctx),

        Expr::Fail { payload } => {
            let payload = eval_expr(payload, ctx)?;
            Err(EvalError::OperationalFailure(Box::new(
                operational_failure_for_payload(payload, ctx),
            )))
        }

        Expr::WithError { body, arms } => match eval_expr(body, ctx) {
            Ok(value) => Ok(value),
            Err(EvalError::OperationalFailure(original)) => {
                for arm in arms {
                    match crate::pattern::match_pattern(&arm.pattern, &original.payload) {
                        Ok(bindings) => {
                            let mut new_ctx = ctx.extend();
                            for (name, val) in bindings {
                                new_ctx.set(name, val);
                            }
                            return match eval_expr(&arm.body, &new_ctx) {
                                Err(EvalError::OperationalFailure(raised)) => {
                                    Err(EvalError::OperationalFailure(
                                        preserve_caught_failure_as_tail_cause(
                                            raised,
                                            original.as_ref(),
                                        ),
                                    ))
                                }
                                other => other,
                            };
                        }
                        Err(_) => continue,
                    }
                }
                Err(EvalError::OperationalFailure(original))
            }
            Err(error) => Err(error),
        },

        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        } => eval_if_let(pattern, expr, then_branch, else_branch, ctx),

        Expr::Spawn {
            workflow_type,
            init: _,
        } => eval_spawn(workflow_type),

        Expr::Split(expr) => eval_split(expr, ctx),

        Expr::CheckObligation { obligation, .. } => {
            // Check if obligation exists and discharge it (linear consumption)
            // Returns true if obligation was found and discharged, false otherwise
            let discharged = ctx.discharge_obligation(obligation);
            Ok(Value::Bool(discharged))
        }

        Expr::FnDef {
            params,
            return_type: _,
            body,
        } => {
            let env_frame = ctx.to_env_frame();
            // SPEC-088: Capture-based effect rule replaces blanket ban.
            // A closure in a pure context may only capture values whose effect level ≤ Pure.
            if ctx.is_pure() {
                for (name, value) in env_frame.all_bindings() {
                    if !value.is_pure() {
                        return Err(EvalError::CaptureEffectViolation {
                            var: name.to_string(),
                            var_effect: value.effect_level(),
                            context_effect: "Pure".to_string(),
                            context: "closure created inside pure-function boundary".into(),
                        });
                    }
                }
            }
            let closure = Value::Closure {
                params: params.clone(),
                body: body.clone(),
                env: env_frame,
            };
            Ok(closure)
        }

        Expr::FnApply { func, args } => {
            let callee = eval_expr(func, ctx)?;
            match callee {
                Value::Closure { params, body, env } => {
                    let arg_vals: Vec<Value> = args
                        .iter()
                        .map(|arg| eval_expr(arg, ctx))
                        .collect::<Result<Vec<_>, _>>()?;
                    apply_closure(&params, &body, &env, arg_vals, ctx)
                }
                _ => Err(EvalError::NotCallable {
                    value: Box::new(callee),
                }),
            }
        }

        // EXPR-LET (SPEC-004 §4.6): pure scope extension
        // Evaluate expr, match pattern, extend env, evaluate body
        Expr::Let {
            pattern,
            expr,
            body,
            span: _,
        } => {
            let value = eval_expr(expr, ctx)?;
            match crate::pattern::match_pattern(pattern, &value) {
                Ok(bindings) => {
                    let mut child_ctx = ctx.extend();
                    for (name, val) in bindings {
                        child_ctx.set(name, val);
                    }
                    eval_expr(body, &child_ctx)
                }
                Err(_) => Err(EvalError::LetPatternBindFailed {
                    pattern: format!("{:?}", pattern),
                    value: format!("{:?}", value),
                }),
            }
        }
    }
}

mod control;
use control::{eval_if_let, eval_match, eval_spawn, eval_split};

/// Evaluate a built-in function call
pub fn eval_function_call(
    func: &str,
    module: Option<&str>,
    args: &[Value],
    ctx: &Context,
) -> EvalResult<Value> {
    match (module, func) {
        (Some("act"), "unit") => runtime_unit(args, ctx),
        (Some("act"), "bind") => runtime_bind(args, ctx),
        (Some("proc"), "unit") => runtime_proc_unit(args, ctx),
        (Some("proc"), "from_act") => runtime_proc_from_act(args, ctx),
        (Some("proc"), "bind") => runtime_proc_bind(args, ctx),
        (Some("proc"), "then") => runtime_proc_then(args, ctx),
        (Some("proc"), "await") => runtime_proc_await(args, ctx),
        (Some("proc"), "yield") => runtime_proc_yield(args, ctx),
        (Some("proc"), "par") => runtime_proc_par(args, ctx),
        (Some("proc"), "scatter") => runtime_proc_scatter(args, ctx),
        (Some("proc"), "join") => runtime_proc_join(args, ctx),
        (Some("proc"), "gather") => runtime_proc_gather(args, ctx),
        (Some("act"), "__guard") | (None, "__guard") => runtime_guard(args, ctx),
        (Some("act"), "policy_check") | (None, "policy_check") => runtime_policy_check(args, ctx),
        (Some("result"), "and_then") => runtime_result_and_then(args, ctx),
        (None, "Ok") => runtime_result_ok(args),
        (None, "Err") => runtime_result_err(args),
        (Some("string"), "concat") => {
            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::String(s) => result.push_str(s),
                    _ => {
                        return Err(EvalError::TypeMismatch {
                            expected: "string".to_string(),
                            actual: format!("{:?}", arg),
                        });
                    }
                }
            }
            Ok(Value::String(result))
        }
        (Some("string"), "starts_with") => {
            if args.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: args.len(),
                    callee: None,
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string, string".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }
        (Some("string"), "ends_with") => {
            if args.len() != 2 {
                return Err(EvalError::WrongArity {
                    expected: 2,
                    actual: args.len(),
                    callee: None,
                });
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string, string".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }
        (Some("string"), "is_empty") => {
            if args.len() != 1 {
                return Err(EvalError::WrongArity {
                    expected: 1,
                    actual: args.len(),
                    callee: None,
                });
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Bool(s.is_empty())),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }
        (Some("string"), "to_upper") => {
            if args.len() != 1 {
                return builtin_arity_error("string::to_upper", 1, args.len());
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_uppercase())),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }
        (Some("string"), "to_lower") => {
            if args.len() != 1 {
                return builtin_arity_error("string::to_lower", 1, args.len());
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_lowercase())),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }
        (Some("string"), "trim") => {
            if args.len() != 1 {
                return builtin_arity_error("string::trim", 1, args.len());
            }
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim().to_string())),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }
        (Some("regex"), "find") => {
            if args.len() != 2 {
                return builtin_arity_error("regex::find", 2, args.len());
            }
            let pattern = expect_string_arg(args, 0, "string")?;
            let text = expect_string_arg(args, 1, "string")?;
            regex_find(pattern, text)
        }
        (Some("regex"), "matches") => {
            if args.len() != 2 {
                return builtin_arity_error("regex::matches", 2, args.len());
            }
            let pattern = expect_string_arg(args, 0, "string")?;
            let text = expect_string_arg(args, 1, "string")?;
            regex_matches(pattern, text)
        }
        (Some("regex"), "replace") => {
            if args.len() != 3 {
                return builtin_arity_error("regex::replace", 3, args.len());
            }
            let pattern = expect_string_arg(args, 0, "string")?;
            let replacement = expect_string_arg(args, 1, "string")?;
            let text = expect_string_arg(args, 2, "string")?;
            regex_replace(pattern, replacement, text)
        }
        (Some("process"), "run") => {
            if args.len() != 2 {
                return builtin_arity_error("process::run", 2, args.len());
            }
            let cmd = expect_string_arg(args, 0, "string")?;
            let list = match &args[1] {
                Value::List(items) => items,
                other => {
                    return Err(EvalError::TypeMismatch {
                        expected: "list".to_string(),
                        actual: format!("{other:?}"),
                    });
                }
            };

            let mut command = std::process::Command::new(cmd);
            for item in list.iter() {
                match item {
                    Value::String(s) => {
                        command.arg(s);
                    }
                    other => {
                        return Err(EvalError::TypeMismatch {
                            expected: "string".to_string(),
                            actual: format!("{other:?}"),
                        });
                    }
                }
            }

            let output = command
                .output()
                .map_err(|e| EvalError::ExecutionFailed(format!("process::run failed: {e}")))?;

            let mut result = HashMap::new();
            result.insert(
                "stdout".to_string(),
                Value::String(String::from_utf8_lossy(&output.stdout).to_string()),
            );
            result.insert(
                "stderr".to_string(),
                Value::String(String::from_utf8_lossy(&output.stderr).to_string()),
            );
            result.insert(
                "exit_code".to_string(),
                Value::Int(i64::from(output.status.code().unwrap_or(-1))),
            );
            Ok(Value::Record(Box::new(result)))
        }
        (Some("process"), "which") => {
            if args.len() != 1 {
                return builtin_arity_error("process::which", 1, args.len());
            }
            let cmd = expect_string_arg(args, 0, "string")?;
            Ok(process_which_value(cmd))
        }
        // List operations
        (_, "len") => {
            if args.len() != 1 {
                return builtin_arity_error("len", 1, args.len());
            }
            match &args[0] {
                Value::List(list) => Ok(Value::Int(list.len() as i64)),
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                other => {
                    // Try Cons/Nil variant representation
                    match crate::list_helpers::list_len(other) {
                        Some(len) => Ok(Value::Int(len as i64)),
                        None => Err(EvalError::TypeMismatch {
                            expected: "list or string".to_string(),
                            actual: format!("{:?}", args[0]),
                        }),
                    }
                }
            }
        }

        (_, "head") => {
            if args.len() != 1 {
                return builtin_arity_error("head", 1, args.len());
            }
            match &args[0] {
                Value::List(list) => {
                    if list.is_empty() {
                        Err(EvalError::ExecutionFailed("head on empty list".to_string()))
                    } else {
                        Ok(list[0].clone())
                    }
                }
                other => {
                    // Try Cons/Nil variant representation
                    match crate::list_helpers::list_head(other) {
                        Some(head) => Ok(head.clone()),
                        None => Err(EvalError::TypeMismatch {
                            expected: "list".to_string(),
                            actual: format!("{:?}", args[0]),
                        }),
                    }
                }
            }
        }

        (_, "tail") => {
            if args.len() != 1 {
                return builtin_arity_error("tail", 1, args.len());
            }
            match &args[0] {
                Value::List(list) => {
                    if list.is_empty() {
                        Err(EvalError::ExecutionFailed("tail on empty list".to_string()))
                    } else {
                        Ok(Value::List(Box::new(list[1..].to_vec())))
                    }
                }
                other => {
                    // Try Cons/Nil variant representation
                    match crate::list_helpers::list_tail(other) {
                        Some(tail) => {
                            // Convert back to Value::List for backward compatibility
                            match crate::list_helpers::list_to_vec(tail) {
                                Some(vec) => Ok(Value::List(Box::new(vec))),
                                None => Ok(tail.clone()),
                            }
                        }
                        None => Err(EvalError::TypeMismatch {
                            expected: "list".to_string(),
                            actual: format!("{:?}", args[0]),
                        }),
                    }
                }
            }
        }

        (_, "append") => {
            if args.len() != 2 {
                return builtin_arity_error("append", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::List(list), elem) => {
                    let mut new_list = list.to_vec();
                    new_list.push(elem.clone());
                    Ok(Value::List(Box::new(new_list)))
                }
                (other, elem) => {
                    // Try Cons/Nil variant representation
                    match crate::list_helpers::list_append(other, elem.clone()) {
                        Some(result) => {
                            // Convert back to Value::List for backward compatibility
                            match crate::list_helpers::list_to_vec(&result) {
                                Some(vec) => Ok(Value::List(Box::new(vec))),
                                None => Ok(result),
                            }
                        }
                        None => Err(EvalError::TypeMismatch {
                            expected: "list".to_string(),
                            actual: format!("{:?}", args[0]),
                        }),
                    }
                }
            }
        }

        (_, "concat") => {
            if args.len() != 2 {
                return builtin_arity_error("concat", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::List(l1), Value::List(l2)) => {
                    let mut new_list = l1.to_vec();
                    new_list.extend(l2.iter().cloned());
                    Ok(Value::List(Box::new(new_list)))
                }
                (other1, other2) => {
                    // Try Cons/Nil variant representation
                    match crate::list_helpers::list_concat(other1, other2) {
                        Some(result) => {
                            // Convert back to Value::List for backward compatibility
                            match crate::list_helpers::list_to_vec(&result) {
                                Some(vec) => Ok(Value::List(Box::new(vec))),
                                None => Ok(result),
                            }
                        }
                        None => Err(EvalError::TypeMismatch {
                            expected: "list, list".to_string(),
                            actual: format!("{:?}, {:?}", args[0], args[1]),
                        }),
                    }
                }
            }
        }

        (_, "filter") => {
            if args.len() != 2 {
                return builtin_arity_error("filter", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::List(list), Value::Closure { params, body, env }) => {
                    if params.len() != 1 {
                        return Err(EvalError::WrongArity {
                            expected: params.len(),
                            actual: 1,
                            callee: None,
                        });
                    }
                    let mut result = Vec::new();
                    for item in list.iter() {
                        let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
                        call_env.insert(params[0].0.clone(), item.clone());
                        let call_ctx =
                            call_context_from_env(std::sync::Arc::new(call_env), ctx, false);
                        match eval_expr(body, &call_ctx)? {
                            Value::Bool(true) => result.push(item.clone()),
                            Value::Bool(false) => {}
                            other => {
                                return Err(EvalError::TypeMismatch {
                                    expected: "bool".to_string(),
                                    actual: format!("{other:?}"),
                                });
                            }
                        }
                    }
                    Ok(Value::List(Box::new(result)))
                }
                (other, Value::Closure { params, body, env }) => {
                    // Try Cons/Nil variant representation
                    if params.len() != 1 {
                        return Err(EvalError::WrongArity {
                            expected: params.len(),
                            actual: 1,
                            callee: None,
                        });
                    }
                    let list = match crate::list_helpers::list_to_vec(other) {
                        Some(list) => list,
                        None => return Err(EvalError::TypeMismatch {
                            expected: "list, function".to_string(),
                            actual: format!("{:?}, {:?}", args[0], args[1]),
                        }),
                    };
                    let mut result = Vec::new();
                    for item in list.iter() {
                        let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
                        call_env.insert(params[0].0.clone(), item.clone());
                        let call_ctx =
                            call_context_from_env(std::sync::Arc::new(call_env), ctx, false);
                        match eval_expr(body, &call_ctx)? {
                            Value::Bool(true) => result.push(item.clone()),
                            Value::Bool(false) => {}
                            other => {
                                return Err(EvalError::TypeMismatch {
                                    expected: "bool".to_string(),
                                    actual: format!("{other:?}"),
                                });
                            }
                        }
                    }
                    Ok(Value::List(Box::new(result)))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "list, function".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }

        (_, "map") => {
            if args.len() != 2 {
                return builtin_arity_error("map", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::List(list), Value::Closure { params, body, env }) => {
                    if params.len() != 1 {
                        return Err(EvalError::WrongArity {
                            expected: params.len(),
                            actual: 1,
                            callee: None,
                        });
                    }
                    let mut result = Vec::new();
                    for item in list.iter() {
                        let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
                        call_env.insert(params[0].0.clone(), item.clone());
                        let call_ctx =
                            call_context_from_env(std::sync::Arc::new(call_env), ctx, false);
                        result.push(eval_expr(body, &call_ctx)?);
                    }
                    Ok(Value::List(Box::new(result)))
                }
                (other, Value::Closure { params, body, env }) => {
                    // Try Cons/Nil variant representation
                    if params.len() != 1 {
                        return Err(EvalError::WrongArity {
                            expected: params.len(),
                            actual: 1,
                            callee: None,
                        });
                    }
                    let list = match crate::list_helpers::list_to_vec(other) {
                        Some(list) => list,
                        None => return Err(EvalError::TypeMismatch {
                            expected: "list, function".to_string(),
                            actual: format!("{:?}, {:?}", args[0], args[1]),
                        }),
                    };
                    let mut result = Vec::new();
                    for item in list.iter() {
                        let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
                        call_env.insert(params[0].0.clone(), item.clone());
                        let call_ctx =
                            call_context_from_env(std::sync::Arc::new(call_env), ctx, false);
                        result.push(eval_expr(body, &call_ctx)?);
                    }
                    Ok(Value::List(Box::new(result)))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "list, function".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }

        (_, "starts_with") => {
            if args.len() != 2 {
                return builtin_arity_error("starts_with", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string, string".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }

        (_, "ends_with") => {
            if args.len() != 2 {
                return builtin_arity_error("ends_with", 2, args.len());
            }
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err(EvalError::TypeMismatch {
                    expected: "string, string".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
            }
        }

        // Record operations
        (_, "keys") => {
            if args.len() != 1 {
                return builtin_arity_error("keys", 1, args.len());
            }
            match &args[0] {
                Value::Record(fields) => {
                    let keys: Vec<Value> =
                        fields.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::List(Box::new(keys)))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        (_, "values") => {
            if args.len() != 1 {
                return builtin_arity_error("values", 1, args.len());
            }
            match &args[0] {
                Value::Record(fields) => {
                    let values: Vec<Value> = fields.values().cloned().collect();
                    Ok(Value::List(Box::new(values)))
                }
                _ => Err(EvalError::TypeMismatch {
                    expected: "record".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
            }
        }

        // Type checking
        (_, "is_int") => {
            if args.len() != 1 {
                return builtin_arity_error("is_int", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::Int(_))))
        }

        (_, "is_string") => {
            if args.len() != 1 {
                return builtin_arity_error("is_string", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::String(_))))
        }

        (_, "is_bool") => {
            if args.len() != 1 {
                return builtin_arity_error("is_bool", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
        }

        (_, "is_list") => {
            if args.len() != 1 {
                return builtin_arity_error("is_list", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::List(_))))
        }

        (_, "is_record") => {
            if args.len() != 1 {
                return builtin_arity_error("is_record", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::Record(_))))
        }

        (_, "is_null") => {
            if args.len() != 1 {
                return builtin_arity_error("is_null", 1, args.len());
            }
            Ok(Value::Bool(matches!(args[0], Value::Null)))
        }

        // Record constructor
        (_, "record") => {
            let mut fields = HashMap::new();
            // Arguments come in pairs: key, value, key, value, ...
            if !args.len().is_multiple_of(2) {
                return Err(EvalError::ExecutionFailed(
                    "record requires even number of arguments (key, value pairs)".to_string(),
                ));
            }
            for i in (0..args.len()).step_by(2) {
                let key = match &args[i] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::TypeMismatch {
                            expected: "string".to_string(),
                            actual: format!("{:?}", args[i]),
                        });
                    }
                };
                fields.insert(key, args[i + 1].clone());
            }
            Ok(Value::Record(Box::new(fields)))
        }

        // JSON operations
        (Some("json"), "parse") => {
            if args.len() != 1 {
                return builtin_arity_error("json::parse", 1, args.len());
            }
            let text = expect_string_arg(args, 0, "string")?;
            json_parse(text)
        }
        (Some("json"), "stringify") => {
            if args.len() != 1 {
                return builtin_arity_error("json::stringify", 1, args.len());
            }
            let text = expect_string_arg(args, 0, "string")?;
            json_stringify(text)
        }
        (Some("json"), "stringify_pretty") => {
            if args.len() != 1 {
                return builtin_arity_error("json::stringify_pretty", 1, args.len());
            }
            let text = expect_string_arg(args, 0, "string")?;
            json_stringify_pretty(text)
        }

        // Markdown operations
        (Some("markdown"), "parse") => {
            if args.len() != 1 {
                return builtin_arity_error("markdown::parse", 1, args.len());
            }
            let text = expect_string_arg(args, 0, "string")?;
            markdown_parse(text)
        }

        // Unknown function
        _ => Err(EvalError::UnknownFunction(func.to_string())),
    }
}

fn expect_string_arg<'a>(args: &'a [Value], index: usize, expected: &str) -> EvalResult<&'a str> {
    match &args[index] {
        Value::String(s) => Ok(s),
        other => Err(EvalError::TypeMismatch {
            expected: expected.to_string(),
            actual: format!("{other:?}"),
        }),
    }
}

fn option_some(value: Value) -> Value {
    Value::Variant {
        name: "Some".to_string(),
        fields: Box::new(vec![("value".to_string(), value)]),
    }
}

fn option_none() -> Value {
    Value::Variant {
        name: "None".to_string(),
        fields: Box::new(vec![]),
    }
}

fn process_which_value(cmd: &str) -> Value {
    let command_path = std::path::Path::new(cmd);
    if command_path.components().count() > 1 && command_path.is_file() {
        return option_some(Value::String(cmd.to_string()));
    }
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|path| path.join(cmd))
                .find(|candidate| candidate.is_file())
        })
        .map_or_else(option_none, |path| {
            option_some(Value::String(path.display().to_string()))
        })
}

fn compile_regex(pattern: &str) -> EvalResult<regex::Regex> {
    regex::Regex::new(pattern)
        .map_err(|err| EvalError::ExecutionFailed(format!("Invalid regex pattern: {err}")))
}

fn regex_find(pattern: &str, text: &str) -> EvalResult<Value> {
    let regex = compile_regex(pattern)?;
    Ok(regex.find(text).map_or_else(
        || Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        },
        |matched| Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::String(matched.as_str().to_string()),
            )]),
        },
    ))
}

fn regex_matches(pattern: &str, text: &str) -> EvalResult<Value> {
    let regex = compile_regex(pattern)?;
    Ok(Value::Bool(regex.is_match(text)))
}

fn regex_replace(pattern: &str, replacement: &str, text: &str) -> EvalResult<Value> {
    let regex = compile_regex(pattern)?;
    Ok(Value::String(
        regex.replace_all(text, replacement).to_string(),
    ))
}

fn json_parse(text: &str) -> EvalResult<Value> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(_) => Ok(Value::String(text.to_string())),
        Err(e) => Err(EvalError::ExecutionFailed(format!("JSON parse error: {e}"))),
    }
}

fn json_stringify(text: &str) -> EvalResult<Value> {
    let val: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| EvalError::ExecutionFailed(format!("JSON parse error: {e}")))?;
    Ok(Value::String(serde_json::to_string(&val).map_err(|e| {
        EvalError::ExecutionFailed(format!("JSON stringify error: {e}"))
    })?))
}

fn json_stringify_pretty(text: &str) -> EvalResult<Value> {
    let val: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| EvalError::ExecutionFailed(format!("JSON parse error: {e}")))?;
    Ok(Value::String(serde_json::to_string_pretty(&val).map_err(
        |e| EvalError::ExecutionFailed(format!("JSON stringify error: {e}")),
    )?))
}

/// Parse CommonMark markdown text into a JSON AST string.
///
/// Returns a JSON string with the structure:
/// `{ "blocks": [ { "type": "heading", "level": 1, "text": "..." }, ... ] }`
fn markdown_parse(text: &str) -> EvalResult<Value> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

    let mut blocks: Vec<serde_json::Value> = Vec::new();
    let parser = Parser::new(text);

    let mut current_block: Option<serde_json::Value> = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_text.clear();
                let _ = level; // level is used on End
            }
            Event::End(TagEnd::Heading(level)) => {
                blocks.push(serde_json::json!({
                    "type": "heading",
                    "level": level as u8,
                    "text": current_text.trim()
                }));
                current_text.clear();
            }
            Event::Start(Tag::Paragraph) => {
                current_text.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                blocks.push(serde_json::json!({
                    "type": "paragraph",
                    "text": current_text.trim()
                }));
                current_text.clear();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    _ => String::new(),
                };
                current_block = Some(serde_json::json!({
                    "type": "code_block",
                    "language": lang,
                }));
                current_text.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(mut block) = current_block.take() {
                    block
                        .as_object_mut()
                        .expect("code_block should be a JSON object")
                        .insert(
                            "text".to_string(),
                            serde_json::Value::String(current_text.trim().to_string()),
                        );
                    blocks.push(block);
                }
                current_text.clear();
            }
            Event::Text(t) => {
                current_text.push_str(&t);
            }
            Event::Code(c) => {
                current_text.push_str(&c);
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    let result = serde_json::json!({ "blocks": blocks });
    let json_str = serde_json::to_string(&result)
        .map_err(|e| EvalError::ExecutionFailed(format!("markdown serialization error: {e}")))?;
    Ok(Value::String(json_str))
}

/// Return a WrongArity error for built-ins, preserving the expected arity.
fn builtin_arity_error(name: &str, expected: usize, actual: usize) -> EvalResult<Value> {
    Err(EvalError::WrongArity {
        expected,
        actual,
        callee: Some(name.to_string()),
    })
}

/// Apply a closure with exact arity.
fn apply_closure(
    params: &[(String, Option<String>)],
    body: &Expr,
    env: &std::sync::Arc<ash_core::env_frame::EnvFrame>,
    args: Vec<Value>,
    runtime_ctx: &Context,
) -> EvalResult<Value> {
    let validates_hidden_act_env = |params: &[(String, Option<String>)], args: &[Value]| {
        for ((name, _ty), val) in params.iter().zip(args.iter()) {
            if name == "__act_env" {
                if !matches!(val, Value::ActEnvToken) {
                    return Err(EvalError::TypeMismatch {
                        expected: "hidden ActEnv token".to_string(),
                        actual: format!("{val:?}"),
                    });
                }
                if runtime_ctx.act_env().is_none() {
                    return Err(EvalError::TypeMismatch {
                        expected: "hidden runtime ActEnv".to_string(),
                        actual: "missing runtime ActEnv".to_string(),
                    });
                }
            }
        }
        Ok(())
    };

    if args.len() == params.len() {
        validates_hidden_act_env(params, &args)?;
        let enters_effect_scope = params
            .iter()
            .take(args.len())
            .any(|(name, _ty)| name == "__act_env");
        let mut call_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
        for ((name, _ty), val) in params.iter().zip(args) {
            call_env.insert(name.clone(), val);
        }
        let call_ctx = call_context_from_env(
            std::sync::Arc::new(call_env),
            runtime_ctx,
            enters_effect_scope,
        );
        let result = eval_expr(body, &call_ctx)?;
        let result = maybe_execute_invoke_capture(result, &call_ctx)?;
        let result = maybe_execute_proc_await_capture(result, &call_ctx)?;
        maybe_execute_proc_yield_capture(result, &call_ctx)
    } else {
        Err(EvalError::WrongArity {
            expected: params.len(),
            actual: args.len(),
            callee: None,
        })
    }
}

#[cfg(test)]
mod tests;
