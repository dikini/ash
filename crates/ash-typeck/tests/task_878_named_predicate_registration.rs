use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, PropositionPredicateId, PropositionPredicateParamSummary,
    PropositionPredicateSummary, SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NamedPredicateProposition, PropositionDeferredKind, PropositionEvidenceRule,
    PropositionOutcome, TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{
    PropositionClause, PropositionClauseKind, PropositionPredicateDecl, PropositionPredicateParam,
    PropositionTail, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::error::TypeEnvError;
use ash_typeck::solver::TypeError;
use ash_typeck::type_env::{PropositionCheckingSite, PropositionCheckingSiteKind};
use ash_typeck::{Kind, TypeEnv};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-878-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-878-test".into(),
    }
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end, 1, start + 1)
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
}

fn anchor_with_span(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(origin(), Some(ash_core::ast::Span { start, end }), label)
}

fn param(name: &str, domain: SurfaceType, start: usize, end: usize) -> PropositionPredicateParam {
    PropositionPredicateParam {
        name: name.into(),
        domain,
        span: span(start, end),
    }
}

fn proposition_decl(name: &str, visibility: Visibility) -> PropositionPredicateDecl {
    PropositionPredicateDecl {
        visibility,
        name: name.into(),
        params: vec![param("T", SurfaceType::Name("Int".into()), 12, 18)],
        span: span(0, 20),
    }
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
            source_anchor: anchor(&format!("{name}<T> param")),
        }],
        source_anchor: anchor(&format!("prop {name}<T: Int>")),
    }
}

fn predicate_tail(name: &str, name_start: usize, args: Vec<SurfaceType>) -> PropositionTail {
    PropositionTail {
        where_span: span(0, 5),
        span: span(0, 40),
        clauses: vec![PropositionClause {
            span: span(name_start, 40),
            kind: PropositionClauseKind::NamedPredicate {
                name: name.into(),
                name_span: span(name_start, name_start + name.len()),
                args,
            },
        }],
    }
}

fn named(predicate: PropositionPredicateId, arg: TypePropositionTerm) -> TypeProposition {
    TypeProposition::NamedPredicate(NamedPredicateProposition {
        predicate,
        args: vec![arg],
    })
}

fn named_args(
    predicate: PropositionPredicateId,
    args: Vec<TypePropositionTerm>,
) -> TypeProposition {
    TypeProposition::NamedPredicate(NamedPredicateProposition { predicate, args })
}

fn primitive(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive(name.into()))
}

#[test]
fn task_878_registers_predicate_identity_parameter_domain_visibility_and_source_anchor() {
    let module = module_identity(878_001, &["pkg", "predicates"]);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module.clone());

    let id = env
        .register_proposition_predicate_decl(&proposition_decl("NonEmpty", Visibility::Public))
        .expect("predicate declaration should register");

    assert_eq!(id, PropositionPredicateId::new(module, "NonEmpty"));
    let registered = env
        .lookup_proposition_predicate("NonEmpty")
        .expect("source-visible predicate should be registered");
    assert_eq!(registered.summary.id, id);
    assert_eq!(registered.summary.exported_name.as_str(), "NonEmpty");
    assert_eq!(registered.summary.visibility, CoreVisibility::Public);
    assert_eq!(registered.summary.params.len(), 1);
    assert_eq!(registered.summary.params[0].name.as_str(), "T");
    assert_eq!(
        registered.summary.params[0].ty,
        CanonicalTypeExpr::Primitive("Int".into())
    );
    assert_eq!(registered.summary.params[0].kind, Kind::Type);
    assert!(registered.summary.params[0].source_anchor.span.is_some());
    assert!(registered.summary.source_anchor.span.is_some());
}

#[test]
fn task_878_lowers_known_predicate_use_to_canonical_named_predicate_and_defers_solving() {
    let module = module_identity(878_002, &["pkg", "predicates"]);
    let mut env = TypeEnv::with_builtin_types();
    env.register_proposition_predicate_summary(&predicate_summary(
        &module,
        "Normalized",
        CoreVisibility::Public,
    ))
    .expect("known named predicate summary should register");

    let lowered = env
        .lower_proposition_tail(
            &predicate_tail("Normalized", 6, vec![SurfaceType::Name("Int".into())]),
            origin(),
        )
        .expect("known named predicate use should lower");

    assert_eq!(lowered.len(), 1);
    match (&lowered[0].proposition, &lowered[0].outcome) {
        (TypeProposition::NamedPredicate(named), Some(PropositionOutcome::Deferred(reason))) => {
            assert_eq!(
                named.predicate,
                PropositionPredicateId::new(module, "Normalized")
            );
            assert_eq!(named.args, vec![primitive("Int")]);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::UnsupportedNamedPredicate
            );
            assert_eq!(reason.proposition, lowered[0].proposition);
            assert!(reason.source_anchor.is_some());
            assert!(reason.no_inversion_boundary);
        }
        other => {
            panic!("expected canonical named predicate plus unsupported deferral, got {other:?}")
        }
    }
}

#[test]
fn task_878_rejects_unknown_predicate_with_name_span_diagnostic_distinct_from_deferred_known() {
    let env = TypeEnv::with_builtin_types();
    let err = env
        .lower_proposition_tail(
            &predicate_tail(
                "MissingPredicate",
                11,
                vec![SurfaceType::Name("Int".into())],
            ),
            origin(),
        )
        .expect_err("unknown named predicate use must be rejected");

    match err {
        TypeError::TypeEnv(err) => match *err {
            TypeEnvError::UnknownPropositionPredicate {
                name,
                span: diagnostic_span,
            } => {
                assert_eq!(name, "MissingPredicate");
                assert_eq!(diagnostic_span, span(11, 27));
            }
            other => panic!("expected unknown predicate diagnostic, got {other:?}"),
        },
        other => panic!("expected TypeEnv unknown predicate diagnostic, got {other:?}"),
    }
}

#[test]
fn task_878_only_explicitly_registered_builtin_predicates_are_satisfied() {
    let module = module_identity(878_003, &["pkg", "builtins"]);
    let mut env = TypeEnv::with_builtin_types();
    let summary = predicate_summary(&module, "CompilerKnown", CoreVisibility::Public);
    let predicate = summary.id.clone();
    env.register_builtin_proposition_predicate_summary(&summary)
        .expect("compiler-known builtin predicate must be explicitly registered");
    let proposition = named(predicate, primitive("Int"));

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("CompilerKnown<Int>")))
        .expect("registered builtin named predicate should solve");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::NamedPredicateAssumption
            );
            assert!(evidence.source_anchor.is_some());
        }
        other => panic!("expected registered builtin predicate to be satisfied, got {other:?}"),
    }
}

#[test]
fn task_878_direct_unknown_named_predicate_errors_instead_of_deferring() {
    let module = module_identity(878_004, &["pkg", "unknown"]);
    let env = TypeEnv::with_builtin_types();
    let predicate = PropositionPredicateId::new(module, "NotRegistered");
    let proposition = named(predicate, primitive("Int"));
    let source_anchor = anchor_with_span("NotRegistered<Int>", 33, 52);

    let err = env
        .solve_proposition(&proposition, Some(source_anchor))
        .expect_err("direct unknown named predicate must error, not defer");

    match err {
        TypeError::TypeEnv(err) => match *err {
            TypeEnvError::UnknownPropositionPredicate { name, span } => {
                assert_eq!(name, "NotRegistered");
                assert_eq!(span, Span::new(33, 52, 0, 0));
            }
            other => panic!("expected unknown predicate diagnostic, got {other:?}"),
        },
        other => panic!("expected TypeEnv unknown predicate diagnostic, got {other:?}"),
    }
}

#[test]
fn task_878_direct_builtin_named_predicate_wrong_arity_errors_instead_of_satisfying() {
    let module = module_identity(878_005, &["pkg", "builtins"]);
    let mut env = TypeEnv::with_builtin_types();
    let summary = predicate_summary(&module, "CompilerKnownArity", CoreVisibility::Public);
    let predicate = summary.id.clone();
    env.register_builtin_proposition_predicate_summary(&summary)
        .expect("compiler-known builtin predicate must be explicitly registered");
    let proposition = named_args(predicate, vec![]);
    let source_anchor = anchor_with_span("CompilerKnownArity<> malformed", 61, 82);

    let err = env
        .solve_proposition(&proposition, Some(source_anchor))
        .expect_err("registered builtin named predicate with wrong arity must error");

    match err {
        TypeError::TypeEnv(err) => match *err {
            TypeEnvError::PropositionPredicateArityMismatch {
                name,
                expected,
                actual,
                span,
            } => {
                assert_eq!(name, "CompilerKnownArity");
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
                assert_eq!(span, Span::new(61, 82, 0, 0));
            }
            other => panic!("expected arity mismatch diagnostic, got {other:?}"),
        },
        other => panic!("expected TypeEnv arity mismatch diagnostic, got {other:?}"),
    }
}

#[test]
fn task_878_does_not_use_arbitrary_named_predicate_proof_search_or_assumptions() {
    let module = module_identity(878_006, &["pkg", "opaque"]);
    let mut env = TypeEnv::with_builtin_types();
    let summary = predicate_summary(&module, "Opaque", CoreVisibility::Public);
    let predicate = summary.id.clone();
    env.register_proposition_predicate_summary(&summary)
        .expect("ordinary named predicate summary should register");
    let proposition = named(predicate, primitive("Int"));
    env.add_proposition_obligation(
        proposition.clone(),
        anchor("Opaque<Int> obligation"),
        PropositionCheckingSite::new(
            878_004,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("task-878 ordinary named predicate".into()),
        ),
    );

    let outcomes = env
        .solve_proposition_obligations()
        .expect("ordinary named predicates should produce deferred outcomes");

    assert_eq!(outcomes.len(), 1);
    match &outcomes[0] {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::UnsupportedNamedPredicate
            );
            assert!(reason.no_inversion_boundary);
        }
        other => panic!("expected unsupported ordinary named predicate to defer, got {other:?}"),
    }
}
