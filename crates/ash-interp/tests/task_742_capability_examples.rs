use std::collections::HashMap;
use std::sync::Arc;

use ash_core::runtime::{
    CapabilityBindingId, CapabilityImplementationId, CapabilityInterfaceId, ResourceId,
    ResourceTypeId,
};
use ash_core::{CapabilityBinding, Effect, Expr, Provenance, Value, WorkflowId};
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::eval::eval_expr_async;
use ash_interp::{
    ImplementationBindingAdmission, ImplementationBindingDependencySource,
    ImplementationOperationBody, PolicyEvaluator, RuntimeState, WorkflowOwnedResourceAdmission,
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

fn field_expr(name: &str, expr: Expr) -> (String, Expr) {
    (name.to_string(), expr)
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.to_string(),
        span: Default::default(),
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

async fn admit_resource(runtime_state: &RuntimeState, name: &str) -> ResourceId {
    let resources = runtime_state
        .admit_workflow_owned_resources(
            WorkflowId::new(),
            vec![WorkflowOwnedResourceAdmission::new(
                name,
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("workflow-owned resource admission succeeds");
    resources[name]
}

#[tokio::test]
async fn mock_internal_kv_can_substitute_for_host_provider_binding() {
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
            invoke_expr("kv", "get", vec![Value::String("k".to_string())],),
            &host_ctx
        )
        .await
        .expect("host-backed kv binding executes"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("prod-value".to_string()),
        ]))
    );

    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("MockInternalKV"),
            "get",
            ImplementationOperationBody::new(vec!["key"], var("fixture")),
        )
        .await
        .expect("mock body registration succeeds");
    let resource_id = admit_resource(&runtime_state, "store").await;
    let mut resources = HashMap::new();
    resources.insert("store".to_string(), resource_id);
    let mock_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "mock-kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("MockInternalKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "fixture",
                Value::String("mock-value".to_string()),
            ))
            .with_requested_operations(["get"]),
            &resources,
        )
        .await
        .expect("mock implementation binding admission succeeds");

    let mock_ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![mock_binding_id]);
    assert_eq!(
        eval_invoke_act(
            invoke_expr("mock-kv", "get", vec![Value::String("k".to_string())]),
            &mock_ctx,
        )
        .await
        .expect("mock implementation binding executes"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("mock-value".to_string()),
        ]))
    );
}

#[tokio::test]
async fn logging_cache_adapter_invokes_inner_key_value_dependency() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");

    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("InnerKV"),
            "get",
            ImplementationOperationBody::new(vec!["key"], var("fixture")),
        )
        .await
        .expect("inner body registration succeeds");
    let backing_resource_id = admit_resource(&runtime_state, "backing").await;
    let mut backing_resources = HashMap::new();
    backing_resources.insert("store".to_string(), backing_resource_id);
    let inner_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "inner-kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("InnerKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "fixture",
                Value::String("inner-value".to_string()),
            ))
            .with_requested_operations(["get"]),
            &backing_resources,
        )
        .await
        .expect("inner implementation binding admission succeeds");

    let adapter_body = invoke_expr("inner", "get", vec![Value::String("cache-key".to_string())]);
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("LoggingCacheKV"),
            "get",
            ImplementationOperationBody::new(vec!["key"], adapter_body).returns_act(),
        )
        .await
        .expect("adapter body registration succeeds");
    let cache_resource_id = admit_resource(&runtime_state, "cache").await;
    let mut cache_resources = HashMap::new();
    cache_resources.insert("cache".to_string(), cache_resource_id);
    let adapter_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "cached-kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("LoggingCacheKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "cache",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::capability(
                "inner",
                "inner-kv",
                CapabilityInterfaceId::new("KeyValue"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "prefix",
                Value::String("cache".to_string()),
            ))
            .with_requested_operations(["get"]),
            &cache_resources,
        )
        .await
        .expect("adapter implementation binding admission succeeds");

    let ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![adapter_binding_id, inner_binding_id]);
    assert_eq!(
        eval_invoke_act(
            invoke_expr("cached-kv", "get", vec![Value::String("k".to_string())]),
            &ctx,
        )
        .await
        .expect("adapter invokes inner dependency alias"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("inner-value".to_string()),
        ]))
    );
}

#[tokio::test]
async fn recording_adapter_pilot_returns_replayable_envelope_without_persistent_replay_claim() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");
    let envelope_body = Expr::Constructor {
        name: "RecordedCall".to_string(),
        fields: vec![
            field_expr("operation", Expr::Literal(Value::String("get".to_string()))),
            field_expr("key", var("key")),
            field_expr("label", var("label")),
        ],
    };
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("RecordingKV"),
            "get",
            ImplementationOperationBody::new(vec!["key"], envelope_body),
        )
        .await
        .expect("recording body registration succeeds");

    let log_resource_id = admit_resource(&runtime_state, "log").await;
    let mut resources = HashMap::new();
    resources.insert("log".to_string(), log_resource_id);
    let recording_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "recording-kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("RecordingKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "log",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "label",
                Value::String("session-a".to_string()),
            ))
            .with_requested_operations(["get"]),
            &resources,
        )
        .await
        .expect("recording implementation binding admission succeeds");

    let ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![recording_binding_id]);
    assert_eq!(
        eval_invoke_act(
            invoke_expr(
                "recording-kv",
                "get",
                vec![Value::String("alpha".to_string())]
            ),
            &ctx,
        )
        .await
        .expect("recording pilot returns envelope"),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::variant(
                "RecordedCall",
                vec![
                    ("operation", Value::String("get".to_string())),
                    ("key", Value::String("alpha".to_string())),
                    ("label", Value::String("session-a".to_string())),
                ],
            ),
        ]))
    );
}
