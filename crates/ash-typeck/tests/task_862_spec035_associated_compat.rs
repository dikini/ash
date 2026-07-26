use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, ModuleIdentity, ModuleSourceOrigin,
    SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, CanonicalTypeExpr,
    ProjectionRigidity,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, Expr,
    ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, InterfaceTypeParam, Literal,
    Type as SurfaceType, Visibility, WhereBound,
};
use ash_parser::token::Span;
use ash_typeck::Type;
use ash_typeck::type_env::TypeEnv;

fn span() -> Span {
    Span::default()
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-862 test".to_string(),
        },
        None,
        label,
    )
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(862)),
        ModuleId(id),
        vec![name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-862 {name}"),
        },
    )
}

fn domain(module: &ModuleIdentity, name: &str) -> SealedDomainSummary {
    let id = SealedDomainId::new(module.clone(), name.to_string());
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(id.clone(), "Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    );
    SealedDomainSummary::new(id, name, CoreVisibility::Public, anchor(name)).with_constructor(nil)
}

fn param(name: &str, domain: Option<&str>) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: domain.map(|name| SurfaceType::Name(name.into())),
        kind: None,
        span: span(),
    }
}

fn sealed_family_interface() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Append".into(),
        type_params: vec![param("Xs", Some("TypeList")), param("Ys", Some("TypeList"))],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("TypeList".into()),
                decreases: Some(AssociatedFamilyDecreases {
                    param: "Xs".into(),
                    span: span(),
                }),
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn env_with_family() -> (TypeEnv, ModuleIdentity, SealedDomainId) {
    let owner = module("owner", 1);
    let type_list = domain(&owner, "TypeList");
    let domain_id = type_list.id.clone();
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner.clone());
    env.register_local_sealed_domain_summary(&type_list)
        .expect("sealed domain precondition");
    env.register_interface(&sealed_family_interface())
        .expect("sealed family declaration registers");
    (env, owner, domain_id)
}

fn serializer_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Serializer".into(),
        type_params: vec![param("S", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: AssociatedTypeKind::Ordinary,
            span: span(),
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
            span: span(),
        }],
        laws: Vec::new(),
        span: span(),
    }
}

fn formatter_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Formatter".into(),
        type_params: vec![param("F", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Ok".into(),
            kind: AssociatedTypeKind::Ordinary,
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn serializer_string_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Serializer".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Ok".into(),
            ty: SurfaceType::Name("String".into()),
            span: span(),
        }],
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: Expr::Literal(Literal::String("serialized".into())),
            span: span(),
        }],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    }
}

fn ambiguous_serializer_impl() -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Serializer".into(),
        type_params: vec![param("T", None)],
        type_args: vec![SurfaceType::Name("String".into())],
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
        methods: vec![ImplMethodDef {
            name: "serialize_bool".into(),
            params: vec!["writer".into(), "value".into()],
            body: Expr::Literal(Literal::String("ambiguous".into())),
            span: span(),
        }],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    }
}

fn type_kind_family_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Compute".into(),
        type_params: vec![param("X", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None,
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn explicit_compute_string_out_projection() -> SurfaceType {
    SurfaceType::AssociatedFamilyProjection {
        interface: "Compute".into(),
        args: vec![SurfaceType::Name("String".into())],
        member: "Out".into(),
        span: span(),
    }
}

fn explicit_append_out_projection() -> SurfaceType {
    SurfaceType::AssociatedFamilyProjection {
        interface: "Append".into(),
        args: vec![
            SurfaceType::Name("Xs".into()),
            SurfaceType::Name("Ys".into()),
        ],
        member: "Out".into(),
        span: span(),
    }
}

#[test]
fn task_862_spec035_selected_impl_substitution_survives_with_family_table_present() {
    let (mut env, _, _) = env_with_family();
    env.register_interface(&serializer_interface_def())
        .expect("ordinary associated interface registers beside family table");
    env.register_impl(&serializer_string_impl())
        .expect("ordinary associated impl registers beside family table");

    assert!(
        env.lookup_associated_family_declaration("Append", "Out")
            .is_some()
    );
    assert!(
        env.lookup_associated_family_declaration("Serializer", "Ok")
            .is_none()
    );
    assert!(
        env.associated_family_schemes(
            &env.lookup_associated_family_declaration("Append", "Out")
                .expect("family declaration")
                .head,
        )
        .is_none()
    );

    let (selected, scheme) = env
        .select_impl_scheme("Serializer", "serialize_bool", &[Type::String, Type::Bool])
        .expect("concrete Serializer<String> impl selection should still work");
    let raw_return = scheme.methods[0].return_type.clone();
    let normalized = env
        .normalize_associated_types(&raw_return, scheme, &selected.substitution)
        .expect("ordinary associated type substitution should normalize");
    assert_eq!(normalized, Type::String);

    let resolved = env
        .resolve_interface_method_call("Serializer", "serialize_bool", &[Type::String, Type::Bool])
        .expect("method call return should use SPEC-035 substitution");
    assert_eq!(resolved, Type::String);
}

#[test]
fn task_862_explicit_family_projection_lowers_to_canonical_projection_identity() {
    let (env, _, _) = env_with_family();
    let decl = env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("family declaration precondition");

    let lowered = env
        .lower_surface_type_to_canonical(&explicit_append_out_projection())
        .expect("explicit family projection should lower after TASK-862 bridge");

    assert_eq!(
        lowered,
        CanonicalTypeExpr::Projection {
            interface: decl.head.interface.clone(),
            member: decl.head.member.clone(),
            args: vec![
                CanonicalTypeExpr::Var("Xs".to_string()),
                CanonicalTypeExpr::Var("Ys".to_string()),
            ],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Neutral,
        }
    );
}

#[test]
fn task_862_family_impl_rhs_explicit_projection_publishes_family_projection_identity() {
    let owner = module("mirror", 2);
    let type_list = domain(&owner, "TypeList");
    let type_list_id = type_list.id.clone();
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_local_sealed_domain_summary(&type_list)
        .expect("sealed domain precondition");

    let mut interface = sealed_family_interface();
    interface.associated_types.push(AssociatedTypeDecl {
        name: "Mirror".into(),
        kind: AssociatedTypeKind::SealedFamily {
            result_domain: SurfaceType::Name("TypeList".into()),
            decreases: Some(AssociatedFamilyDecreases {
                param: "Xs".into(),
                span: span(),
            }),
            span: span(),
        },
        span: span(),
    });
    env.register_interface(&interface)
        .expect("two-family interface registers");

    let impl_def = ImplDef {
        visibility: Visibility::Inherited,
        interface: "Append".into(),
        type_params: vec![param("Xs", Some("TypeList")), param("Ys", Some("TypeList"))],
        type_args: vec![
            SurfaceType::Name("Xs".into()),
            SurfaceType::Name("Ys".into()),
        ],
        where_bounds: vec![],
        associated_type_bindings: vec![
            AssociatedTypeBinding {
                name: "Out".into(),
                ty: SurfaceType::Name("Nil".into()),
                span: span(),
            },
            AssociatedTypeBinding {
                name: "Mirror".into(),
                ty: explicit_append_out_projection(),
                span: span(),
            },
        ],
        methods: vec![],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    };
    env.register_impl(&impl_def)
        .expect("family impl RHS explicit projection should publish");

    let out_decl = env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("Out declaration");
    let mirror_decl = env
        .lookup_associated_family_declaration("Append", "Mirror")
        .expect("Mirror declaration");
    let mirror_schemes = env
        .associated_family_schemes(&mirror_decl.head)
        .expect("Mirror scheme published");
    assert_eq!(mirror_schemes.len(), 1);
    let result = &mirror_schemes[0].scheme.equations[0].result;

    match result {
        AssociatedFamilyResultExpr::AssociatedFamilyProjection {
            head,
            interface_args,
            kind,
            constraint,
            rigidity,
            ..
        } => {
            assert_eq!(head, &out_decl.head);
            assert_eq!(interface_args.len(), 2);
            match &interface_args[0] {
                AssociatedFamilyResultExpr::Var {
                    name,
                    constraint,
                    kind,
                    ..
                } => {
                    assert_eq!(name, "Xs");
                    assert_eq!(
                        constraint,
                        &AssociatedFamilyResultConstraint::Domain(type_list_id.clone())
                    );
                    assert_eq!(kind, &Kind::Type);
                }
                other => panic!("expected Xs var argument, got {other:?}"),
            }
            match &interface_args[1] {
                AssociatedFamilyResultExpr::Var {
                    name,
                    constraint,
                    kind,
                    ..
                } => {
                    assert_eq!(name, "Ys");
                    assert_eq!(
                        constraint,
                        &AssociatedFamilyResultConstraint::Domain(type_list_id.clone())
                    );
                    assert_eq!(kind, &Kind::Type);
                }
                other => panic!("expected Ys var argument, got {other:?}"),
            }
            assert_eq!(kind, &Kind::Type);
            assert_eq!(
                constraint,
                &AssociatedFamilyResultConstraint::Domain(type_list_id)
            );
            assert_eq!(rigidity, &ProjectionRigidity::Neutral);
        }
        other => panic!("expected family projection RHS, got {other:?}"),
    }
}

#[test]
fn task_862_ambiguous_t_assoc_diagnostic_remains_stable_with_family_bridge_present() {
    let (mut env, _, _) = env_with_family();
    env.register_interface(&serializer_interface_def())
        .expect("Serializer registers");
    env.register_interface(&formatter_interface_def())
        .expect("Formatter registers");

    let err = env
        .register_impl(&ambiguous_serializer_impl())
        .expect_err("ambiguous T::Ok must remain rejected");
    let message = err.to_string();
    assert!(
        message.contains("ambiguous associated type 'Ok'"),
        "expected stable ambiguity diagnostic, got: {message}"
    );
    assert!(
        !message.contains("family"),
        "ordinary ambiguity diagnostic should not claim family reduction/search: {message}"
    );
}

#[test]
fn task_862_ordinary_associated_type_does_not_publish_family_scheme_in_mixed_interface() {
    let owner = module("mixed", 3);
    let type_list = domain(&owner, "TypeList");
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_local_sealed_domain_summary(&type_list)
        .expect("sealed domain precondition");

    let mut mixed = sealed_family_interface();
    mixed.name = "MixedAppend".into();
    mixed.associated_types.push(AssociatedTypeDecl {
        name: "Trace".into(),
        kind: AssociatedTypeKind::Ordinary,
        span: span(),
    });
    env.register_interface(&mixed)
        .expect("mixed ordinary/family interface registers");
    assert!(
        env.lookup_associated_family_declaration("MixedAppend", "Out")
            .is_some()
    );
    assert!(
        env.lookup_associated_family_declaration("MixedAppend", "Trace")
            .is_none()
    );

    let impl_def = ImplDef {
        visibility: Visibility::Inherited,
        interface: "MixedAppend".into(),
        type_params: vec![param("Xs", Some("TypeList")), param("Ys", Some("TypeList"))],
        type_args: vec![
            SurfaceType::Name("Xs".into()),
            SurfaceType::Name("Ys".into()),
        ],
        where_bounds: vec![],
        associated_type_bindings: vec![
            AssociatedTypeBinding {
                name: "Out".into(),
                ty: SurfaceType::Name("Nil".into()),
                span: span(),
            },
            AssociatedTypeBinding {
                name: "Trace".into(),
                ty: SurfaceType::Name("String".into()),
                span: span(),
            },
        ],
        methods: vec![],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    };
    env.register_impl(&impl_def)
        .expect("mixed ordinary/family impl registers");

    let out_decl = env
        .lookup_associated_family_declaration("MixedAppend", "Out")
        .expect("Out family declaration");
    assert_eq!(
        env.associated_family_schemes(&out_decl.head)
            .expect("Out scheme published")
            .len(),
        1
    );

    let mixed_impl = env
        .impl_schemes()
        .iter()
        .find(|scheme| scheme.interface == "MixedAppend")
        .expect("ordinary impl scheme still exists");
    assert!(mixed_impl.associated_type_bindings.contains_key("Trace"));
    assert!(!mixed_impl.associated_type_bindings.contains_key("Out"));
    assert_eq!(mixed_impl.associated_type_bindings["Trace"], Type::String);
}

#[test]
fn task_862_explicit_projection_syntax_rejects_ordinary_associated_members() {
    let owner = module("ordinary_projection", 5);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&serializer_interface_def())
        .expect("ordinary associated interface registers");

    let err = env
        .lower_surface_type_to_canonical(&SurfaceType::AssociatedFamilyProjection {
            interface: "Serializer".into(),
            args: vec![SurfaceType::Name("S".into())],
            member: "Ok".into(),
            span: span(),
        })
        .expect_err("explicit family syntax must not target ordinary associated types");
    let message = err.to_string();
    assert!(
        message.contains("registered sealed associated-family projection")
            || message.contains("Serializer"),
        "expected sealed-family-only diagnostic, got: {message}"
    );
    assert!(
        env.lookup_associated_family_declaration("Serializer", "Ok")
            .is_none()
    );
}

#[test]
fn task_862_type_kind_family_projection_preserves_concrete_type_arguments() {
    let owner = module("type_kind", 6);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&type_kind_family_interface_def())
        .expect("Type-kind associated family registers");

    let impl_def = ImplDef {
        visibility: Visibility::Inherited,
        interface: "Compute".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("String".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Out".into(),
            ty: explicit_compute_string_out_projection(),
            span: span(),
        }],
        methods: vec![],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    };
    env.register_impl(&impl_def)
        .expect("Type-kind family projection with concrete argument should publish");

    let decl = env
        .lookup_associated_family_declaration("Compute", "Out")
        .expect("family declaration");
    let schemes = env
        .associated_family_schemes(&decl.head)
        .expect("scheme published");
    let result = &schemes[0].scheme.equations[0].result;
    match result {
        AssociatedFamilyResultExpr::AssociatedFamilyProjection { interface_args, .. } => {
            assert_eq!(interface_args.len(), 1);
            match &interface_args[0] {
                AssociatedFamilyResultExpr::Primitive {
                    name,
                    kind,
                    constraint,
                    ..
                } => {
                    assert_eq!(name, "String");
                    assert_eq!(kind, &Kind::Type);
                    assert_eq!(
                        constraint,
                        &AssociatedFamilyResultConstraint::Kind(Kind::Type)
                    );
                }
                other => panic!("expected concrete primitive String argument, got {other:?}"),
            }
        }
        other => panic!("expected associated family projection result, got {other:?}"),
    }
}

#[test]
fn task_862_unselected_ordinary_projection_remains_neutral_not_family_scheme() {
    let owner = module("ordinary", 4);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&serializer_interface_def())
        .expect("ordinary associated interface registers");

    let lowered = env
        .lower_surface_type_to_canonical(&SurfaceType::Associated {
            base: Box::new(SurfaceType::Name("S".into())),
            name: "Ok".into(),
        })
        .expect("ordinary associated projection lowers neutrally");

    match lowered {
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
        } => {
            assert_eq!(interface.name, "Serializer");
            assert_eq!(member.name, "Ok");
            assert_eq!(args, vec![CanonicalTypeExpr::Var("S".to_string())]);
            assert_eq!(kind, Kind::Type);
            assert_eq!(rigidity, ProjectionRigidity::Neutral);
        }
        other => panic!("ordinary associated projection should stay neutral, got {other:?}"),
    }
    assert!(
        env.lookup_associated_family_declaration("Serializer", "Ok")
            .is_none()
    );
    assert!(env.impl_schemes().is_empty());
}
