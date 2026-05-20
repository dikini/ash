use std::sync::Arc;

use ash_core::capability::CapabilityError;
use ash_core::runtime::{FailureEntity, ProcessId, TowerLevel};
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Effect, Expr, Provenance, Value,
    ast::Pattern,
};
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{ChildEnvProjection, PolicyEvaluator, RuntimeState, derive_child_env};
use ash_parser::lower::lower_expr;
use ash_parser::surface::{ActStmt, Expr as SurfaceExpr, Literal};

fn span() -> ash_parser::token::Span {
    ash_parser::token::Span::default()
}

fn invoke_expr() -> Expr {
    Expr::Call {
        func: "invoke".into(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String("sensor".to_string())),
            Expr::Literal(Value::String("read".to_string())),
            Expr::Literal(Value::List(Box::default())),
        ],
    }
}

fn return_act_surface(value: i64) -> SurfaceExpr {
    SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(Literal::Int(value))),
            span: span(),
        }],
        span: span(),
    }
}

fn proc_from_act_expr(act_value: Value) -> Expr {
    Expr::Call {
        func: "from_act".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(act_value)],
    }
}

fn process_context(runtime_state: RuntimeState, process_id: ProcessId) -> Context {
    derive_child_env(
        &Context::new().with_runtime_state(runtime_state),
        ChildEnvProjection::new(process_id, 0),
    )
    .expect("process context projection should succeed")
}

async fn admit_sensor_binding(runtime_state: &RuntimeState) -> CapabilityBindingId {
    let binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "sensor",
        CapabilityInterfaceId::new("Sensor"),
        "sensor",
        vec!["sensor.read".to_string()],
    );
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("sensor host binding should admit");
    binding_id
}

async fn admitted_sensor_act_env(
    runtime_state: &RuntimeState,
    binding_id: CapabilityBindingId,
) -> ActEnv {
    ActEnv::from_runtime_state_with_admitted_bindings(
        runtime_state,
        &[binding_id],
        PolicyEvaluator::new(),
        Provenance::new(),
    )
    .await
    .expect("admitted sensor binding should project into ActEnv")
}

fn process_context_with_admission(
    runtime_state: RuntimeState,
    process_id: ProcessId,
    binding_id: CapabilityBindingId,
) -> Context {
    process_context(runtime_state, process_id).with_admitted_capability_bindings(vec![binding_id])
}

async fn force_proc_with_sensor_admission(
    runtime_state: &RuntimeState,
    process_id: ProcessId,
    binding_id: CapabilityBindingId,
    proc_value: Value,
) -> ash_interp::EvalResult<Value> {
    force_proc_in_context(
        process_context_with_admission(runtime_state.clone(), process_id, binding_id)
            .with_act_env(admitted_sensor_act_env(runtime_state, binding_id).await),
        proc_value,
        Value::Null,
    )
    .await
}

async fn force_proc_in_context(
    ctx: Context,
    proc_value: Value,
    proc_env: Value,
) -> ash_interp::EvalResult<Value> {
    let mut call_ctx = ctx;
    call_ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(proc_env)],
        },
        &call_ctx,
    )
    .await
}

fn proc_unit_expr(value: Value) -> Expr {
    Expr::Call {
        func: "unit".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(value)],
    }
}

fn proc_bind_expr(left: Expr, continuation_body: Expr) -> Expr {
    Expr::Call {
        func: "bind".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![
            left,
            Expr::FnDef {
                params: vec![("x".to_string(), None)],
                return_type: None,
                body: Box::new(continuation_body),
            },
        ],
    }
}

fn proc_do_like_fail_expr() -> Expr {
    proc_bind_expr(
        proc_unit_expr(Value::Int(1)),
        Expr::Call {
            func: "unit".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![Expr::Fail {
                payload: Box::new(Expr::Literal(Value::String("proc boom".to_string()))),
            }],
        },
    )
}

fn proc_do_like_handled_fail_expr() -> Expr {
    proc_bind_expr(
        proc_unit_expr(Value::Int(1)),
        Expr::Call {
            func: "unit".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![Expr::WithError {
                body: Box::new(Expr::Fail {
                    payload: Box::new(Expr::Literal(Value::String("handled".to_string()))),
                }),
                arms: vec![ash_core::ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Expr::Literal(Value::Int(99)),
                }],
            }],
        },
    )
}

#[tokio::test]
async fn forcing_embedded_act_via_hidden_act_env_succeeds_with_runtime_kernel_admission() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("done".to_string()))),
        ),
    );
    let binding_id = admit_sensor_binding(&runtime_state).await;
    let act_env = admitted_sensor_act_env(&runtime_state, binding_id).await;
    let act_value =
        eval_expr(&invoke_expr(), &Context::new()).expect("invoke should build an Act closure");
    let proc_value = eval_expr(
        &proc_from_act_expr(act_value),
        &Context::new().with_act_env(act_env),
    )
    .expect("proc::from_act should build a Proc closure from an Act value");

    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let before_children = runtime_state.process_children(process_id).await;

    let forced =
        force_proc_with_sensor_admission(&runtime_state, process_id, binding_id, proc_value)
            .await
            .expect("forcing proc::from_act should reuse the hidden runtime ActEnv path");

    assert_eq!(forced, Value::String("done".to_string()));
    assert_eq!(
        runtime_state.process_children(process_id).await,
        before_children,
        "proc::from_act should not admit child processes just to force an embedded Act"
    );
}

#[tokio::test]
async fn forcing_without_hidden_act_env_rejects_fake_visible_carrier() {
    let lowered = lower_expr(&return_act_surface(5)).expect("single-return act block should lower");
    let act_value = eval_expr(&lowered, &Context::new()).expect("lowered act should evaluate");
    let proc_value = eval_expr(&proc_from_act_expr(act_value), &Context::new())
        .expect("proc::from_act should build a Proc closure before forcing");

    let forced = force_proc_in_context(Context::new(), proc_value, Value::ActEnvToken).await;
    assert!(
        forced.is_err(),
        "a visible fake carrier alone must not satisfy proc::from_act's hidden ActEnv boundary"
    );
}

#[tokio::test]
async fn proc_from_act_does_not_inflate_into_child_processes_or_public_handles() {
    let lowered =
        lower_expr(&return_act_surface(11)).expect("single-return act block should lower");
    let act_value = eval_expr(&lowered, &Context::new()).expect("lowered act should evaluate");
    let proc_value = eval_expr(&proc_from_act_expr(act_value), &Context::new())
        .expect("proc::from_act should build a Proc closure");

    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let before_children = runtime_state.process_children(process_id).await;
    let forced = force_proc_in_context(
        process_context(runtime_state.clone(), process_id).with_act_env(ActEnv::default()),
        proc_value,
        Value::Null,
    )
    .await
    .expect(
        "forcing proc::from_act should return the embedded Act value without process inflation",
    );

    assert_eq!(forced, Value::Int(11));
    assert!(
        !matches!(forced, Value::ProcessHandle(_)),
        "proc::from_act must not silently inflate plain Act results into public process handles"
    );
    assert_eq!(
        runtime_state.process_children(process_id).await,
        before_children,
        "proc::from_act must not register child processes for plain embedded Act forcing"
    );
}

#[tokio::test]
async fn proc_do_like_bind_fail_is_operational_bottom_not_domain_value() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let proc_value = eval_expr(&proc_do_like_fail_expr(), &Context::new())
        .expect("proc do-like bind should build a Proc closure");

    let err = force_proc_in_context(
        process_context(runtime_state.clone(), process_id),
        proc_value,
        Value::Null,
    )
    .await
    .expect_err("forcing proc do-like fail should raise operational bottom");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected structured operational failure, got {err:?}");
    };

    assert_eq!(failure.tower, TowerLevel::Proc);
    assert_eq!(failure.entity, FailureEntity::Process(process_id));
    assert_eq!(failure.payload, Value::String("proc boom".to_string()));
    assert_eq!(failure.payload_type, "String");
}

#[tokio::test]
async fn proc_do_like_bind_fail_can_be_handled_only_by_operational_handler() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let proc_value = eval_expr(&proc_do_like_handled_fail_expr(), &Context::new())
        .expect("proc do-like handled fail should build a Proc closure");

    let forced = force_proc_in_context(
        process_context(runtime_state.clone(), process_id),
        proc_value,
        Value::Null,
    )
    .await
    .expect("with_error is the existing operational surface for handling fail");

    assert_eq!(forced, Value::Int(99));
}

#[tokio::test]
async fn proc_from_act_failing_embedded_act_preserves_effect_scope_operational_failure() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Err(CapabilityError::ExecutionFailed("boom".to_string()))),
        ),
    );
    let binding_id = admit_sensor_binding(&runtime_state).await;
    let act_env = admitted_sensor_act_env(&runtime_state, binding_id).await;
    let act_value =
        eval_expr(&invoke_expr(), &Context::new()).expect("invoke should build an Act closure");
    let proc_value = eval_expr(
        &proc_from_act_expr(act_value),
        &Context::new().with_act_env(act_env),
    )
    .expect("proc::from_act should build a Proc closure around the failing Act");

    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");

    let err = force_proc_with_sensor_admission(&runtime_state, process_id, binding_id, proc_value)
        .await
        .expect_err("forcing proc::from_act should propagate the embedded Act failure honestly");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected structured operational failure, got {err:?}");
    };

    assert_eq!(failure.tower, TowerLevel::Effectful);
    assert!(matches!(failure.entity, FailureEntity::EffectScope(_)));
    assert_eq!(
        failure.payload,
        Value::String("workflow execution failed: boom".to_string())
    );
    assert_eq!(failure.payload_type, "String");
    assert!(
        failure.cause.is_none(),
        "proc::from_act should preserve the lower Act-side failure directly rather than wrapping it"
    );
}
