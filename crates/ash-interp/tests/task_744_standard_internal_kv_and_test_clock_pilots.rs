//! TASK-744: standard internal WorkflowKV and FrozenClock/TestClock pilots.

use std::sync::Arc;

use ash_core::runtime::{
    CapabilityAuthorityProvenance, CapabilityBindingId, CapabilityInterfaceId, ResourceLifecycle,
    ResourceProvenance, ResourceTypeId,
};
use ash_core::{CapabilityBinding, Effect, Expr, Provenance, Value};
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::eval::eval_expr_async;
use ash_interp::{
    PolicyEvaluator, RuntimeState, StandardInternalPilot, StandardPilotBinding,
    StandardPilotResource,
};

fn invoke_expr(binding_name: &str, operation: &str, args: Vec<Value>) -> Expr {
    Expr::Call {
        func: "invoke".to_string(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String(binding_name.to_string())),
            Expr::Literal(Value::String(operation.to_string())),
            Expr::Literal(Value::List(Box::new(args))),
        ],
    }
}

async fn act_context(runtime_state: &RuntimeState) -> Context {
    let act_env =
        ActEnv::from_runtime_state(runtime_state, PolicyEvaluator::new(), Provenance::new()).await;
    Context::new().with_act_env(act_env)
}

async fn act_context_with_admitted(
    runtime_state: &RuntimeState,
    admitted_bindings: &[CapabilityBindingId],
) -> Context {
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        runtime_state,
        admitted_bindings,
        PolicyEvaluator::new(),
        Provenance::new(),
    )
    .await
    .expect("admitted capability bindings should project into ActEnv");
    Context::new().with_act_env(act_env)
}

async fn eval_invoke_act(expr: Expr, ctx: &Context) -> Result<Value, ash_interp::EvalError> {
    let closure = eval_expr_async(&expr, ctx).await?;
    let mut call_ctx = ctx.clone();
    call_ctx.set("act".to_string(), closure);
    eval_expr_async(
        &Expr::Call {
            func: "act".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .await
}

#[tokio::test]
async fn workflow_kv_pilot_substitutes_internal_binding_for_host_binding() {
    let runtime_state = RuntimeState::new().with_provider(
        "prod-kv-provider",
        Arc::new(
            MockProvider::new("prod-kv-provider", Effect::Operational)
                .with_execute_result(Ok(Value::String("prod-value".to_string()))),
        ),
    );
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");

    let host_binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "kv",
        CapabilityInterfaceId::new("KeyValue"),
        "prod-kv-provider",
        vec!["prod-kv-provider.get".to_string()],
    );
    let host_binding_id = host_binding.id;
    runtime_state
        .admit_capability_binding(host_binding)
        .await
        .expect("host binding admission succeeds");

    let host_ctx = act_context_with_admitted(&runtime_state, &[host_binding_id])
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![host_binding_id]);
    assert_eq!(
        eval_invoke_act(
            invoke_expr("kv", "get", vec![Value::String("key".to_string())]),
            &host_ctx,
        )
        .await
        .expect("host-backed kv binding executes"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("prod-value".to_string()),
        ]))
    );

    let StandardPilotBinding {
        binding_id,
        resource_id,
    } = runtime_state
        .admit_standard_internal_pilot(StandardInternalPilot::workflow_kv(
            "internal-kv",
            "store",
            Value::String("internal-value".to_string()),
        ))
        .await
        .expect("workflow kv pilot admits internal resource and binding");

    let internal_ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![binding_id]);
    assert_eq!(
        eval_invoke_act(
            invoke_expr("internal-kv", "get", vec![Value::String("key".to_string())]),
            &internal_ctx,
        )
        .await
        .expect("internal WorkflowKV implementation executes"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("internal-value".to_string()),
        ]))
    );

    let binding = runtime_state
        .capability_binding(binding_id)
        .await
        .expect("pilot binding is registered");
    assert!(matches!(
        binding.authority,
        CapabilityAuthorityProvenance::DerivedAuthority { ref dependency_names, .. }
            if dependency_names.contains(&"store".to_string())
    ));

    let resource = runtime_state
        .resource_instance(resource_id)
        .await
        .expect("pilot resource is registered");
    assert_eq!(resource.type_id, ResourceTypeId::new("WorkflowKV"));
    assert_eq!(resource.lifecycle, ResourceLifecycle::Admitted);
    assert!(matches!(
        resource.provenance,
        ResourceProvenance::InternalAuthority { ref notes }
            if notes.iter().any(|note| note.contains("standard WorkflowKV pilot"))
    ));
}

#[tokio::test]
async fn workflow_kv_pilot_preserves_explicit_admission_boundary() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");

    let StandardPilotBinding { binding_id, .. } = runtime_state
        .admit_standard_internal_pilot(StandardInternalPilot::workflow_kv(
            "kv",
            "store",
            Value::String("internal-value".to_string()),
        ))
        .await
        .expect("workflow kv pilot admits internal resource and binding");

    let unadmitted_ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone());
    let error = eval_invoke_act(
        invoke_expr("kv", "get", vec![Value::String("key".to_string())]),
        &unadmitted_ctx,
    )
    .await
    .expect_err("pilot binding must not be available without explicit admission");
    assert!(
        error.to_string().contains("capability kv not available"),
        "unexpected error: {error}"
    );

    let admitted_ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![binding_id]);
    assert!(
        eval_invoke_act(
            invoke_expr("kv", "get", vec![Value::String("key".to_string())]),
            &admitted_ctx,
        )
        .await
        .is_ok()
    );
}

#[tokio::test]
async fn frozen_clock_pilot_is_deterministic_without_host_time_provider() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_capability_interface_operations(
            CapabilityInterfaceId::new("Clock"),
            ["epoch_millis"],
        )
        .await
        .expect("interface operations registered");

    let StandardPilotBinding {
        binding_id,
        resource_id,
    } = runtime_state
        .admit_standard_internal_pilot(StandardInternalPilot::frozen_clock(
            "clock",
            "test-clock",
            1_700_000_000_000,
        ))
        .await
        .expect("frozen clock pilot admits internal resource and binding");

    let ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![binding_id]);
    assert_eq!(
        eval_invoke_act(invoke_expr("clock", "epoch_millis", vec![]), &ctx)
            .await
            .expect("internal frozen clock implementation executes"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::Int(1_700_000_000_000),
        ]))
    );

    assert!(
        !runtime_state.has_provider("time"),
        "internal frozen clock pilot should not require a host time provider"
    );

    let resource = runtime_state
        .resource_instance(resource_id)
        .await
        .expect("pilot clock resource is registered");
    assert_eq!(resource.type_id, ResourceTypeId::new("FrozenClock"));
    assert!(matches!(
        resource.provenance,
        ResourceProvenance::InternalAuthority { ref notes }
            if notes.iter().any(|note| note.contains("standard FrozenClock pilot"))
    ));
}

#[tokio::test]
async fn standard_pilot_rejects_pre_registered_internal_body() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");
    runtime_state
        .register_implementation_operation_body(
            ash_core::runtime::CapabilityImplementationId::new("__ash_standard_pilot.WorkflowKV"),
            "get",
            ash_interp::ImplementationOperationBody::new(
                Vec::<String>::new(),
                Expr::Literal(Value::String("evil".to_string())),
            ),
        )
        .await
        .expect("pre-existing body registration succeeds");

    let error = runtime_state
        .admit_standard_internal_pilot(StandardInternalPilot::workflow_kv(
            "kv",
            "store",
            Value::String("internal-value".to_string()),
        ))
        .await
        .expect_err("standard pilot must reject pre-registered internal body collisions");
    assert!(
        error.to_string().contains(
            "standard internal pilot body __ash_standard_pilot.WorkflowKV.get is already registered"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn standard_pilot_resource_metadata_is_stable() {
    assert_eq!(
        StandardPilotResource::WorkflowKv.resource_type_id(),
        ResourceTypeId::new("WorkflowKV")
    );
    assert_eq!(
        StandardPilotResource::FrozenClock.resource_type_id(),
        ResourceTypeId::new("FrozenClock")
    );
}
