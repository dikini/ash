use ash_parser::surface::{
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Param, Program,
    Type as SurfaceType, Visibility, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::Type;
use ash_typeck::error::TypeEnvError;
use ash_typeck::type_env::TypeEnv;

fn span() -> Span {
    Span::default()
}

fn param(name: &str, ty: SurfaceType) -> Param {
    Param {
        name: name.into(),
        ty,
    }
}

fn operation(
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

fn key_value_store_interface() -> CapabilityInterfaceDef {
    interface(
        "KeyValueStore",
        vec![
            operation(
                CapabilityOperationMode::Observe,
                "get",
                vec![param("key", SurfaceType::Name("String".into()))],
                SurfaceType::Name("String".into()),
            ),
            operation(
                CapabilityOperationMode::Execute,
                "put",
                vec![
                    param("key", SurfaceType::Name("String".into())),
                    param("value", SurfaceType::Name("String".into())),
                ],
                SurfaceType::Name("Unit".into()),
            ),
        ],
    )
}

#[test]
fn registers_visible_capability_interface_and_exposes_operation_signatures() {
    let mut env = TypeEnv::with_builtin_types();

    env.register_capability_interface(&key_value_store_interface())
        .expect("public capability interface should register");

    let get = env
        .lookup_capability_operation("KeyValueStore", "get")
        .expect("registered operation should be available for call/type checking");
    assert_eq!(get.mode, CapabilityOperationMode::Observe);
    assert_eq!(get.param_names, vec!["key"]);
    assert_eq!(get.params, vec![Type::String]);
    assert_eq!(get.return_type, Type::String);

    let put = env
        .lookup_capability_operation("KeyValueStore", "put")
        .expect("registered execute operation should be available for conformance checking");
    assert_eq!(put.mode, CapabilityOperationMode::Execute);
    assert_eq!(put.param_names, vec!["key", "value"]);
    assert_eq!(put.params, vec![Type::String, Type::String]);
    assert_eq!(put.return_type, Type::Null);
}

#[test]
fn rejects_duplicate_capability_operation_names_within_interface() {
    let mut env = TypeEnv::with_builtin_types();
    let duplicated = interface(
        "Clock",
        vec![
            operation(
                CapabilityOperationMode::Observe,
                "now",
                vec![],
                SurfaceType::Name("Time".into()),
            ),
            operation(
                CapabilityOperationMode::Execute,
                "now",
                vec![],
                SurfaceType::Name("Time".into()),
            ),
        ],
    );

    let err = env
        .register_capability_interface(&duplicated)
        .expect_err("duplicate operation names must be rejected before lookup/conformance use");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(
        format!("{err}").contains("duplicate") && format!("{err}").contains("now"),
        "diagnostic should identify the duplicate operation, got: {err}"
    );
}

#[test]
fn rejects_capability_operation_signatures_with_unknown_types() {
    let mut env = TypeEnv::with_builtin_types();
    let unknown_type = interface(
        "BlobStore",
        vec![operation(
            CapabilityOperationMode::Execute,
            "write",
            vec![param("blob", SurfaceType::Name("MissingBlob".into()))],
            SurfaceType::Name("Unit".into()),
        )],
    );

    let err = env
        .register_capability_interface(&unknown_type)
        .expect_err("operation parameter and return types must be validated at registration");

    assert!(matches!(err, TypeEnvError::InvalidDefinition(_, _)));
    assert!(
        format!("{err}").contains("MissingBlob"),
        "diagnostic should identify the unknown operation type, got: {err}"
    );
}

#[test]
fn lookup_distinguishes_missing_capability_interfaces_from_missing_operations() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_capability_interface(&key_value_store_interface())
        .expect("fixture should register");

    assert!(
        env.lookup_capability_operation("MissingStore", "get")
            .is_none(),
        "missing interface should not resolve an operation"
    );
    assert!(
        env.lookup_capability_operation("KeyValueStore", "delete")
            .is_none(),
        "missing operation should not resolve from an existing interface"
    );
}

#[test]
fn child_type_environments_inherit_capability_interface_signatures_without_leaking_new_bindings() {
    let mut parent = TypeEnv::with_builtin_types();
    parent
        .register_capability_interface(&key_value_store_interface())
        .expect("fixture should register in parent");

    let mut child = parent.extend();
    assert!(
        child
            .lookup_capability_operation("KeyValueStore", "get")
            .is_some(),
        "child environments should inherit registered capability interface signatures"
    );

    child
        .register_capability_interface(&interface(
            "Clock",
            vec![operation(
                CapabilityOperationMode::Observe,
                "now",
                vec![],
                SurfaceType::Name("Time".into()),
            )],
        ))
        .expect("child-local capability interface should register");

    assert!(child.lookup_capability_interface("Clock").is_some());
    assert!(
        parent.lookup_capability_interface("Clock").is_none(),
        "child-local capability interface registration must not mutate the parent environment"
    );
}

#[test]
fn type_check_program_registers_capability_interface_definitions_before_workflow_checking() {
    let program = Program {
        definitions: vec![ash_parser::surface::Definition::CapabilityInterface(
            key_value_store_interface(),
        )],
        helper_workflows: vec![],
        workflow: WorkflowDef {
            name: "main".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: None,
            plays_roles: vec![],
            capabilities: vec![],
            owned_resources: vec![],
            used_bindings: vec![],
            body: Workflow::Done { span: span() },
            contract: None,
            span: span(),
        },
    };

    ash_typeck::type_check_program(&program)
        .expect("program-level type checking should accept capability interface definitions");
}
