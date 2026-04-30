use ash_parser::surface::{
    CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Definition, Expr,
    Literal, Program, ResourceField, ResourceTypeDef, Type as SurfaceType, Visibility, Workflow,
    WorkflowDef, WorkflowOwnedResource, WorkflowUsedBinding,
};
use ash_parser::token::Span;
use ash_typeck::error::TypeEnvError;
use ash_typeck::type_env::TypeEnv;

fn span() -> Span {
    Span::default()
}

fn ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        module: None,
        func: name.into(),
        args,
        span: span(),
    }
}

fn cap_iface(name: &str) -> CapabilityInterfaceDef {
    CapabilityInterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        operations: vec![CapabilityOperationSig {
            mode: CapabilityOperationMode::Observe,
            name: "read".into(),
            params: vec![],
            return_type: ty("String"),
            span: span(),
        }],
        span: span(),
    }
}

fn impl_op() -> CapabilityImplementationOperation {
    CapabilityImplementationOperation {
        mode: CapabilityOperationMode::Observe,
        name: "read".into(),
        params: vec![],
        return_type: ty("String"),
        body: Expr::Literal(Literal::String("ok".into())),
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

fn cap_impl(
    name: &str,
    interface: &str,
    dependencies: Vec<CapabilityImplementationDependency>,
) -> CapabilityImplementationDef {
    CapabilityImplementationDef {
        visibility: Visibility::Public,
        name: name.into(),
        interface: interface.into(),
        dependencies,
        operations: vec![impl_op()],
        span: span(),
    }
}

fn resource_type(name: &str, fields: Vec<(&str, SurfaceType)>) -> ResourceTypeDef {
    ResourceTypeDef {
        visibility: Visibility::Public,
        name: name.into(),
        fields: fields
            .into_iter()
            .map(|(name, ty)| ResourceField {
                name: name.into(),
                ty,
                span: span(),
            })
            .collect(),
        span: span(),
    }
}

fn workflow() -> WorkflowDef {
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

fn own(name: &str, ty: &str) -> WorkflowOwnedResource {
    WorkflowOwnedResource {
        name: name.into(),
        ty: SurfaceType::Name(ty.into()),
        span: span(),
    }
}

fn use_binding(name: &str, interface: &str, implementation: Expr) -> WorkflowUsedBinding {
    WorkflowUsedBinding {
        name: name.into(),
        interface: SurfaceType::Name(interface.into()),
        implementation,
        span: span(),
    }
}

fn env_with_fs() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("File", vec![("path", ty("String"))]))
        .unwrap();
    env.register_capability_interface(&cap_iface("Storage"))
        .unwrap();
    env
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

fn assert_typecheck_err_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    needles: &[&str],
) {
    let err = result.expect_err("typecheck should be rejected");
    let message = err.to_string();
    for needle in needles {
        assert!(
            message.contains(needle),
            "diagnostic should contain {needle:?}; got: {message}"
        );
    }
}

#[test]
fn resource_type_declaration_registers_in_type_env() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("File", vec![("path", ty("String"))]))
        .unwrap();
    assert!(env.has_resource_type("File"));
    assert_eq!(env.lookup_resource_type("File").unwrap().fields.len(), 1);
}

#[test]
fn duplicate_resource_type_declaration_is_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("File", vec![]))
        .unwrap();
    assert_invalid_contains(
        env.register_resource_type(&resource_type("File", vec![])),
        &["resource type", "File", "already"],
    );
}

#[test]
fn duplicate_resource_field_names_are_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    assert_invalid_contains(
        env.register_resource_type(&resource_type(
            "File",
            vec![("path", ty("String")), ("path", ty("String"))],
        )),
        &["resource type", "File", "duplicate field", "path"],
    );
}

#[test]
fn resource_type_field_with_unknown_ordinary_type_is_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    assert_invalid_contains(
        env.register_resource_type(&resource_type("File", vec![("x", ty("Missing"))])),
        &["resource type", "File"],
    );
}

#[test]
fn resource_requirement_names_registered_resource_type() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    assert!(env.has_capability_implementation("FileStorage"));
}

#[test]
fn resource_requirement_unknown_resource_type_is_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&cap_iface("Storage"))
        .unwrap();
    assert_invalid_contains(
        env.register_capability_implementation(&cap_impl(
            "Bad",
            "Storage",
            vec![dep(
                CapabilityImplementationDependencyKind::Resource,
                "file",
                ty("Missing"),
            )],
        )),
        &[
            "resource dependency",
            "file",
            "unknown resource type",
            "Missing",
        ],
    );
}

#[test]
fn resource_requirement_must_not_use_ordinary_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&cap_iface("Storage"))
        .unwrap();

    assert_invalid_contains(
        env.register_capability_implementation(&cap_impl(
            "Bad",
            "Storage",
            vec![dep(
                CapabilityImplementationDependencyKind::Resource,
                "file",
                ty("String"),
            )],
        )),
        &[
            "resource dependency",
            "file",
            "unknown resource type",
            "String",
        ],
    );
}

#[test]
fn capability_requirement_unknown_interface_is_rejected() {
    let mut env = env_with_fs();
    assert_invalid_contains(
        env.register_capability_implementation(&cap_impl(
            "Bad",
            "Storage",
            vec![dep(
                CapabilityImplementationDependencyKind::Capability,
                "other",
                ty("MissingIface"),
            )],
        )),
        &[
            "capability dependency",
            "other",
            "unknown capability interface",
            "MissingIface",
        ],
    );
}

#[test]
fn resource_and_capability_dependencies_remain_unavailable_as_operation_body_values() {
    let mut env = env_with_fs();
    env.register_capability_interface(&cap_iface("Logger"))
        .unwrap();
    assert_invalid_contains(
        env.register_capability_implementation(&CapabilityImplementationDef {
            operations: vec![CapabilityImplementationOperation {
                body: var("file"),
                ..impl_op()
            }],
            ..cap_impl(
                "Bad",
                "Storage",
                vec![
                    dep(
                        CapabilityImplementationDependencyKind::Resource,
                        "file",
                        ty("File"),
                    ),
                    dep(
                        CapabilityImplementationDependencyKind::Capability,
                        "log",
                        ty("Logger"),
                    ),
                ],
            )
        }),
        &[
            "invalid capability implementation operation body",
            "unbound variable",
        ],
    );
}

#[test]
fn type_check_program_registers_resource_types_before_capability_implementations() {
    let program = Program {
        definitions: vec![
            Definition::CapabilityInterface(cap_iface("Storage")),
            Definition::CapabilityImplementation(cap_impl(
                "FileStorage",
                "Storage",
                vec![dep(
                    CapabilityImplementationDependencyKind::Resource,
                    "file",
                    ty("File"),
                )],
            )),
            Definition::ResourceType(resource_type("File", vec![("path", ty("String"))])),
        ],
        helper_workflows: vec![],
        workflow: workflow(),
    };
    ash_typeck::type_check_program(&program).unwrap();
}

#[test]
fn workflow_owns_registered_resource_type_is_accepted() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
}

#[test]
fn workflow_owns_unknown_resource_type_is_rejected() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "Missing")];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "owned resource",
            "input",
            "unknown resource type",
            "Missing",
        ],
    );
}

#[test]
fn workflow_duplicate_owned_resource_names_are_rejected() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File"), own("input", "File")];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["duplicate owned resource", "input"],
    );
}

#[test]
fn owned_resource_is_not_pure_workflow_variable() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    wf.body = Workflow::Ret {
        expr: var("input"),
        span: span(),
    };
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["UnboundVariable", "input"],
    );
}

#[test]
fn workflow_uses_matching_impl_with_owned_resource_dependency_is_accepted() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("FileStorage", vec![var("input")]),
    )];
    ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
}

#[test]
fn workflow_uses_unknown_interface_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl("FileStorage", "Storage", vec![]))
        .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding("store", "Missing", call("FileStorage", vec![]))];

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "used binding",
            "store",
            "unknown capability interface",
            "Missing",
        ],
    );
}

#[test]
fn workflow_uses_unknown_implementation_is_rejected() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding("store", "Storage", call("MissingImpl", vec![]))];

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "used binding",
            "store",
            "unknown implementation",
            "MissingImpl",
        ],
    );
}

#[test]
fn workflow_duplicate_used_binding_names_are_rejected() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl("FileStorage", "Storage", vec![]))
        .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![
        use_binding("store", "Storage", call("FileStorage", vec![])),
        use_binding("store", "Storage", call("FileStorage", vec![])),
    ];

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["duplicate used binding", "store"],
    );
}

#[test]
fn workflow_uses_resource_dependency_arg_must_be_owned_resource_binding() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("FileStorage", vec![var("input")]),
    )];

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["resource dependency", "file", "owned resource", "input"],
    );
}

#[test]
fn workflow_uses_impl_for_wrong_interface_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_interface(&cap_iface("Other"))
        .unwrap();
    env.register_capability_implementation(&cap_impl("FileStorage", "Storage", vec![]))
        .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding("store", "Other", call("FileStorage", vec![]))];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "implementation",
            "FileStorage",
            "targets",
            "Storage",
            "not",
            "Other",
        ],
    );
}

#[test]
fn workflow_uses_dependency_arity_mismatch_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    wf.used_bindings = vec![use_binding("store", "Storage", call("FileStorage", vec![]))];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["dependency arity", "expected 1", "found 0"],
    );
}

#[test]
fn workflow_uses_resource_dependency_type_mismatch_is_rejected() {
    let mut env = env_with_fs();
    env.register_resource_type(&resource_type("Socket", vec![]))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.owned_resources = vec![own("sock", "Socket")];
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("FileStorage", vec![var("sock")]),
    )];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "resource dependency",
            "file",
            "expected",
            "File",
            "found",
            "Socket",
        ],
    );
}

#[test]
fn workflow_uses_non_call_implementation_expression_is_rejected() {
    let env = env_with_fs();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding("store", "Storage", var("FileStorage"))];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["unsupported", "uses", "implementation", "call"],
    );
}

#[test]
fn workflow_uses_capability_dependency_from_previous_binding_is_accepted() {
    let mut env = env_with_fs();
    env.register_capability_interface(&cap_iface("Cache"))
        .unwrap();
    env.register_capability_implementation(&cap_impl("BaseStorage", "Storage", vec![]))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "CachedStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Capability,
            "cache",
            ty("Cache"),
        )],
    ))
    .unwrap();
    env.register_capability_implementation(&cap_impl("MemoryCache", "Cache", vec![]))
        .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![
        use_binding("cache", "Cache", call("MemoryCache", vec![])),
        use_binding(
            "store",
            "Storage",
            call("CachedStorage", vec![var("cache")]),
        ),
    ];
    ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
}

#[test]
fn workflow_uses_capability_dependency_forward_reference_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_interface(&cap_iface("Cache"))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "CachedStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Capability,
            "cache",
            ty("Cache"),
        )],
    ))
    .unwrap();
    env.register_capability_implementation(&cap_impl("MemoryCache", "Cache", vec![]))
        .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![
        use_binding(
            "store",
            "Storage",
            call("CachedStorage", vec![var("cache")]),
        ),
        use_binding("cache", "Cache", call("MemoryCache", vec![])),
    ];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &["capability dependency", "cache", "earlier used binding"],
    );
}

#[test]
fn workflow_uses_capability_dependency_interface_mismatch_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_interface(&cap_iface("Cache"))
        .unwrap();
    env.register_capability_interface(&cap_iface("Logger"))
        .unwrap();
    env.register_capability_implementation(&cap_impl("LoggerImpl", "Logger", vec![]))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "CachedStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Capability,
            "cache",
            ty("Cache"),
        )],
    ))
    .unwrap();

    let mut wf = workflow();
    wf.used_bindings = vec![
        use_binding("logger", "Logger", call("LoggerImpl", vec![])),
        use_binding(
            "store",
            "Storage",
            call("CachedStorage", vec![var("logger")]),
        ),
    ];

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "capability dependency",
            "cache",
            "expected",
            "Cache",
            "found",
            "Logger",
        ],
    );
}

#[test]
fn workflow_uses_config_dependency_expression_type_is_checked() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "ConfiguredStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Config,
            "prefix",
            ty("String"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call(
            "ConfiguredStorage",
            vec![Expr::Literal(Literal::String("/tmp".into()))],
        ),
    )];
    ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
}

#[test]
fn workflow_uses_config_dependency_type_mismatch_is_rejected() {
    let mut env = env_with_fs();
    env.register_capability_implementation(&cap_impl(
        "ConfiguredStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Config,
            "prefix",
            ty("String"),
        )],
    ))
    .unwrap();
    let mut wf = workflow();
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("ConfiguredStorage", vec![Expr::Literal(Literal::Int(1))]),
    )];
    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &wf),
        &[
            "config dependency",
            "prefix",
            "expected",
            "String",
            "found",
            "Int",
        ],
    );
}
