//! TASK-881: structured proposition diagnostics.

use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary,
    ModuleSourceOrigin, PropositionFactSummary, PropositionPredicateId,
    PropositionPredicateParamSummary, PropositionPredicateSummary, SourceAnchor, SourceOrigin,
    SummaryVersion,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, NamedPredicateProposition, ProjectionRigidity,
    TypeComputationHeadId, TypeDisequalityProposition, TypeEqualityProposition, TypeProposition,
    TypePropositionTerm,
};
use ash_diagnostic::{AshLspError, Severity};
use ash_parser::surface::{
    PropositionPredicateDecl, PropositionPredicateParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::error::{PropositionDiagnosticKind, TypeEnvError};
use ash_typeck::type_env::{PropositionCheckingSite, PropositionCheckingSiteKind};
use ash_typeck::{Kind, TypeEnv};

fn span() -> Span {
    Span::new(10, 20, 2, 4)
}

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(881)),
        ModuleId(id),
        vec![format!("task_881_{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-881-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-881-typeck-test".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
}

fn primitive(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive(name.into()))
}

fn canonical(expr: CanonicalTypeExpr) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(expr)
}

fn computation_head_app(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: TypeComputationHeadId::new(module_identity(900), name),
        args: vec![CanonicalTypeExpr::Primitive("Int".into())],
        kind: Kind::Type,
    }
}

fn rigid_projection(name: &str) -> CanonicalTypeExpr {
    let interface = InterfaceIdentityId::new(module_identity(901), "RigidInterface");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        name,
        vec!["RigidInterface".into(), name.into()],
    );
    CanonicalTypeExpr::Projection {
        interface,
        member,
        args: vec![CanonicalTypeExpr::Primitive("Int".into())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    }
}

fn proposition_var(name: &str) -> TypePropositionTerm {
    canonical(CanonicalTypeExpr::Var(name.into()))
}

fn assert_required_discharge_code(
    proposition: TypeProposition,
    expected_code: &str,
    expected_message: &str,
) -> String {
    let mut env = TypeEnv::with_builtin_types();
    env.add_proposition_obligation(
        proposition,
        anchor("task-881 live required proposition"),
        site("pub fn diagnostic path"),
    );
    let err = env
        .discharge_required_proposition_obligations()
        .expect_err("required proposition should emit a structured diagnostic");

    assert_eq!(err.code().expect("stable code").0, expected_code);
    let message = err.to_string();
    assert!(
        message.contains(expected_message),
        "expected diagnostic message to contain `{expected_message}`, got `{message}`"
    );
    message
}

fn site(label: &str) -> PropositionCheckingSite {
    PropositionCheckingSite::new(
        881,
        PropositionCheckingSiteKind::ExplicitRequirement,
        Some(label.into()),
    )
}

fn predicate_decl(name: &str) -> PropositionPredicateDecl {
    PropositionPredicateDecl {
        visibility: Visibility::Public,
        name: name.into(),
        params: vec![PropositionPredicateParam {
            name: "T".into(),
            domain: SurfaceType::Name("Int".into()),
            span: span(),
        }],
        span: span(),
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

fn named_fact(module: &ModuleIdentity, name: &str) -> PropositionFactSummary {
    let predicate = PropositionPredicateId::new(module.clone(), name);
    PropositionFactSummary {
        proposition: TypeProposition::NamedPredicate(NamedPredicateProposition {
            predicate: predicate.clone(),
            args: vec![primitive("Int")],
        }),
        role: ash_typeck::type_env::PropositionFactRole::Requirement,
        source_anchor: anchor(&format!("where {name}<Int>")),
        predicate_dependencies: vec![predicate],
        dependency_summary_refs: Vec::new(),
        outcome: None,
    }
}

fn proposition_error(kind: PropositionDiagnosticKind) -> TypeEnvError {
    TypeEnvError::PropositionDiagnostic {
        kind,
        proposition: "Append<Xs, Ys> == Cons<A, Nil>".into(),
        expected: "a proposition that can be discharged by normalized equality evidence".into(),
        found: "deferred proposition requiring type-function inversion".into(),
        solver_rule: "normalize-and-compare without inversion".into(),
        help: "add explicit evidence or use a closed proposition; Ash will not solve inputs from outputs".into(),
        span: span(),
    }
}

#[test]
fn task_881_all_spec064_diagnostic_families_have_stable_codes() {
    let families = [
        (PropositionDiagnosticKind::UnsupportedSurfaceSyntax, "E168"),
        (PropositionDiagnosticKind::UnknownNamedPredicate, "E166"),
        (
            PropositionDiagnosticKind::UnsupportedNamedPredicateSolving,
            "E169",
        ),
        (
            PropositionDiagnosticKind::EqualityBlockedByNeutralHead,
            "E170",
        ),
        (
            PropositionDiagnosticKind::EqualityBlockedByRigidProjection,
            "E171",
        ),
        (PropositionDiagnosticKind::DisequalityOpenOrNeutral, "E172"),
        (
            PropositionDiagnosticKind::DisequalityRefutedByEquality,
            "E173",
        ),
        (PropositionDiagnosticKind::InterfaceBoundNotFound, "E174"),
        (
            PropositionDiagnosticKind::MalformedPropositionSummary,
            "E175",
        ),
        (
            PropositionDiagnosticKind::PrivatePropositionDependencyLeak,
            "E176",
        ),
        (PropositionDiagnosticKind::NoInversionBoundary, "E177"),
    ];

    for (kind, expected_code) in families {
        let err = proposition_error(kind);
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(AshLspError::span(&err), Some(span().into()));
        assert_eq!(err.code().expect("stable code").0, expected_code);
    }
}

#[test]
fn task_881_proposition_diagnostic_message_includes_shape_rule_help_and_no_inversion_note() {
    let err = proposition_error(PropositionDiagnosticKind::NoInversionBoundary);
    let message = err.to_string();

    assert!(message.contains("Append<Xs, Ys> == Cons<A, Nil>"));
    assert!(message.contains("expected"));
    assert!(message.contains("found"));
    assert!(message.contains("normalize-and-compare"));
    assert!(message.contains("next step"));
    assert!(message.contains("will not solve inputs from outputs"));
    assert!(message.contains("did not solve under type functions or associated families"));
}

#[test]
fn task_881_required_discharge_of_unsupported_named_predicate_uses_structured_e169() {
    let module = module_identity(1);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module);
    let predicate = env
        .register_proposition_predicate_decl(&predicate_decl("Opaque"))
        .expect("predicate declaration registers");
    env.add_proposition_obligation(
        TypeProposition::NamedPredicate(NamedPredicateProposition {
            predicate,
            args: vec![primitive("Int")],
        }),
        anchor("where Opaque<Int>"),
        site("pub fn needs_opaque"),
    );

    let err = env
        .discharge_required_proposition_obligations()
        .expect_err("opaque named predicates defer at required checking points");

    assert_eq!(err.code().expect("stable code").0, "E169");
    let message = err.to_string();
    assert!(message.contains("unsupported named predicate solving"));
    assert!(message.contains("expected shape"));
    assert!(message.contains("found shape"));
    assert!(message.contains("next step"));
    assert!(!message.contains("normalized both sides"));
    assert!(!message.contains("did not solve under type functions or associated families"));
    assert!(!message.contains("no_inversion"));
}

#[test]
fn task_881_required_discharge_of_missing_interface_bound_uses_structured_e174() {
    let module = module_identity(2);
    let mut env = TypeEnv::with_builtin_types();
    env.add_proposition_obligation(
        TypeProposition::InterfaceBound(InterfaceBoundProposition {
            subject: primitive("Int"),
            interface: ash_core::semantic_summary::InterfaceIdentityId::new(module, "Display"),
            interface_args: Vec::new(),
        }),
        anchor("where Int: Display"),
        site("pub fn needs_display"),
    );

    let err = env
        .discharge_required_proposition_obligations()
        .expect_err("missing interface evidence defers at required checking points");

    assert_eq!(err.code().expect("stable code").0, "E174");
    let message = err.to_string();
    assert!(message.contains("interface bound not found"));
    assert!(message.contains("interface evidence lookup"));
    assert!(!message.contains("normalized both sides"));
    assert!(!message.contains("did not solve under type functions or associated families"));
    assert!(!message.contains("no_inversion"));
}

#[test]
fn task_881_unknown_named_predicate_uses_structured_e166_message() {
    let module = module_identity(20);
    let env = TypeEnv::with_builtin_types();
    let predicate = PropositionPredicateId::new(module, "MissingPredicate");
    let proposition = TypeProposition::NamedPredicate(NamedPredicateProposition {
        predicate,
        args: vec![primitive("Int")],
    });

    let err = env
        .solve_proposition(&proposition, Some(anchor("MissingPredicate<Int>")))
        .expect_err("unknown named predicate should fail at registry lookup");

    assert_eq!(err.code().expect("stable code").0, "E166");
    let message = err.to_string();
    assert!(message.contains("proposition diagnostic"));
    assert!(message.contains("expected shape"));
    assert!(message.contains("found shape"));
    assert!(message.contains("solver rule"));
    assert!(message.contains("next step"));
}

#[test]
fn task_881_required_equality_on_neutral_computation_head_uses_live_e170() {
    let message = assert_required_discharge_code(
        TypeProposition::Equality(TypeEqualityProposition {
            lhs: canonical(computation_head_app("Append")),
            rhs: primitive("Int"),
        }),
        "E170",
        "equality blocked by neutral computation head",
    );

    assert!(message.contains("normalize-and-compare deferred on neutral head"));
    assert!(message.contains("did not solve under type functions or associated families"));
}

#[test]
fn task_881_required_equality_on_rigid_projection_uses_live_e171() {
    let message = assert_required_discharge_code(
        TypeProposition::Equality(TypeEqualityProposition {
            lhs: canonical(rigid_projection("Item")),
            rhs: primitive("Int"),
        }),
        "E171",
        "equality blocked by rigid associated projection",
    );

    assert!(message.contains("deferred on rigid associated projection"));
    assert!(message.contains("did not solve under type functions or associated families"));
}

#[test]
fn task_881_required_disequality_on_open_var_uses_live_e172() {
    let message = assert_required_discharge_code(
        TypeProposition::Disequality(TypeDisequalityProposition {
            lhs: proposition_var("T"),
            rhs: primitive("Int"),
        }),
        "E172",
        "disequality blocked by open or neutral side",
    );

    assert!(message.contains("unsupported proof search boundary"));
    assert!(!message.contains("normalized both sides"));
    assert!(!message.contains("did not solve under type functions or associated families"));
    assert!(!message.contains("no_inversion"));
}

#[test]
fn task_881_required_neutral_disequality_uses_live_e172_without_no_inversion_note() {
    let message = assert_required_discharge_code(
        TypeProposition::Disequality(TypeDisequalityProposition {
            lhs: canonical(computation_head_app("Append")),
            rhs: primitive("Int"),
        }),
        "E172",
        "disequality blocked by open or neutral side",
    );

    assert!(message.contains("normalize-and-compare deferred on neutral head"));
    assert!(!message.contains("equality blocked by neutral computation head"));
    assert!(!message.contains("normalized both sides"));
    assert!(!message.contains("did not solve under type functions or associated families"));
    assert!(!message.contains("no_inversion"));
}

#[test]
fn task_881_required_disequality_refuted_by_equality_uses_live_e173() {
    let message = assert_required_discharge_code(
        TypeProposition::Disequality(TypeDisequalityProposition {
            lhs: primitive("Int"),
            rhs: primitive("Int"),
        }),
        "E173",
        "disequality refuted because both sides are equal",
    );

    assert!(message.contains("normalize-and-compare disequality refutation"));
}

#[test]
fn task_881_v4_proposition_summary_malformed_uses_structured_e175_fail_closed() {
    let module = module_identity(3);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "ShouldNotRegister",
            CoreVisibility::Public,
        ))
        .with_exported_proposition_fact(named_fact(&module, "ShouldNotRegister"));
    let mut env = TypeEnv::with_builtin_types();

    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("V4 proposition payload must be rejected before partial registration");

    assert_eq!(err.code().expect("stable code").0, "E175");
    assert!(
        env.lookup_proposition_predicate("ShouldNotRegister")
            .is_none()
    );
    assert!(env.proposition_obligations().is_empty());
}

#[test]
fn task_881_private_predicate_leak_uses_structured_e176_fail_closed() {
    let module = module_identity(4);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "HiddenReq",
            CoreVisibility::Private,
        ))
        .with_exported_proposition_fact(named_fact(&module, "HiddenReq"));
    let mut env = TypeEnv::with_builtin_types();

    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("private predicates must not leak in public proposition summaries");

    assert_eq!(err.code().expect("stable code").0, "E176");
    assert!(env.proposition_obligations().is_empty());
    let message = err.to_string();
    assert!(message.contains("private proposition dependency leak"));
    assert!(message.contains("HiddenReq"));
}
