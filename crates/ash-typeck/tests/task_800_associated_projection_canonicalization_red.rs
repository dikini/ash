use ash_core::ast::{TypeExpr, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    RepresentationExposure, SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{CanonicalTypeExpr, ProjectionRigidity};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, BuiltinFnDef, Expr, FnDef, ImplDef, InterfaceBound,
    InterfaceDef as SurfaceInterfaceDef, InterfaceMethodSig, Param, Parameter, Type as SurfaceType,
    TypeParam, Visibility as SurfaceVisibility, WhereBound, Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::type_env::type_expr_to_type;
use ash_typeck::{Kind, Type, TypeEnv, builtin_fn_signature_type, fn_signature_type};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-800-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-800-test".into(),
        },
        None,
        label,
    )
}

fn exported_type_summary(
    module: ModuleIdentity,
    origin_name: &str,
    exported_name: &str,
    params: &[&str],
) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module, origin_name),
        exported_name,
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor(exported_name),
    )
    .with_params(params.iter().map(|param| (*param).to_string()).collect())
}

fn pair_interface_identity(module: &ModuleIdentity) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), "Pair")
}

fn item_member_identity(interface: &InterfaceIdentityId) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Pair".into(), "Item".into()],
    )
}

fn ok_member_identity(interface: &InterfaceIdentityId) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Ok",
        vec![interface.name.clone(), "Ok".into()],
    )
}

fn pair_interface_def() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            span: ash_parser::token::Span::default(),
        }],
        methods: vec![],
        span: ash_parser::token::Span::default(),
    }
}

fn register_pair_projection_metadata(env: &mut TypeEnv, module: &ModuleIdentity) {
    let interface = pair_interface_identity(module);
    let member = item_member_identity(&interface);
    let summary = ModuleSemanticSummary::new(module.clone()).with_exported_type(
        exported_type_summary(module.clone(), "Pair", "Pair", &["A", "B"]),
    );

    env.register_module_semantic_summary(&summary)
        .expect("test precondition: Pair carrier type summary should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "Pair",
        vec!["Pair".into()],
        anchor("interface Pair"),
    ))
    .expect("test precondition: Pair interface identity should register");
    env.register_interface(&pair_interface_def())
        .expect("test precondition: Pair interface definition should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member,
        "Item",
        anchor("associated type Item"),
    ))
    .expect("test precondition: Pair::Item identity should register");
}

fn span() -> Span {
    Span::default()
}

fn serializer_interface_def() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Serializer".into(),
        type_params: vec!["S".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn formatter_interface_def() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Formatter".into(),
        type_params: vec!["S".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn projection_fn(return_type: SurfaceType) -> FnDef {
    FnDef {
        visibility: SurfaceVisibility::Inherited,
        name: "project".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type: Some(return_type),
        contract: None,
        body: Expr::Variable {
            name: "value".into(),
            span: span(),
        },
        span: span(),
    }
}

fn projection_builtin(return_type: SurfaceType) -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: SurfaceVisibility::Public,
        name: "project".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type,
        span: span(),
    }
}

fn projection_fn_without_bounds(return_type: SurfaceType) -> FnDef {
    FnDef {
        visibility: SurfaceVisibility::Inherited,
        name: "project_unbounded".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "value".into(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type: Some(return_type),
        contract: None,
        body: Expr::Variable {
            name: "value".into(),
            span: span(),
        },
        span: span(),
    }
}

fn projection_workflow_with_declared_return() -> WorkflowDef {
    WorkflowDef {
        name: "project".into(),
        type_params: vec![TypeParam {
            name: "T".into(),
            bounds: vec![InterfaceBound {
                interface: "Serializer".into(),
                span: span(),
            }],
            span: span(),
        }],
        params: vec![Parameter {
            name: "value".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: "Ok".into(),
            },
            span: span(),
        }],
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
                name: "value".into(),
                span: span(),
            },
            span: span(),
        },
        contract: None,
        span: span(),
    }
}

fn serializer_impl_with_associated_projection_binding(member_name: &str) -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec!["T".into()],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: member_name.into(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn serializer_impl_with_in_bounds_projection_binding(member_name: &str) -> ImplDef {
    serializer_impl_with_where_bound_projection_binding("Serializer", member_name)
}

fn serializer_impl_with_where_bound_projection_binding(bound: &str, member_name: &str) -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec!["T".into()],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: bound.into(),
            span: span(),
        }],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: member_name.into(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn serializer_impl_with_ambiguous_in_bounds_projection_binding() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Serializer".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        type_params: vec!["T".into()],
        where_bounds: vec![
            WhereBound {
                param: "T".into(),
                bound: "Serializer".into(),
                span: span(),
            },
            WhereBound {
                param: "T".into(),
                bound: "Formatter".into(),
                span: span(),
            },
        ],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("T".into())),
                name: "Ok".into(),
            },
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn serializer_interface_with_projection_method() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Serializer".into(),
        type_params: vec!["S".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            span: span(),
        }],
        methods: vec![InterfaceMethodSig {
            name: "finish".into(),
            params: vec![SurfaceType::Name("S".into())],
            return_type: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("S".into())),
                name: "Ok".into(),
            },
            span: span(),
        }],
        span: span(),
    }
}

fn serializer_interface_with_unknown_projection_method() -> SurfaceInterfaceDef {
    SurfaceInterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Serializer".into(),
        type_params: vec!["S".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            span: span(),
        }],
        methods: vec![InterfaceMethodSig {
            name: "finish".into(),
            params: vec![SurfaceType::Name("S".into())],
            return_type: SurfaceType::Associated {
                base: Box::new(SurfaceType::Name("S".into())),
                name: "Missing".into(),
            },
            span: span(),
        }],
        span: span(),
    }
}

#[test]
fn task800_projection_lowering_preserves_nominal_base_argument_spine_order() {
    let module = module_identity(8001, &["pkg", "pair"]);

    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let interface = pair_interface_identity(&module);
    let member = item_member_identity(&interface);
    let lowered = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Constructor {
                name: "Pair".into(),
                args: vec![
                    TypeExpr::Named("Left".into()),
                    TypeExpr::Named("Right".into()),
                ],
            }),
            name: "Item".into(),
        })
        .expect(
            "TASK-800 should lower Pair<Left, Right>::Item through canonical identity plumbing",
        );

    assert_eq!(
        lowered,
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args: vec![
                CanonicalTypeExpr::Var("Left".into()),
                CanonicalTypeExpr::Var("Right".into()),
            ],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Rigid,
        },
        "TASK-800 should preserve the nominal base spine instead of collapsing it to a single base node"
    );
}

#[test]
fn task800_nested_projection_base_reports_projection_specific_lowering_error() {
    let module = module_identity(8002, &["pkg", "pair"]);

    let mut env = TypeEnv::new();
    register_pair_projection_metadata(&mut env, &module);

    let err = env
        .lower_core_type_expr_to_canonical(&TypeExpr::Associated {
            base: Box::new(TypeExpr::Associated {
                base: Box::new(TypeExpr::Named("T".into())),
                name: "Item".into(),
            }),
            name: "Item".into(),
        })
        .expect_err(
            "TASK-800 should reject nested projection bases explicitly instead of silently stringifying them",
        );

    let message = err.to_string();
    assert!(
        message.contains("projection")
            && (message.contains("nested")
                || message.contains("base")
                || message.contains("unsupported")),
        "expected projection-specific unsupported-base diagnostic, got: {message}"
    );
    assert!(
        message.contains("Item"),
        "diagnostic should mention the associated member name, got: {message}"
    );
}

#[test]
fn task800_associated_projection_aliases_with_same_canonical_identity_compare_equal_at_equality_boundary()
 {
    let module = module_identity(8003, &["pkg", "serializer"]);
    let interface = InterfaceIdentityId::new(module.clone(), "Serializer");
    let member = ok_member_identity(&interface);

    let summary = ModuleSemanticSummary::new(module.clone())
        .with_interface_identity(InterfaceIdentitySummary::new(
            interface.clone(),
            "Serializer",
            vec!["Serializer".into()],
            anchor("interface Serializer"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            member.clone(),
            "Ok",
            anchor("associated type Ok"),
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("test precondition: primary summary should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        interface.clone(),
        "SerializerAlias",
        vec!["SerializerAlias".into()],
        anchor("interface SerializerAlias"),
    ))
    .expect("test precondition: alias interface identity should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        AssociatedMemberIdentityId::associated_type(
            interface,
            "Ok",
            vec!["SerializerAlias".into(), "Ok".into()],
        ),
        "Ok",
        anchor("associated type Ok alias"),
    ))
    .expect("test precondition: alias associated member identity should register");

    let canonical = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(ash_typeck::TypeVar(99))),
        name: "Ok".into(),
    };
    let alias = Type::Associated {
        interface: "SerializerAlias".into(),
        base: Box::new(Type::Var(ash_typeck::TypeVar(99))),
        name: "Ok".into(),
    };

    assert!(
        env.types_equivalent_for_equality(&canonical, &alias),
        "TASK-800/TASK-802 should canonicalize imported alias names that point at the same projection identity"
    );
}

#[test]
fn task800_distinct_projection_identities_remain_distinct_at_equality_boundary() {
    let first_module = module_identity(8004, &["pkg", "serializer"]);
    let second_module = module_identity(8005, &["pkg", "encoder"]);
    let first_interface = InterfaceIdentityId::new(first_module.clone(), "Serializer");
    let second_interface = InterfaceIdentityId::new(second_module.clone(), "SerializerV2");

    let mut env = TypeEnv::new();
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        first_interface.clone(),
        "Serializer",
        vec!["Serializer".into()],
        anchor("interface serializer one"),
    ))
    .expect("test precondition: first interface should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        ok_member_identity(&first_interface),
        "Ok",
        anchor("associated type Ok one"),
    ))
    .expect("test precondition: first member should register");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        second_interface.clone(),
        "SerializerV2",
        vec!["SerializerV2".into()],
        anchor("interface serializer two"),
    ))
    .expect("test precondition: second interface should register");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        ok_member_identity(&second_interface),
        "Ok",
        anchor("associated type Ok two"),
    ))
    .expect("test precondition: second member should register");

    let left = Type::Associated {
        interface: "Serializer".into(),
        base: Box::new(Type::Var(ash_typeck::TypeVar(1))),
        name: "Ok".into(),
    };
    let right = Type::Associated {
        interface: "SerializerV2".into(),
        base: Box::new(Type::Var(ash_typeck::TypeVar(1))),
        name: "Ok".into(),
    };

    assert!(
        !env.types_equivalent_for_equality(&left, &right),
        "TASK-800/TASK-802 must not collapse distinct canonical projection identities just because their visible names overlap"
    );
}

#[test]
fn task800_fn_signature_type_rejects_unresolved_projection_member_in_public_surface() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let function = projection_fn(SurfaceType::Associated {
        base: Box::new(SurfaceType::Name("T".into())),
        name: "Missing".into(),
    });

    let err = fn_signature_type(&env, &function)
        .expect_err("TASK-800 should reject unresolved public-surface projection members");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_builtin_fn_signature_type_rejects_unresolved_projection_member_in_public_surface() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");
    env.register_interface(&formatter_interface_def())
        .expect("test precondition: Formatter should register");

    let builtin = projection_builtin(SurfaceType::Associated {
        base: Box::new(SurfaceType::Name("T".into())),
        name: "Missing".into(),
    });

    let err = builtin_fn_signature_type(&env, &builtin)
        .expect_err("TASK-800 should reject unresolved builtin projection members");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_workflow_declared_projection_return_rejects_unresolved_member_in_public_surface() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let mut workflow = projection_workflow_with_declared_return();
    workflow.declared_return_type = Some(SurfaceType::Associated {
        base: Box::new(SurfaceType::Name("T".into())),
        name: "Missing".into(),
    });

    let err = ash_typeck::type_check_workflow_def_in_env(&env, &workflow)
        .expect_err("TASK-800 should reject unresolved declared workflow projection returns");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_fn_signature_type_does_not_fall_back_to_global_interface_scan_for_unbounded_projection()
{
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let function = projection_fn_without_bounds(SurfaceType::Associated {
        base: Box::new(SurfaceType::Name("T".into())),
        name: "Ok".into(),
    });

    let err = fn_signature_type(&env, &function)
        .expect_err("TASK-800 should keep unbounded generic projections unresolved");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Ok'"));
}

#[test]
fn task800_fn_signature_type_does_not_escape_declared_bounds_when_member_is_missing() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let function = projection_fn(SurfaceType::Associated {
        base: Box::new(SurfaceType::Name("T".into())),
        name: "Missing".into(),
    });

    let err = fn_signature_type(&env, &function)
        .expect_err("TASK-800 should reject members outside declared bounds instead of fabricating a projection");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
    assert!(!message.contains("Serializer::Missing"));
}

#[test]
fn task800_register_impl_accepts_in_bounds_projection_binding_member() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    env.register_impl(&serializer_impl_with_in_bounds_projection_binding("Ok"))
        .expect("TASK-800 should keep impl-local in-bounds projection bindings accepted");
}

#[test]
fn task800_register_impl_rejects_ambiguous_impl_local_projection_binding_member() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");
    env.register_interface(&formatter_interface_def())
        .expect("test precondition: Formatter should register");

    let err = env
        .register_impl(&serializer_impl_with_ambiguous_in_bounds_projection_binding())
        .expect_err("TASK-800 should reject ambiguous impl-local projection members");
    let message = err.to_string();
    assert!(message.contains("ambiguous associated type 'Ok'"));
}

#[test]
fn task800_register_impl_rejects_unresolved_projection_binding_member() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let err = env
        .register_impl(&serializer_impl_with_associated_projection_binding(
            "Missing",
        ))
        .expect_err("TASK-800 should reject unresolved impl-local projection members");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_register_impl_rejects_unknown_impl_local_bound_interface_in_projection_binding_seam() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let err = env
        .register_impl(&serializer_impl_with_where_bound_projection_binding(
            "UnknownBound",
            "Ok",
        ))
        .expect_err(
            "TASK-800 should reject unknown interfaces in impl-local projection where bounds",
        );
    let message = err.to_string();
    assert!(message.contains("unknown interface 'UnknownBound' in where bound"));
}

#[test]
fn task800_register_impl_rejects_out_of_bounds_projection_binding_member() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("test precondition: Serializer should register");

    let err = env
        .register_impl(&serializer_impl_with_where_bound_projection_binding(
            "Serializer",
            "Missing",
        ))
        .expect_err("TASK-800 should reject out-of-bounds impl-local projection members");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_register_interface_accepts_projection_method_return_in_interface_bounds() {
    let mut env = TypeEnv::with_builtin_types();

    env.register_interface(&serializer_interface_with_projection_method())
        .expect("TASK-800 should keep interface method associated projection returns explicit and in-bounds");

    let iface = env
        .lookup_interface("Serializer")
        .expect("Serializer registered");
    let method = iface
        .methods
        .get("finish")
        .expect("finish method registered");
    assert_eq!(
        method.return_type,
        Type::Associated {
            interface: "Serializer".into(),
            base: Box::new(Type::Var(method.type_params[0])),
            name: "Ok".into(),
        }
    );
}

#[test]
fn task800_register_interface_rejects_unknown_projection_member_in_method_signature() {
    let mut env = TypeEnv::with_builtin_types();

    let err = env
        .register_interface(&serializer_interface_with_unknown_projection_method())
        .expect_err("TASK-800 should reject interface methods whose associated projection member is unresolved");
    let message = err.to_string();
    assert!(message.contains("unresolved associated type 'Missing'"));
}

#[test]
fn task800_type_expr_to_type_rejects_unresolved_associated_projection_without_bounds() {
    let env = TypeEnv::with_builtin_types();
    let err = type_expr_to_type(
        &TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Ok".into(),
        },
        &std::collections::HashMap::new(),
        &env,
    )
    .expect_err(
        "TASK-800 should not emit unresolved empty-interface sentinels from type_expr_to_type",
    );
    assert!(err.to_string().contains("unresolved associated type 'Ok'"));
}

#[test]
fn task800_type_expr_to_type_accepts_in_bounds_associated_projection_with_typevar_mapping() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&serializer_interface_def())
        .expect("register Serializer");

    let type_var = ash_typeck::TypeVar(9000);
    env.bind_type_var_interface_bound(type_var, "Serializer");

    let mut mapping = std::collections::HashMap::new();
    mapping.insert("T".to_string(), type_var);

    let ty = type_expr_to_type(
        &TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Ok".into(),
        },
        &mapping,
        &env,
    )
    .expect("TASK-800 should resolve associated projections through explicit typevar bounds");

    assert_eq!(
        ty,
        Type::Associated {
            interface: "Serializer".into(),
            base: Box::new(Type::Var(type_var)),
            name: "Ok".into(),
        }
    );
}
