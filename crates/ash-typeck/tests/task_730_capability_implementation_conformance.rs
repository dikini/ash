use ash_parser::surface::{
    BuiltinFnDef, CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Definition, Expr,
    FnDef, Literal, Param, Program, ResourceTypeDef, Type as SurfaceType, Visibility, Workflow,
    WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::Type;
use ash_typeck::error::TypeEnvError;
use ash_typeck::type_env::TypeEnv;

fn span() -> Span {
    Span::default()
}

fn ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn fn_ty(params: Vec<SurfaceType>, return_type: SurfaceType) -> SurfaceType {
    SurfaceType::Fn(params, Box::new(return_type))
}

fn param(name: &str, ty: SurfaceType) -> Param {
    Param {
        name: name.into(),
        ty,
    }
}

fn operation_sig(
    mode: CapabilityOperationMode,
    name: &str,
    params: Vec<Param>,
    return_type: SurfaceType,
) -> CapabilityOperationSig {
    CapabilityOperationSig {
        mode,
        name: name.into(),
        params,
        return_type,
        span: span(),
    }
}

fn interface(name: &str, operations: Vec<CapabilityOperationSig>) -> CapabilityInterfaceDef {
    CapabilityInterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        operations,
        span: span(),
    }
}

fn literal_string(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn literal_int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn literal_list(values: Vec<Literal>) -> Expr {
    Expr::Literal(Literal::List(values))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn impl_op(
    mode: CapabilityOperationMode,
    name: &str,
    params: Vec<Param>,
    return_type: SurfaceType,
    body: Expr,
) -> CapabilityImplementationOperation {
    CapabilityImplementationOperation {
        mode,
        name: name.into(),
        params,
        return_type,
        body,
        span: span(),
    }
}

fn dep(
    kind: CapabilityImplementationDependencyKind,
    name: &str,
    ty: SurfaceType,
) -> CapabilityImplementationDependency {
    CapabilityImplementationDependency {
        kind,
        name: name.into(),
        ty,
        span: span(),
    }
}

fn config_dep(name: &str, ty: SurfaceType) -> CapabilityImplementationDependency {
    dep(CapabilityImplementationDependencyKind::Config, name, ty)
}

fn resource_dep(name: &str, ty: SurfaceType) -> CapabilityImplementationDependency {
    dep(CapabilityImplementationDependencyKind::Resource, name, ty)
}

fn implementation(
    name: &str,
    interface: &str,
    dependencies: Vec<CapabilityImplementationDependency>,
    operations: Vec<CapabilityImplementationOperation>,
) -> CapabilityImplementationDef {
    CapabilityImplementationDef {
        visibility: Visibility::Public,
        name: name.into(),
        interface: interface.into(),
        dependencies,
        operations,
        span: span(),
    }
}

fn kv_interface() -> CapabilityInterfaceDef {
    interface(
        "KeyValueStore",
        vec![
            operation_sig(
                CapabilityOperationMode::Observe,
                "get",
                vec![param("key", ty("String"))],
                ty("String"),
            ),
            operation_sig(
                CapabilityOperationMode::Execute,
                "put",
                vec![param("key", ty("String")), param("value", ty("String"))],
                ty("Unit"),
            ),
        ],
    )
}

fn valid_kv_impl() -> CapabilityImplementationDef {
    implementation(
        "MemoryKV",
        "KeyValueStore",
        vec![],
        vec![
            impl_op(
                CapabilityOperationMode::Observe,
                "get",
                vec![param("key", ty("String"))],
                ty("String"),
                var("key"),
            ),
            impl_op(
                CapabilityOperationMode::Execute,
                "put",
                vec![param("key", ty("String")), param("value", ty("String"))],
                ty("Unit"),
                Expr::Literal(Literal::Null),
            ),
        ],
    )
}

fn env_with_kv_interface() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&kv_interface())
        .expect("fixture interface should register");
    env
}

fn fn_expr(param_name: &str, param_ty: &str, return_ty: &str, body: Expr) -> Expr {
    Expr::FnDef {
        params: vec![(param_name.into(), Some(param_ty.into()))],
        return_type: Some(return_ty.into()),
        body: Box::new(body),
        span: span(),
    }
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        func: name.into(),
        module: None,
        args,
        span: span(),
    }
}

fn main_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: None,
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Done { span: span() },
        contract: None,
        span: span(),
    }
}

fn assert_invalid_contains<T: std::fmt::Debug>(result: Result<T, TypeEnvError>, needles: &[&str]) {
    let err = result.expect_err("definition should be rejected");
    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    let message = err.to_string();
    for needle in needles {
        assert!(
            message.contains(needle),
            "diagnostic should contain {needle:?}; got: {message}"
        );
    }
}

#[test]
fn valid_capability_implementation_registers_against_known_interface() {
    let mut env = env_with_kv_interface();

    env.register_capability_implementation(&valid_kv_impl())
        .expect("implementation matching every interface operation should register");

    assert!(env.has_capability_implementation("MemoryKV"));
    let info = env
        .lookup_capability_implementation("MemoryKV")
        .expect("registered implementation should be available for binding checks");
    assert_eq!(info.name, "MemoryKV");
    assert_eq!(info.interface, "KeyValueStore");
    assert_eq!(info.operations.len(), 2);
    let get = info.operations.get("get").expect("get op should be stored");
    assert_eq!(get.mode, CapabilityOperationMode::Observe);
    assert_eq!(get.param_names, vec!["key"]);
    assert_eq!(get.params, vec![Type::String]);
    assert_eq!(get.return_type, Type::String);
}

#[test]
fn unknown_target_interface_is_rejected() {
    let mut env = TypeEnv::with_builtin_types();

    let err = env
        .register_capability_implementation(&valid_kv_impl())
        .expect_err("implementation target must name a known capability interface");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(err.to_string().contains("KeyValueStore"));
}

#[test]
fn missing_required_operation_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation
        .operations
        .retain(|op| op.name.as_ref() != "put");

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["missing", "put"],
    );
}

#[test]
fn extra_operation_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations.push(impl_op(
        CapabilityOperationMode::Observe,
        "delete",
        vec![param("key", ty("String"))],
        ty("Unit"),
        Expr::Literal(Literal::Null),
    ));

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["extra", "delete"],
    );
}

#[test]
fn duplicate_operation_is_rejected_before_overwrite() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations.push(impl_op(
        CapabilityOperationMode::Observe,
        "get",
        vec![param("key", ty("String"))],
        ty("String"),
        literal_string("fallback"),
    ));

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["duplicate", "get"],
    );
}

#[test]
fn mode_mismatch_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0].mode = CapabilityOperationMode::Execute;

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["mode", "get"],
    );
}

#[test]
fn arity_mismatch_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0]
        .params
        .push(param("extra", ty("String")));

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["arity", "get"],
    );
}

#[test]
fn parameter_type_mismatch_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0].params[0].ty = ty("Int");

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["parameter", "get"],
    );
}

#[test]
fn return_type_mismatch_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0].return_type = ty("Int");

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["return", "get"],
    );
}

#[test]
fn operation_body_type_mismatch_is_rejected() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0].body = literal_int(42);

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["body", "get", "String"],
    );
}

#[test]
fn direct_invoke_is_not_ambient_authority_for_operation_body() {
    let mut env = env_with_kv_interface();
    let mut implementation = valid_kv_impl();
    implementation.operations[0].body = Expr::Call {
        func: "invoke".into(),
        module: None,
        args: vec![
            literal_string("host"),
            literal_string("read"),
            literal_list(vec![]),
        ],
        span: span(),
    };

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["body", "invoke", "capability implementation"],
    );
}

#[test]
fn declared_config_dependency_is_visible_to_body_but_undeclared_identifier_is_rejected() {
    let counter = interface(
        "Counter",
        vec![operation_sig(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
        )],
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&counter)
        .expect("fixture interface should register");

    let uses_declared_config = implementation(
        "ConfiguredCounter",
        "Counter",
        vec![config_dep("limit_value", ty("Int"))],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
            var("limit_value"),
        )],
    );
    env.register_capability_implementation(&uses_declared_config)
        .expect("declared config dependency should be visible in operation body");

    let uses_undeclared_identifier = implementation(
        "BrokenCounter",
        "Counter",
        vec![],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
            var("limit_value"),
        )],
    );
    assert_invalid_contains(
        env.register_capability_implementation(&uses_undeclared_identifier),
        &["body", "limit", "limit_value"],
    );
}

#[test]
fn child_environments_inherit_implementations_without_leaking_new_child_registrations() {
    let mut parent = env_with_kv_interface();
    parent
        .register_capability_implementation(&valid_kv_impl())
        .expect("fixture implementation should register");

    let mut child = parent.extend();
    assert!(child.has_capability_implementation("MemoryKV"));

    let counter = interface(
        "Counter",
        vec![operation_sig(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
        )],
    );
    child
        .register_capability_interface(&counter)
        .expect("child-local interface should register");
    child
        .register_capability_implementation(&implementation(
            "ConfiguredCounter",
            "Counter",
            vec![config_dep("limit_value", ty("Int"))],
            vec![impl_op(
                CapabilityOperationMode::Observe,
                "limit",
                vec![],
                ty("Int"),
                var("limit_value"),
            )],
        ))
        .expect("child-local implementation should register");

    assert!(child.has_capability_implementation("ConfiguredCounter"));
    assert!(!parent.has_capability_implementation("ConfiguredCounter"));
}

#[test]
fn type_check_program_registers_capability_implementations_after_interfaces() {
    let program = Program {
        definitions: vec![
            Definition::CapabilityInterface(kv_interface()),
            Definition::CapabilityImplementation(valid_kv_impl()),
        ],
        helper_workflows: vec![],
        workflow: main_workflow(),
    };

    ash_typeck::type_check_program(&program)
        .expect("program type checking should accept conforming capability implementations");
}

#[test]
fn operation_body_accepts_pure_closure_result_in_effectful_context() {
    let callback_interface = interface(
        "CallbackSource",
        vec![operation_sig(
            CapabilityOperationMode::Observe,
            "callback",
            vec![],
            fn_ty(vec![ty("Int")], ty("Int")),
        )],
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&callback_interface)
        .expect("fixture interface should register");

    let implementation = implementation(
        "CallbackImpl",
        "CallbackSource",
        vec![],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "callback",
            vec![],
            fn_ty(vec![ty("Int")], ty("Int")),
            fn_expr("value", "Int", "Int", var("value")),
        )],
    );

    env.register_capability_implementation(&implementation)
        .expect("SPEC-072 pure closures stay Type::Fn even when checked inside effectful operation bodies");
}

#[test]
fn ambient_variable_and_function_authority_are_not_visible_to_operation_body() {
    let counter = interface(
        "Counter",
        vec![operation_sig(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
        )],
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&counter)
        .expect("fixture interface should register");
    env.bind_variable("ambient_limit", Type::Int);
    env.bind_variable("ambient_helper", Type::Fn(vec![], Box::new(Type::Int)));

    let uses_ambient_variable = implementation(
        "AmbientVariableCounter",
        "Counter",
        vec![],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
            var("ambient_limit"),
        )],
    );
    assert_invalid_contains(
        env.register_capability_implementation(&uses_ambient_variable),
        &["body", "limit", "ambient_limit"],
    );

    let uses_ambient_function = implementation(
        "AmbientFunctionCounter",
        "Counter",
        vec![],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("Int"),
            call("ambient_helper", vec![]),
        )],
    );
    assert_invalid_contains(
        env.register_capability_implementation(&uses_ambient_function),
        &["body", "limit", "ambient_helper"],
    );
}

#[test]
fn duplicate_dependency_names_are_rejected() {
    let mut env = env_with_kv_interface();
    let implementation = implementation(
        "DuplicateDepsKV",
        "KeyValueStore",
        vec![
            config_dep("endpoint", ty("String")),
            config_dep("endpoint", ty("String")),
        ],
        valid_kv_impl().operations,
    );

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["duplicate", "dependency", "endpoint"],
    );
}

#[test]
fn dependency_name_colliding_with_operation_parameter_is_rejected() {
    let mut env = env_with_kv_interface();
    let implementation = implementation(
        "ShadowingKV",
        "KeyValueStore",
        vec![config_dep("key", ty("String"))],
        valid_kv_impl().operations,
    );

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["dependency", "key", "parameter", "get"],
    );
}

#[test]
fn same_program_helper_function_is_not_ambient_authority_for_operation_body() {
    let program = Program {
        definitions: vec![
            Definition::Function(FnDef {
                visibility: Visibility::Public,
                name: "same_program_helper".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(ty("String")),
                proposition_tail: None,
                contract: None,
                body: literal_string("ambient"),
                span: span(),
            }),
            Definition::CapabilityInterface(kv_interface()),
            Definition::CapabilityImplementation(implementation(
                "HelperKV",
                "KeyValueStore",
                vec![],
                vec![
                    impl_op(
                        CapabilityOperationMode::Observe,
                        "get",
                        vec![param("key", ty("String"))],
                        ty("String"),
                        call("same_program_helper", vec![]),
                    ),
                    impl_op(
                        CapabilityOperationMode::Execute,
                        "put",
                        vec![param("key", ty("String")), param("value", ty("String"))],
                        ty("Unit"),
                        Expr::Literal(Literal::Null),
                    ),
                ],
            )),
        ],
        helper_workflows: vec![],
        workflow: main_workflow(),
    };

    let err = ash_typeck::type_check_program(&program)
        .expect_err("same-program pure helpers must not leak into capability impl bodies");
    let message = err.to_string();
    assert!(
        message.contains("same_program_helper"),
        "diagnostic should mention helper name; got: {message}"
    );
}

#[test]
fn builtin_function_is_not_ambient_authority_for_operation_body() {
    let program = Program {
        definitions: vec![
            Definition::BuiltinFn(BuiltinFnDef {
                visibility: Visibility::Public,
                name: "ambient_builtin".into(),
                type_params: vec![],
                params: vec![],
                return_type: ty("String"),
                proposition_tail: None,
                span: span(),
            }),
            Definition::CapabilityInterface(kv_interface()),
            Definition::CapabilityImplementation(implementation(
                "BuiltinKV",
                "KeyValueStore",
                vec![],
                vec![
                    impl_op(
                        CapabilityOperationMode::Observe,
                        "get",
                        vec![param("key", ty("String"))],
                        ty("String"),
                        call("ambient_builtin", vec![]),
                    ),
                    impl_op(
                        CapabilityOperationMode::Execute,
                        "put",
                        vec![param("key", ty("String")), param("value", ty("String"))],
                        ty("Unit"),
                        Expr::Literal(Literal::Null),
                    ),
                ],
            )),
        ],
        helper_workflows: vec![],
        workflow: main_workflow(),
    };

    let err = ash_typeck::type_check_program(&program)
        .expect_err("program builtins must not leak into capability impl bodies");
    let message = err.to_string();
    assert!(
        message.contains("ambient_builtin"),
        "diagnostic should mention builtin name; got: {message}"
    );
}

#[test]
fn resource_dependencies_are_metadata_not_body_values() {
    let counter = interface(
        "Counter",
        vec![operation_sig(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("String"),
        )],
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&ResourceTypeDef {
        visibility: Visibility::Public,
        name: "ResourceStore".into(),
        fields: vec![],
        span: span(),
    })
    .expect("fixture resource type should register");
    env.register_capability_interface(&counter)
        .expect("fixture interface should register");

    let implementation = implementation(
        "ResourceCounter",
        "Counter",
        vec![resource_dep("store", ty("ResourceStore"))],
        vec![impl_op(
            CapabilityOperationMode::Observe,
            "limit",
            vec![],
            ty("String"),
            var("store"),
        )],
    );

    assert_invalid_contains(
        env.register_capability_implementation(&implementation),
        &["body", "limit", "store"],
    );
}
