use std::sync::Arc;

use ash_core::capability::CapabilityError;
use ash_core::runtime::{FailureBoundary, FailureEntity, ProcessId};
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

fn invoke_expr() -> Expr {
    Expr::Call {
        func: "invoke".into(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String("sensor".to_string())),
            Expr::Literal(Value::String("read".to_string())),
            Expr::Literal(Value::list_nil()),
        ],
    }
}

fn sensor_binding() -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "sensor",
        CapabilityInterfaceId::new("Sensor"),
        "sensor",
        vec!["sensor.read".to_string()],
    )
}

async fn admit_sensor(runtime_state: &RuntimeState) -> CapabilityBindingId {
    let binding = sensor_binding();
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("sensor binding should be admitted");
    binding_id
}

fn constant_act_expr(value: Value) -> Expr {
    Expr::FnDef {
        params: vec![("__act_env".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Constructor {
            name: "Cons".to_string(),
            fields: vec![
                ("head".to_string(), Expr::Literal(Value::ActEnvToken)),
                (
                    "tail".to_string(),
                    Expr::Literal(Value::list_from_vec(vec![value])),
                ),
            ],
        }),
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
async fn forcing_embedded_act_via_hidden_act_env_succeeds_without_child_admission() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("{};".to_string()))),
        ),
    );
    let binding_id = admit_sensor(&runtime_state).await;
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        PolicyEvaluator::new(),
        Provenance::new(),
    )
    .await
    .expect("admitted sensor binding should build ActEnv");
    let act_value =
        eval_expr(&invoke_expr(), &Context::new()).expect("invoke should build an Act closure");
    let proc_value = eval_expr(
        &proc_from_act_expr(act_value),
        &Context::new()
            .with_admitted_capability_bindings(vec![binding_id])
            .with_act_env(act_env),
    )
    .expect("proc::from_act should build a Proc closure from an Act value");

    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let process_ctx = process_context(runtime_state.clone(), process_id);
    let before_children = runtime_state.process_children(process_id).await;

    let forced = force_proc_in_context(
        process_ctx
            .with_admitted_capability_bindings(vec![binding_id])
            .with_act_env(
                ActEnv::from_runtime_state_with_admitted_bindings(
                    &runtime_state,
                    &[binding_id],
                    PolicyEvaluator::new(),
                    Provenance::new(),
                )
                .await
                .expect("admitted sensor binding should build process ActEnv"),
            ),
        proc_value,
        Value::Null,
    )
    .await
    .expect("forcing proc::from_act should reuse the hidden runtime ActEnv path");

    assert_eq!(forced, Value::String("{};".to_string()));
    assert_eq!(
        runtime_state.process_children(process_id).await,
        before_children,
        "proc::from_act should not admit child processes just to force an embedded Act"
    );
}

#[tokio::test]
async fn forcing_without_hidden_act_env_rejects_fake_visible_carrier() {
    let act_value =
        eval_expr(&constant_act_expr(Value::Int(5)), &Context::new()).expect("act builds");
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
    let act_value =
        eval_expr(&constant_act_expr(Value::Int(11)), &Context::new()).expect("act builds");
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

    assert_eq!(failure.boundary, FailureBoundary::Process);
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
    let binding_id = admit_sensor(&runtime_state).await;
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        PolicyEvaluator::new(),
        Provenance::new(),
    )
    .await
    .expect("admitted sensor binding should build ActEnv");
    let act_value =
        eval_expr(&invoke_expr(), &Context::new()).expect("invoke should build an Act closure");
    let proc_value = eval_expr(
        &proc_from_act_expr(act_value),
        &Context::new()
            .with_admitted_capability_bindings(vec![binding_id])
            .with_act_env(act_env),
    )
    .expect("proc::from_act should build a Proc closure around the failing Act");

    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");

    let err = force_proc_in_context(
        process_context(runtime_state.clone(), process_id)
            .with_admitted_capability_bindings(vec![binding_id])
            .with_act_env(
                ActEnv::from_runtime_state_with_admitted_bindings(
                    &runtime_state,
                    &[binding_id],
                    PolicyEvaluator::new(),
                    Provenance::new(),
                )
                .await
                .expect("admitted sensor binding should build process ActEnv"),
            ),
        proc_value,
        Value::Null,
    )
    .await
    .expect_err("forcing proc::from_act should propagate the embedded Act failure honestly");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected structured operational failure, got {err:?}");
    };

    assert_eq!(failure.boundary, FailureBoundary::Effectful);
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
