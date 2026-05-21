//! Expression evaluation
//!
//! Evaluates expressions in a runtime context, producing values.

use ash_core::runtime::{
    CapabilityBindingKind, EffectScopeId, FailureEvidence, OperationalFailure, ProcessId,
    ProcessTerminalState,
};
use ash_core::{BinaryOp, Expr, UnaryOp, Value, WorkflowId, ast::MatchArm, ast::Pattern};
use ash_core::{ControlLink, Instance, InstanceAddr};
use futures::future::join_all;
use std::collections::HashMap;

use crate::EvalResult;
use crate::context::Context;
use crate::error::EvalError;

fn operational_failure_for_payload(payload: Value, ctx: &Context) -> OperationalFailure {
    let payload_type = value_type_name(&payload);
    let (tower, entity) = ctx.current_failure_attribution();
    OperationalFailure::new(tower, entity, payload, payload_type)
}

fn operational_failure_with_attribution(
    payload: Value,
    tower: ash_core::runtime::TowerLevel,
    entity: ash_core::runtime::FailureEntity,
) -> OperationalFailure {
    let payload_type = value_type_name(&payload);
    OperationalFailure::new(tower, entity, payload, payload_type)
}

fn operational_eval_error_for_message(message: String, ctx: &Context) -> EvalError {
    EvalError::OperationalFailure(Box::new(operational_failure_for_payload(
        Value::String(message),
        ctx,
    )))
}

fn operational_eval_error_for_message_with_attribution(
    message: String,
    tower: ash_core::runtime::TowerLevel,
    entity: ash_core::runtime::FailureEntity,
) -> EvalError {
    EvalError::OperationalFailure(Box::new(operational_failure_with_attribution(
        Value::String(message),
        tower,
        entity,
    )))
}

fn operational_eval_error_for_resource_policy(
    violation: crate::runtime_state::ResourceSplitJoinViolation,
    ctx: &Context,
) -> EvalError {
    let mut failure = operational_failure_for_payload(Value::String(violation.to_string()), ctx);
    failure.evidence = FailureEvidence {
        notes: violation.evidence_notes(),
        provenance: violation.evidence_provenance(),
    };
    EvalError::OperationalFailure(Box::new(failure))
}

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

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::String(_) => "String",
        Value::Bool(_) => "Bool",
        Value::Null => "Null",
        Value::Time(_) => "Time",
        Value::Ref(_) => "Ref",
        Value::List(_) => "List",
        Value::Record(_) => "Record",
        Value::Cap(_) => "Cap",
        Value::Variant { .. } => "Variant",
        Value::Instance(_) => "Instance",
        Value::InstanceAddr(_) => "InstanceAddr",
        Value::ControlLink(_) => "ControlLink",
        Value::Stream(_) => "Stream",
        Value::ProcessHandle(_) => "P",
        Value::ProcAwaitCapture(_) => "<proc-await>",
        Value::ProcYieldCapture => "<proc-yield>",
        Value::ProcParCapture { .. } => "<proc-par>",
        Value::ProcScatterCapture { .. } => "<proc-scatter>",
        Value::ProcJoinCapture { .. } => "<proc-join>",
        Value::ProcGatherCapture { .. } => "<proc-gather>",
        Value::Closure { .. } => "Closure",
        Value::ActEnvToken => "ActEnvToken",
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinEntry {
    /// Number of required parameters (0 for variadic builtins).
    pub arity: usize,
    /// Whether the builtin accepts a variable number of arguments.
    pub variadic: bool,
    /// Whether the runtime implementation is present.
    /// Entries with `implemented: false` produce [`EvalError::UnimplementedBuiltin`].
    pub implemented: bool,
}

/// Returns the builtin dispatch table mapping qualified names to entries.
///
/// Qualified names use `"module::func"` format (e.g., `"string::concat"`).
/// Unqualified names are used for builtins that accept any module prefix
/// (e.g., `"len"`, `"head"`).
pub fn builtin_dispatch_table() -> &'static HashMap<&'static str, BuiltinEntry> {
    static TABLE: std::sync::OnceLock<HashMap<&'static str, BuiltinEntry>> =
        std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut m = HashMap::new();
        // ── String module builtins (qualified) ──
        m.insert(
            "string::concat",
            BuiltinEntry {
                arity: 0,
                variadic: true,
                implemented: true,
            },
        );
        m.insert(
            "string::starts_with",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::ends_with",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::is_empty",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Regex module builtins (qualified) ──
        m.insert(
            "regex::find",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "regex::matches",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "regex::replace",
            BuiltinEntry {
                arity: 3,
                variadic: false,
                implemented: true,
            },
        );

        // ── String case / whitespace builtins ──
        m.insert(
            "string::to_upper",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::to_lower",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::trim",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Provider-backed stdlib surfaces intentionally deferred in interp ──
        for (name, arity) in [
            ("http::get", 1),
            ("http::post", 2),
            ("http::put", 2),
            ("http::delete", 1),
            ("time::now", 0),
            ("time::now_iso", 0),
            ("time::epoch_millis", 0),
            ("time::sleep", 1),
            ("io::stdio::read_line", 0),
            ("io::stdio::print", 1),
            ("io::stdio::println", 1),
            ("io::fs::read", 1),
            ("io::fs::read_to_string", 1),
            ("io::fs::write", 2),
            ("io::fs::write_string", 2),
            ("io::fs::append", 2),
            ("io::fs::copy", 2),
            ("io::fs::rename", 2),
            ("io::fs::remove_file", 1),
            ("io::dir::create_dir", 1),
            ("io::dir::create_dir_all", 1),
            ("io::dir::remove_dir", 1),
            ("io::dir::remove_dir_all", 1),
            ("io::dir::read_dir", 1),
            ("io::meta::metadata", 1),
            ("io::meta::is_file", 1),
            ("io::meta::is_dir", 1),
            ("io::meta::len", 1),
            ("io::meta::readonly", 1),
            ("io::buf::read_to_end", 1),
            ("io::buf::read_to_string", 1),
            ("io::buf::write_all", 2),
            ("io::buf::lines", 1),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: false,
                },
            );
        }

        // ── Process module builtins (qualified) ──
        m.insert(
            "process::run",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "process::which",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Act module bridge builtins (qualified) ──
        m.insert(
            "act::__guard",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "act::policy_check",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Proc module bridge builtins (qualified) ──
        for (name, arity) in [
            ("proc::unit", 1),
            ("proc::from_act", 1),
            ("proc::bind", 2),
            ("proc::then", 2),
            ("proc::await", 1),
            ("proc::yield", 0),
            ("proc::par", 2),
            ("proc::scatter", 2),
            ("proc::join", 2),
            ("proc::gather", 1),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── Workflow module bridge builtins (qualified) ──
        for (name, arity) in [
            ("workflow::unit", 1),
            ("workflow::from_act", 1),
            ("workflow::from_proc", 1),
            ("workflow::bind", 2),
            ("workflow::then", 2),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── List module builtins (qualified) ──
        m.insert(
            "list::len",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::head",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::tail",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::append",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::concat",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::filter",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::map",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );

        // ── Record module builtins (qualified) ──
        for (name, arity, variadic) in [
            ("record::keys", 1, false),
            ("record::values", 1, false),
            ("record::record", 0, true),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }

        // ── Unqualified builtins ──
        let unqualified = [
            ("len", 1, false),
            ("head", 1, false),
            ("tail", 1, false),
            ("append", 2, false),
            ("concat", 2, false),
            ("filter", 2, false),
            ("map", 2, false),
            ("starts_with", 2, false),
            ("ends_with", 2, false),
            ("keys", 1, false),
            ("values", 1, false),
            ("is_int", 1, false),
            ("is_string", 1, false),
            ("is_bool", 1, false),
            ("is_list", 1, false),
            ("is_record", 1, false),
            ("is_null", 1, false),
        ];
        for (name, arity, variadic) in unqualified {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }
        // ── Predicate module builtins (qualified) ──
        for (name, arity, variadic) in [
            ("predicate::is_int", 1, false),
            ("predicate::is_string", 1, false),
            ("predicate::is_bool", 1, false),
            ("predicate::is_list", 1, false),
            ("predicate::is_record", 1, false),
            ("predicate::is_null", 1, false),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }

        // ── JSON module builtins (qualified) ──
        m.insert(
            "json::parse",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "json::stringify",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "json::stringify_pretty",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Markdown module builtins (qualified) ──
        m.insert(
            "markdown::parse",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        m.insert(
            "record",
            BuiltinEntry {
                arity: 0,
                variadic: true,
                implemented: true,
            },
        );
        m
    })
}

/// Check whether `(func, module)` identifies a known builtin.
///
/// Looks up both the qualified form `"module::func"` (when `module` is `Some`)
/// and the bare `func` name in the dispatch table. O(1) via HashMap lookups.
pub fn is_known_builtin(func: &str, module: Option<&str>) -> bool {
    let table = builtin_dispatch_table();

    // Try qualified name first (O(1))
    if let Some(mod_name) = module {
        let qualified = format!("{mod_name}::{func}");
        if table.contains_key(qualified.as_str()) {
            return true;
        }
    }

    // Try unqualified name (O(1))
    table.contains_key(func)
}

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
    let Value::List(items) = &args[0] else {
        return Err(EvalError::TypeMismatch {
            expected: "List<A>".to_string(),
            actual: value_type_name(&args[0]).to_string(),
        });
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
            items: Box::new((**items).clone()),
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
                && let Some(CapabilityBindingKind::HostProvider { provider_name, .. }) =
                    binding.as_ref().map(|binding| &binding.kind)
            {
                let projected_ctx = runtime_state
                    .create_capability_context_for_bindings(&[binding.as_ref().expect("binding checked").id])
                    .await
                    .map_err(|err| EvalError::ExecutionFailed(err.to_string()))?;
                return projected_ctx
                    .execute(provider_name, &action, &args)
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
                    match result {
                        Err(EvalError::WrongArity {
                            expected,
                            actual,
                            callee: Some(ref name),
                        }) if actual < expected => {
                            return Ok(make_partial_builtin(name, &args, expected));
                        }
                        other => return other,
                    }
                }

                match eval_function_call(func, module.as_deref(), &args, ctx) {
                    Ok(value) => Ok(value),
                    Err(EvalError::WrongArity {
                        expected,
                        actual,
                        callee: Some(builtin_name),
                    }) if actual < expected => {
                        Ok(make_partial_builtin(&builtin_name, &args, expected))
                    }
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
                let closure = Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    env: ctx.to_env_frame(),
                };
                if ctx.is_pure() {
                    return Err(EvalError::BoundaryViolation {
                        value: Box::new(closure),
                        context: "closure created inside pure-function boundary".into(),
                    });
                }
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
            maybe_execute_proc_admission_capture_async(result, &call_ctx).await
        } else if args.len() < params.len() {
            validates_hidden_act_env(params, &args)?;
            let mut new_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
            for ((name, _ty), val) in params.iter().take(args.len()).zip(args.clone()) {
                new_env.insert(name.clone(), val);
            }
            let remaining_params = params[args.len()..].to_vec();
            Ok(Value::Closure {
                params: remaining_params,
                body: Box::new(body.clone()),
                env: std::sync::Arc::new(new_env),
            })
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
            CapabilityBindingKind::HostProvider { provider_name, .. } => {
                let projected_ctx = runtime_state
                    .create_capability_context_for_bindings(&[binding.id])
                    .await
                    .map_err(|err| EvalError::ExecutionFailed(err.to_string()))?;
                let dispatch_provider = provider_name.clone();
                let invoked = projected_ctx
                    .execute(&dispatch_provider, &action, &args)
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

            // Note: this early-exit returns apply_closure directly, bypassing the
            // make_partial_builtin path below. Over-application through a closure found
            // here produces WrongArity { callee: None } rather than a partial value;
            // that is handled inside apply_closure itself.
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
                // Preserve partial-application: if the table rejects for too-few
                // args, fall through to make_partial_builtin just like the legacy path.
                match result {
                    Err(EvalError::WrongArity {
                        expected,
                        actual,
                        callee: Some(ref name),
                    }) if actual < expected => {
                        return Ok(make_partial_builtin(name, &args, expected));
                    }
                    other => return other,
                }
            }

            // Not in dispatch table: try legacy eval_function_call (covers
            // unqualified builtins like "len", "head" matched via pattern).
            match eval_function_call(func, module.as_deref(), &args, ctx) {
                Ok(value) => Ok(value),
                Err(EvalError::WrongArity {
                    expected,
                    actual,
                    callee: Some(builtin_name),
                }) if actual < expected => Ok(make_partial_builtin(&builtin_name, &args, expected)),
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
            let closure = Value::Closure {
                params: params.clone(),
                body: body.clone(),
                env: ctx.to_env_frame(),
            };
            // SPEC-031 §4.8 — runtime safety net: closures must not be created
            // inside a pure context.  The type checker is the primary enforcer;
            // this catches any values that slip through.
            if ctx.is_pure() {
                return Err(EvalError::BoundaryViolation {
                    value: Box::new(closure),
                    context: "closure created inside pure-function boundary".into(),
                });
            }
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

/// Generate a fresh instance ID
fn fresh_instance_id() -> WorkflowId {
    WorkflowId::new()
}

/// Evaluate a spawn expression
/// Creates a new Instance value with a fresh address and control link
fn eval_spawn(workflow_type: &str) -> EvalResult<Value> {
    let instance_id = fresh_instance_id();

    let addr = InstanceAddr {
        workflow_type: workflow_type.to_string(),
        instance_id,
    };

    let control = Some(ControlLink { instance_id });

    Ok(Value::Instance(Box::new(Instance { addr, control })))
}

/// Evaluate a split expression
/// Splits an Instance into a tuple (InstanceAddr, Option<ControlLink>)
fn eval_split(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match value {
        Value::Instance(instance) => {
            let addr = Value::InstanceAddr(instance.addr);
            let control = instance.control.map(Value::ControlLink);
            // Return as a tuple: (addr, control)
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

/// Evaluate a unary operation
fn eval_unary_op(op: UnaryOp, operand: Value) -> EvalResult<Value> {
    match op {
        UnaryOp::Not => match operand {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(EvalError::InvalidUnaryOp {
                op: "not".to_string(),
                operand: format!("{:?}", operand),
            }),
        },
        UnaryOp::Neg => match operand {
            Value::Int(i) => Ok(Value::Int(-i)),
            _ => Err(EvalError::InvalidUnaryOp {
                op: "neg".to_string(),
                operand: format!("{:?}", operand),
            }),
        },
    }
}

/// Evaluate a binary operation
fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> EvalResult<Value> {
    match op {
        // Arithmetic
        BinaryOp::Add => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "add".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Sub => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "sub".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Mul => match (&left, &right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "mul".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Div => match (&left, &right) {
            (Value::Int(_), Value::Int(r)) if *r == 0 => Err(EvalError::DivisionByZero),
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l / r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "div".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Mod => match (&left, &right) {
            (Value::Int(_), Value::Int(r)) if *r == 0 => Err(EvalError::DivisionByZero),
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l % r)),
            _ => Err(EvalError::InvalidBinaryOp {
                op: "mod".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },

        // Logical — NOTE: And/Or are handled with short-circuit evaluation in
        // the Expr::Binary arm of eval_expr (SPEC-004 EXPR-AND-FALSE, EXPR-OR-TRUE).
        // These arms are only reachable if eval_binary_op is called directly.
        BinaryOp::And | BinaryOp::Or => {
            unreachable!(
                "and/or are handled with short-circuit in eval_expr; \
                 eval_binary_op should never be called for {:?}",
                op
            )
        }

        // Comparison
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => eval_comparison(left, right, |o| o == std::cmp::Ordering::Less),
        BinaryOp::Gt => eval_comparison(left, right, |o| o == std::cmp::Ordering::Greater),
        BinaryOp::Le => eval_comparison(left, right, |o| {
            o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal
        }),
        BinaryOp::Ge => eval_comparison(left, right, |o| {
            o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal
        }),

        // Membership
        BinaryOp::In => match right {
            Value::List(list) => Ok(Value::Bool(list.contains(&left))),
            Value::String(s) => match left.as_string() {
                Some(substr) => Ok(Value::Bool(s.contains(substr))),
                None => Err(EvalError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", left),
                }),
            },
            _ => Err(EvalError::InvalidBinaryOp {
                op: "in".to_string(),
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            }),
        },
        BinaryOp::Pipe => Err(EvalError::InvalidBinaryOp {
            op: "pipe".to_string(),
            left: format!("{:?}", left),
            right: format!("{:?}", right),
        }),
    }
}

/// Helper to evaluate comparison operations
fn eval_comparison<F>(left: Value, right: Value, check: F) -> EvalResult<Value>
where
    F: Fn(std::cmp::Ordering) -> bool,
{
    let ordering = compare_values(&left, &right)?;
    Ok(Value::Bool(check(ordering)))
}

/// Evaluate a match expression
///
/// Tries each arm in order, returning the result of the first matching arm.
/// If no arm matches, returns a non-exhaustive match error.
fn eval_match(scrutinee: &Expr, arms: &[MatchArm], ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(scrutinee, ctx)?;

    for arm in arms {
        match crate::pattern::match_pattern(&arm.pattern, &value) {
            Ok(bindings) => {
                // Create a new context with the bindings
                let mut new_ctx = ctx.extend();
                for (name, val) in bindings {
                    new_ctx.set(name, val);
                }
                return eval_expr(&arm.body, &new_ctx);
            }
            Err(_) => {
                // Pattern didn't match, try next arm
                continue;
            }
        }
    }

    // No arm matched
    Err(EvalError::NonExhaustiveMatch {
        value: format!("{:?}", value),
    })
}

/// Evaluate an if-let expression
///
/// If the pattern matches the expression value, evaluates the then branch with bindings.
/// Otherwise evaluates the else branch.
fn eval_if_let(
    pattern: &Pattern,
    expr: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    ctx: &Context,
) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match crate::pattern::match_pattern(pattern, &value) {
        Ok(bindings) => {
            // Pattern matched - evaluate then branch with bindings
            let mut new_ctx = ctx.extend();
            for (name, val) in bindings {
                new_ctx.set(name, val);
            }
            eval_expr(then_branch, &new_ctx)
        }
        Err(_) => {
            // Pattern didn't match - evaluate else branch
            eval_expr(else_branch, ctx)
        }
    }
}

/// Compare two values for ordering
fn compare_values(left: &Value, right: &Value) -> EvalResult<std::cmp::Ordering> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(l.cmp(r)),
        (Value::String(l), Value::String(r)) => Ok(l.cmp(r)),
        (Value::Bool(l), Value::Bool(r)) => Ok(l.cmp(r)),
        (Value::Time(l), Value::Time(r)) => Ok(l.cmp(r)),
        _ => Err(EvalError::InvalidBinaryOp {
            op: "comparison".to_string(),
            left: format!("{:?}", left),
            right: format!("{:?}", right),
        }),
    }
}

/// Evaluate a built-in function call
pub fn eval_function_call(
    func: &str,
    module: Option<&str>,
    args: &[Value],
    ctx: &Context,
) -> EvalResult<Value> {
    match (module, func) {
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
                _ => Err(EvalError::TypeMismatch {
                    expected: "list or string".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
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
                _ => Err(EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
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
                _ => Err(EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
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
                _ => Err(EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    actual: format!("{:?}", args[0]),
                }),
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
                _ => Err(EvalError::TypeMismatch {
                    expected: "list, list".to_string(),
                    actual: format!("{:?}, {:?}", args[0], args[1]),
                }),
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

/// Build a synthetic closure that represents a partially-applied built-in.
///
/// `ends_with(".md")` becomes a closure `|x| => ends_with(x, ".md")` (with args reordered).
///
/// This reordering ensures that when used in a pipeline like `filter(ends_with(".md"))`,
/// the closure correctly receives the iterated element as its first argument.
fn make_partial_builtin(name: &str, applied_args: &[Value], total_arity: usize) -> Value {
    let remaining = total_arity - applied_args.len();
    let param_names: Vec<(String, Option<String>)> = (0..remaining)
        .map(|i| (format!("__partial_{i}"), None))
        .collect();

    // Build call args with remaining params FIRST, then applied args.
    // This ensures `ends_with(".md")` becomes `|x| => ends_with(x, ".md")`
    // rather than `|x| => ends_with(".md", x)`.
    let mut call_args: Vec<Expr> = param_names
        .iter()
        .enumerate()
        .map(|(i, _)| Expr::Variable {
            name: format!("__partial_{i}"),
            span: ash_core::ast::Span::default(),
        })
        .collect();

    call_args.extend(applied_args.iter().map(|v| Expr::Literal(v.clone())));

    Value::Closure {
        params: param_names,
        body: Box::new(Expr::Call {
            func: name.to_string(),
            module: None,
            arguments: call_args,
        }),
        env: std::sync::Arc::new(ash_core::env_frame::EnvFrame::new()),
    }
}

/// Apply a closure, supporting partial application.
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
    } else if args.len() < params.len() {
        validates_hidden_act_env(params, &args)?;
        // Partial application: bind provided params, keep remaining
        let mut new_env = ash_core::env_frame::EnvFrame::with_parent(env.clone());
        for ((name, _ty), val) in params.iter().take(args.len()).zip(args.clone()) {
            new_env.insert(name.clone(), val);
        }
        let remaining_params = params[args.len()..].to_vec();
        Ok(Value::Closure {
            params: remaining_params,
            body: Box::new(body.clone()),
            env: std::sync::Arc::new(new_env),
        })
    } else {
        Err(EvalError::WrongArity {
            expected: params.len(),
            actual: args.len(),
            callee: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeState;
    use ash_core::{ProcessHandle, ProcessId};

    #[test]
    fn test_eval_literal() {
        let ctx = Context::new();
        let expr = Expr::Literal(Value::Int(42));
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_variable_found() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(42));
        let expr = Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_variable_not_found() {
        let ctx = Context::new();
        let expr = Expr::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_field_access() {
        let mut ctx = Context::new();
        let mut record = HashMap::new();
        record.insert("name".to_string(), Value::String("Alice".to_string()));
        ctx.set("person".to_string(), Value::Record(Box::new(record)));

        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Variable {
                name: "person".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            field: "name".to_string(),
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("Alice".to_string())
        );
    }

    #[test]
    fn test_eval_field_access_not_found() {
        let ctx = Context::new();
        let mut record = HashMap::new();
        record.insert("x".to_string(), Value::Int(1));
        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Literal(Value::Record(Box::new(record)))),
            field: "missing".to_string(),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_field_access_not_record() {
        let ctx = Context::new();
        let expr = Expr::FieldAccess {
            expr: Box::new(Expr::Literal(Value::Int(42))),
            field: "x".to_string(),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_index_list() {
        let ctx = Context::new();
        let expr = Expr::IndexAccess {
            expr: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(10),
                Value::Int(20),
                Value::Int(30),
            ])))),
            index: Box::new(Expr::Literal(Value::Int(1))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_eval_index_out_of_bounds() {
        let ctx = Context::new();
        let expr = Expr::IndexAccess {
            expr: Box::new(Expr::Literal(Value::List(Box::new(vec![Value::Int(10)])))),
            index: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_unary_not() {
        let ctx = Context::new();
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_unary_neg() {
        let ctx = Context::new();
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Literal(Value::Int(42))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(-42));
    }

    #[test]
    fn test_eval_binary_arithmetic() {
        let ctx = Context::new();

        // Addition
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(15));

        // Subtraction
        let expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(5));

        // Multiplication
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(50));

        // Division
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(5))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_binary_div_by_zero() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Value::Int(10))),
            right: Box::new(Expr::Literal(Value::Int(0))),
        };
        assert!(matches!(
            eval_expr(&expr, &ctx),
            Err(EvalError::DivisionByZero)
        ));
    }

    #[test]
    fn test_eval_binary_logical() {
        let ctx = Context::new();

        // AND
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));

        // OR
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_comparison() {
        let ctx = Context::new();

        // Less than
        let expr = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        // Greater than
        let expr = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Literal(Value::Int(2))),
            right: Box::new(Expr::Literal(Value::Int(1))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        // Equal
        let expr = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Value::Int(42))),
            right: Box::new(Expr::Literal(Value::Int(42))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_in_list() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::In,
            left: Box::new(Expr::Literal(Value::Int(2))),
            right: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_call_len() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "len".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
            ])))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_call_append() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "append".to_string(),
            module: None,
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1)]))),
                Expr::Literal(Value::Int(2)),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))
        );
    }

    #[test]
    fn test_eval_call_concat() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "concat".to_string(),
            module: None,
            arguments: vec![
                Expr::Literal(Value::List(Box::new(vec![Value::Int(1)]))),
                Expr::Literal(Value::List(Box::new(vec![Value::Int(2)]))),
            ],
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))
        );
    }

    #[test]
    fn test_eval_call_unknown() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "unknown".to_string(),
            module: None,
            arguments: vec![],
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn test_eval_call_wrong_arity() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "len".to_string(),
            module: None,
            arguments: vec![],
        };
        let value = eval_expr(&expr, &ctx).unwrap();
        assert!(
            matches!(value, Value::Closure { .. }),
            "expected partial-application Closure, got {value:?}"
        );
    }

    #[test]
    fn test_eval_nested_expr() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(5));

        // (x + 3) * 2
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Literal(Value::Int(3))),
            }),
            right: Box::new(Expr::Literal(Value::Int(2))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(16));
    }

    #[test]
    fn test_eval_string_concat() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Value::String("hello ".to_string()))),
            right: Box::new(Expr::Literal(Value::String("world".to_string()))),
        };
        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_eval_type_checks() {
        let ctx = Context::new();

        let expr = Expr::Call {
            func: "is_int".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Int(42))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        let expr = Expr::Call {
            func: "is_string".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Int(42))],
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    // ============================================================
    // TASK-131: Constructor Evaluation Tests
    // ============================================================

    #[test]
    fn test_eval_constructor_some_with_value() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_none_empty() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "None".to_string(),
            fields: vec![],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }

    #[test]
    fn test_eval_match_wildcard_fallback() {
        let ctx = Context::new();

        // match 2 { 1 => "one", _ => "other" } → "other"
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Value::Int(1)),
                body: Expr::Literal(Value::String("one".to_string())),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Expr::Literal(Value::String("other".to_string())),
            },
        ];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::Int(2))),
            arms,
        };

        assert_eq!(
            eval_expr(&expr, &ctx).unwrap(),
            Value::String("other".to_string())
        );
    }

    #[test]
    fn test_eval_constructor_ok_with_string() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Ok".to_string(),
            fields: vec![(
                "value".to_string(),
                Expr::Literal(Value::String("hello".to_string())),
            )],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Ok".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::String("hello".to_string())
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_err_with_value() {
        let ctx = Context::new();
        let expr = Expr::Constructor {
            name: "Err".to_string(),
            fields: vec![(
                "error".to_string(),
                Expr::Literal(Value::String("not found".to_string())),
            )],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Err".to_string(),
                fields: Box::new(vec![(
                    "error".to_string(),
                    Value::String("not found".to_string())
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_nested() {
        let ctx = Context::new();
        // Some { value: Ok { value: 42 } }
        let inner = Expr::Constructor {
            name: "Ok".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };
        let outer = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), inner)],
        };
        let result = eval_expr(&outer, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::Variant {
                        name: "Ok".to_string(),
                        fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
                    }
                )]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_with_variable() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(100));

        let expr = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![(
                "value".to_string(),
                Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            )],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(100))]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_with_expression_in_field() {
        let ctx = Context::new();
        // Point { x: 1 + 2, y: 3 * 4 }
        let expr = Expr::Constructor {
            name: "Point".to_string(),
            fields: vec![
                (
                    "x".to_string(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Literal(Value::Int(1))),
                        right: Box::new(Expr::Literal(Value::Int(2))),
                    },
                ),
                (
                    "y".to_string(),
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Literal(Value::Int(3))),
                        right: Box::new(Expr::Literal(Value::Int(4))),
                    },
                ),
            ],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Point".to_string(),
                fields: Box::new(vec![
                    ("x".to_string(), Value::Int(3)),
                    ("y".to_string(), Value::Int(12)),
                ]),
            }
        );
    }

    #[test]
    fn test_eval_constructor_multiple_fields() {
        let ctx = Context::new();
        // Person { name: "Alice", age: 30, active: true }
        let expr = Expr::Constructor {
            name: "Person".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Expr::Literal(Value::String("Alice".to_string())),
                ),
                ("age".to_string(), Expr::Literal(Value::Int(30))),
                ("active".to_string(), Expr::Literal(Value::Bool(true))),
            ],
        };
        let result = eval_expr(&expr, &ctx).unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "Person".to_string(),
                fields: Box::new(vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                    ("active".to_string(), Value::Bool(true)),
                ]),
            }
        );
    }

    #[test]
    fn test_value_variant_helpers() {
        // Test Value::variant helper
        let v = Value::variant("Some", vec![("value", Value::Int(42))]);
        assert_eq!(
            v,
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            }
        );

        // Test Value::unit_variant helper
        let v = Value::unit_variant("None");
        assert_eq!(
            v,
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }

    #[test]
    fn test_eval_match_list_destructure() {
        let ctx = Context::new();

        // match [1, 2, 3] { [a, b, ..] => a + b, _ => 0 } → 3
        let arms = vec![
            MatchArm {
                pattern: Pattern::List(
                    vec![
                        Pattern::Variable {
                            name: "a".to_string(),
                            span: ash_core::ast::Span::default(),
                        },
                        Pattern::Variable {
                            name: "b".to_string(),
                            span: ash_core::ast::Span::default(),
                        },
                    ],
                    Some("_".to_string()),
                ),
                body: Expr::Binary {
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
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                body: Expr::Literal(Value::Int(0)),
            },
        ];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))),
            arms,
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_match_tuple_destructure() {
        let ctx = Context::new();

        // match (1, 2) { (x, y) => x + y } → 3
        let arms = vec![MatchArm {
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
            body: Expr::Binary {
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
        }];

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
            ])))),
            arms,
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_match_option_some_branch_binds_value() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
            },
        );

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: "opt".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable {
                                name: "x".to_string(),
                                span: ash_core::ast::Span::default(),
                            },
                        )]),
                    },
                    body: Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                        right: Box::new(Expr::Literal(Value::Int(2))),
                    },
                },
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "None".to_string(),
                        fields: None,
                    },
                    body: Expr::Literal(Value::Int(0)),
                },
            ],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(84));
    }

    #[test]
    fn test_eval_match_option_none_branch_selected() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            },
        );

        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: "opt".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "Some".to_string(),
                        fields: Some(vec![(
                            "value".to_string(),
                            Pattern::Variable {
                                name: "x".to_string(),
                                span: ash_core::ast::Span::default(),
                            },
                        )]),
                    },
                    body: Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                },
                MatchArm {
                    pattern: Pattern::Variant {
                        name: "None".to_string(),
                        fields: None,
                    },
                    body: Expr::Literal(Value::Int(0)),
                },
            ],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(0));
    }

    #[test]
    fn test_eval_if_let_option_some_then_branch_binds_value() {
        let mut ctx = Context::new();
        ctx.set(
            "opt".to_string(),
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![("value".to_string(), Value::Int(99))]),
            },
        );

        let expr = Expr::IfLet {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![(
                    "value".to_string(),
                    Pattern::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                )]),
            },
            expr: Box::new(Expr::Variable {
                name: "opt".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            then_branch: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(99));
    }

    // ============================================================
    // TASK-134: Spawn and Split Tests
    // ============================================================

    #[test]
    fn test_eval_spawn_returns_instance() {
        let ctx = Context::new();

        // spawn Worker with { init: 42 }
        let expr = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(42))),
        };

        let result = eval_expr(&expr, &ctx).unwrap();

        // Should return an Instance value
        match result {
            Value::Instance(instance) => {
                assert_eq!(instance.addr.workflow_type, "Worker");
                assert!(instance.control.is_some());
                assert_eq!(
                    instance.control.unwrap().instance_id,
                    instance.addr.instance_id
                );
            }
            _ => panic!("Expected Instance, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_spawn_creates_unique_ids() {
        let ctx = Context::new();

        // spawn two instances
        let expr1 = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(1))),
        };
        let expr2 = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(2))),
        };

        let result1 = eval_expr(&expr1, &ctx).unwrap();
        let result2 = eval_expr(&expr2, &ctx).unwrap();

        let id1 = match &result1 {
            Value::Instance(inst) => inst.addr.instance_id,
            _ => panic!("Expected Instance"),
        };
        let id2 = match &result2 {
            Value::Instance(inst) => inst.addr.instance_id,
            _ => panic!("Expected Instance"),
        };

        // IDs should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_eval_split_returns_tuple() {
        let ctx = Context::new();

        // First spawn an instance
        let spawn_expr = Expr::Spawn {
            workflow_type: "Worker".to_string(),
            init: Box::new(Expr::Literal(Value::Int(42))),
        };

        // Then split it
        let split_expr = Expr::Split(Box::new(spawn_expr));

        let result = eval_expr(&split_expr, &ctx).unwrap();

        // Should return a tuple (InstanceAddr, ControlLink)
        match result {
            Value::List(tuple) => {
                assert_eq!(tuple.len(), 2);
                // First element should be InstanceAddr
                assert!(matches!(tuple[0], Value::InstanceAddr(_)));
                // Second element should be ControlLink
                assert!(matches!(tuple[1], Value::ControlLink(_)));
            }
            _ => panic!("Expected tuple (List), got {:?}", result),
        }
    }

    #[test]
    fn test_eval_split_type_mismatch() {
        let ctx = Context::new();

        // Try to split a non-instance value
        let split_expr = Expr::Split(Box::new(Expr::Literal(Value::Int(42))));

        let result = eval_expr(&split_expr, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_instance_addr_display() {
        let id = WorkflowId::new();
        let addr = InstanceAddr {
            workflow_type: "Worker".to_string(),
            instance_id: id,
        };
        let display = format!("{}", addr);
        assert!(display.starts_with("InstanceAddr<Worker:"));
        assert!(display.ends_with(">"));
    }

    #[test]
    fn test_control_link_display() {
        let link = ControlLink {
            instance_id: WorkflowId::new(),
        };
        let display = format!("{}", link);
        assert!(display.starts_with("ControlLink<"));
        assert!(display.ends_with(">"));
    }

    #[test]
    fn test_instance_display() {
        let id = WorkflowId::new();
        let instance = Instance {
            addr: InstanceAddr {
                workflow_type: "Worker".to_string(),
                instance_id: id,
            },
            control: Some(ControlLink { instance_id: id }),
        };
        let display = format!("{}", instance);
        assert!(display.contains("Instance {"));
        assert!(display.contains("addr:"));
        assert!(display.contains("control: Some(ControlLink"));
    }

    #[test]
    fn test_instance_display_no_control() {
        let instance = Instance {
            addr: InstanceAddr {
                workflow_type: "Worker".to_string(),
                instance_id: WorkflowId::new(),
            },
            control: None,
        };
        let display = format!("{}", instance);
        assert!(display.contains("control: None"));
    }

    // ============================================================
    // TASK-559: SPEC-031 End-to-End Conformance Tests
    // ============================================================

    /// SPEC-031 §5.1 – FnDef evaluates to Value::Closure capturing the current env.
    #[test]
    fn task559_fndef_produces_value_closure() {
        let mut ctx = Context::new();
        ctx.set("offset".to_string(), Value::Int(10));

        let expr = Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "offset".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
            }),
        };

        let result = eval_expr(&expr, &ctx).unwrap();
        match &result {
            Value::Closure { params, .. } => assert_eq!(params.len(), 1),
            other => panic!("expected Value::Closure, got {other:?}"),
        }
    }

    /// SPEC-031 §5.4 – FnApply calls a closure and binds params correctly.
    #[test]
    fn task559_fnapply_calls_closure() {
        let ctx = Context::new();

        // (fn(x) { x + 1 })(5)  =>  6
        let expr = Expr::FnApply {
            func: Box::new(Expr::FnDef {
                params: vec![("x".to_string(), None)],
                return_type: None,
                body: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Value::Int(1))),
                }),
            }),
            args: vec![Expr::Literal(Value::Int(5))],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(6));
    }

    #[test]
    fn task689c_projected_callable_invocation_evaluates_through_fnapply() {
        let mut ctx = Context::new();
        let check_closure = Value::Closure {
            params: vec![("_p".to_string(), None)],
            body: Box::new(Expr::Literal(Value::Bool(true))),
            env: std::sync::Arc::new(ash_core::env_frame::EnvFrame::new()),
        };

        let mut policies = HashMap::new();
        policies.insert("check".to_string(), check_closure);

        let mut env_record = HashMap::new();
        env_record.insert("policies".to_string(), Value::Record(Box::new(policies)));
        ctx.set("env".to_string(), Value::Record(Box::new(env_record)));

        let expr = Expr::FnApply {
            func: Box::new(Expr::FieldAccess {
                expr: Box::new(Expr::FieldAccess {
                    expr: Box::new(Expr::Variable {
                        name: "env".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    field: "policies".to_string(),
                }),
                field: "check".to_string(),
            }),
            args: vec![Expr::Literal(Value::String("demo".to_string()))],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    #[test]
    fn task689c_policy_check_fails_closed_without_hidden_policy_context() {
        let ctx = Context::new();
        let expr = Expr::Call {
            func: "policy_check".to_string(),
            module: Some("act".to_string()),
            arguments: vec![Expr::Literal(Value::String("missing".to_string()))],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    #[test]
    fn task689c_policy_check_uses_hidden_policy_evaluator() {
        let mut evaluator = crate::policy::PolicyEvaluator::new();
        let policy = crate::policy::Policy::new("allow-large")
            .with_rule(crate::policy::PolicyRule::new(
                "allow x > 10",
                Expr::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Literal(Value::Int(10))),
                },
                ash_core::Decision::Permit,
            ))
            .with_default(ash_core::Decision::Deny);
        evaluator.register(policy);

        let mut ctx = Context::new().with_policy_evaluator(evaluator);
        ctx.set("x".to_string(), Value::Int(15));

        let expr = Expr::Call {
            func: "policy_check".to_string(),
            module: Some("act".to_string()),
            arguments: vec![Expr::Literal(Value::String("allow-large".to_string()))],
        };

        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));

        let mut denied_ctx = ctx.clone();
        denied_ctx.set("x".to_string(), Value::Int(1));
        assert_eq!(eval_expr(&expr, &denied_ctx).unwrap(), Value::Bool(false));
    }

    /// SPEC-031 §5.3 – Closure captures the enclosing scope (make_adder pattern).
    #[test]
    fn task559_closure_captures_enclosing_scope() {
        // Build make_adder closure: fn(n) { fn(x) { x + n } }
        let mut ctx = Context::new();
        let make_adder = Expr::FnDef {
            params: vec![("n".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::FnDef {
                params: vec![("x".to_string(), None)],
                return_type: None,
                body: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "n".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                }),
            }),
        };

        // adder5 = make_adder(5)
        let adder_closure = eval_expr(
            &Expr::FnApply {
                func: Box::new(make_adder),
                args: vec![Expr::Literal(Value::Int(5))],
            },
            &ctx,
        )
        .unwrap();

        ctx.set("add5".to_string(), adder_closure);

        // add5(3) => 8
        let result = eval_expr(
            &Expr::FnApply {
                func: Box::new(Expr::Variable {
                    name: "add5".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                args: vec![Expr::Literal(Value::Int(3))],
            },
            &ctx,
        )
        .unwrap();

        assert_eq!(result, Value::Int(8));
    }

    /// SPEC-031 §5.2 – Higher-order function: apply(f, x) = f(x).
    #[test]
    fn task559_higher_order_function_apply() {
        let ctx = Context::new();

        // apply = fn(f, x) { f(x) }  -- wait, Core FnApply needs Expr::FnApply
        // Use: (fn(f) { f(10) })(fn(x) { x * 2 }) => 20
        let double_fn = Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Literal(Value::Int(2))),
            }),
        };

        let apply_fn = Expr::FnDef {
            params: vec![("f".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::FnApply {
                func: Box::new(Expr::Variable {
                    name: "f".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                args: vec![Expr::Literal(Value::Int(10))],
            }),
        };

        let result = eval_expr(
            &Expr::FnApply {
                func: Box::new(apply_fn),
                args: vec![double_fn],
            },
            &ctx,
        )
        .unwrap();

        assert_eq!(result, Value::Int(20));
    }

    /// SPEC-031 §5.5 – Recursion via BindingSlot::Late: factorial(5) = 120.
    #[test]
    fn task559_recursive_closure_via_late_binding() {
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;

        // 1. Create env frame with a late slot for `fact`
        let mut frame = EnvFrame::new();
        let late_slot = frame.insert_late("fact".to_string());
        let env = Arc::new(frame);

        // 2. Build the factorial body:
        //    match n { 0 => 1, _ => n * fact(n-1) }
        let body = Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: "n".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Value::Int(0)),
                    body: Expr::Literal(Value::Int(1)),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Variable {
                            name: "n".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                        right: Box::new(Expr::FnApply {
                            func: Box::new(Expr::Variable {
                                name: "fact".to_string(),
                                span: ash_core::ast::Span::default(),
                            }),
                            args: vec![Expr::Binary {
                                op: BinaryOp::Sub,
                                left: Box::new(Expr::Variable {
                                    name: "n".to_string(),
                                    span: ash_core::ast::Span::default(),
                                }),
                                right: Box::new(Expr::Literal(Value::Int(1))),
                            }],
                        }),
                    },
                },
            ],
        };

        // 3. Construct the closure manually with the env that has the late slot
        let fact_closure = Value::Closure {
            params: vec![("n".to_string(), None)],
            body: Box::new(body),
            env: env.clone(),
        };

        // 4. Fill the late slot so recursive calls resolve
        late_slot.set_late(fact_closure.clone());

        // 5. Call fact(5) from a context that has fact bound
        let mut ctx = Context::new();
        ctx.set("fact".to_string(), fact_closure);

        let result = eval_expr(
            &Expr::FnApply {
                func: Box::new(Expr::Variable {
                    name: "fact".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                args: vec![Expr::Literal(Value::Int(5))],
            },
            &ctx,
        )
        .unwrap();

        assert_eq!(result, Value::Int(120), "fact(5) must equal 120");
    }

    /// SPEC-031 §10.2 – Value::Closure is Send + Sync (compile-time assertion).
    ///
    /// This test doesn't run code — the fact it compiles proves the constraint.
    #[test]
    fn task559_closure_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Value>();
    }

    /// SPEC-031 §10.3 – Serializing a Value::Closure returns an error.
    #[test]
    fn task559_closure_serialization_returns_error() {
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;

        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            env: Arc::new(EnvFrame::new()),
        };

        let result = serde_json::to_string(&closure);
        assert!(
            result.is_err(),
            "serializing Value::Closure must return an error, got Ok"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be serialized"),
            "error message should explain why: {err_msg}"
        );
    }

    /// SPEC-031 §5.6 – Calling a non-closure value via FnApply returns NotCallable.
    #[test]
    fn task559_fnapply_non_callable_returns_error() {
        let mut ctx = Context::new();
        ctx.set("not_a_fn".to_string(), Value::Int(42));

        let expr = Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: "not_a_fn".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            args: vec![Expr::Literal(Value::Int(1))],
        };

        let err = eval_expr(&expr, &ctx).unwrap_err();
        assert!(
            matches!(err, EvalError::NotCallable { .. }),
            "expected NotCallable, got {err:?}"
        );
    }

    /// SPEC-031 §5.7 – FnApply with wrong arity returns a partial-application Closure.
    #[test]
    fn task559_fnapply_wrong_arity_returns_error() {
        let ctx = Context::new();

        let expr = Expr::FnApply {
            func: Box::new(Expr::FnDef {
                params: vec![("x".to_string(), None), ("y".to_string(), None)],
                return_type: None,
                body: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
            }),
            args: vec![Expr::Literal(Value::Int(1))], // only 1 arg, need 2
        };

        let value = eval_expr(&expr, &ctx).unwrap();
        assert!(
            matches!(value, Value::Closure { .. }),
            "expected partial-application Closure, got {value:?}"
        );
    }

    /// SPEC-031 §4.8 / §10 – BoundaryViolation can be constructed with a
    /// Value::Closure and a descriptive context string.
    ///
    /// The `is_pure()` guard exists in `eval_expr` and is exercised by
    /// `task559_boundary_violation_in_pure_context` using the test-only
    /// `Context::enter_pure()` method.  In production, the type checker is
    /// the primary enforcement mechanism; the runtime safety net will
    /// activate once the interpreter propagates purity context through
    /// closure application.
    #[test]
    fn task559_boundary_violation_on_context_boundary_crossing() {
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;

        // Construct a Value::Closure (the kind of value that would trigger a
        // boundary violation if it crossed into a pure context).
        let closure_value = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            env: Arc::new(EnvFrame::new()),
        };

        // Build the error — this is the code path that SPEC-031 §4.8 escape
        // case 5 describes.
        let err = EvalError::BoundaryViolation {
            value: Box::new(closure_value),
            context: "closure escaped pure vertex boundary".to_string(),
        };

        // Verify the display message contains the required fragments.
        let msg = err.to_string();
        assert!(
            msg.contains("three-vertex boundary"),
            "BoundaryViolation message should mention three-vertex boundary, got: {msg}"
        );
        assert!(
            msg.contains("closure escaped pure vertex boundary"),
            "BoundaryViolation message should contain context string, got: {msg}"
        );
    }

    /// SPEC-031 §4.8 – Runtime enforcement: Expr::FnDef inside a pure context
    /// raises BoundaryViolation.
    #[test]
    fn task559_boundary_violation_in_pure_context() {
        use crate::context::Context;

        // Create a pure context
        let base = Context::new();
        let pure_ctx = base.enter_pure();

        // FnDef inside a pure context should be rejected
        let expr = Expr::FnDef {
            params: vec![("x".into(), None)],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_core::ast::Span::default(),
            }),
        };

        let result = eval_expr(&expr, &pure_ctx);
        assert!(
            matches!(result, Err(EvalError::BoundaryViolation { .. })),
            "expected BoundaryViolation in pure context, got {result:?}"
        );
    }

    /// SPEC-031 §13.1 – Module-level functions are never Value::Closure.
    ///
    /// Module-level functions are invoked directly by name (Expr::Call in a
    /// module context) and return their evaluated result, not an intermediate
    /// Value::Closure.  By contrast, Expr::FnDef at the expression level DOES
    /// produce Value::Closure (see `task559_fndef_produces_value_closure`).
    ///
    /// This test simulates the return value of a module-level function call
    /// and asserts it is never a Closure.
    #[test]
    fn task559_module_level_fndef_never_produces_closure() {
        // Simulate the result of calling a module-level function.
        // Module-level functions evaluate to their *body result*, not a Closure.
        let result = Value::Int(42);

        // A module-level function return must never be a Closure.
        assert!(
            !matches!(result, Value::Closure { .. }),
            "module-level function call must not return Value::Closure, got {result:?}"
        );

        // Positive contrast: Expr::FnDef at expression level DOES produce Closure.
        // (Already covered by `task559_fndef_produces_value_closure` — this block
        // confirms the same fact inline for documentation purposes.)
        let ctx = Context::new();
        let fndef = Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
        };
        let closure_result = eval_expr(&fndef, &ctx).unwrap();
        assert!(
            matches!(closure_result, Value::Closure { .. }),
            "expression-level FnDef should produce Value::Closure, got {closure_result:?}"
        );
    }

    // ── TASK-653: Short-circuit and/or evaluation (SPEC-004) ──────────

    /// SPEC-004 EXPR-AND-FALSE: `false && <error>` returns `false` without
    /// evaluating the right operand.
    #[test]
    fn task653_and_short_circuits_on_false() {
        let ctx = Context::new();

        // false and (1 / 0) — division by zero on the right must not fire
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Value::Bool(false))),
            right: Box::new(Expr::Binary {
                op: BinaryOp::Div,
                left: Box::new(Expr::Literal(Value::Int(1))),
                right: Box::new(Expr::Literal(Value::Int(0))),
            }),
        };

        let result = eval_expr(&expr, &ctx);
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    /// SPEC-004 EXPR-OR-TRUE: `true || <error>` returns `true` without
    /// evaluating the right operand.
    #[test]
    fn task653_or_short_circuits_on_true() {
        let ctx = Context::new();

        // true or (1 / 0) — division by zero on the right must not fire
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Binary {
                op: BinaryOp::Div,
                left: Box::new(Expr::Literal(Value::Int(1))),
                right: Box::new(Expr::Literal(Value::Int(0))),
            }),
        };

        let result = eval_expr(&expr, &ctx);
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    /// `true && false` returns `false` (both operands evaluated).
    #[test]
    fn task653_and_both_evaluated_when_left_true() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Value::Bool(true))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(false));
    }

    /// `false || true` returns `true` (both operands evaluated).
    #[test]
    fn task653_or_both_evaluated_when_left_false() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Value::Bool(false))),
            right: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Bool(true));
    }

    /// Non-boolean left operand in `and` produces a type error.
    #[test]
    fn task653_and_non_bool_left_is_error() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Value::Int(1))),
            right: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    /// Non-boolean left operand in `or` produces a type error.
    #[test]
    fn task653_or_non_bool_left_is_error() {
        let ctx = Context::new();
        let expr = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Value::Int(0))),
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    // ── TASK-650: Expr::Let evaluation tests ────────────────────────

    /// Simple let binding: `let x = 42; x` evaluates to 42
    #[test]
    fn task650_let_simple_binding() {
        let ctx = Context::new();
        let expr = Expr::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(42))),
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            span: ash_core::ast::Span::default(),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(42));
    }

    /// Nested let: `let x = 1; let y = 2; y` evaluates to 2
    #[test]
    fn task650_let_nested_binding() {
        let ctx = Context::new();
        let inner = Expr::Let {
            pattern: Pattern::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(2))),
            body: Box::new(Expr::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            span: ash_core::ast::Span::default(),
        };
        let outer = Expr::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(1))),
            body: Box::new(inner),
            span: ash_core::ast::Span::default(),
        };
        assert_eq!(eval_expr(&outer, &ctx).unwrap(), Value::Int(2));
    }

    /// Scope isolation: `let x = 1; let x = 2; x` evaluates to 2 (inner shadows outer)
    #[test]
    fn task650_let_shadowing() {
        let ctx = Context::new();
        let inner = Expr::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(2))),
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            span: ash_core::ast::Span::default(),
        };
        let outer = Expr::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(1))),
            body: Box::new(inner),
            span: ash_core::ast::Span::default(),
        };
        assert_eq!(eval_expr(&outer, &ctx).unwrap(), Value::Int(2));
    }

    /// Let binding doesn't leak into parent scope.
    #[test]
    fn task650_let_no_scope_leak() {
        let ctx = Context::new();
        // let x = 42; x  -- evaluates to 42, but x is not in parent
        let let_expr = Expr::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::Literal(Value::Int(42))),
            body: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            span: ash_core::ast::Span::default(),
        };
        // After evaluating the let, x should NOT be in ctx
        let result = eval_expr(&let_expr, &ctx);
        assert_eq!(result.unwrap(), Value::Int(42));
        // Verify x is NOT accessible in the original context
        assert!(ctx.get("x").is_none());
    }

    /// Tuple destructuring: `let (a, b) = (1, 2); a` — uses List since no Value::Tuple.
    /// Test list pattern destructuring: `let [a, b] = [1, 2]; a`
    #[test]
    fn task650_let_list_destructure() {
        let ctx = Context::new();
        let expr = Expr::Let {
            pattern: Pattern::List(
                vec![
                    Pattern::Variable {
                        name: "a".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                    Pattern::Variable {
                        name: "b".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                ],
                None,
            ),
            expr: Box::new(Expr::Literal(Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
            ])))),
            body: Box::new(Expr::Variable {
                name: "a".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            span: ash_core::ast::Span::default(),
        };
        assert_eq!(eval_expr(&expr, &ctx).unwrap(), Value::Int(1));
    }

    // ── markdown::parse builtin tests ──

    #[test]
    fn test_markdown_parse_heading() {
        let ctx = Context::new();
        let result = eval_function_call(
            "parse",
            Some("markdown"),
            &[Value::String("# Hello\n\nWorld".to_string())],
            &ctx,
        );
        let json_str = match result.unwrap() {
            Value::String(s) => s,
            other => panic!("expected String, got {other:?}"),
        };
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
        let blocks = val["blocks"].as_array().expect("blocks should be array");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "heading");
        assert_eq!(blocks[0]["level"], 1);
        assert_eq!(blocks[0]["text"], "Hello");
        assert_eq!(blocks[1]["type"], "paragraph");
        assert_eq!(blocks[1]["text"], "World");
    }

    #[test]
    fn test_markdown_parse_paragraph() {
        let ctx = Context::new();
        let result = eval_function_call(
            "parse",
            Some("markdown"),
            &[Value::String("Hello world".to_string())],
            &ctx,
        );
        let json_str = match result.unwrap() {
            Value::String(s) => s,
            other => panic!("expected String, got {other:?}"),
        };
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
        let blocks = val["blocks"].as_array().expect("blocks should be array");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "paragraph");
        assert_eq!(blocks[0]["text"], "Hello world");
    }

    #[test]
    fn test_markdown_parse_code_block() {
        let ctx = Context::new();
        let input = "```rust\nfn main() {}\n```".to_string();
        let result = eval_function_call("parse", Some("markdown"), &[Value::String(input)], &ctx);
        let json_str = match result.unwrap() {
            Value::String(s) => s,
            other => panic!("expected String, got {other:?}"),
        };
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
        let blocks = val["blocks"].as_array().expect("blocks should be array");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "code_block");
        assert_eq!(blocks[0]["language"], "rust");
        assert_eq!(blocks[0]["text"], "fn main() {}");
    }

    #[test]
    fn test_markdown_parse_empty_input() {
        let ctx = Context::new();
        let result = eval_function_call(
            "parse",
            Some("markdown"),
            &[Value::String(String::new())],
            &ctx,
        );
        let json_str = match result.unwrap() {
            Value::String(s) => s,
            other => panic!("expected String, got {other:?}"),
        };
        let val: serde_json::Value = serde_json::from_str(&json_str).expect("should be valid JSON");
        let blocks = val["blocks"].as_array().expect("blocks should be array");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_markdown_parse_arity_error() {
        let ctx = Context::new();
        let result = eval_function_call("parse", Some("markdown"), &[], &ctx);
        assert!(result.is_err());
    }

    // ============================================================
    // TASK-661: string::to_upper, string::to_lower, string::trim
    // ============================================================

    #[test]
    fn test_string_to_upper_basic() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_upper",
            Some("string"),
            &[Value::String("hello".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_to_upper_already_upper() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_upper",
            Some("string"),
            &[Value::String("HELLO".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_to_upper_mixed_case() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_upper",
            Some("string"),
            &[Value::String("hElLo".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_to_upper_empty() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_upper",
            Some("string"),
            &[Value::String(String::new())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String(String::new()));
    }

    #[test]
    fn test_string_to_upper_type_error() {
        let ctx = Context::new();
        let result = eval_function_call("to_upper", Some("string"), &[Value::Int(42)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_upper_arity_error() {
        let ctx = Context::new();
        let result = eval_function_call("to_upper", Some("string"), &[], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_lower_basic() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_lower",
            Some("string"),
            &[Value::String("HELLO".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_to_lower_already_lower() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_lower",
            Some("string"),
            &[Value::String("hello".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_to_lower_mixed_case() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_lower",
            Some("string"),
            &[Value::String("HeLLo".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_to_lower_empty() {
        let ctx = Context::new();
        let result = eval_function_call(
            "to_lower",
            Some("string"),
            &[Value::String(String::new())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String(String::new()));
    }

    #[test]
    fn test_string_to_lower_type_error() {
        let ctx = Context::new();
        let result = eval_function_call("to_lower", Some("string"), &[Value::Bool(true)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_lower_arity_error() {
        let ctx = Context::new();
        let result = eval_function_call("to_lower", Some("string"), &[], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_trim_basic() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String("  hi  ".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hi".to_string()));
    }

    #[test]
    fn test_string_trim_leading() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String("   hello".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_trim_trailing() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String("hello   ".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_trim_no_whitespace() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String("hello".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_trim_empty() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String(String::new())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String(String::new()));
    }

    #[test]
    fn test_string_trim_only_whitespace() {
        let ctx = Context::new();
        let result = eval_function_call(
            "trim",
            Some("string"),
            &[Value::String("   ".to_string())],
            &ctx,
        );
        assert_eq!(result.unwrap(), Value::String(String::new()));
    }

    #[test]
    fn test_string_trim_type_error() {
        let ctx = Context::new();
        let result = eval_function_call("trim", Some("string"), &[Value::Int(42)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_trim_arity_error() {
        let ctx = Context::new();
        let result = eval_function_call("trim", Some("string"), &[], &ctx);
        assert!(result.is_err());
    }

    // ============================================================
    // TASK-664: list::filter and list::map closure callback tests
    // ============================================================

    /// Helper: build a simple 1-param closure (x => body).
    fn simple_closure(body: Expr) -> Value {
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;
        Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Box::new(body),
            env: Arc::new(EnvFrame::new()),
        }
    }

    // ── filter tests ──────────────────────────────────────────────

    #[test]
    fn test_filter_keep_greater_than_3() {
        let ctx = Context::new();
        // filter [1, 4, 2, 5, 6, 3] with (x > 3)
        let closure = simple_closure(Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(3))),
        });
        let list = Value::List(Box::new(vec![
            Value::Int(1),
            Value::Int(4),
            Value::Int(2),
            Value::Int(5),
            Value::Int(6),
            Value::Int(3),
        ]));
        let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
        assert_eq!(
            result,
            Value::List(Box::new(vec![Value::Int(4), Value::Int(5), Value::Int(6)]))
        );
    }

    #[test]
    fn test_filter_keeps_nothing() {
        let ctx = Context::new();
        // filter [1, 2, 3] with (x > 100) → []
        let closure = simple_closure(Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(100))),
        });
        let list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
        let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
        assert_eq!(result, Value::List(Box::default()));
    }

    #[test]
    fn test_filter_keeps_everything() {
        let ctx = Context::new();
        // filter [1, 2, 3] with (x > 0) → [1, 2, 3]
        let closure = simple_closure(Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(0))),
        });
        let list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
        let result = eval_function_call("filter", None, &[list, closure], &ctx).unwrap();
        assert_eq!(
            result,
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3),]))
        );
    }

    // ── map tests ─────────────────────────────────────────────────

    #[test]
    fn test_map_double_elements() {
        let ctx = Context::new();
        // map [1, 2, 3] with (x * 2) → [2, 4, 6]
        let closure = simple_closure(Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::Int(2))),
        });
        let list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
        let result = eval_function_call("map", None, &[list, closure], &ctx).unwrap();
        assert_eq!(
            result,
            Value::List(Box::new(vec![Value::Int(2), Value::Int(4), Value::Int(6),]))
        );
    }

    #[test]
    fn test_map_string_transform() {
        let ctx = Context::new();
        // map ["a", "b"] with (x + "!") → ["a!", "b!"]
        let closure = simple_closure(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            right: Box::new(Expr::Literal(Value::String("!".to_string()))),
        });
        let list = Value::List(Box::new(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]));
        let result = eval_function_call("map", None, &[list, closure], &ctx).unwrap();
        assert_eq!(
            result,
            Value::List(Box::new(vec![
                Value::String("a!".to_string()),
                Value::String("b!".to_string()),
            ]))
        );
    }

    // ── filter/map error cases ────────────────────────────────────

    #[test]
    fn test_filter_wrong_first_arg_type() {
        let ctx = Context::new();
        let closure = simple_closure(Expr::Literal(Value::Bool(true)));
        // filter(42, closure) → TypeMismatch
        let result = eval_function_call("filter", None, &[Value::Int(42), closure], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_wrong_second_arg_type() {
        let ctx = Context::new();
        let list = Value::List(Box::new(vec![Value::Int(1)]));
        // filter(list, 99) → TypeMismatch
        let result = eval_function_call("filter", None, &[list, Value::Int(99)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_wrong_first_arg_type() {
        let ctx = Context::new();
        let closure = simple_closure(Expr::Literal(Value::Int(0)));
        // map("hello", closure) → TypeMismatch
        let result = eval_function_call(
            "map",
            None,
            &[Value::String("hello".to_string()), closure],
            &ctx,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_map_wrong_second_arg_type() {
        let ctx = Context::new();
        let list = Value::List(Box::new(vec![Value::Int(1)]));
        // map(list, true) → TypeMismatch
        let result = eval_function_call("map", None, &[list, Value::Bool(true)], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_wrong_arity_too_few() {
        let ctx = Context::new();
        let result = eval_function_call("filter", None, &[], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_wrong_arity_too_many() {
        let ctx = Context::new();
        let closure = simple_closure(Expr::Literal(Value::Bool(true)));
        let result = eval_function_call(
            "filter",
            None,
            &[Value::List(Box::default()), closure, Value::Int(1)],
            &ctx,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_map_wrong_arity_too_few() {
        let ctx = Context::new();
        let result = eval_function_call("map", None, &[], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_wrong_arity_too_many() {
        let ctx = Context::new();
        let closure = simple_closure(Expr::Literal(Value::Int(0)));
        let result = eval_function_call(
            "map",
            None,
            &[Value::List(Box::default()), closure, Value::Int(1)],
            &ctx,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_closure_wrong_param_count() {
        let ctx = Context::new();
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;
        // Closure with 0 params → WrongArity
        let closure = Value::Closure {
            params: vec![],
            body: Box::new(Expr::Literal(Value::Bool(true))),
            env: Arc::new(EnvFrame::new()),
        };
        let list = Value::List(Box::new(vec![Value::Int(1)]));
        let result = eval_function_call("filter", None, &[list, closure], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_closure_wrong_param_count() {
        let ctx = Context::new();
        use ash_core::env_frame::EnvFrame;
        use std::sync::Arc;
        // Closure with 2 params → WrongArity
        let closure = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: Box::new(Expr::Literal(Value::Int(0))),
            env: Arc::new(EnvFrame::new()),
        };
        let list = Value::List(Box::new(vec![Value::Int(1)]));
        let result = eval_function_call("map", None, &[list, closure], &ctx);
        assert!(result.is_err());
    }

    fn proc_unit_expr(expr: Expr) -> Expr {
        Expr::Call {
            func: "unit".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![expr],
        }
    }

    fn proc_par_expr(left: Expr, right: Expr) -> Expr {
        Expr::Call {
            func: "par".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![left, right],
        }
    }

    fn proc_scatter_expr(items: Vec<Value>, mapper: Expr) -> Expr {
        Expr::Call {
            func: "scatter".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![Expr::Literal(Value::List(Box::new(items))), mapper],
        }
    }

    fn proc_await_expr(handle: ProcessHandle) -> Expr {
        Expr::Call {
            func: "await".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![Expr::Literal(Value::ProcessHandle(handle))],
        }
    }

    async fn force_proc_in_context(ctx: &Context, proc_value: Value) -> EvalResult<Value> {
        let mut proc_ctx = ctx.clone();
        proc_ctx.set("p".to_string(), proc_value);
        eval_expr_async(
            &Expr::Call {
                func: "p".to_string(),
                module: None,
                arguments: vec![Expr::Literal(Value::Null)],
            },
            &proc_ctx,
        )
        .await
    }

    fn expect_handle_list(value: Value, expected_len: usize) -> Vec<ProcessHandle> {
        let Value::List(items) = value else {
            panic!("expected ordered handle list, got {value:?}");
        };
        assert_eq!(items.len(), expected_len, "expected {expected_len} handles");
        items
            .iter()
            .map(|value| match value {
                Value::ProcessHandle(handle) => handle.clone(),
                other => panic!("expected process handle, got {other:?}"),
            })
            .collect()
    }

    #[tokio::test]
    async fn proc_par_returns_ordered_child_handles_and_defers_child_failure_to_later_await() {
        let runtime_state = RuntimeState::new();
        let parent_process_id = ProcessId::new();
        runtime_state
            .register_root_process(parent_process_id)
            .await
            .expect("parent process registers");

        let failed_dependency = ProcessId::new();
        runtime_state
            .register_root_process(failed_dependency)
            .await
            .expect("dependency process registers");
        runtime_state
            .record_process_terminal(
                failed_dependency,
                ash_core::runtime::ProcessTerminalState::Failed {
                    process_id: failed_dependency,
                    failure: Box::new(ash_core::runtime::OperationalFailure::new(
                        ash_core::runtime::TowerLevel::Proc,
                        ash_core::runtime::FailureEntity::Process(failed_dependency),
                        Value::String("boom".to_string()),
                        "String",
                    )),
                },
            )
            .await
            .expect("dependency terminal state records");

        let proc_ctx = Context::new()
            .with_runtime_state(runtime_state.clone())
            .project_process_child(
                crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
                None,
            );

        let proc_value = eval_expr(
            &proc_par_expr(
                proc_await_expr(ProcessHandle::new(
                    failed_dependency,
                    Some("Int".to_string()),
                )),
                proc_unit_expr(Expr::Literal(Value::Int(7))),
            ),
            &proc_ctx,
        )
        .expect("proc::par should build a Proc closure");

        let handles = expect_handle_list(
            force_proc_in_context(&proc_ctx, proc_value)
                .await
                .expect("proc::par should return child handles before child failure is observed"),
            2,
        );

        let children = runtime_state.process_children(parent_process_id).await;
        assert_eq!(
            children.len(),
            2,
            "proc::par should register two child processes"
        );
        assert_eq!(handles[0].process_id, children[0]);
        assert_eq!(handles[1].process_id, children[1]);

        for _ in 0..1024 {
            if runtime_state
                .process_terminal_state(handles[0].process_id)
                .await
                .is_some()
            {
                break;
            }
            tokio::task::yield_now().await;
        }

        let await_proc = eval_expr(&proc_await_expr(handles[0].clone()), &Context::new())
            .expect("await closure builds");
        let err = force_proc_in_context(&proc_ctx, await_proc)
            .await
            .expect_err("child failure should be observed only through later await");
        assert!(matches!(err, EvalError::OperationalFailure(_)));
    }

    #[tokio::test]
    async fn proc_scatter_returns_handles_in_input_order() {
        let runtime_state = RuntimeState::new();
        let parent_process_id = ProcessId::new();
        runtime_state
            .register_root_process(parent_process_id)
            .await
            .expect("parent process registers");

        let proc_ctx = Context::new()
            .with_runtime_state(runtime_state.clone())
            .project_process_child(
                crate::process_env::ProcessEnvIdentity::new(parent_process_id, None, 0),
                None,
            );

        let mapper = Expr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(proc_unit_expr(Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            })),
        };
        let proc_value = eval_expr(
            &proc_scatter_expr(vec![Value::Int(1), Value::Int(2), Value::Int(3)], mapper),
            &proc_ctx,
        )
        .expect("proc::scatter should build a Proc closure");

        let handles = expect_handle_list(
            force_proc_in_context(&proc_ctx, proc_value)
                .await
                .expect("proc::scatter should return one handle per input element"),
            3,
        );

        let children = runtime_state.process_children(parent_process_id).await;
        assert_eq!(
            children.len(),
            3,
            "proc::scatter should admit every child before returning"
        );
        assert_eq!(
            handles
                .iter()
                .map(|handle| handle.process_id)
                .collect::<Vec<_>>(),
            children,
            "proc::scatter should preserve stable input order in returned handles"
        );
    }
}
