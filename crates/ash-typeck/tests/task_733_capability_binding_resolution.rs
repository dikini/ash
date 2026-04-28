use ash_parser::surface::{
    CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Definition, Expr,
    Literal, Param, Program, ResourceField, ResourceTypeDef, Type as SurfaceType, Visibility,
    Workflow, WorkflowDef, WorkflowOwnedResource, WorkflowUsedBinding,
};
use ash_parser::token::Span;
use ash_typeck::type_env::TypeEnv;
use proptest::prelude::*;
use proptest::test_runner::Config;

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

fn string_lit(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        module: None,
        func: name.into(),
        args,
        span: span(),
    }
}

fn param(name: &str, ty: SurfaceType) -> Param {
    Param {
        name: name.into(),
        ty,
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

fn cap_iface(name: &str) -> CapabilityInterfaceDef {
    CapabilityInterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        operations: vec![
            CapabilityOperationSig {
                mode: CapabilityOperationMode::Observe,
                name: "read".into(),
                params: vec![param("key", ty("String"))],
                return_type: ty("String"),
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
            .map(|(name, ty)| ash_parser::surface::Param {
                name: name.into(),
                ty,
            })
            .collect(),
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

fn cap_impl(name: &str, interface: &str) -> CapabilityImplementationDef {
    CapabilityImplementationDef {
        visibility: Visibility::Public,
        name: name.into(),
        interface: interface.into(),
        dependencies: vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "kv",
            ty("KvResource"),
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
                Expr::Literal(Literal::Null),
            ),
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

fn workflow_with_body(body: Workflow, used_bindings: Vec<WorkflowUsedBinding>) -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(ty("String")),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![own("kv", "KvResource")],
        used_bindings,
        body,
        contract: None,
        span: span(),
    }
}

fn env_with_kv() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("KvResource"))
        .unwrap();
    env.register_capability_interface(&cap_iface("Storage"))
        .unwrap();
    env.register_capability_implementation(&cap_impl("MemoryStorage", "Storage"))
        .unwrap();
    env
}

fn program_with_workflow(workflow: WorkflowDef) -> Program {
    Program {
        definitions: vec![
            Definition::ResourceType(resource_type("KvResource")),
            Definition::CapabilityInterface(cap_iface("Storage")),
            Definition::CapabilityImplementation(cap_impl("MemoryStorage", "Storage")),
        ],
        helper_workflows: vec![],
        workflow,
    }
}

fn admitted_store_binding() -> WorkflowUsedBinding {
    use_binding("store", "Storage", call("MemoryStorage", vec![var("kv")]))
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
fn admitted_binding_operation_call_resolves_to_interface_return_type() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "read", vec![string_lit("alpha")]),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    ash_typeck::type_check_workflow_def_in_env(&env, &workflow)
        .expect("admitted capability binding operation should typecheck");
}

#[test]
fn program_typecheck_resolves_binding_operation_calls_through_workflow_header() {
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "read", vec![string_lit("alpha")]),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    ash_typeck::type_check_program(&program_with_workflow(workflow))
        .expect("program-level interface/impl/resource metadata should admit binding calls");
}

#[test]
fn admitted_binding_operation_call_enforces_argument_types() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "read", vec![int_lit(42)]),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &workflow),
        &["store", "read", "String", "Int"],
    );
}

#[test]
fn admitted_binding_operation_call_enforces_arity() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "read", vec![]),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &workflow),
        &["store", "read", "expected 1", "found 0"],
    );
}

#[test]
fn admitted_binding_unknown_operation_is_rejected_against_interface() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "delete", vec![string_lit("alpha")]),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &workflow),
        &["store", "delete", "Storage"],
    );
}

#[test]
fn unadmitted_capability_binding_operation_is_rejected() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: binding_call("store", "read", vec![string_lit("alpha")]),
            span: span(),
        },
        vec![],
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &workflow),
        &["unadmitted capability binding", "store", "uses"],
    );
}

#[test]
fn admitted_capability_binding_is_not_available_as_first_class_value() {
    let env = env_with_kv();
    let workflow = workflow_with_body(
        Workflow::Ret {
            expr: var("store"),
            span: span(),
        },
        vec![admitted_store_binding()],
    );

    assert_typecheck_err_contains(
        ash_typeck::type_check_workflow_def_in_env(&env, &workflow),
        &["store"],
    );
}

proptest! {
    #![proptest_config(Config::with_cases(32))]

    #[test]
    fn only_names_declared_in_workflow_uses_are_capability_bindings(
        rejected in "[a-z][a-z0-9_]{0,8}"
            .prop_filter("not the admitted binding", |name| name != "store" && name != "kv")
    ) {
        let env = env_with_kv();
        let workflow = workflow_with_body(
            Workflow::Ret {
                expr: binding_call(&rejected, "read", vec![string_lit("alpha")]),
                span: span(),
            },
            vec![admitted_store_binding()],
        );

        prop_assert!(
            ash_typeck::type_check_workflow_def_in_env(&env, &workflow)
                .expect_err("undeclared binding-like field call should be rejected")
                .to_string()
                .contains("unadmitted capability binding")
        );
    }
}
