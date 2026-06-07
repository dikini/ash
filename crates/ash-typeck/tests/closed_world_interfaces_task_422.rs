use ash_core::ast::{
    TypeBody, TypeDef as CoreTypeDef, VariantDef, VariantPayload, Visibility as CoreVisibility,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, Definition, Expr, ImplDef, ImplMethodDef,
    InterfaceBound, InterfaceDef, InterfaceMethodSig, Literal, MatchArm, Parameter, Pattern,
    Program, Type as SurfaceType, TypeParam, VariantPatternPayload,
    Visibility as SurfaceVisibility, WhereBound, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::TypeEnvError;
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
        builtin: false,
    }
}

fn explain_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Explain".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
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
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
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
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
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
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
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
            kind: None,
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
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Variable {
                name: "value".into(),
                span: ash_parser::token::Span::default(),
            },
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
            kind: None,
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
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Call {
                func: "explain".into(),
                module: Some("Explain".into()),
                args: vec![Expr::Variable {
                    name: "value".into(),
                    span: ash_parser::token::Span::default(),
                }],
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
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Match {
                scrutinee: Box::new(Expr::Variable {
                    name: "value".into(),
                    span: ash_parser::token::Span::default(),
                }),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Variant {
                            name: "Some".into(),
                            fields: Some(vec![(
                                "value".into(),
                                Pattern::Variable {
                                    name: "x".into(),
                                    span: ash_parser::token::Span::default(),
                                },
                            )]),
                            payload: VariantPatternPayload::Record(vec![(
                                "value".into(),
                                Pattern::Variable {
                                    name: "x".into(),
                                    span: ash_parser::token::Span::default(),
                                },
                            )]),
                        },
                        body: Box::new(Expr::Call {
                            func: "explain".into(),
                            module: Some("Explain".into()),
                            args: vec![Expr::Variable {
                                name: "x".into(),
                                span: ash_parser::token::Span::default(),
                            }],
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
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
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
        helper_workflows: vec![],
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
        args: vec![Expr::Variable {
            name: "decision".into(),
            span: ash_parser::token::Span::default(),
        }],
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
        helper_workflows: vec![],
        workflow: workflow_with_declared_return_type_mismatch(),
    };

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_err(),
        "program typechecking should reject a workflow whose body type does not match the declared return type"
    );
}

// ---------------------------------------------------------------------------
// TASK-563: Multi-parameter interfaces and impl registry redesign
// ---------------------------------------------------------------------------

fn pair_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "first".into(),
            params: vec![SurfaceType::Constructor {
                name: "Pair".into(),
                args: vec![SurfaceType::Name("A".into()), SurfaceType::Name("B".into())],
            }],
            return_type: SurfaceType::Name("A".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

#[test]
fn task563_pair_two_param_interface_registers() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&CoreTypeDef {
        name: "Pair".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .unwrap();
    env.register_interface(&pair_interface_def())
        .expect("register Pair");
    assert!(env.lookup_interface("Pair").is_some());
}

#[test]
fn task563_concrete_multi_param_impl_resolves() {
    let mut env = TypeEnv::with_builtin_types();
    // Register the type Pair
    env.register_type(&CoreTypeDef {
        name: "Pair".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .unwrap();
    // Register Pair<A,B> interface
    env.register_interface(&pair_interface_def()).unwrap();
    // Register impl Pair<Int, String>
    let impl_def = ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Pair".into(),
        type_args: vec![
            SurfaceType::Name("Int".into()),
            SurfaceType::Name("String".into()),
        ],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "first".into(),
            params: vec!["p".into()],
            body: Expr::Literal(Literal::Int(42)),
            span: test_span(),
        }],
        span: test_span(),
    };
    env.register_impl(&impl_def)
        .expect("register impl Pair<Int,String>");

    // Call Pair::first(my_pair) where my_pair: Pair<Int, String>
    let return_ty = env
        .resolve_interface_method_call(
            "Pair",
            "first",
            &[Type::Constructor {
                name: QualifiedName::root("Pair"),
                args: vec![Type::Int, Type::String],
                kind: Kind::Type,
            }],
        )
        .expect("resolve Pair::first");
    assert_eq!(return_ty, Type::Int);
}

#[test]
fn task563_wrong_arity_impl_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&CoreTypeDef {
        name: "Pair".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .unwrap();
    env.register_interface(&pair_interface_def()).unwrap();
    // Wrong: 1 type arg for 2-param interface
    let bad_impl = ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Pair".into(),
        type_args: vec![SurfaceType::Name("Int".into())],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "first".into(),
            params: vec!["p".into()],
            body: Expr::Literal(Literal::Int(42)),
            span: test_span(),
        }],
        span: test_span(),
    };
    let err = env.register_impl(&bad_impl).unwrap_err();
    assert!(err.to_string().contains("type parameters"));
}

#[test]
fn task563_from_underdetermined_param_errors() {
    let mut env = TypeEnv::with_builtin_types();
    // From<A,B> { from(A) -> B }
    let from_iface = InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "From".into(),
        type_params: vec!["A".into(), "B".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "from".into(),
            params: vec![SurfaceType::Name("A".into())],
            return_type: SurfaceType::Name("B".into()),
            span: test_span(),
        }],
        span: test_span(),
    };
    env.register_interface(&from_iface).unwrap();
    // From::from("hello") only determines A=String, B is underdetermined
    let result = env.resolve_interface_method_call("From", "from", &[Type::String]);
    assert!(result.is_err(), "underdetermined type params should error");
}

#[test]
fn task563_duplicate_multi_param_impl_rejected_with_full_application() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&CoreTypeDef {
        name: "Pair".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .unwrap();
    env.register_interface(&pair_interface_def()).unwrap();

    let impl_def = ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Pair".into(),
        type_args: vec![
            SurfaceType::Name("Int".into()),
            SurfaceType::Name("String".into()),
        ],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "first".into(),
            params: vec!["p".into()],
            body: Expr::Literal(Literal::Int(42)),
            span: test_span(),
        }],
        span: test_span(),
    };
    env.register_impl(&impl_def).expect("first impl");

    let duplicate = env.register_impl(&impl_def);
    assert!(
        duplicate.is_err(),
        "duplicate multi-param impl must be rejected"
    );
    let msg = duplicate.unwrap_err().to_string();
    assert!(
        msg.contains("Pair"),
        "error must mention interface name: got '{msg}'"
    );
    assert!(
        msg.contains("Int") && msg.contains("String"),
        "error must contain full application types: got '{msg}'"
    );
}

// ---------------------------------------------------------------------------
// TASK-565: Generic impl schemes, overlap checking, recursive resolution
// ---------------------------------------------------------------------------

fn serialize_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Serialize".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "serialize".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serialize_int_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serialize".into(),
        type_args: vec![SurfaceType::Name("Int".into())],
        type_params: vec![],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "serialize".into(),
            params: vec!["x".into()],
            body: Expr::Literal(Literal::String("int".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serialize_list_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serialize".into(),
        type_args: vec![SurfaceType::List(Box::new(SurfaceType::Name("T".into())))],
        type_params: vec!["T".into()],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: "Serialize".into(),
            span: test_span(),
        }],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "serialize".into(),
            params: vec!["items".into()],
            body: Expr::Literal(Literal::String("list".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

#[test]
fn task565_generic_impl_scheme_registers() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serialize_interface_def())
        .expect("register Serialize");
    env.register_impl(&serialize_list_impl())
        .expect("register generic impl");

    let schemes = env.impl_schemes();
    assert_eq!(schemes.len(), 1, "expected exactly one impl scheme");
    assert_eq!(
        schemes[0].type_params.len(),
        1,
        "expected scheme to have one type parameter"
    );
    assert_eq!(schemes[0].interface, "Serialize");
}

#[test]
fn task565_overlapping_impls_rejected() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serialize_interface_def())
        .expect("register Serialize");
    env.register_impl(&serialize_list_impl())
        .expect("register first generic impl");

    let result = env.register_impl(&serialize_list_impl());
    assert!(
        matches!(
            result,
            Err(TypeEnvError::OverlappingImpls { ref interface, .. }) if interface == "Serialize"
        ),
        "expected OverlappingImpls error, got {:?}",
        result
    );
}

#[test]
fn task565_recursive_where_bound_resolution() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serialize_interface_def())
        .expect("register Serialize");
    env.register_impl(&serialize_int_impl())
        .expect("register concrete Int impl");
    env.register_impl(&serialize_list_impl())
        .expect("register generic List impl");

    let nested_list_type = Type::List(Box::new(Type::List(Box::new(Type::Int))));

    let return_ty = env
        .resolve_interface_method_call("Serialize", "serialize", &[nested_list_type])
        .expect("recursive bound resolution should succeed for List<List<Int>>");
    assert_eq!(return_ty, Type::String);
}

fn cyclic_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Cyclic".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![InterfaceMethodSig {
            name: "m".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("T".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn cyclic_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Cyclic".into(),
        type_args: vec![SurfaceType::Name("T".into())],
        type_params: vec!["T".into()],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: "Cyclic".into(),
            span: test_span(),
        }],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "m".into(),
            params: vec!["x".into()],
            body: Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            span: test_span(),
        }],
        span: test_span(),
    }
}

#[test]
fn task565_recursive_bound_depth_limit_errors() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&cyclic_interface_def())
        .expect("register Cyclic");
    env.register_impl(&cyclic_impl())
        .expect("register cyclic generic impl");

    let result = env.resolve_interface_method_call("Cyclic", "m", &[Type::Int]);
    assert!(
        matches!(result, Err(TypeEnvError::RecursiveBound { .. })),
        "expected RecursiveBound error after depth 32, got {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// TASK-567: Associated types, normalization, rigid projections
// ---------------------------------------------------------------------------

fn serializer_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Serializer".into(),
        type_params: vec!["S".into()],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: ash_parser::surface::AssociatedTypeKind::Ordinary,
            span: test_span(),
        }],
        methods: vec![InterfaceMethodSig {
            name: "serialize_bool".into(),
            params: vec![
                SurfaceType::Name("S".into()),
                SurfaceType::Name("Bool".into()),
            ],
            return_type: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("S".into())),
                name: "Ok".into(),
            },
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serializer_json_writer_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: Expr::Literal(Literal::String("serialized".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn serializer_impl_missing_associated_type() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: Expr::Literal(Literal::String("serialized".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn rigid_projection_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "test_rigid".into(),
        type_params: vec![TypeParam {
            name: "T".into(),
            kind: None,
            bounds: vec![InterfaceBound {
                interface: "Serializer".into(),
                span: test_span(),
            }],
            span: test_span(),
        }],
        params: vec![
            Parameter {
                name: "a".into(),
                ty: SurfaceType::Associated {
                    base: Box::new(SurfaceType::Name("T".into())),
                    name: "Ok".into(),
                },
                span: test_span(),
            },
            Parameter {
                name: "b".into(),
                ty: SurfaceType::Associated {
                    base: Box::new(SurfaceType::Name("T".into())),
                    name: "Ok".into(),
                },
                span: test_span(),
            },
        ],
        declared_return_type: Some(SurfaceType::Associated {
            base: Box::new(SurfaceType::Name("T".into())),
            name: "Ok".into(),
        }),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            },
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn rigid_projection_concrete_mismatch_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "test_rigid_mismatch".into(),
        type_params: vec![TypeParam {
            name: "T".into(),
            kind: None,
            bounds: vec![InterfaceBound {
                interface: "Serializer".into(),
                span: test_span(),
            }],
            span: test_span(),
        }],
        params: vec![Parameter {
            name: "a".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: "Ok".into(),
            },
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        header_events: vec![],
        body: Workflow::Ret {
            expr: Expr::Variable {
                name: "a".into(),
                span: ash_parser::token::Span::default(),
            },
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

#[test]
fn task567_associated_type_normalizes_in_return_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("register Serializer");
    env.register_impl(&serializer_json_writer_impl())
        .expect("register Serializer<String>");
    env.bind_variable("writer", Type::String);

    let expr = Expr::Call {
        func: "serialize_bool".into(),
        module: Some("Serializer".into()),
        args: vec![
            Expr::Variable {
                name: "writer".into(),
                span: ash_parser::token::Span::default(),
            },
            Expr::Literal(Literal::Bool(true)),
        ],
        span: test_span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "expected Serializer::serialize_bool(writer, true) to typecheck, got errors: {:?}",
        result.errors
    );
    assert_eq!(
        result.ty,
        Type::String,
        "expected return type to normalize to String, got {:?}",
        result.ty
    );
}

#[test]
fn task567_rigid_projection_unifies_with_itself() {
    let workflow = rigid_projection_workflow();

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("register Serializer");

    let result = ash_typeck::type_check_workflow_def_in_env(&env, &workflow);

    assert!(
        result.is_ok(),
        "rigid projection T::Ok should unify with itself, got: {:?}",
        result
    );
}

#[test]
fn task567_rigid_projection_rejects_concrete_match() {
    let workflow = rigid_projection_concrete_mismatch_workflow();

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("register Serializer");

    let result = ash_typeck::type_check_workflow_def_in_env(&env, &workflow);

    assert!(
        result.is_err(),
        "rigid projection T::Ok should not unify with concrete String"
    );
}

#[test]
fn task567_missing_associated_type_in_impl_errors() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("register Serializer");

    let result = env.register_impl(&serializer_impl_missing_associated_type());

    assert!(
        matches!(
            result,
            Err(TypeEnvError::MissingAssociatedType { ref interface, ref name, .. })
            if interface == "Serializer" && name == "Ok"
        ),
        "expected MissingAssociatedType error for Ok in Serializer, got {:?}",
        result
    );
}
