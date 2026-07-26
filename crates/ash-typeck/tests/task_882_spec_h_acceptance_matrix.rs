//! TASK-882: focused SPEC-064 H1-H8/H11 typeck acceptance matrix smoke tests.

use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, DomainConstructorSummary, DomainFieldSummary,
    InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    PropositionFactSummary, PropositionPredicateId, PropositionPredicateParamSummary,
    PropositionPredicateSummary, SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin,
    SummaryVersion,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, NamedPredicateProposition, NormalTypeExpr,
    ProjectionRigidity, PropositionBoundary, PropositionDeferredKind, PropositionEvidenceRule,
    PropositionOutcome, TypeComputationHeadId, TypeDisequalityProposition, TypeEqualityProposition,
    TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, Definition, ImplDef,
    InterfaceDef, InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};
use ash_typeck::{Kind, TypeEnv};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(882)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-882-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-882-typeck-acceptance".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
}

fn span() -> Span {
    Span::default()
}

fn primitive(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive(name.to_string()))
}

fn var(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(name.to_string()))
}

fn canonical_var(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Var(name.to_string())
}

fn equality(lhs: TypePropositionTerm, rhs: TypePropositionTerm) -> TypeProposition {
    TypeProposition::Equality(TypeEqualityProposition { lhs, rhs })
}

fn disequality(lhs: TypePropositionTerm, rhs: TypePropositionTerm) -> TypeProposition {
    TypeProposition::Disequality(TypeDisequalityProposition { lhs, rhs })
}

fn interface_bound(
    subject: TypePropositionTerm,
    interface: InterfaceIdentityId,
    interface_args: Vec<TypePropositionTerm>,
) -> TypeProposition {
    TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject,
        interface,
        interface_args,
    })
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

fn registered_type_list() -> (
    TypeEnv,
    SealedDomainId,
    DomainConstructorId,
    DomainConstructorId,
) {
    let mut env = TypeEnv::with_builtin_types();
    let module = module_identity(882_100, &["pkg", "typelist"]);
    let domain = type_list_domain(&module);
    let domain_id = domain.id.clone();
    let nil_id = domain.constructors[0].id.clone();
    let cons_id = domain.constructors[1].id.clone();
    env.register_local_sealed_domain_summary(&domain)
        .expect("sealed domain fixture should register");
    (env, domain_id, nil_id, cons_id)
}

fn nil_term(domain: &SealedDomainId, nil: &DomainConstructorId) -> TypePropositionTerm {
    TypePropositionTerm::DomainConstructorApp {
        constructor: nil.clone(),
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_term(
    domain: &SealedDomainId,
    cons: &DomainConstructorId,
    head: TypePropositionTerm,
    tail: TypePropositionTerm,
) -> TypePropositionTerm {
    TypePropositionTerm::DomainConstructorApp {
        constructor: cons.clone(),
        domain: domain.clone(),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn append_term(module: &ModuleIdentity, xs: &str, ys: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::ComputationHeadApp {
        head: TypeComputationHeadId::new(module.clone(), "Append"),
        args: vec![canonical_var(xs), canonical_var(ys)],
        kind: Kind::Type,
    })
}

fn interface_param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: span(),
    }
}

fn surface_name(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn surface_list(item: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "List".into(),
        args: vec![item],
    }
}

fn explicit_iterator_item(arg: SurfaceType) -> SurfaceType {
    SurfaceType::AssociatedFamilyProjection {
        interface: "Iterator".into(),
        args: vec![arg],
        member: "Item".into(),
        span: span(),
    }
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![interface_param("Self")],
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
        type_params: vec![interface_param(param_name)],
        type_args: vec![surface_list(surface_name(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: surface_name(param_name),
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
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(882_101, &["pkg", "iterator"]));
    env.register_interface(&iterator_interface_def())
        .expect("Iterator associated family registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("Iterator<List<A>>::Item family scheme registers");
    env
}

fn display_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Displayable".into(),
        type_params: vec![interface_param("Self")],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn source_append_module() -> ModuleIdentity {
    module_identity(882_102, &["pkg", "append"])
}

fn source_append_domain() -> SealedDomainId {
    SealedDomainId::new(source_append_module(), "TypeList")
}

fn source_append_ctor(name: &str) -> DomainConstructorId {
    DomainConstructorId::new(source_append_domain(), name)
}

fn source_append_summary() -> ModuleSemanticSummary {
    let domain = source_append_domain();
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
    ModuleSemanticSummary::new(source_append_module())
        .with_version(SummaryVersion::SPEC059_SEALED_DOMAIN_V2)
        .with_exported_sealed_domain(
            SealedDomainSummary::new(
                domain,
                "TypeList",
                CoreVisibility::Public,
                anchor("TypeList"),
            )
            .with_constructor(nil)
            .with_constructor(cons),
        )
}

fn source_type_fns() -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    )
    .expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn env_with_source_append() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&source_append_summary())
        .expect("sealed domain registers");
    env.register_local_type_functions(&source_append_module(), &source_type_fns())
        .expect("source type functions validate and publish");
    env
}

fn source_append_head(env: &TypeEnv) -> TypeComputationHeadId {
    env.lookup_local_type_function("Append")
        .expect("type function exists")
        .head
        .clone()
}

fn normal_nil() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: source_append_ctor("Nil"),
        domain: source_append_domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn normal_cons(head: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: source_append_ctor("Cons"),
        domain: source_append_domain(),
        args: vec![head, tail],
        kind: Kind::Type,
    }
}

fn normal_prim(name: &str) -> NormalTypeExpr {
    NormalTypeExpr::Primitive(name.to_string())
}

fn predicate_summary(
    module: &ModuleIdentity,
    name: &str,
    visibility: CoreVisibility,
) -> PropositionPredicateSummary {
    PropositionPredicateSummary {
        id: PropositionPredicateId::new(module.clone(), name),
        exported_name: name.into(),
        visibility,
        params: vec![PropositionPredicateParamSummary {
            name: "T".into(),
            ty: CanonicalTypeExpr::Primitive("Int".into()),
            kind: Kind::Type,
            source_anchor: anchor("predicate param"),
        }],
        source_anchor: anchor("predicate summary"),
    }
}

fn named_fact(module: &ModuleIdentity, name: &str) -> PropositionFactSummary {
    let predicate = PropositionPredicateId::new(module.clone(), name);
    PropositionFactSummary {
        proposition: TypeProposition::NamedPredicate(NamedPredicateProposition {
            predicate: predicate.clone(),
            args: vec![primitive("Int")],
        }),
        role: ash_typeck::type_env::PropositionFactRole::Requirement,
        source_anchor: anchor("where HiddenReq<Int>"),
        predicate_dependencies: vec![predicate],
        dependency_summary_refs: Vec::new(),
        outcome: None,
    }
}

#[test]
fn task_882_h1_constructor_disequality_and_h2_open_append_equality_are_conservative() {
    let (env, domain, nil, cons) = registered_type_list();
    let disjoint = disequality(
        cons_term(&domain, &cons, var("A"), var("Tail")),
        nil_term(&domain, &nil),
    );
    let outcome = env
        .solve_proposition(&disjoint, Some(anchor("H1 Cons<A,T> != Nil")))
        .expect("H1 disequality solver should run");
    assert!(matches!(
        outcome,
        PropositionOutcome::Satisfied(evidence)
            if evidence.rule == PropositionEvidenceRule::SealedDomainConstructorDisjointness
                && evidence.boundary == PropositionBoundary::Local
    ));

    let append_module = module_identity(882_103, &["pkg", "neutral_append"]);
    let no_inversion = equality(
        append_term(&append_module, "Xs", "Ys"),
        cons_term(&domain, &cons, var("A"), nil_term(&domain, &nil)),
    );
    let outcome = env
        .solve_proposition(
            &no_inversion,
            Some(anchor("H2 Append<Xs,Ys> == Cons<A,Nil>")),
        )
        .expect("H2 neutral equality should defer");
    assert!(matches!(
        outcome,
        PropositionOutcome::Deferred(reason)
            if reason.no_inversion_boundary
                && matches!(reason.kind, PropositionDeferredKind::BlockedByNeutrality { .. })
    ));
}

#[test]
fn task_882_h3_named_predicate_defers_and_h11_private_predicate_leak_rejects() {
    let module = module_identity(882_104, &["pkg", "predicates"]);
    let mut env = TypeEnv::with_builtin_types();
    let summary = predicate_summary(&module, "Opaque", CoreVisibility::Public);
    let predicate = summary.id.clone();
    env.register_proposition_predicate_summary(&summary)
        .expect("ordinary named predicate summary registers");
    let proposition = TypeProposition::NamedPredicate(NamedPredicateProposition {
        predicate,
        args: vec![primitive("Int")],
    });
    let outcome = env
        .solve_proposition(&proposition, Some(anchor("H3 Opaque<Int>")))
        .expect("registered ordinary named predicate should defer");
    assert!(matches!(
        outcome,
        PropositionOutcome::Deferred(reason)
            if reason.kind == PropositionDeferredKind::UnsupportedNamedPredicate
                && reason.no_inversion_boundary
    ));

    let hidden = module_identity(882_105, &["pkg", "hidden_predicate"]);
    let summary = ModuleSemanticSummary::new(hidden.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate_summary(
            &hidden,
            "HiddenReq",
            CoreVisibility::Private,
        ))
        .with_exported_proposition_fact(named_fact(&hidden, "HiddenReq"));
    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("H11 private predicate must not leak through public proposition summary");
    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("HiddenReq"),
        "expected private-predicate leak diagnostic, got {msg}"
    );
    assert!(env.proposition_obligations().is_empty());
}

#[test]
fn task_882_h4_direct_type_fn_normalization_satisfies_without_unification_fallback() {
    let env = env_with_source_append();
    let ys = normal_cons(normal_prim("B"), normal_nil());
    let lhs = Normalizer::new(&env)
        .normalize_known_computation_app(
            &source_append_head(&env),
            vec![normal_nil(), ys.clone()],
            &Kind::Type,
        )
        .expect("H4 source Append<Nil, Ys> should normalize");

    assert_eq!(lhs, ys);
    assert_eq!(
        Normalizer::new(&env).definitional_equality_normal_forms(&lhs, &ys),
        DefinitionalEqualityResult::Equal
    );
}

#[test]
fn task_882_h5_associated_family_equality_satisfies_and_h6_rigid_projection_defers() {
    let env = env_with_iterator_family();
    let projected = env
        .lower_surface_type_to_canonical(&explicit_iterator_item(surface_list(surface_name(
            "String",
        ))))
        .expect("Iterator<List<String>>::Item projection lowers");
    let proposition = equality(
        TypePropositionTerm::Canonical(projected),
        primitive("String"),
    );
    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor("H5 Iterator<List<String>>::Item == String")),
        )
        .expect("H5 associated-family equality should solve");
    assert!(matches!(
        outcome,
        PropositionOutcome::Satisfied(evidence)
            if evidence.rule == PropositionEvidenceRule::DefinitionalEquality
    ));

    let module = module_identity(882_106, &["pkg", "rigid"]);
    let interface = InterfaceIdentityId::new(module.clone(), "Iterator");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Item",
        vec!["Iterator".into(), "Item".into()],
    );
    let proposition = equality(
        TypePropositionTerm::Canonical(CanonicalTypeExpr::Projection {
            interface,
            member,
            args: vec![canonical_var("T")],
            kind: Kind::Type,
            rigidity: ProjectionRigidity::Rigid,
        }),
        var("A"),
    );
    let outcome = env
        .solve_proposition(&proposition, Some(anchor("H6 T::Item == A")))
        .expect("H6 rigid projection equality should defer");
    assert!(matches!(
        outcome,
        PropositionOutcome::Deferred(reason)
            if reason.kind == PropositionDeferredKind::RigidAssociatedProjection
                && reason.no_inversion_boundary
    ));
}

#[test]
fn task_882_h7_known_interface_bound_satisfies_and_h8_missing_bound_defers_without_search() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(882_107, &["pkg", "interfaces"]));
    env.register_interface(&display_interface_def())
        .expect("Displayable registers");
    let interface_id = env
        .interface_identity_for_name("Displayable")
        .expect("interface identity registered")
        .clone();
    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Displayable".into(),
        type_params: vec![],
        type_args: vec![surface_name("Int")],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        handlers: Vec::new(),
        derived_handlers: Vec::new(),
        proofs: Vec::new(),
        span: span(),
    })
    .expect("concrete impl evidence should register");

    let known = interface_bound(primitive("Int"), interface_id.clone(), vec![]);
    let outcome = env
        .solve_proposition(&known, Some(anchor("H7 Int: Displayable")))
        .expect("H7 exact concrete impl should satisfy");
    assert!(matches!(
        outcome,
        PropositionOutcome::Satisfied(evidence)
            if evidence.rule == PropositionEvidenceRule::ConcreteImplEvidence
    ));

    let missing = interface_bound(primitive("String"), interface_id, vec![]);
    let outcome = env
        .solve_proposition(&missing, Some(anchor("H8 String: Displayable")))
        .expect("H8 missing evidence should defer without search");
    assert!(matches!(
        outcome,
        PropositionOutcome::Deferred(reason)
            if reason.kind == PropositionDeferredKind::MissingInterfaceEvidence
                && reason.no_inversion_boundary
    ));
}
