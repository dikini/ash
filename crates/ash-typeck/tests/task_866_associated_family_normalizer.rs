use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility as CoreVisibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, DomainConstructorSummary, DomainFieldSummary,
    InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId,
    SealedDomainSummary, SourceAnchor, SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, AssociatedFamilyScheme,
    AssociatedFamilySchemeParam, CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr,
    ProjectionRigidity, TypeComputationHeadId,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind,
    ImplDef, InterfaceDef, InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    DefinitionalEqualityResult, FixtureEquation, FixtureEquationRegistry, FixtureResultExpr,
    NormalizationConfig, NormalizationError, NormalizationEvidence, NormalizationFuel, Normalizer,
};

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(866)),
        ModuleId(id),
        vec!["task866".into(), name.into()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-866 {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-866-test".into(),
        },
        None,
        label,
    )
}

fn span() -> Span {
    Span::default()
}

fn param(name: &str, domain: Option<&str>) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: domain.map(|name| SurfaceType::Name(name.into())),
        kind: None,
        span: span(),
    }
}

fn name_ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn list_ty(item: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "List".into(),
        args: vec![item],
    }
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![param("Self", None)],
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
    }
}

fn iterator_list_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param(param_name, None)],
        type_args: vec![list_ty(name_ty(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    }
}

fn env_with_iterator_family() -> TypeEnv {
    let owner = module("iterator_owner", 1);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("Iterator<List<A>>::Item family impl registers");
    env
}

fn family_head(env: &TypeEnv, interface: &str, member: &str) -> AssociatedFamilyHeadId {
    env.lookup_associated_family_declaration(interface, member)
        .expect("family declaration exists")
        .head
        .clone()
}

fn canonical(env: &TypeEnv, ty: SurfaceType) -> CanonicalTypeExpr {
    env.lower_surface_type_to_canonical(&ty)
        .expect("surface type lowers")
}

fn projection(
    head: &AssociatedFamilyHeadId,
    args: Vec<CanonicalTypeExpr>,
    rigidity: ProjectionRigidity,
) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Projection {
        interface: head.interface.clone(),
        member: head.member.clone(),
        args,
        kind: Kind::Type,
        rigidity,
    }
}

#[test]
fn task_866_reduces_local_iterator_list_item_projection() {
    let env = env_with_iterator_family();
    let head = family_head(&env, "Iterator", "Item");
    let expr = projection(
        &head,
        vec![canonical(&env, list_ty(name_ty("String")))],
        ProjectionRigidity::Neutral,
    );

    let outcome = Normalizer::new(&env)
        .normalize(&expr)
        .expect("local associated-family projection should normalize");

    assert_eq!(
        outcome.evidence,
        NormalizationEvidence::AssociatedFamilyProjectionReduced
    );
    assert_eq!(outcome.normal, NormalTypeExpr::Primitive("String".into()));
}

#[test]
fn task_866_normalizes_projection_arguments_before_family_lookup() {
    let mut env = env_with_iterator_family();
    env.register_type(&TypeDef {
        name: "Strings".into(),
        params: vec![],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "List".into(),
            args: vec![TypeExpr::Named("String".into())],
        }),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("register transparent list alias");
    let head = family_head(&env, "Iterator", "Item");
    let alias_arg = canonical(&env, name_ty("Strings"));

    let outcome = Normalizer::new(&env)
        .normalize(&projection(
            &head,
            vec![alias_arg],
            ProjectionRigidity::Neutral,
        ))
        .expect("alias-normalized argument should be selected by family lookup");

    assert_eq!(outcome.normal, NormalTypeExpr::Primitive("String".into()));
}

fn type_list_domain(module: &ModuleIdentity) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), "TypeList");
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("Cons"),
    );
    SealedDomainSummary::new(
        domain,
        "TypeList",
        CoreVisibility::Public,
        anchor("TypeList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn register_type_list_domain(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(type_list_domain(module));
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    env.register_module_semantic_summary(&summary)
        .expect("sealed domain registers");
}

fn append_interface() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "AppendFamily".into(),
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

fn constraint(domain: &SealedDomainId) -> AssociatedFamilyResultConstraint {
    AssociatedFamilyResultConstraint::Domain(domain.clone())
}

fn param_meta(name: &str, domain: &SealedDomainId) -> AssociatedFamilySchemeParam {
    AssociatedFamilySchemeParam {
        name: name.into(),
        ty: CanonicalTypeExpr::Var(name.into()),
        kind: Kind::Type,
        domain_constraint: Some(domain.clone()),
        source_anchor: anchor(name),
    }
}

fn var_pat(name: &str, domain: &SealedDomainId) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::Var {
        name: name.into(),
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn kind_var_pat(name: &str) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::Var {
        name: name.into(),
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn ctor_pat(
    name: &str,
    domain: &SealedDomainId,
    fields: Vec<AssociatedFamilyPattern>,
) -> AssociatedFamilyPattern {
    AssociatedFamilyPattern::DomainConstructor {
        constructor: Box::new(DomainConstructorId::new(domain.clone(), name)),
        domain: Box::new(domain.clone()),
        fields,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn var_result(name: &str, domain: &SealedDomainId) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Var {
        name: name.into(),
        kind: Kind::Type,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn kind_var_result(name: &str) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::Var {
        name: name.into(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor(name),
    }
}

fn ctor_result(
    name: &str,
    domain: &SealedDomainId,
    args: Vec<AssociatedFamilyResultExpr>,
) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::DomainConstructorApp {
        constructor: DomainConstructorId::new(domain.clone(), name),
        domain: domain.clone(),
        args,
        kind: Kind::Type,
        constraint: constraint(domain),
        source_anchor: anchor(name),
    }
}

fn recursive_call(
    head: &AssociatedFamilyHeadId,
    arg: AssociatedFamilyResultExpr,
    ys: AssociatedFamilyResultExpr,
) -> AssociatedFamilyResultExpr {
    AssociatedFamilyResultExpr::AssociatedFamilyProjection {
        head: head.clone(),
        interface_args: vec![arg, ys],
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        rigidity: ProjectionRigidity::Rigid,
        source_anchor: anchor("recursive call"),
    }
}

fn append_scheme(
    head: &AssociatedFamilyHeadId,
    module: &ModuleIdentity,
    domain: &SealedDomainId,
) -> AssociatedFamilyScheme {
    let equations = vec![
        AssociatedFamilyEquation {
            head: head.clone(),
            ordinal: 0,
            interface_arg_patterns: vec![ctor_pat("Nil", domain, vec![]), var_pat("Ys", domain)],
            result: var_result("Ys", domain),
            decreases: None,
            source_anchor: anchor("append-nil"),
            case_head_anchor: anchor("append-nil-case"),
        },
        AssociatedFamilyEquation {
            head: head.clone(),
            ordinal: 1,
            interface_arg_patterns: vec![
                ctor_pat(
                    "Cons",
                    domain,
                    vec![kind_var_pat("Head"), var_pat("Tail", domain)],
                ),
                var_pat("Ys", domain),
            ],
            result: ctor_result(
                "Cons",
                domain,
                vec![
                    kind_var_result("Head"),
                    recursive_call(head, var_result("Tail", domain), var_result("Ys", domain)),
                ],
            ),
            decreases: None,
            source_anchor: anchor("append-cons"),
            case_head_anchor: anchor("append-cons-case"),
        },
    ];
    AssociatedFamilyScheme {
        head: head.clone(),
        params: vec![param_meta("Xs", domain), param_meta("Ys", domain)],
        result_domain: CanonicalTypeExpr::Var("TypeList".into()),
        result_kind: Kind::Type,
        equations,
        source_anchor: anchor(&format!("append-scheme-{}", module.module_id.0)),
    }
}

fn env_with_append_family() -> (
    TypeEnv,
    ModuleIdentity,
    SealedDomainId,
    AssociatedFamilyHeadId,
) {
    let owner = module("append_owner", 2);
    let domain = SealedDomainId::new(owner.clone(), "TypeList");
    let mut env = TypeEnv::new();
    register_type_list_domain(&mut env, &owner);
    env.set_current_module_identity(owner.clone());
    env.register_interface(&append_interface())
        .expect("AppendFamily declaration registers");
    let head = family_head(&env, "AppendFamily", "Out");
    env.register_associated_family_scheme(append_scheme(&head, &owner, &domain), owner.clone())
        .expect("append recursive family scheme registers");
    (env, owner, domain, head)
}

fn make_nil_head(owner: &ModuleIdentity) -> TypeComputationHeadId {
    TypeComputationHeadId::new(owner.clone(), "MakeNil")
}

fn make_cons_head(owner: &ModuleIdentity) -> TypeComputationHeadId {
    TypeComputationHeadId::new(owner.clone(), "MakeCons")
}

fn make_list_registry(owner: &ModuleIdentity, domain: &SealedDomainId) -> FixtureEquationRegistry {
    let nil_head = make_nil_head(owner);
    let cons_head = make_cons_head(owner);
    FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(
                nil_head,
                vec![],
                FixtureResultExpr::DomainConstructor {
                    constructor: DomainConstructorId::new(domain.clone(), "Nil"),
                    domain: domain.clone(),
                    args: vec![],
                    kind: Kind::Type,
                },
            )
            .expect("nil fixture equation validates"),
        )
        .expect("nil fixture registers")
        .with_equation(
            FixtureEquation::new(
                cons_head,
                vec![
                    ash_typeck::normalizer::FixturePattern::Var("Head".into()),
                    ash_typeck::normalizer::FixturePattern::Var("Tail".into()),
                ],
                FixtureResultExpr::DomainConstructor {
                    constructor: DomainConstructorId::new(domain.clone(), "Cons"),
                    domain: domain.clone(),
                    args: vec![
                        FixtureResultExpr::BoundVar("Head".into()),
                        FixtureResultExpr::BoundVar("Tail".into()),
                    ],
                    kind: Kind::Type,
                },
            )
            .expect("cons fixture equation validates"),
        )
        .expect("cons fixture registers")
}

fn nil_expr(owner: &ModuleIdentity) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: make_nil_head(owner),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_expr(
    owner: &ModuleIdentity,
    head: CanonicalTypeExpr,
    tail: CanonicalTypeExpr,
) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: make_cons_head(owner),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn nil_normal(domain: &SealedDomainId) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: DomainConstructorId::new(domain.clone(), "Nil"),
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_normal(
    domain: &SealedDomainId,
    head: NormalTypeExpr,
    tail: NormalTypeExpr,
) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: DomainConstructorId::new(domain.clone(), "Cons"),
        domain: domain.clone(),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

#[test]
fn task_866_reduces_recursive_append_family_with_fuel() {
    let (env, owner, domain, head) = env_with_append_family();
    let registry = make_list_registry(&owner, &domain);
    let xs = cons_expr(
        &owner,
        CanonicalTypeExpr::Primitive("String".into()),
        nil_expr(&owner),
    );
    let ys = nil_expr(&owner);
    let expr = projection(&head, vec![xs, ys], ProjectionRigidity::Neutral);

    let outcome = Normalizer::with_registry(&env, &registry)
        .normalize(&expr)
        .expect("recursive append family should normalize under sufficient fuel");

    assert_eq!(
        outcome.normal,
        cons_normal(
            &domain,
            NormalTypeExpr::Primitive("String".into()),
            nil_normal(&domain)
        )
    );
}

#[test]
fn task_866_recursive_family_reduction_respects_fuel() {
    let (env, owner, domain, head) = env_with_append_family();
    let registry = make_list_registry(&owner, &domain);
    let expr = projection(
        &head,
        vec![
            cons_expr(&owner, CanonicalTypeExpr::Var("A".into()), nil_expr(&owner)),
            nil_expr(&owner),
        ],
        ProjectionRigidity::Neutral,
    );
    let config = NormalizationConfig {
        fuel: NormalizationFuel::new(2),
        ..NormalizationConfig::default()
    };

    let err = Normalizer::with_config_and_registry(&env, config, &registry)
        .normalize(&expr)
        .expect_err("recursive family reduction must consume fuel");

    assert!(matches!(err, NormalizationError::FuelExhausted { .. }));
}

#[test]
fn task_866_open_projection_input_is_preserved_without_inversion() {
    let env = env_with_iterator_family();
    let head = family_head(&env, "Iterator", "Item");
    let expr = projection(
        &head,
        vec![CanonicalTypeExpr::Var("I".into())],
        ProjectionRigidity::Neutral,
    );

    let outcome = Normalizer::new(&env)
        .normalize(&expr)
        .expect("open projection should normalize to a blocker");

    match outcome.normal {
        NormalTypeExpr::Projection { reason, args, .. } => {
            assert_eq!(reason, Some(NormalFormBlockReason::AbstractScrutinee));
            assert_eq!(args, vec![NormalTypeExpr::Var("I".into())]);
        }
        other => panic!("expected preserved projection, got {other:?}"),
    }

    let equality = Normalizer::new(&env)
        .definitional_equality(&expr, &CanonicalTypeExpr::Primitive("String".into()))
        .expect("definitional equality should report semantic evidence");
    assert!(matches!(
        equality,
        DefinitionalEqualityResult::BlockedByNeutrality { .. }
    ));
}

#[test]
fn task_866_rigid_generic_projection_remains_rigid_and_unselected() {
    let env = env_with_iterator_family();
    let head = family_head(&env, "Iterator", "Item");
    let expr = projection(
        &head,
        vec![CanonicalTypeExpr::Var("T".into())],
        ProjectionRigidity::Rigid,
    );

    let outcome = Normalizer::new(&env)
        .normalize(&expr)
        .expect("rigid generic projection should be preserved");

    match outcome.normal {
        NormalTypeExpr::Projection {
            rigidity, reason, ..
        } => {
            assert_eq!(rigidity, ProjectionRigidity::Rigid);
            assert_eq!(reason, Some(NormalFormBlockReason::RigidProjection));
        }
        other => panic!("expected rigid projection, got {other:?}"),
    }
}

fn ordinary_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Plain".into(),
        type_params: vec![param("Self", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: AssociatedTypeKind::Ordinary,
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

#[test]
fn task_866_ordinary_associated_type_projection_reports_not_sealed() {
    let owner = module("ordinary_owner", 3);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner.clone());
    env.register_interface(&ordinary_interface_def())
        .expect("ordinary interface registers");
    let interface = InterfaceIdentityId::new(owner, "Plain");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Plain".into(), "Item".into()],
    );
    let expr = CanonicalTypeExpr::Projection {
        interface,
        member,
        args: vec![CanonicalTypeExpr::Primitive("String".into())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Neutral,
    };

    let outcome = Normalizer::new(&env)
        .normalize(&expr)
        .expect("ordinary associated projection should be preserved");

    match outcome.normal {
        NormalTypeExpr::Projection { reason, .. } => {
            assert_eq!(
                reason,
                Some(NormalFormBlockReason::AssociatedFamilyNotSealed)
            );
        }
        other => panic!("expected preserved ordinary projection, got {other:?}"),
    }
}

#[test]
fn task_866_imported_family_projection_reduces_after_v4_import_task() {
    let mut env = env_with_iterator_family();
    let head = family_head(&env, "Iterator", "Item");
    env.set_current_module_identity(module("downstream_importer", 4));
    let expr = projection(
        &head,
        vec![canonical(&env, list_ty(name_ty("String")))],
        ProjectionRigidity::Neutral,
    );

    let outcome = Normalizer::new(&env)
        .normalize(&expr)
        .expect("validated associated family should reduce across module boundaries");

    assert_eq!(
        outcome.evidence,
        NormalizationEvidence::AssociatedFamilyProjectionReduced
    );
    assert_eq!(outcome.normal, NormalTypeExpr::Primitive("String".into()));
}
