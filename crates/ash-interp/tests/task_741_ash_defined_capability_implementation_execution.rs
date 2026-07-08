use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use ash_core::runtime::{
    CapabilityImplementationId, CapabilityInterfaceId, ResourceId, ResourceTypeId,
};
use ash_core::{
    CapabilityBinding, CapabilityBindingId, Effect, EnvFrame, Expr, Provenance, Value, WorkflowId,
};
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::eval::eval_expr_async;
use ash_interp::{
    EntryOwnedResourceAdmission, ImplementationBindingAdmission,
    ImplementationBindingDependencySource, ImplementationOperationBody, PolicyEvaluator,
    RuntimeState,
};

fn invoke_expr(binding_name: &str, operation: &str, args: Vec<Value>) -> Expr {
    Expr::Call {
        func: "invoke".to_string(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String(binding_name.to_string())),
            Expr::Literal(Value::String(operation.to_string())),
            Expr::Literal(Value::list_from_vec(args)),
        ],
    }
}

async fn act_context(runtime_state: &RuntimeState) -> Context {
    let act_env =
        ActEnv::from_runtime_state(runtime_state, PolicyEvaluator::new(), Provenance::new()).await;
    Context::new().with_act_env(act_env)
}

async fn act_context_with_admitted_bindings(
    runtime_state: &RuntimeState,
    binding_ids: Vec<CapabilityBindingId>,
) -> Context {
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        runtime_state,
        &binding_ids,
        PolicyEvaluator::new(),
        Provenance::new(),
    )
    .await
    .expect("admitted bindings should project into ActEnv");
    Context::new()
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(binding_ids)
        .with_act_env(act_env)
}

async fn admit_host_provider_binding(
    runtime_state: &RuntimeState,
    name: &str,
    interface: &str,
    provider_name: &str,
    capability: &str,
) -> CapabilityBindingId {
    let binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        name,
        CapabilityInterfaceId::new(interface),
        provider_name,
        vec![capability.to_string()],
    );
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("host provider binding should admit");
    binding_id
}

async fn admit_store_resource(runtime_state: &RuntimeState) -> ResourceId {
    let resources = runtime_state
        .admit_entry_owned_resources(
            WorkflowId::new(),
            vec![EntryOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("entry-owned resource admission should succeed");
    resources["store"]
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
async fn registered_implementation_operation_body_executes_with_params_and_dependencies_in_scope() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface operations registered");

    let body = Expr::Constructor {
        name: "BodyResult".to_string(),
        fields: vec![
            (
                "key".to_string(),
                Expr::Variable {
                    name: "key".to_string(),
                    span: Default::default(),
                },
            ),
            (
                "prefix".to_string(),
                Expr::Variable {
                    name: "prefix".to_string(),
                    span: Default::default(),
                },
            ),
        ],
    };
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(vec!["key"], body),
        )
        .await
        .expect("body registration succeeds");

    let mut resource_sources = HashMap::new();
    resource_sources.insert("store".to_string(), resource_id);
    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "prefix",
                Value::String("pfx".to_string()),
            ))
            .with_requested_operations(["get"]),
            &resource_sources,
        )
        .await
        .expect("implementation binding can be admitted");

    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;
    let result = eval_invoke_act(
        invoke_expr("kv", "get", vec![Value::String("a".to_string())]),
        &ctx,
    )
    .await
    .expect("registered implementation body executes");

    assert_eq!(
        result,
        Value::list_from_vec(vec![
            Value::ActEnvToken,
            Value::Variant {
                name: "BodyResult".to_string(),
                fields: Box::new(vec![
                    ("key".to_string(), Value::String("a".to_string())),
                    ("prefix".to_string(), Value::String("pfx".to_string())),
                ]),
            }
        ])
    );
}

#[tokio::test]
async fn implementation_binding_without_registered_body_fails_as_operational_failure() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    let mut resource_sources = HashMap::new();
    resource_sources.insert("store".to_string(), resource_id);
    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &resource_sources,
        )
        .await
        .expect("binding admission succeeds");

    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;
    let err = eval_invoke_act(invoke_expr("kv", "get", vec![]), &ctx)
        .await
        .expect_err("missing body should fail");
    assert!(
        err.to_string().contains("operational failure")
            && err
                .to_string()
                .contains("no Ash-defined operation body registered"),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn implementation_binding_invocation_requires_explicit_admission() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(
                Vec::<String>::new(),
                Expr::Literal(Value::String("ok".to_string())),
            ),
        )
        .await
        .expect("body registration succeeds");
    let mut resource_sources = HashMap::new();
    resource_sources.insert("store".to_string(), resource_id);
    runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &resource_sources,
        )
        .await
        .expect("binding admission succeeds");

    let ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![]);
    let err = eval_invoke_act(invoke_expr("kv", "get", vec![]), &ctx)
        .await
        .expect_err("ambient lookup by binding name must not bypass explicit admission");
    assert!(
        err.to_string().contains("lacks RuntimeKernel admission"),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn host_provider_binding_behavior_remains_unchanged() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("host".to_string()))),
        ),
    );
    let binding_id =
        admit_host_provider_binding(&runtime_state, "sensor", "Sensor", "sensor", "sensor.read")
            .await;
    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;
    let result = eval_invoke_act(invoke_expr("sensor", "read", vec![]), &ctx)
        .await
        .expect("host provider invoke should still dispatch");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("host".to_string())])
    );
}

#[tokio::test]
async fn operation_body_can_invoke_only_explicit_capability_dependency_aliases() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock-provider",
        Arc::new(
            MockProvider::new("clock-provider", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let host_binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "entry-clock",
        CapabilityInterfaceId::new("Clock"),
        "clock-provider",
        vec!["clock-provider.read".to_string()],
    );
    let host_binding_id = host_binding.id;
    runtime_state
        .admit_capability_binding(host_binding)
        .await
        .expect("host binding admitted");

    let body = invoke_expr("clock", "read", vec![]);
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(Vec::<String>::new(), body).returns_act(),
        )
        .await
        .expect("body registration succeeds");

    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "entry-clock",
                CapabilityInterfaceId::new("Clock"),
            )),
            &HashMap::<String, ResourceId>::new(),
        )
        .await
        .expect("implementation binding can be admitted");

    let ctx =
        act_context_with_admitted_bindings(&runtime_state, vec![binding_id, host_binding_id]).await;
    let result = eval_invoke_act(invoke_expr("kv", "get", vec![]), &ctx)
        .await
        .expect("body may invoke dependency alias admitted for the implementation");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("tick".to_string())])
    );
}

#[tokio::test]
async fn resource_dependencies_are_not_exposed_as_pure_body_variables() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(
                Vec::<String>::new(),
                Expr::Variable {
                    name: "store".to_string(),
                    span: Default::default(),
                },
            ),
        )
        .await
        .expect("body registration succeeds");

    let mut resource_sources = HashMap::new();
    resource_sources.insert("store".to_string(), resource_id);
    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &resource_sources,
        )
        .await
        .expect("implementation binding can be admitted");

    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;
    let err = eval_invoke_act(invoke_expr("kv", "get", vec![]), &ctx)
        .await
        .expect_err("resource handles must remain environment-owned, not pure variables");
    assert!(
        err.to_string().contains("operational failure")
            && err.to_string().contains("undefined variable: store"),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn capability_dependency_alias_variable_resolves_to_declared_alias() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock-provider",
        Arc::new(
            MockProvider::new("clock-provider", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let host_binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "entry-clock",
        CapabilityInterfaceId::new("Clock"),
        "clock-provider",
        vec!["clock-provider.read".to_string()],
    );
    let host_binding_id = host_binding.id;
    runtime_state
        .admit_capability_binding(host_binding)
        .await
        .expect("host binding admitted");

    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(
                Vec::<String>::new(),
                invoke_expr("clock", "read", vec![Value::String("ignored".to_string())]),
            )
            .returns_act(),
        )
        .await
        .expect("body registration succeeds");

    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "entry-clock",
                CapabilityInterfaceId::new("Clock"),
            )),
            &HashMap::<String, ResourceId>::new(),
        )
        .await
        .expect("implementation binding can be admitted");

    let body = invoke_expr("clock", "read", vec![]);
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplAlias"),
            "read_alias",
            ImplementationOperationBody::new(Vec::<String>::new(), body).returns_act(),
        )
        .await
        .expect("alias body registration succeeds");
    let alias_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "alias",
                CapabilityInterfaceId::new("Alias"),
                CapabilityImplementationId::new("ImplAlias"),
            )
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "entry-clock",
                CapabilityInterfaceId::new("Clock"),
            )),
            &HashMap::<String, ResourceId>::new(),
        )
        .await
        .expect("alias implementation binding can be admitted");

    let ctx = act_context_with_admitted_bindings(
        &runtime_state,
        vec![binding_id, alias_binding_id, host_binding_id],
    )
    .await;
    let result = eval_invoke_act(invoke_expr("alias", "read_alias", vec![]), &ctx)
        .await
        .expect("capability dependency variable resolves to declared alias");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("tick".to_string())])
    );
}

#[tokio::test]
async fn pure_closure_results_are_returned_without_implicit_act_forcing() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "make_closure",
            ImplementationOperationBody::new(
                Vec::<String>::new(),
                Expr::Literal(Value::Closure {
                    params: Vec::new(),
                    body: Box::new(Expr::Literal(Value::String("pure".to_string()))),
                    env: Arc::new(EnvFrame::new()),
                }),
            ),
        )
        .await
        .expect("body registration succeeds");
    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &HashMap::from([("store".to_string(), resource_id)]),
        )
        .await
        .expect("binding admission succeeds");

    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;
    let result = eval_invoke_act(invoke_expr("kv", "make_closure", vec![]), &ctx)
        .await
        .expect("pure closure result should be transported");
    let items = result
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected Act result wrapper"));
    assert!(matches!(
        items.as_slice(),
        [Value::ActEnvToken, Value::Closure { .. }]
    ));
}

#[tokio::test]
async fn body_arity_and_evaluation_failures_are_operationally_attributed() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("ImplKV"),
            "get",
            ImplementationOperationBody::new(
                vec!["key"],
                Expr::Variable {
                    name: "missing".to_string(),
                    span: Default::default(),
                },
            ),
        )
        .await
        .expect("body registration succeeds");
    let binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("ImplKV"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &HashMap::from([("store".to_string(), resource_id)]),
        )
        .await
        .expect("binding admission succeeds");
    let ctx = act_context_with_admitted_bindings(&runtime_state, vec![binding_id]).await;

    let arity_err = eval_invoke_act(invoke_expr("kv", "get", vec![]), &ctx)
        .await
        .expect_err("wrong arity should fail");
    assert!(
        arity_err.to_string().contains("operational failure")
            && arity_err.to_string().contains("expected 1 arguments"),
        "unexpected error: {arity_err:?}"
    );

    let body_err = eval_invoke_act(
        invoke_expr("kv", "get", vec![Value::String("a".to_string())]),
        &ctx,
    )
    .await
    .expect_err("body evaluation should fail");
    assert!(
        body_err.to_string().contains("operational failure")
            && body_err
                .to_string()
                .contains("Ash-defined operation body kv.get failed"),
        "unexpected error: {body_err:?}"
    );
}

#[tokio::test]
async fn implementation_dependency_can_invoke_nested_implementation_binding_by_alias() {
    let runtime_state = RuntimeState::new();
    let resource_id = admit_store_resource(&runtime_state).await;
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("InnerImpl"),
            "read",
            ImplementationOperationBody::new(
                Vec::<String>::new(),
                Expr::Literal(Value::String("inner".to_string())),
            ),
        )
        .await
        .expect("inner body registration succeeds");
    let inner_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "inner-source",
                CapabilityInterfaceId::new("Inner"),
                CapabilityImplementationId::new("InnerImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &HashMap::from([("store".to_string(), resource_id)]),
        )
        .await
        .expect("inner binding admission succeeds");

    let outer_body = Expr::Call {
        func: "invoke".to_string(),
        module: None,
        arguments: vec![
            Expr::Variable {
                name: "inner".to_string(),
                span: Default::default(),
            },
            Expr::Literal(Value::String("read".to_string())),
            Expr::Literal(Value::list_nil()),
        ],
    };
    runtime_state
        .register_implementation_operation_body(
            CapabilityImplementationId::new("OuterImpl"),
            "read_outer",
            ImplementationOperationBody::new(Vec::<String>::new(), outer_body).returns_act(),
        )
        .await
        .expect("outer body registration succeeds");
    let outer_binding_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "outer",
                CapabilityInterfaceId::new("Outer"),
                CapabilityImplementationId::new("OuterImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::capability(
                "inner",
                "inner-source",
                CapabilityInterfaceId::new("Inner"),
            )),
            &HashMap::<String, ResourceId>::new(),
        )
        .await
        .expect("outer binding admission succeeds");

    let ctx = act_context(&runtime_state)
        .await
        .with_runtime_state(runtime_state.clone())
        .with_admitted_capability_bindings(vec![outer_binding_id]);
    let result = eval_invoke_act(invoke_expr("outer", "read_outer", vec![]), &ctx)
        .await
        .expect("outer body may invoke nested implementation dependency by alias");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("inner".to_string())])
    );
    assert_ne!(inner_binding_id, outer_binding_id);
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        failure_persistence: None,
        cases: 32,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn operation_body_dependency_scope_preserves_config_values_without_exposing_resources(
        config_value in "[a-z][a-z0-9_]{0,12}"
    ) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let runtime_state = RuntimeState::new();
            let resource_id = admit_store_resource(&runtime_state).await;
            let mut resource_sources = HashMap::new();
            resource_sources.insert("store".to_string(), resource_id);
            let binding_id = runtime_state
                .admit_implementation_binding(
                    ImplementationBindingAdmission::new(
                        "kv",
                        CapabilityInterfaceId::new("KeyValue"),
                        CapabilityImplementationId::new("ImplKV"),
                    )
                    .with_dependency(ImplementationBindingDependencySource::resource(
                        "store",
                        ResourceTypeId::new("KvStore"),
                    ))
                    .with_dependency(ImplementationBindingDependencySource::config(
                        "cfg",
                        Value::String(config_value.clone()),
                    )),
                    &resource_sources,
                )
                .await
                .expect("binding admission should preserve resource authority without widening");

            runtime_state
                .register_implementation_operation_body(
                    CapabilityImplementationId::new("ImplKV"),
                    "read",
                    ImplementationOperationBody::new(
                        Vec::<String>::new(),
                        Expr::Constructor {
                            name: "Scope".to_string(),
                            fields: vec![
                                (
                                    "config".to_string(),
                                    Expr::Variable {
                                        name: "cfg".to_string(),
                                        span: Default::default(),
                                    },
                                ),
                                (
                                    "resource".to_string(),
                                    Expr::Variable {
                                        name: "store".to_string(),
                                        span: Default::default(),
                                    },
                                ),
                            ],
                        },
                    ),
                )
                .await
                .expect("operation body registration succeeds");

            let ctx = act_context(&runtime_state)
                .await
                .with_runtime_state(runtime_state.clone())
                .with_admitted_capability_bindings(vec![binding_id]);
            let err = eval_invoke_act(invoke_expr("kv", "read", vec![]), &ctx)
                .await
                .expect_err("resource dependency aliases are authority, not pure body variables");
            let rendered = err.to_string();
            prop_assert!(rendered.contains("store") || rendered.contains("Ash-defined operation body"));
            prop_assert!(rendered.contains("kv") || rendered.contains("ImplKV"));
            Ok(())
        })?;
    }
}
