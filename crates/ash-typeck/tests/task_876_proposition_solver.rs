use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, DomainConstructorSummary, DomainFieldSummary,
    InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary,
    SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    PropositionBoundary, PropositionDeferredKind, PropositionEvidenceRule, PropositionOutcome,
    PropositionRefutationReason, TypeComputationHeadId, TypeDisequalityProposition,
    TypeEqualityProposition, TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::Kind;
use ash_typeck::TypeEnv;

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-876-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-876-test".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
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

fn nil_term(domain: &SealedDomainId, nil: &DomainConstructorId) -> TypePropositionTerm {
    TypePropositionTerm::DomainConstructorApp {
        constructor: nil.clone(),
        domain: domain.clone(),
        args: vec![],
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

fn registered_type_list() -> (
    TypeEnv,
    SealedDomainId,
    DomainConstructorId,
    DomainConstructorId,
) {
    let mut env = TypeEnv::with_builtin_types();
    let module = module_identity(876_100, &["pkg", "constraints"]);
    let domain = type_list_domain(&module);
    let domain_id = domain.id.clone();
    let nil_id = domain.constructors[0].id.clone();
    let cons_id = domain.constructors[1].id.clone();
    env.register_local_sealed_domain_summary(&domain)
        .expect("sealed domain fixture should register");
    (env, domain_id, nil_id, cons_id)
}

fn surface_span() -> Span {
    Span::default()
}

fn interface_param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: surface_span(),
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
        span: surface_span(),
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
                span: surface_span(),
            },
            span: surface_span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: surface_span(),
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
            span: surface_span(),
        }],
        methods: vec![],
        span: surface_span(),
    }
}

fn env_with_iterator_family() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(876_103, &["pkg", "iterator"]));
    env.register_interface(&iterator_interface_def())
        .expect("Iterator associated family registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("Iterator<List<A>>::Item family scheme registers");
    env
}

#[test]
fn task_876_equality_satisfied_when_normalized_forms_are_definitionally_equal() {
    let env = TypeEnv::with_builtin_types();
    let proposition = equality(primitive("Int"), primitive("Int"));

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Int == Int")))
        .expect("proposition solver should normalize and compare equality");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(evidence.rule, PropositionEvidenceRule::DefinitionalEquality);
            assert_eq!(evidence.boundary, PropositionBoundary::Local);
            let terms = evidence
                .normalized_terms
                .expect("equality evidence should include normalized terms");
            assert_eq!(terms.lhs, NormalTypeExpr::Primitive("Int".into()));
            assert_eq!(terms.rhs, NormalTypeExpr::Primitive("Int".into()));
            assert!(evidence.source_anchor.is_some());
        }
        other => panic!("expected satisfied equality, got {other:?}"),
    }
}

#[test]
fn task_876_equality_refuted_when_closed_normal_forms_are_not_equal() {
    let env = TypeEnv::with_builtin_types();
    let proposition = equality(primitive("Int"), primitive("String"));

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Int == String")))
        .expect("proposition solver should return refutation evidence");

    match outcome {
        PropositionOutcome::Refuted(refutation) => {
            assert_eq!(refutation.proposition, proposition);
            assert_eq!(
                refutation.reason,
                PropositionRefutationReason::DefinitionalEquality
            );
            let terms = refutation
                .normalized_terms
                .expect("refutation should include normalized terms");
            assert_eq!(terms.lhs, NormalTypeExpr::Primitive("Int".into()));
            assert_eq!(terms.rhs, NormalTypeExpr::Primitive("String".into()));
        }
        other => panic!("expected refuted equality, got {other:?}"),
    }
}

#[test]
fn task_876_equality_deferred_at_neutral_no_inversion_boundary_without_solving_inputs() {
    let (env, domain, nil, cons) = registered_type_list();
    let module = module_identity(876_101, &["pkg", "constraints"]);
    let rhs = cons_term(&domain, &cons, var("A"), nil_term(&domain, &nil));
    let proposition = equality(append_term(&module, "Xs", "Ys"), rhs);

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Append<Xs,Ys> == Cons<A,Nil>")))
        .expect("neutral equality should defer without inversion");

    match outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert!(reason.no_inversion_boundary);
            assert!(matches!(
                reason.kind,
                PropositionDeferredKind::BlockedByNeutrality {
                    blocker: NormalFormBlockReason::Unsupported
                }
            ));
            match &reason.proposition {
                TypeProposition::Equality(eq) => match &eq.lhs {
                    TypePropositionTerm::Canonical(CanonicalTypeExpr::ComputationHeadApp {
                        args,
                        ..
                    }) => {
                        assert_eq!(args, &vec![canonical_var("Xs"), canonical_var("Ys")]);
                    }
                    other => panic!(
                        "expected original neutral Append lhs to be preserved, got {other:?}"
                    ),
                },
                other => panic!("expected equality proposition, got {other:?}"),
            }
        }
        other => panic!("expected deferred equality, got {other:?}"),
    }
}

#[test]
fn task_876_equality_deferred_for_rigid_associated_projection() {
    let env = TypeEnv::with_builtin_types();
    let module = module_identity(876_102, &["pkg", "iter"]);
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
        .solve_proposition(&proposition, Some(anchor("T::Item == A")))
        .expect("rigid projection equality should defer");

    match outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert!(reason.no_inversion_boundary);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::RigidAssociatedProjection
            );
        }
        other => panic!("expected deferred rigid projection equality, got {other:?}"),
    }
}

#[test]
fn task_876_equality_satisfied_after_associated_family_projection_normalization() {
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
            Some(anchor("Iterator<List<String>>::Item == String")),
        )
        .expect("associated-family projection equality should normalize and solve");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(evidence.rule, PropositionEvidenceRule::DefinitionalEquality);
            let terms = evidence
                .normalized_terms
                .expect("projection equality evidence should include normalized terms");
            assert_eq!(terms.lhs, NormalTypeExpr::Primitive("String".into()));
            assert_eq!(terms.rhs, NormalTypeExpr::Primitive("String".into()));
        }
        other => panic!("expected satisfied associated-family equality, got {other:?}"),
    }
}

#[test]
fn task_876_disequality_satisfied_for_sealed_domain_constructor_head_disjointness_with_open_args() {
    let (env, domain, nil, cons) = registered_type_list();
    let lhs = cons_term(&domain, &cons, var("A"), var("Tail"));
    let rhs = nil_term(&domain, &nil);
    let proposition = disequality(lhs, rhs);

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Cons<A,Tail> != Nil")))
        .expect("sealed-domain constructor disjointness should solve disequality");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::SealedDomainConstructorDisjointness
            );
            let terms = evidence
                .normalized_terms
                .expect("disequality evidence should include normalized terms");
            match (terms.lhs, terms.rhs) {
                (
                    NormalTypeExpr::DomainConstructorApp {
                        constructor: lhs_constructor,
                        domain: lhs_domain,
                        args,
                        ..
                    },
                    NormalTypeExpr::DomainConstructorApp {
                        constructor: rhs_constructor,
                        domain: rhs_domain,
                        args: rhs_args,
                        ..
                    },
                ) => {
                    assert_eq!(lhs_constructor, cons);
                    assert_eq!(rhs_constructor, nil);
                    assert_eq!(lhs_domain, domain);
                    assert_eq!(rhs_domain, domain);
                    assert!(matches!(args.first(), Some(NormalTypeExpr::Var(name)) if name == "A"));
                    assert!(rhs_args.is_empty());
                }
                other => panic!("expected constructor normal forms, got {other:?}"),
            }
        }
        other => panic!("expected satisfied disequality, got {other:?}"),
    }
}

#[test]
fn task_876_disequality_refuted_when_both_sides_normalize_equal() {
    let (env, domain, nil, cons) = registered_type_list();
    let lhs = cons_term(&domain, &cons, primitive("Int"), nil_term(&domain, &nil));
    let rhs = cons_term(&domain, &cons, primitive("Int"), nil_term(&domain, &nil));
    let proposition = disequality(lhs, rhs);

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Cons<Int,Nil> != Cons<Int,Nil>")))
        .expect("equal normal forms should refute disequality");

    match outcome {
        PropositionOutcome::Refuted(refutation) => {
            assert_eq!(refutation.proposition, proposition);
            assert_eq!(
                refutation.reason,
                PropositionRefutationReason::DefinitionalEquality
            );
            assert!(refutation.normalized_terms.is_some());
        }
        other => panic!("expected refuted disequality, got {other:?}"),
    }
}

#[test]
fn task_876_disequality_deferred_for_neutral_and_open_cases() {
    let (env, domain, nil, cons) = registered_type_list();
    let module = module_identity(876_103, &["pkg", "constraints"]);
    let neutral_proposition = disequality(
        append_term(&module, "Xs", "Ys"),
        cons_term(&domain, &cons, var("A"), nil_term(&domain, &nil)),
    );
    let open_proposition = disequality(var("A"), primitive("Int"));

    let neutral_outcome = env
        .solve_proposition(
            &neutral_proposition,
            Some(anchor("Append<Xs,Ys> != Cons<A,Nil>")),
        )
        .expect("neutral disequality should defer");
    let open_outcome = env
        .solve_proposition(&open_proposition, Some(anchor("A != Int")))
        .expect("open disequality should defer");

    match neutral_outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, neutral_proposition);
            assert!(reason.no_inversion_boundary);
            assert!(matches!(
                reason.kind,
                PropositionDeferredKind::BlockedByNeutrality { .. }
            ));
        }
        other => panic!("expected neutral disequality to defer, got {other:?}"),
    }
    match open_outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, open_proposition);
            assert!(reason.no_inversion_boundary);
            assert_eq!(reason.kind, PropositionDeferredKind::UnsupportedProofSearch);
        }
        other => panic!("expected open disequality to defer, got {other:?}"),
    }
}

#[test]
fn task_876_solving_obligations_records_outcomes_without_creating_unification_evidence() {
    let (mut env, domain, nil, cons) = registered_type_list();
    let module = module_identity(876_104, &["pkg", "constraints"]);
    let proposition = equality(
        append_term(&module, "Xs", "Ys"),
        cons_term(&domain, &cons, var("A"), nil_term(&domain, &nil)),
    );
    env.add_proposition_obligation(
        proposition.clone(),
        anchor("Append<Xs,Ys> == Cons<A,Nil> obligation"),
        ash_typeck::type_env::PropositionCheckingSite::new(
            876_104,
            ash_typeck::type_env::PropositionCheckingSiteKind::ExplicitRequirement,
            Some("task-876 no inversion".into()),
        ),
    );

    let outcomes = env
        .solve_proposition_obligations()
        .expect("obligation solving should produce conservative outcomes");

    assert_eq!(outcomes.len(), 1);
    assert!(matches!(outcomes[0], PropositionOutcome::Deferred(_)));
    assert_eq!(env.proposition_obligations().len(), 1);
    assert!(matches!(
        env.proposition_obligations()[0].outcome,
        Some(PropositionOutcome::Deferred(_))
    ));
    assert!(
        env.proposition_assumptions().is_empty(),
        "solver must not create legacy unification/substitution/meta evidence facts"
    );
    assert_eq!(env.proposition_obligations()[0].proposition, proposition);
}
