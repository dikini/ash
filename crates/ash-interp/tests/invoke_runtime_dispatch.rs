use std::sync::Arc;

use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Effect, Expr, Value,
};
use ash_interp::RuntimeState;
use ash_interp::act_env::ActEnv;
use ash_interp::capability::{CapabilityContext, MockProvider};
use ash_interp::context::Context;
use ash_interp::eval::{eval_expr, eval_expr_async};

fn shadowing_closure(marker: &str) -> Value {
    eval_expr(
        &Expr::FnDef {
            params: vec![("_ignored".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Literal(Value::String(marker.to_string()))),
        },
        &Context::new(),
    )
    .expect("closure definition should evaluate")
}

fn one_arg_call(name: &str) -> Expr {
    Expr::Call {
        func: name.to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::String("arg".to_string()))],
    }
}

fn invoke_expr() -> Expr {
    invoke_expr_for("sensor", "read")
}

fn invoke_expr_for(provider: &str, action: &str) -> Expr {
    Expr::Call {
        func: "invoke".into(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String(provider.to_string())),
            Expr::Literal(Value::String(action.to_string())),
            Expr::Literal(Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])),
        ],
    }
}

fn host_binding_alias(name: &str, provider_name: &str, admitted: Vec<&str>) -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        name,
        CapabilityInterfaceId::new("Sensor"),
        provider_name,
        admitted.into_iter().map(str::to_string).collect(),
    )
}

#[tokio::test]
async fn invoke_dispatch_returns_closure_with_captured_state() {
    let ctx = Context::new();
    let result = eval_expr(&invoke_expr(), &ctx).expect("invoke should dispatch");

    let Value::Closure { params, .. } = result.clone() else {
        panic!("expected closure from invoke, got {result:?}");
    };
    assert!(
        params == vec![("__act_env".to_string(), None)],
        "invoke closure should require the hidden ActEnv carrier"
    );

    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("done".to_string()))),
        ),
    );
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
        .expect("sensor binding admission succeeds");
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    )
    .await
    .expect("act env projection succeeds");

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id])
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect("invoke closure should be callable");

    assert_eq!(
        applied,
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("done".to_string())])
    );
}

#[tokio::test]
async fn admitted_host_provider_sync_invoke_uses_projected_runtime_surface_not_manual_act_env() {
    let ctx = Context::new();
    let result =
        eval_expr(&invoke_expr_for("sensor", "read"), &ctx).expect("invoke should dispatch");
    let mut capability_ctx = CapabilityContext::new();
    capability_ctx.register(Box::new(
        MockProvider::new("sensor", Effect::Operational)
            .with_execute_result(Ok(Value::String("leaked".to_string()))),
    ));
    let act_env = ActEnv::new(
        capability_ctx,
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    );
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("projected".to_string()))),
        ),
    );
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
        .expect("binding admission succeeds");

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id])
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect("admitted sync invoke should use projected runtime provider");

    assert_eq!(
        applied,
        Value::list_from_vec(vec![
            Value::ActEnvToken,
            Value::String("projected".to_string())
        ])
    );
}

#[tokio::test]
async fn alias_only_invoke_rejects_direct_backing_provider_call() {
    let ctx = Context::new();
    let result =
        eval_expr(&invoke_expr_for("sensor", "read"), &ctx).expect("invoke should dispatch");
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("leaked".to_string()))),
        ),
    );
    let binding = host_binding_alias("workflow-sensor", "sensor", vec!["sensor.read"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("alias binding admission succeeds");
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    )
    .await
    .expect("act env projection succeeds");

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id])
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect_err("alias-only admission must not authorize direct backing-provider invoke");
    let message = applied.to_string();
    assert!(
        message.contains("authority boundary")
            && message.contains("admission")
            && message.contains("sensor"),
        "diagnostic should identify the direct provider admission boundary: {message}"
    );
}

#[tokio::test]
async fn alias_invoke_uses_projected_binding_alias_not_manual_act_env_provider() {
    let ctx = Context::new();
    let result = eval_expr(&invoke_expr_for("workflow-sensor", "read"), &ctx)
        .expect("invoke should dispatch");
    let mut capability_ctx = CapabilityContext::new();
    capability_ctx.register(Box::new(
        MockProvider::new("workflow-sensor", Effect::Operational)
            .with_execute_result(Ok(Value::String("leaked".to_string()))),
    ));
    let act_env = ActEnv::new(
        capability_ctx,
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    );
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("projected".to_string()))),
        ),
    );
    let binding = host_binding_alias("workflow-sensor", "sensor", vec!["sensor.read"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("alias binding admission succeeds");

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id])
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect("admitted alias invoke should use the projected runtime provider");

    assert_eq!(
        applied,
        Value::list_from_vec(vec![
            Value::ActEnvToken,
            Value::String("projected".to_string())
        ])
    );
}

#[tokio::test]
async fn registered_provider_without_admitted_binding_cannot_execute_through_invoke_fallback() {
    let ctx = Context::new();
    let result =
        eval_expr(&invoke_expr_for("sensor", "read"), &ctx).expect("invoke should dispatch");
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("leaked".to_string()))),
        ),
    );
    let binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "sensor",
        CapabilityInterfaceId::new("Sensor"),
        "sensor",
        vec!["sensor.read".to_string()],
    );
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("binding exists in runtime registry but is not admitted to this context");
    let act_env = ActEnv::from_runtime_state(
        &runtime_state,
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    )
    .await;

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect_err("registered provider existence alone must not grant authority");
    let message = applied.to_string();
    assert!(
        message.contains("authority boundary")
            && message.contains("admission")
            && message.contains("sensor"),
        "diagnostic should distinguish authority-boundary admission failure: {message}"
    );
}

async fn assert_provider_in_act_env_without_runtime_state_binding_is_denied(
    evaluate: impl AsyncFnOnce(&Expr, &Context) -> Result<Value, ash_interp::EvalError>,
) {
    let ctx = Context::new();
    let result =
        eval_expr(&invoke_expr_for("sensor", "read"), &ctx).expect("invoke should dispatch");
    let mut capability_ctx = CapabilityContext::new();
    capability_ctx.register(Box::new(
        MockProvider::new("sensor", Effect::Operational)
            .with_execute_result(Ok(Value::String("leaked".to_string()))),
    ));
    let act_env = ActEnv::new(
        capability_ctx,
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    );
    let runtime_state = RuntimeState::new();

    let mut call_ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = evaluate(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .await
    .expect_err("ActEnv provider registration without runtime admission must not grant authority");
    let message = applied.to_string();
    assert!(
        message.contains("authority boundary")
            && message.contains("admission")
            && message.contains("sensor"),
        "diagnostic should distinguish authority-boundary admission failure: {message}"
    );
}

#[tokio::test]
async fn provider_in_act_env_without_runtime_state_binding_cannot_execute_through_invoke_fallback()
{
    assert_provider_in_act_env_without_runtime_state_binding_is_denied(async |expr, ctx| {
        eval_expr(expr, ctx)
    })
    .await;
}

#[tokio::test]
async fn provider_in_act_env_without_runtime_state_binding_cannot_execute_through_async_invoke_fallback()
 {
    assert_provider_in_act_env_without_runtime_state_binding_is_denied(async |expr, ctx| {
        eval_expr_async(expr, ctx).await
    })
    .await;
}

#[test]
fn invoke_closure_rejects_visible_token_without_hidden_runtime_act_env() {
    let ctx = Context::new();
    let result = eval_expr(&invoke_expr(), &ctx).expect("invoke should dispatch");

    let mut call_ctx = Context::new();
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    );

    assert!(
        applied.is_err(),
        "visible ActEnvToken alone should not satisfy invoke's hidden runtime carrier boundary"
    );
}

#[test]
fn invoke_does_not_break_existing_builtin_dispatch() {
    let ctx = Context::new();
    let concat = Expr::Call {
        func: "concat".into(),
        module: Some("string".into()),
        arguments: vec![
            Expr::Literal(Value::String("a".to_string())),
            Expr::Literal(Value::String("b".to_string())),
        ],
    };

    let result = eval_expr(&concat, &ctx).expect("string::concat should still work");
    assert_eq!(result, Value::String("ab".to_string()));
}

#[test]
fn unqualified_public_runtime_names_resolve_context_closures_before_primitives_sync() {
    for name in ["unit", "bind", "invoke", "policy_check"] {
        let marker = format!("user {name}");
        let mut ctx = Context::new();
        ctx.set(name.to_string(), shadowing_closure(&marker));

        let result = eval_expr(&one_arg_call(name), &ctx)
            .unwrap_or_else(|err| panic!("{name} should resolve through context first: {err:?}"));

        assert_eq!(result, Value::String(marker));
    }
}

#[tokio::test]
async fn unqualified_public_runtime_names_resolve_context_closures_before_primitives_async() {
    for name in ["unit", "bind", "invoke", "policy_check"] {
        let marker = format!("async user {name}");
        let mut ctx = Context::new();
        ctx.set(name.to_string(), shadowing_closure(&marker));

        let result = eval_expr_async(&one_arg_call(name), &ctx)
            .await
            .unwrap_or_else(|err| panic!("{name} should resolve through context first: {err:?}"));

        assert_eq!(result, Value::String(marker));
    }
}
