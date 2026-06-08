use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, InterfaceIdentityId, ModuleIdentity,
    ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyPattern, AssociatedFamilyResultConstraint,
    AssociatedFamilyResultExpr, AssociatedFamilyScheme, AssociatedFamilySchemeParam,
    CanonicalTypeExpr,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind,
    ImplDef, InterfaceDef, InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::error::TypeEnvError;

fn span() -> Span {
    Span::default()
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-861 test".to_string(),
        },
        None,
        label,
    )
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(861)),
        ModuleId(id),
        vec![name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-861 {name}"),
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

fn impl_with_binding(binding_name: &str, rhs: SurfaceType) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Append".into(),
        type_params: vec![param("Xs", Some("TypeList")), param("Ys", Some("TypeList"))],
        type_args: vec![
            SurfaceType::Name("Xs".into()),
            SurfaceType::Name("Ys".into()),
        ],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: binding_name.into(),
            ty: rhs,
            span: span(),
        }],
        methods: vec![],
        proofs: Vec::new(),
        span: span(),
    }
}

fn env_with_family() -> (TypeEnv, ModuleIdentity, SealedDomainId) {
    let owner = module("owner", 1);
    let type_list = domain(&owner, "TypeList");
    let domain_id = type_list.id.clone();
    let mut env = TypeEnv::new();
    env.set_current_module_identity(owner.clone());
    env.register_local_sealed_domain_summary(&type_list)
        .expect("sealed domain precondition");
    env.register_interface(&sealed_family_interface())
        .expect("sealed associated family declaration should register");
    (env, owner, domain_id)
}

fn nil_result(domain: &SealedDomainId) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::DomainConstructorApp {
        constructor: DomainConstructorId::new(domain.clone(), "Nil"),
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
        source_anchor: anchor("rhs Nil"),
    }
}

fn generic_scheme_for(
    env: &TypeEnv,
    interface: &str,
    family: &str,
    domain: &SealedDomainId,
    result: AssociatedFamilyResultExpr,
) -> AssociatedFamilyScheme {
    let decl = env
        .lookup_associated_family_declaration(interface, family)
        .expect("family declaration precondition");
    AssociatedFamilyScheme {
        head: decl.head.clone(),
        params: vec![
            AssociatedFamilySchemeParam {
                name: "Xs".to_string(),
                ty: CanonicalTypeExpr::Var("Xs".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(domain.clone()),
                source_anchor: anchor("Xs"),
            },
            AssociatedFamilySchemeParam {
                name: "Ys".to_string(),
                ty: CanonicalTypeExpr::Var("Ys".to_string()),
                kind: Kind::Type,
                domain_constraint: Some(domain.clone()),
                source_anchor: anchor("Ys"),
            },
        ],
        result_domain: CanonicalTypeExpr::Var("TypeList".to_string()),
        result_kind: Kind::Type,
        equations: vec![AssociatedFamilyEquation {
            head: decl.head.clone(),
            ordinal: 0,
            interface_arg_patterns: vec![
                AssociatedFamilyPattern::Var {
                    name: "Xs".to_string(),
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("pattern Xs"),
                },
                AssociatedFamilyPattern::Var {
                    name: "Ys".to_string(),
                    constraint: AssociatedFamilyResultConstraint::Domain(domain.clone()),
                    source_anchor: anchor("pattern Ys"),
                },
            ],
            result,
            decreases: None,
            source_anchor: anchor("equation"),
            case_head_anchor: anchor("case head"),
        }],
        source_anchor: anchor("scheme"),
    }
}

fn generic_scheme(
    env: &TypeEnv,
    domain: &SealedDomainId,
    result: AssociatedFamilyResultExpr,
) -> AssociatedFamilyScheme {
    generic_scheme_for(env, "Append", "Out", domain, result)
}

#[test]
fn task_861_associated_family_registration_preserves_domains_and_registers_impl_scheme() {
    let (mut env, owner, type_list) = env_with_family();

    let decl = env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("sealed family declaration metadata should be queryable");
    assert_eq!(decl.defining_module, owner);
    assert_eq!(
        decl.result_domain,
        AssociatedFamilyResultConstraint::Domain(type_list.clone())
    );
    assert_eq!(decl.decreases.as_deref(), Some("Xs"));
    assert_eq!(decl.interface_params[0].name, "Xs");
    assert_eq!(
        decl.interface_params[0].domain_constraint,
        Some(type_list.clone())
    );
    assert_eq!(
        decl.head.interface,
        InterfaceIdentityId::new(owner.clone(), "Append")
    );
    let family_head = decl.head.clone();

    env.register_impl(&impl_with_binding("Out", SurfaceType::Name("Nil".into())))
        .expect("impl family binding should publish a dedicated family scheme");

    let schemes = env
        .associated_family_schemes(&family_head)
        .expect("scheme table should be keyed by the declared family head");
    assert_eq!(schemes.len(), 1);
    assert_eq!(schemes[0].defining_module, owner);
    assert_eq!(schemes[0].scheme.head, family_head);
    assert_eq!(
        schemes[0].scheme.params[0].domain_constraint,
        Some(type_list)
    );
}

#[test]
fn task_861_accepts_kind_type_result_domain_for_common_associated_family_shape() {
    let owner = module("kind_result", 20);
    let mut env = TypeEnv::new();
    env.set_current_module_identity(owner);
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![param("I", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
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
    })
    .expect("sealed associated family result domain ': Type' should register");

    let decl = env
        .lookup_associated_family_declaration("Iterator", "Item")
        .expect("kind-result family declaration should be registered");
    assert_eq!(
        decl.result_domain,
        AssociatedFamilyResultConstraint::Kind(Kind::Type)
    );

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param("I", None)],
        type_args: vec![SurfaceType::Name("I".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: SurfaceType::Name("String".into()),
            span: span(),
        }],
        methods: vec![],
        proofs: Vec::new(),
        span: span(),
    })
    .expect("kind-result family impl should accept a Type-kind RHS");
}

#[test]
fn task_861_reports_missing_extra_and_duplicate_family_bindings_precisely() {
    let (mut missing_env, _, _) = env_with_family();
    let missing = ImplDef {
        associated_type_bindings: vec![],
        ..impl_with_binding("Out", SurfaceType::Name("Nil".into()))
    };
    let err = missing_env
        .register_impl(&missing)
        .expect_err("sealed family impl must bind the family member");
    assert!(
        matches!(err, TypeEnvError::MissingAssociatedFamilyBinding { family, .. } if family == "Out")
    );

    let (mut extra_env, _, _) = env_with_family();
    let err = extra_env
        .register_impl(&impl_with_binding("Bogus", SurfaceType::Name("Nil".into())))
        .expect_err("extra impl family binding must be rejected precisely");
    assert!(
        matches!(err, TypeEnvError::ExtraAssociatedFamilyBinding { family, .. } if family == "Bogus")
    );

    let owner = module("dup", 2);
    let mut dup_env = TypeEnv::new();
    dup_env.set_current_module_identity(owner.clone());
    dup_env
        .register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");
    let mut duplicate = sealed_family_interface();
    duplicate
        .associated_types
        .push(duplicate.associated_types[0].clone());
    let err = dup_env
        .register_interface(&duplicate)
        .expect_err("duplicate sealed associated-family heads must be rejected");
    assert!(
        matches!(err, TypeEnvError::DuplicateAssociatedFamilyHead { family, .. } if family == "Out")
    );
}

#[test]
fn task_861_validates_decreases_parameter_and_malformed_direct_scheme_shapes() {
    let owner = module("bad_decreases", 30);
    let mut decreases_env = TypeEnv::new();
    decreases_env.set_current_module_identity(owner.clone());
    decreases_env
        .register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");
    let err = decreases_env
        .register_interface(&InterfaceDef {
            visibility: Visibility::Inherited,
            name: "BadAppend".into(),
            type_params: vec![param("Xs", None)],
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
        })
        .expect_err("decreases must name a sealed-domain-constrained interface parameter");
    assert!(
        err.to_string()
            .contains("must have a sealed-domain constraint")
    );

    let (mut env, owner, type_list) = env_with_family();
    let mut empty = generic_scheme(&env, &type_list, nil_result(&type_list));
    empty.equations.clear();
    let err = env
        .register_associated_family_scheme(empty, owner.clone())
        .expect_err("empty direct family scheme must be rejected");
    let message = err.to_string();
    assert!(message.contains("must contain at least one equation"));
    assert!(!message.contains("exactly one equation"));

    let mut wrong_arity = generic_scheme(&env, &type_list, nil_result(&type_list));
    wrong_arity.equations[0].interface_arg_patterns.pop();
    let err = env
        .register_associated_family_scheme(wrong_arity, owner)
        .expect_err("wrong-arity direct family scheme must be rejected");
    assert!(err.to_string().contains("interface argument patterns"));
}

#[test]
fn task_861_rejects_missing_owner_context_and_downstream_sealed_family_extension() {
    let owner = module("no_context", 3);
    let mut no_context = TypeEnv::new();
    no_context
        .register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");
    let err = no_context
        .register_interface(&sealed_family_interface())
        .expect_err("sealed family declaration requires module owner context");
    assert!(matches!(
        err,
        TypeEnvError::AssociatedFamilyModuleOwnerViolation { .. }
    ));

    let (mut env, owner, type_list) = env_with_family();
    let downstream = module("downstream", 4);
    let scheme = generic_scheme(&env, &type_list, nil_result(&type_list));
    let err = env
        .register_associated_family_scheme(scheme, downstream)
        .expect_err("sealed family equations cannot be extended outside the owning module");
    assert!(
        matches!(err, TypeEnvError::UnauthorizedAssociatedFamilyExtension { owner_module, .. } if owner_module == owner)
    );
}

#[test]
fn task_861_rejected_impl_does_not_publish_family_scheme() {
    let mut mixed = sealed_family_interface();
    mixed.name = "MixedAppend".into();
    mixed.associated_types.push(AssociatedTypeDecl {
        name: "Trace".into(),
        kind: AssociatedTypeKind::Ordinary,
        span: span(),
    });

    let owner = module("mixed", 40);
    let mut mixed_env = TypeEnv::new();
    mixed_env.set_current_module_identity(owner.clone());
    mixed_env
        .register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");
    mixed_env
        .register_interface(&mixed)
        .expect("mixed ordinary/family interface should register");
    let head = mixed_env
        .lookup_associated_family_declaration("MixedAppend", "Out")
        .expect("family declaration should exist")
        .head
        .clone();

    let mut invalid_impl = impl_with_binding("Out", SurfaceType::Name("Nil".into()));
    invalid_impl.interface = "MixedAppend".into();
    let err = mixed_env
        .register_impl(&invalid_impl)
        .expect_err("impl missing ordinary associated type must be rejected");
    assert!(matches!(err, TypeEnvError::MissingAssociatedType { name, .. } if name == "Trace"));
    assert!(
        mixed_env
            .associated_family_schemes(&head)
            .is_none_or(Vec::is_empty),
        "rejected impl must not publish associated-family schemes"
    );
}

#[test]
fn task_861_impl_family_publication_rolls_back_if_later_family_scheme_fails() {
    let owner = module("rollback", 41);
    let mut env = TypeEnv::new();
    env.set_current_module_identity(owner.clone());
    let type_list = domain(&owner, "TypeList");
    let type_list_id = type_list.id.clone();
    env.register_local_sealed_domain_summary(&type_list)
        .expect("sealed domain precondition");

    let mut interface = sealed_family_interface();
    interface.name = "RollbackAppend".into();
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
        .expect("two-family interface should register");

    let out_head = env
        .lookup_associated_family_declaration("RollbackAppend", "Out")
        .expect("Out declaration should exist")
        .head
        .clone();
    let mirror_head = env
        .lookup_associated_family_declaration("RollbackAppend", "Mirror")
        .expect("Mirror declaration should exist")
        .head
        .clone();

    let existing_mirror = generic_scheme_for(
        &env,
        "RollbackAppend",
        "Mirror",
        &type_list_id,
        nil_result(&type_list_id),
    );
    env.register_associated_family_scheme(existing_mirror, owner.clone())
        .expect("preexisting Mirror scheme should publish");

    let mut impl_def = impl_with_binding("Out", SurfaceType::Name("Nil".into()));
    impl_def.interface = "RollbackAppend".into();
    impl_def
        .associated_type_bindings
        .push(AssociatedTypeBinding {
            name: "Mirror".into(),
            ty: SurfaceType::Name("Nil".into()),
            span: span(),
        });

    let err = env
        .register_impl(&impl_def)
        .expect_err("later overlapping Mirror scheme should reject the whole impl");
    assert!(matches!(
        err,
        TypeEnvError::OverlappingAssociatedFamilyScheme { family, .. } if family == "Mirror"
    ));
    assert!(
        env.associated_family_schemes(&out_head)
            .is_none_or(Vec::is_empty),
        "failed second family publication must roll back the earlier Out scheme"
    );
    assert_eq!(
        env.associated_family_schemes(&mirror_head)
            .expect("preexisting Mirror scheme should remain")
            .len(),
        1
    );
}

#[test]
fn task_861_validates_family_overlap_result_kind_and_result_domain_before_publication() {
    let (mut env, owner, type_list) = env_with_family();
    let scheme = generic_scheme(&env, &type_list, nil_result(&type_list));
    env.register_associated_family_scheme(scheme.clone(), owner.clone())
        .expect("first family scheme should publish");
    let err = env
        .register_associated_family_scheme(scheme, owner.clone())
        .expect_err("same-head same-pattern family scheme must be rejected as overlap");
    assert!(
        matches!(err, TypeEnvError::OverlappingAssociatedFamilyScheme { family, .. } if family == "Out")
    );

    let (mut kind_env, owner, type_list) = env_with_family();
    let mut wrong_kind = generic_scheme(&kind_env, &type_list, nil_result(&type_list));
    wrong_kind.result_kind = Kind::n_ary(1);
    let err = kind_env
        .register_associated_family_scheme(wrong_kind, owner.clone())
        .expect_err("associated family schemes must have Type result kind");
    assert!(
        matches!(err, TypeEnvError::WrongAssociatedFamilyResultKind { family, .. } if family == "Out")
    );

    let (mut domain_env, owner, type_list) = env_with_family();
    let other = domain(&owner, "OtherDomain");
    let other_id = other.id.clone();
    domain_env
        .register_local_sealed_domain_summary(&other)
        .expect("second sealed domain precondition");
    let wrong_domain = generic_scheme(&domain_env, &type_list, nil_result(&other_id));
    let err = domain_env
        .register_associated_family_scheme(wrong_domain, owner)
        .expect_err("family RHS must conform to declared result domain");
    assert!(
        matches!(err, TypeEnvError::WrongAssociatedFamilyResultDomain { family, .. } if family == "Out")
    );
}
