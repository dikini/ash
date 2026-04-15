use ash_core::ast::{
    TypeBody, TypeDef as CoreTypeDef, VariantDef, VariantPayload, Visibility as CoreVisibility,
};
use ash_parser::surface::{
    Definition, Expr, ImplDef, ImplMethodDef, InterfaceBound, InterfaceDef, InterfaceMethodSig,
    Literal, MatchArm, Parameter, Pattern, Program, Type as SurfaceType, TypeParam,
    VariantPatternPayload, Visibility as SurfaceVisibility, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Kind, QualifiedName, Type};

fn test_span() -> Span {
    Span::default()
}

fn policy_decision_type_def() -> CoreTypeDef {
    CoreTypeDef {
        name: "PolicyDecision".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Allow".to_string(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Deny".to_string(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: CoreVisibility::Public,
    }
}

fn explain_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Explain".into(),
        type_params: vec!["T".into()],
        methods: vec![InterfaceMethodSig {
            name: "explain".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn explain_impl_for(type_name: &str) -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Explain".into(),
        type_args: vec![SurfaceType::Name(type_name.into())],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            params: vec!["decision".into()],
            body: Expr::Literal(Literal::String("policy".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn explain_policy_decision_impl() -> ImplDef {
    explain_impl_for("PolicyDecision")
}

fn explain_string_impl() -> ImplDef {
    explain_impl_for("String")
}

fn explain_string_impl_with_int_body() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Explain".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            params: vec!["value".into()],
            body: Expr::Literal(Literal::Int(42)),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn explain_list_string_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Explain".into(),
        type_args: vec![SurfaceType::List(Box::new(SurfaceType::Name(
            "String".into(),
        )))],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            params: vec!["items".into()],
            body: Expr::Literal(Literal::String("list".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn nominal_type(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
        kind: Kind::Type,
    }
}

fn workflow_with_bound(bound_interface: &str) -> WorkflowDef {
    WorkflowDef {
        name: "record_event".into(),
        type_params: vec![TypeParam {
            name: "T".into(),
            bounds: vec![InterfaceBound {
                interface: bound_interface.into(),
                span: test_span(),
            }],
            span: test_span(),
        }],
        params: vec![Parameter {
            name: "value".into(),
            ty: SurfaceType::Name("T".into()),
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("T".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Ret {
            expr: Expr::Variable("value".into()),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn interface_method_call_workflow(type_name: &str) -> WorkflowDef {
    WorkflowDef {
        name: "record_event".into(),
        type_params: vec![TypeParam {
            name: "T".into(),
            bounds: vec![InterfaceBound {
                interface: "Explain".into(),
                span: test_span(),
            }],
            span: test_span(),
        }],
        params: vec![Parameter {
            name: "value".into(),
            ty: SurfaceType::Name(type_name.into()),
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Ret {
            expr: Expr::Call {
                func: "explain".into(),
                module: Some("Explain".into()),
                args: vec![Expr::Variable("value".into())],
                span: test_span(),
            },
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn generic_bound_interface_method_call_workflow() -> WorkflowDef {
    interface_method_call_workflow("T")
}

fn match_bound_interface_method_call_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "record_event".into(),
        type_params: vec![],
        params: vec![Parameter {
            name: "value".into(),
            ty: SurfaceType::Constructor {
                name: "Option".into(),
                args: vec![SurfaceType::Name("String".into())],
            },
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Ret {
            expr: Expr::Match {
                scrutinee: Box::new(Expr::Variable("value".into())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Variant {
                            name: "Some".into(),
                            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
                            payload: VariantPatternPayload::Record(vec![(
                                "value".into(),
                                Pattern::Variable("x".into()),
                            )]),
                        },
                        body: Box::new(Expr::Call {
                            func: "explain".into(),
                            module: Some("Explain".into()),
                            args: vec![Expr::Variable("x".into())],
                            span: test_span(),
                        }),
                        span: test_span(),
                    },
                    MatchArm {
                        pattern: Pattern::Variant {
                            name: "None".into(),
                            fields: None,
                            payload: VariantPatternPayload::Unit,
                        },
                        body: Box::new(Expr::Literal(Literal::String("missing".into()))),
                        span: test_span(),
                    },
                ],
                span: test_span(),
            },
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn workflow_with_declared_return_type_mismatch() -> WorkflowDef {
    WorkflowDef {
        name: "record_event".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Ret {
            expr: Expr::Literal(Literal::Int(42)),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn program_with_interface_impl_and_workflow(workflow: WorkflowDef, impls: Vec<ImplDef>) -> Program {
    let mut definitions = vec![Definition::Interface(explain_interface_def())];
    definitions.extend(impls.into_iter().map(Definition::Impl));

    Program {
        definitions,
        workflow,
    }
}

#[test]
fn interface_environment_requires_interface_registration_before_impl_registration() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&policy_decision_type_def())
        .expect("nominal type should register");

    let result = env.register_impl(&explain_policy_decision_impl());

    assert!(
        result.is_err(),
        "impl registration should fail when the interface environment has no Explain entry"
    );
}

#[test]
fn interface_environment_enforces_single_impl_per_interface_and_nominal_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&policy_decision_type_def())
        .expect("nominal type should register");
    env.register_interface(&explain_interface_def())
        .expect("interface should register");
    env.register_impl(&explain_policy_decision_impl())
        .expect("first impl should register");

    let duplicate = env.register_impl(&explain_policy_decision_impl());

    assert!(
        duplicate.is_err(),
        "duplicate impls for the same (Interface, ConcreteNominalType) pair must be rejected"
    );
}

#[test]
fn interface_environment_rejects_impl_method_body_type_that_mismatches_interface_return_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&explain_interface_def())
        .expect("interface should register");

    let result = env.register_impl(&explain_string_impl_with_int_body());

    assert!(
        result.is_err(),
        "impl registration should reject method bodies whose inferred type disagrees with the interface method return type"
    );
}

#[test]
fn invalid_interface_bounds_are_rejected_during_workflow_typechecking() {
    let workflow = workflow_with_bound("MissingExplain");

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(result.is_err(), "unknown interface bounds must be rejected");
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("MissingExplain"),
        "expected missing bound name in error, got: {error}"
    );
}

#[test]
fn interface_method_call_typechecks_via_registered_impl_and_returns_method_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&policy_decision_type_def())
        .expect("nominal type should register");
    env.register_interface(&explain_interface_def())
        .expect("interface should register");
    env.register_impl(&explain_policy_decision_impl())
        .expect("impl should register");
    env.bind_variable("decision", nominal_type("PolicyDecision"));

    let expr = Expr::Call {
        func: "explain".into(),
        module: Some("Explain".into()),
        args: vec![Expr::Variable("decision".into())],
        span: test_span(),
    };

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "expected Explain::explain(decision) to typecheck, got errors: {:?}",
        result.errors
    );
    assert_eq!(result.ty, Type::String);
}

#[test]
fn program_typecheck_registers_top_level_interface_and_impl_for_workflow_bounds() {
    let program = program_with_interface_impl_and_workflow(
        generic_bound_interface_method_call_workflow(),
        vec![explain_string_impl()],
    );

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_ok(),
        "program typechecking should ingest Program.definitions and WorkflowDef.type_params, got: {:?}",
        result
    );
}

#[test]
fn program_typecheck_accepts_interface_method_call_on_match_bound_name() {
    let program = program_with_interface_impl_and_workflow(
        match_bound_interface_method_call_workflow(),
        vec![explain_string_impl()],
    );

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_ok(),
        "program typechecking should let match-bound names drive Explain::explain(x) typing, got: {:?}",
        result
    );
}

#[test]
fn program_typecheck_rejects_interface_method_call_when_program_impl_is_missing() {
    let program =
        program_with_interface_impl_and_workflow(interface_method_call_workflow("String"), vec![]);

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_err(),
        "program typechecking should fail when Program.definitions omit the required impl"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("Explain") && error.contains("String"),
        "expected missing impl details in error, got: {error}"
    );
}

#[test]
fn interface_environment_rejects_non_nominal_impl_targets() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&explain_interface_def())
        .expect("interface should register");

    let result = env.register_impl(&explain_list_string_impl());

    assert!(
        result.is_err(),
        "impl targets should be restricted to concrete nominal types in the MVP"
    );
}

#[test]
fn workflow_typecheck_rejects_declared_return_type_mismatch() {
    let workflow = workflow_with_declared_return_type_mismatch();

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(
        result.is_err(),
        "workflow typechecking should reject a body whose type does not match the declared return type"
    );
}

#[test]
fn program_typecheck_rejects_declared_workflow_return_type_mismatch() {
    let program = Program {
        definitions: vec![],
        workflow: workflow_with_declared_return_type_mismatch(),
    };

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_err(),
        "program typechecking should reject a workflow whose body type does not match the declared return type"
    );
}
