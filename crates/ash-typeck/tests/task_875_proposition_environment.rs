use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSourceOrigin, SealedDomainId,
    SealedDomainSummary, SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, PropositionDeferredKind, PropositionOutcome, TypeProposition,
    TypePropositionTerm,
};
use ash_parser::surface::{
    ImplDef, InterfaceDef, PropositionClause, PropositionClauseKind, PropositionPredicateDecl,
    PropositionPredicateParam, PropositionTail, Type, Visibility, WhereBound,
};
use ash_parser::token::Span;
use ash_typeck::type_env::{
    PropositionCheckingSite, PropositionCheckingSiteKind, PropositionFactRole,
};
use ash_typeck::{TypeEnv, TypeVar};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-875-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-875-test".into(),
    }
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end, 1, start + 1)
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
}

fn register_interface(env: &mut TypeEnv, name: &str) -> InterfaceIdentityId {
    let module = module_identity(8751, &["pkg", "constraints"]);
    let id = InterfaceIdentityId::new(module, name);
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        id.clone(),
        name,
        vec![name.into()],
        anchor(&format!("interface {name}")),
    ))
    .expect("interface identity should register for proposition lowering");
    id
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
        vec![DomainFieldSummary::unconstrained("head")],
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

fn canonical_var(term: &TypePropositionTerm) -> &str {
    match term {
        TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(name)) => name,
        other => panic!("expected canonical variable term, got {other:?}"),
    }
}

fn canonical_primitive(term: &TypePropositionTerm) -> &str {
    match term {
        TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive(name)) => name,
        other => panic!("expected canonical primitive term, got {other:?}"),
    }
}

#[test]
fn task_875_lowers_all_surface_proposition_clause_families_to_typed_core_carriers() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(8750, &["pkg", "predicates"]));
    env.register_proposition_predicate_decl(&PropositionPredicateDecl {
        visibility: Visibility::Inherited,
        name: "Normalized".into(),
        params: vec![PropositionPredicateParam {
            name: "T".into(),
            domain: Type::Name("Int".into()),
            span: span(57, 67),
        }],
        span: span(57, 79),
    })
    .expect("named predicate must be registered before proposition lowering");
    let interface_id = register_interface(&mut env, "Serializable");

    let tail = PropositionTail {
        where_span: span(0, 5),
        span: span(0, 80),
        clauses: vec![
            PropositionClause {
                span: span(6, 16),
                kind: PropositionClauseKind::Equality {
                    lhs: Type::Name("T".into()),
                    rhs: Type::Name("Int".into()),
                    op_span: span(8, 10),
                },
            },
            PropositionClause {
                span: span(18, 30),
                kind: PropositionClauseKind::Disequality {
                    lhs: Type::Name("T".into()),
                    rhs: Type::Name("String".into()),
                    op_span: span(20, 22),
                },
            },
            PropositionClause {
                span: span(32, 55),
                kind: PropositionClauseKind::InterfaceBound {
                    subject: Type::Name("T".into()),
                    interface: Type::Constructor {
                        name: "Serializable".into(),
                        args: vec![Type::Name("Format".into())],
                    },
                    colon_span: span(33, 34),
                },
            },
            PropositionClause {
                span: span(57, 79),
                kind: PropositionClauseKind::NamedPredicate {
                    name: "Normalized".into(),
                    name_span: span(57, 67),
                    args: vec![Type::Name("T".into())],
                },
            },
        ],
    };

    let lowered = env
        .lower_proposition_tail(&tail, origin())
        .expect("all canonicalizable clauses should lower into typed proposition carriers");

    assert_eq!(lowered.len(), 4);
    match &lowered[0].proposition {
        TypeProposition::Equality(eq) => {
            assert_eq!(canonical_var(&eq.lhs), "T");
            assert_eq!(canonical_primitive(&eq.rhs), "Int");
            assert!(
                lowered[0].outcome.is_none(),
                "TASK-875 must not solve equality"
            );
        }
        other => panic!("expected equality proposition, got {other:?}"),
    }
    match &lowered[1].proposition {
        TypeProposition::Disequality(ne) => {
            assert_eq!(canonical_var(&ne.lhs), "T");
            assert_eq!(canonical_primitive(&ne.rhs), "String");
            assert!(
                lowered[1].outcome.is_none(),
                "TASK-875 must not solve disequality"
            );
        }
        other => panic!("expected disequality proposition, got {other:?}"),
    }
    match &lowered[2].proposition {
        TypeProposition::InterfaceBound(bound) => {
            assert_eq!(canonical_var(&bound.subject), "T");
            assert_eq!(bound.interface, interface_id);
            assert_eq!(bound.interface_args.len(), 1);
            assert_eq!(canonical_var(&bound.interface_args[0]), "Format");
        }
        other => panic!("expected interface-bound proposition, got {other:?}"),
    }
    match (&lowered[3].proposition, &lowered[3].outcome) {
        (TypeProposition::NamedPredicate(named), Some(PropositionOutcome::Deferred(reason))) => {
            assert_eq!(named.predicate.name.as_str(), "Normalized");
            assert_eq!(named.args.len(), 1);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::UnsupportedNamedPredicate
            );
            assert_eq!(reason.proposition, lowered[3].proposition);
            assert!(reason.source_anchor.is_some());
        }
        other => panic!(
            "expected named predicate to lower with a typed unsupported-predicate deferral, got {other:?}"
        ),
    }
}

#[test]
fn task_875_lowers_sealed_domain_constructor_terms_without_nominal_encoding() {
    let mut env = TypeEnv::with_builtin_types();
    let module = module_identity(8752, &["pkg", "domains"]);
    let domain = type_list_domain(&module);
    let nil_id = domain.constructors[0].id.clone();
    let cons_id = domain.constructors[1].id.clone();
    let domain_id = domain.id.clone();
    env.register_local_sealed_domain_summary(&domain)
        .expect("sealed-domain constructors should register");

    let tail = PropositionTail {
        where_span: span(80, 85),
        span: span(80, 110),
        clauses: vec![PropositionClause {
            span: span(86, 110),
            kind: PropositionClauseKind::Disequality {
                lhs: Type::Constructor {
                    name: "Cons".into(),
                    args: vec![Type::Name("Int".into())],
                },
                rhs: Type::Name("Nil".into()),
                op_span: span(96, 98),
            },
        }],
    };

    let lowered = env
        .lower_proposition_tail(&tail, origin())
        .expect("domain constructor proposition terms should lower");
    match &lowered[0].proposition {
        TypeProposition::Disequality(ne) => {
            match &ne.lhs {
                TypePropositionTerm::DomainConstructorApp {
                    constructor,
                    domain,
                    args,
                    ..
                } => {
                    assert_eq!(constructor, &cons_id);
                    assert_eq!(domain, &domain_id);
                    assert_eq!(args.len(), 1);
                    assert_eq!(canonical_primitive(&args[0]), "Int");
                }
                other => panic!("expected Cons to lower as a domain constructor, got {other:?}"),
            }
            match &ne.rhs {
                TypePropositionTerm::DomainConstructorApp {
                    constructor,
                    domain,
                    args,
                    ..
                } => {
                    assert_eq!(constructor, &nil_id);
                    assert_eq!(domain, &domain_id);
                    assert!(args.is_empty());
                }
                other => panic!("expected Nil to lower as a domain constructor, got {other:?}"),
            }
        }
        other => panic!("expected disequality proposition, got {other:?}"),
    }
}

#[test]
fn task_875_generated_obligations_retain_source_anchors_and_owner_sites_without_solving() {
    let mut env = TypeEnv::with_builtin_types();
    let owner = PropositionCheckingSite::new(
        875_200,
        PropositionCheckingSiteKind::ExplicitRequirement,
        Some("check type fn Foo".into()),
    );
    let clause_span = span(11, 25);
    let tail = PropositionTail {
        where_span: span(0, 5),
        span: span(0, 26),
        clauses: vec![PropositionClause {
            span: clause_span,
            kind: PropositionClauseKind::Equality {
                lhs: Type::Name("T".into()),
                rhs: Type::Name("Int".into()),
                op_span: span(13, 15),
            },
        }],
    };

    env.add_proposition_obligations_from_tail(&tail, origin(), owner.clone())
        .expect("canonicalizable where-tail propositions should become obligations");

    assert!(env.proposition_assumptions().is_empty());
    let obligations = env.proposition_obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].owner_site, owner);
    assert_eq!(
        obligations[0]
            .source_anchor
            .span
            .expect("span recorded")
            .start,
        11
    );
    assert_eq!(
        obligations[0]
            .source_anchor
            .span
            .expect("span recorded")
            .end,
        25
    );
    assert!(
        obligations[0].outcome.is_none(),
        "storage/classification must not discharge equality before TASK-876"
    );
}

#[test]
fn task_875_assumptions_are_separate_from_required_obligations_and_include_type_var_bounds() {
    let mut env = TypeEnv::with_builtin_types();
    let interface_id = register_interface(&mut env, "Displayable");
    env.bind_type_var_interface_bound(TypeVar(7), "Displayable");

    let assumptions = env.proposition_assumptions();
    assert_eq!(
        assumptions.len(),
        1,
        "type-var interface bounds become input facts"
    );
    assert!(env.proposition_obligations().is_empty());
    match &assumptions[0].proposition {
        TypeProposition::InterfaceBound(bound) => {
            assert_eq!(bound.interface, interface_id);
            assert_eq!(canonical_var(&bound.subject), "type_var_7");
            assert!(bound.interface_args.is_empty());
        }
        other => panic!("expected interface-bound assumption, got {other:?}"),
    }

    let owner = PropositionCheckingSite::new(
        875_300,
        PropositionCheckingSiteKind::ExplicitRequirement,
        Some("separate required predicate".into()),
    );
    let tail = PropositionTail {
        where_span: span(30, 35),
        span: span(30, 45),
        clauses: vec![PropositionClause {
            span: span(36, 45),
            kind: PropositionClauseKind::Disequality {
                lhs: Type::Name("T".into()),
                rhs: Type::Name("Int".into()),
                op_span: span(38, 40),
            },
        }],
    };
    env.add_proposition_obligations_from_tail(&tail, origin(), owner)
        .expect("adding a required proposition should not mix it with assumptions");

    assert_eq!(env.proposition_assumptions().len(), 1);
    assert_eq!(env.proposition_obligations().len(), 1);
    assert_eq!(
        env.proposition_assumptions()[0].role,
        PropositionFactRole::Assumption
    );
    assert_eq!(
        env.proposition_obligations()[0].role,
        PropositionFactRole::Requirement
    );
}

#[test]
fn task_875_impl_where_bounds_are_preserved_as_proposition_assumptions() {
    let mut env = TypeEnv::with_builtin_types();
    let displayable_id = register_interface(&mut env, "Displayable");
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Displayable".into(),
        type_params: vec!["T".into()],
        associated_types: vec![],
        methods: vec![],
        span: span(0, 11),
    })
    .expect("bound interface should register");
    let container_id = register_interface(&mut env, "Container");
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Container".into(),
        type_params: vec!["T".into()],
        associated_types: vec![],
        methods: vec![],
        span: span(12, 21),
    })
    .expect("generic impl target interface should register");

    assert_eq!(env.proposition_assumptions().len(), 0);

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Container".into(),
        type_params: vec!["T".into()],
        type_args: vec![Type::Name("T".into())],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: "Displayable".into(),
            span: span(22, 36),
        }],
        associated_type_bindings: vec![],
        methods: vec![],
        span: span(22, 50),
    })
    .expect("generic impl with a where-bound should register");

    let assumptions = env.proposition_assumptions();
    assert_eq!(assumptions.len(), 1);
    assert_eq!(assumptions[0].role, PropositionFactRole::Assumption);
    assert_eq!(
        assumptions[0].owner_site.kind,
        PropositionCheckingSiteKind::ImplWhereBound
    );
    assert_eq!(
        assumptions[0]
            .source_anchor
            .span
            .expect("where-bound source span is preserved")
            .start,
        22
    );
    assert_eq!(
        assumptions[0]
            .source_anchor
            .span
            .expect("where-bound source span is preserved")
            .end,
        36
    );
    match &assumptions[0].proposition {
        TypeProposition::InterfaceBound(bound) => {
            assert_eq!(bound.interface, displayable_id);
            assert_ne!(bound.interface, container_id);
            assert!(canonical_var(&bound.subject).starts_with("type_var_"));
            assert!(bound.interface_args.is_empty());
        }
        other => panic!("expected impl where-bound interface assumption, got {other:?}"),
    }
    assert!(env.proposition_obligations().is_empty());
}

#[test]
fn task_875_concrete_impls_are_preserved_as_interface_bound_assumptions() {
    let mut env = TypeEnv::with_builtin_types();
    let displayable_id = register_interface(&mut env, "Displayable");
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Displayable".into(),
        type_params: vec!["T".into()],
        associated_types: vec![],
        methods: vec![],
        span: span(0, 11),
    })
    .expect("interface should register");

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Displayable".into(),
        type_params: vec![],
        type_args: vec![Type::Name("Int".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        span: span(40, 58),
    })
    .expect("concrete impl should register");

    let assumptions = env.proposition_assumptions();
    assert_eq!(assumptions.len(), 1);
    assert_eq!(assumptions[0].role, PropositionFactRole::Assumption);
    assert_eq!(
        assumptions[0].owner_site.kind,
        PropositionCheckingSiteKind::ConcreteImpl
    );
    assert_eq!(
        assumptions[0]
            .source_anchor
            .span
            .expect("impl source span is preserved")
            .start,
        40
    );
    match &assumptions[0].proposition {
        TypeProposition::InterfaceBound(bound) => {
            assert_eq!(bound.interface, displayable_id);
            assert_eq!(canonical_primitive(&bound.subject), "Int");
            assert!(bound.interface_args.is_empty());
        }
        other => panic!("expected concrete impl interface assumption, got {other:?}"),
    }
    assert!(env.proposition_obligations().is_empty());
}
