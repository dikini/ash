use ash_parser::surface::{
    CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Definition, Expr,
    Literal, Param, Program, ResourceField, ResourceTypeDef, Type as SurfaceType, Visibility,
    Workflow, WorkflowDef, WorkflowOwnedResource, WorkflowUsedBinding,
};
use ash_parser::token::Span;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{AuthorityProvenanceKind, ProvenanceSourceKind};

fn span() -> Span {
    Span::default()
}

fn ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn param(name: &str, ty: SurfaceType) -> Param {
    Param {
        name: name.into(),
        ty,
    }
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn string_lit(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn unit_lit() -> Expr {
    Expr::Literal(Literal::Null)
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        module: None,
        func: name.into(),
        args,
        span: span(),
    }
}

fn binding_call(binding: &str, operation: &str, args: Vec<Expr>) -> Expr {
    Expr::FnApply {
        func: Box::new(Expr::FieldAccess {
            base: Box::new(var(binding)),
            field: operation.into(),
            span: span(),
        }),
        args,
        span: span(),
    }
}

fn iface(name: &str, read_return: SurfaceType) -> CapabilityInterfaceDef {
    CapabilityInterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        operations: vec![
            CapabilityOperationSig {
                mode: CapabilityOperationMode::Observe,
                name: "read".into(),
                params: vec![param("key", ty("String"))],
                return_type: read_return,
                span: span(),
            },
            CapabilityOperationSig {
                mode: CapabilityOperationMode::Execute,
                name: "write".into(),
                params: vec![param("key", ty("String")), param("value", ty("String"))],
                return_type: ty("Unit"),
                span: span(),
            },
        ],
        span: span(),
    }
}

fn resource_type(name: &str) -> ResourceTypeDef {
    ResourceTypeDef {
        visibility: Visibility::Public,
        name: name.into(),
        fields: vec![ResourceField {
            name: "path".into(),
            ty: ty("String"),
            span: span(),
        }],
        span: span(),
    }
}

fn dependency(
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

fn impl_op(
    mode: CapabilityOperationMode,
    name: &str,
    params: Vec<(&str, SurfaceType)>,
    return_type: SurfaceType,
    body: Expr,
) -> CapabilityImplementationOperation {
    CapabilityImplementationOperation {
        mode,
        name: name.into(),
        params: params
            .into_iter()
            .map(|(name, ty)| Param {
                name: name.into(),
                ty,
            })
            .collect(),
        return_type,
        body,
        span: span(),
    }
}

fn file_storage_impl(interface: &str) -> CapabilityImplementationDef {
    CapabilityImplementationDef {
        visibility: Visibility::Public,
        name: "FileStorage".into(),
        interface: interface.into(),
        dependencies: vec![dependency(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("FileResource"),
        )],
        operations: vec![
            impl_op(
                CapabilityOperationMode::Observe,
                "read",
                vec![("key", ty("String"))],
                ty("String"),
                string_lit("value"),
            ),
            impl_op(
                CapabilityOperationMode::Execute,
                "write",
                vec![("key", ty("String")), ("value", ty("String"))],
                ty("Unit"),
                unit_lit(),
            ),
        ],
        span: span(),
    }
}

fn wrong_read_param_impl() -> CapabilityImplementationDef {
    let mut implementation = file_storage_impl("Storage");
    implementation.operations[0].params = vec![param("key", ty("Int"))];
    implementation
}

fn host_widening_impl() -> CapabilityImplementationDef {
    let mut implementation = file_storage_impl("Storage");
    implementation.name = "HostWideningStorage".into();
    implementation.operations[0].body =
        call("invoke", vec![string_lit("fs.read"), string_lit("/tmp/x")]);
    implementation
}

fn owned_file() -> WorkflowOwnedResource {
    WorkflowOwnedResource {
        name: "file".into(),
        ty: ty("FileResource"),
        span: span(),
    }
}

fn used_binding(name: &str, interface: &str, implementation: Expr) -> WorkflowUsedBinding {
    WorkflowUsedBinding {
        name: name.into(),
        interface: ty(interface),
        implementation,
        span: span(),
    }
}

fn storage_binding_with(args: Vec<Expr>) -> WorkflowUsedBinding {
    used_binding("store", "Storage", call("FileStorage", args))
}

fn workflow(body_expr: Expr, used_bindings: Vec<WorkflowUsedBinding>) -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(ty("String")),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![owned_file()],
        used_bindings,
        body: Workflow::Ret {
            expr: body_expr,
            span: span(),
        },
        contract: None,
        span: span(),
    }
}

fn packet_program(extra_definitions: Vec<Definition>, workflow: WorkflowDef) -> Program {
    let mut definitions = vec![
        Definition::ResourceType(resource_type("FileResource")),
        Definition::CapabilityInterface(iface("Storage", ty("String"))),
        Definition::CapabilityInterface(iface("Metrics", ty("String"))),
        Definition::CapabilityImplementation(file_storage_impl("Storage")),
    ];
    definitions.extend(extra_definitions);
    Program {
        definitions,
        helper_workflows: vec![],
        workflow,
    }
}

fn assert_typecheck_err_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    needles: &[&str],
) {
    let err = result.expect_err("typecheck should reject the malformed packet");
    let message = err.to_string();
    for needle in needles {
        assert!(
            message.contains(needle),
            "diagnostic should contain {needle:?}; got: {message}"
        );
    }
}

fn prepopulated_import_env() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("FileResource"))
        .expect("imported resource metadata should register");
    env.register_capability_interface(&iface("Storage", ty("String")))
        .expect("imported interface metadata should register");
    env.register_capability_implementation(&file_storage_impl("Storage"))
        .expect("imported implementation metadata should register");
    env
}

#[test]
fn valid_packet_typechecks_and_records_integrated_static_metadata() {
    let program = packet_program(
        vec![],
        workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![storage_binding_with(vec![var("file")])],
        ),
    );

    let result = ash_typeck::type_check_program(&program)
        .expect("complete interface/impl/resource/binding packet should typecheck");

    let file = result
        .authority_provenance
        .resource_bindings
        .iter()
        .find(|binding| binding.name == "file")
        .expect("workflow owned resource provenance should be recorded");
    assert_eq!(file.resource_type, "FileResource");
    assert_eq!(file.authority, AuthorityProvenanceKind::Internal);

    let store = result
        .authority_provenance
        .capability_bindings
        .iter()
        .find(|binding| binding.name == "store")
        .expect("workflow binding provenance should be recorded");
    assert_eq!(store.interface, "Storage");
    assert_eq!(store.implementation, "FileStorage");
    assert_eq!(store.authority, AuthorityProvenanceKind::Internal);
    assert_eq!(store.sources.len(), 1);
    assert_eq!(store.sources[0].kind, ProvenanceSourceKind::Resource);
    assert_eq!(store.sources[0].binding_name, "file");
    assert_eq!(store.sources[0].target_name, "FileResource");
}

#[test]
fn wrong_implementation_target_is_rejected_at_program_integration_boundary() {
    let program = packet_program(
        vec![],
        workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![used_binding(
                "store",
                "Metrics",
                call("FileStorage", vec![var("file")]),
            )],
        ),
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_program(&program),
        &["FileStorage", "targets", "Storage", "not", "Metrics"],
    );
}

#[test]
fn missing_required_dependency_is_rejected_before_binding_operation_resolution() {
    let program = packet_program(
        vec![],
        workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![storage_binding_with(vec![])],
        ),
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_program(&program),
        &["FileStorage", "expected 1", "found 0"],
    );
}

#[test]
fn wrong_operation_call_type_is_rejected_after_valid_binding_admission() {
    let program = packet_program(
        vec![],
        workflow(
            binding_call("store", "read", vec![int_lit(42)]),
            vec![storage_binding_with(vec![var("file")])],
        ),
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_program(&program),
        &["store", "read", "String", "Int"],
    );
}

#[test]
fn wrong_implementation_operation_type_is_rejected_during_program_registration() {
    let program = Program {
        definitions: vec![
            Definition::ResourceType(resource_type("FileResource")),
            Definition::CapabilityInterface(iface("Storage", ty("String"))),
            Definition::CapabilityImplementation(wrong_read_param_impl()),
        ],
        helper_workflows: vec![],
        workflow: workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![storage_binding_with(vec![var("file")])],
        ),
    };

    assert_typecheck_err_contains(
        ash_typeck::type_check_program(&program),
        &[
            "FileStorage",
            "read",
            "parameter 0 type mismatch",
            "String",
            "Int",
        ],
    );
}

#[test]
fn authority_widening_attempt_through_ambient_invoke_is_rejected() {
    let program = Program {
        definitions: vec![
            Definition::ResourceType(resource_type("FileResource")),
            Definition::CapabilityInterface(iface("Storage", ty("String"))),
            Definition::CapabilityImplementation(host_widening_impl()),
        ],
        helper_workflows: vec![],
        workflow: workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![used_binding(
                "store",
                "Storage",
                call("HostWideningStorage", vec![var("file")]),
            )],
        ),
    };

    assert_typecheck_err_contains(
        ash_typeck::type_check_program(&program),
        &[
            "invoke",
            "direct invoke",
            "capability implementation bodies",
        ],
    );
}

#[test]
fn imported_interface_and_implementation_metadata_can_seed_program_typechecking() {
    let env = prepopulated_import_env();
    let program = Program {
        definitions: vec![],
        helper_workflows: vec![],
        workflow: workflow(
            binding_call("store", "read", vec![string_lit("alpha")]),
            vec![storage_binding_with(vec![var("file")])],
        ),
    };

    let result = ash_typeck::type_check_program_in_env(&env, &program)
        .expect("pre-populated imported metadata should admit resource/capability bindings");

    let file = result
        .authority_provenance
        .resource_bindings
        .iter()
        .find(|binding| binding.name == "file")
        .expect("imported resource metadata should validate owned resource provenance");
    assert_eq!(file.resource_type, "FileResource");
    assert_eq!(file.authority, AuthorityProvenanceKind::Internal);

    let store = result
        .authority_provenance
        .capability_bindings
        .iter()
        .find(|binding| binding.name == "store")
        .expect("imported capability implementation metadata should admit store binding");
    assert_eq!(store.interface, "Storage");
    assert_eq!(store.implementation, "FileStorage");
    assert_eq!(store.authority, AuthorityProvenanceKind::Internal);
    assert_eq!(store.sources.len(), 1);
    assert_eq!(store.sources[0].kind, ProvenanceSourceKind::Resource);
    assert_eq!(store.sources[0].dependency_name, "file");
    assert_eq!(store.sources[0].binding_name, "file");
    assert_eq!(store.sources[0].target_name, "FileResource");
}
