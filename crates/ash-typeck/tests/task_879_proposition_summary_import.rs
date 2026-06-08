//! TASK-879: TypeEnv import/export of public proposition summary facts.

use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, PropositionFactSummary, PropositionPredicateId,
    PropositionPredicateParamSummary, PropositionPredicateSummary, SealedDomainId, SourceAnchor,
    SourceOrigin, SummaryVersion, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, NamedPredicateProposition, ProjectionRigidity,
    PropositionDeferredKind, PropositionOutcome, TypeEqualityProposition, TypeProposition,
    TypePropositionTerm,
};
use ash_core::{TypeBody, TypeDef};
use ash_diagnostic::AshLspError;
use ash_parser::surface::{InterfaceDef, Visibility};
use ash_typeck::error::TypeEnvError;
use ash_typeck::type_env::{PropositionCheckingSite, PropositionCheckingSiteKind};
use ash_typeck::{Kind, TypeEnv};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(879)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-879-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-879-typeck-test".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
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

fn named_fact(
    module: &ModuleIdentity,
    name: &str,
    args: Vec<TypePropositionTerm>,
) -> PropositionFactSummary {
    let predicate = PropositionPredicateId::new(module.clone(), name);
    PropositionFactSummary {
        proposition: TypeProposition::NamedPredicate(NamedPredicateProposition {
            predicate: predicate.clone(),
            args,
        }),
        role: ash_typeck::type_env::PropositionFactRole::Requirement,
        source_anchor: anchor(&format!("where {name}<Int>")),
        predicate_dependencies: vec![predicate],
        dependency_summary_refs: Vec::new(),
        outcome: None,
    }
}

fn int_term() -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive("Int".into()))
}

fn nominal_term(module: &ModuleIdentity, name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::NominalApp {
        origin: TypeDeclId::ordinary(module.clone(), name),
        visible_name: name.into(),
        args: Vec::new(),
        kind: Kind::Type,
    })
}

fn equality(lhs: TypePropositionTerm, rhs: TypePropositionTerm) -> TypeProposition {
    TypeProposition::Equality(TypeEqualityProposition { lhs, rhs })
}

#[test]
fn task_879_import_registers_public_predicate_identity_and_revalidates_named_fact() {
    let module = module_identity(1, &["pkg", "predicates"]);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "PublicReq",
            CoreVisibility::Public,
        ))
        .with_exported_proposition_fact(named_fact(&module, "PublicReq", vec![int_term()]));

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary)
        .expect("V5 public proposition summary should import");

    let registered = env
        .lookup_proposition_predicate("PublicReq")
        .expect("public predicate identity should be source-visible after import");
    assert_eq!(
        registered.summary.id,
        PropositionPredicateId::new(module, "PublicReq")
    );
    assert_eq!(env.proposition_obligations().len(), 1);
    assert!(matches!(
        &env.proposition_obligations()[0].outcome,
        Some(PropositionOutcome::Deferred(reason))
            if reason.kind == PropositionDeferredKind::UnsupportedNamedPredicate
    ));
}

#[test]
fn task_879_import_rejects_named_fact_arity_mismatch_before_partial_registration() {
    let module = module_identity(2, &["pkg", "bad_arity"]);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "NeedsOne",
            CoreVisibility::Public,
        ))
        .with_exported_proposition_fact(named_fact(&module, "NeedsOne", Vec::new()));

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("malformed proposition fact arity must reject import atomically");

    assert!(env.lookup_proposition_predicate("NeedsOne").is_none());
    assert!(env.proposition_obligations().is_empty());
    match err {
        TypeEnvError::PropositionPredicateArityMismatch {
            name,
            expected,
            actual,
            ..
        } => {
            assert_eq!(name, "NeedsOne");
            assert_eq!(expected, 1);
            assert_eq!(actual, 0);
        }
        other => panic!("expected proposition predicate arity diagnostic, got {other:?}"),
    }
}

#[test]
fn task_879_import_revalidates_and_overrides_stronger_imported_outcome() {
    let module = module_identity(7, &["pkg", "revalidate_outcome"]);
    let trusted_env = TypeEnv::with_builtin_types();
    let misleading_satisfied = trusted_env
        .solve_proposition(
            &equality(int_term(), int_term()),
            Some(anchor("misleading imported satisfied evidence")),
        )
        .expect("fixture local solver should satisfy Int == Int");
    let proposition = equality(
        int_term(),
        TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive("String".into())),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_fact(PropositionFactSummary {
            proposition: proposition.clone(),
            role: ash_typeck::type_env::PropositionFactRole::Requirement,
            source_anchor: anchor("where Int == String"),
            predicate_dependencies: Vec::new(),
            dependency_summary_refs: Vec::new(),
            outcome: Some(misleading_satisfied),
        });

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary)
        .expect("import must re-solve instead of trusting imported evidence");

    assert_eq!(env.proposition_obligations().len(), 1);
    match &env.proposition_obligations()[0].outcome {
        Some(PropositionOutcome::Refuted(refutation)) => {
            assert_eq!(refutation.proposition, proposition);
        }
        other => panic!("expected locally refuted outcome, got {other:?}"),
    }
}

#[test]
fn task_879_import_rejects_private_predicate_leak_in_public_proposition_summary() {
    let module = module_identity(3, &["pkg", "private_predicate"]);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "HiddenReq",
            CoreVisibility::Private,
        ))
        .with_exported_proposition_fact(named_fact(&module, "HiddenReq", vec![int_term()]));

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("private predicates must not leak in public proposition summaries");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("HiddenReq"),
        "expected private-predicate leak diagnostic, got {msg}"
    );
    assert!(env.proposition_obligations().is_empty());
}

#[test]
fn task_879_import_rejects_private_or_unexported_domain_constructor_dependencies_fail_closed() {
    let module = module_identity(4, &["pkg", "private_domain"]);
    let private_domain = SealedDomainId::new(module.clone(), "HiddenList");
    let private_ctor = DomainConstructorId::new(private_domain.clone(), "HiddenNil");
    let predicate = predicate_summary(&module, "PublicReq", CoreVisibility::Public);
    let fact = named_fact(
        &module,
        "PublicReq",
        vec![TypePropositionTerm::DomainConstructorApp {
            constructor: private_ctor,
            domain: private_domain,
            args: Vec::new(),
            kind: Kind::Type,
        }],
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate)
        .with_exported_proposition_fact(fact);

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("unexported/private domain constructor dependency must reject import");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("HiddenList"),
        "expected private-domain leak diagnostic, got {msg}"
    );
    assert!(env.proposition_obligations().is_empty());
}

#[test]
fn task_879_export_rejects_private_ordinary_type_in_public_proposition_term_fail_closed() {
    let module = module_identity(8, &["pkg", "private_ordinary"]);
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&TypeDef {
        name: "Secret".into(),
        params: Vec::new(),
        body: TypeBody::Struct(Vec::new()),
        visibility: CoreVisibility::Private,
        builtin: false,
    })
    .expect("private ordinary type registers locally");
    let proposition = equality(
        nominal_term(&module, "Secret"),
        nominal_term(&module, "Secret"),
    );
    env.add_proposition_obligation(
        proposition,
        anchor("where Secret == Secret"),
        PropositionCheckingSite::new(
            879_008,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("pub fn leaked_private_type".into()),
        ),
    );

    let err = env
        .export_public_proposition_fact_summaries(&module)
        .expect_err("private ordinary types must not leak through public proposition facts");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("ordinary type") && msg.contains("Secret"),
        "expected private ordinary type diagnostic, got {msg}"
    );
}

#[test]
fn task_879_export_rejects_private_projection_dependencies_fail_closed() {
    let module = module_identity(9, &["pkg", "private_projection"]);
    let interface = InterfaceIdentityId::new(module.clone(), "HiddenInterface");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "HiddenMember",
        vec!["HiddenInterface".into(), "HiddenMember".into()],
    );
    let projection = TypePropositionTerm::Canonical(CanonicalTypeExpr::Projection {
        interface,
        member,
        args: Vec::new(),
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    });
    let proposition = equality(projection.clone(), projection);
    let mut env = TypeEnv::with_builtin_types();
    env.add_proposition_obligation(
        proposition,
        anchor("where HiddenInterface::HiddenMember == HiddenInterface::HiddenMember"),
        PropositionCheckingSite::new(
            879_009,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("pub fn leaked_private_projection".into()),
        ),
    );

    let err = env
        .export_public_proposition_fact_summaries(&module)
        .expect_err("unproven projection identities must reject fail-closed");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("interface") && msg.contains("HiddenInterface"),
        "expected private interface/projection diagnostic, got {msg}"
    );
}

#[test]
fn task_879_export_rejects_private_interface_bound_dependencies_fail_closed() {
    let module = module_identity(11, &["pkg", "private_interface_bound"]);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module.clone());
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "HiddenInterface".into(),
        type_params: Vec::new(),
        evidence_constraints: vec![],
        associated_types: Vec::new(),
        methods: Vec::new(),
        laws: Vec::new(),
        span: Default::default(),
    })
    .expect("private interface registers locally");
    let interface = InterfaceIdentityId::new(module.clone(), "HiddenInterface");
    let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject: int_term(),
        interface,
        interface_args: Vec::new(),
    });
    env.add_proposition_obligation(
        proposition,
        anchor("where Int: HiddenInterface"),
        PropositionCheckingSite::new(
            879_011,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("pub fn leaked_private_interface_bound".into()),
        ),
    );

    let err = env
        .export_public_proposition_fact_summaries(&module)
        .expect_err("private interface bounds must not leak through public proposition facts");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("interface") && msg.contains("HiddenInterface"),
        "expected private interface-bound diagnostic, got {msg}"
    );
}

#[test]
fn task_879_import_rejects_private_type_dependency_in_public_predicate_param() {
    let module = module_identity(10, &["pkg", "predicate_param_private"]);
    let mut predicate = predicate_summary(&module, "LeaksParam", CoreVisibility::Public);
    predicate.params[0].ty = match nominal_term(&module, "Secret") {
        TypePropositionTerm::Canonical(expr) => expr,
        TypePropositionTerm::DomainConstructorApp { .. } => unreachable!(),
    };
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate);

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("public predicate params must reject private ordinary type dependencies");

    let msg = err.to_string();
    assert!(
        msg.contains("private") && msg.contains("ordinary type") && msg.contains("Secret"),
        "expected private predicate-param dependency diagnostic, got {msg}"
    );
    assert!(env.lookup_proposition_predicate("LeaksParam").is_none());
}

#[test]
fn task_879_export_public_requirements_keeps_deferred_evidence_diagnostic() {
    let module = module_identity(5, &["pkg", "export"]);
    let predicate = predicate_summary(&module, "DeferredReq", CoreVisibility::Public);
    let predicate_id = predicate.id.clone();
    let mut env = TypeEnv::with_builtin_types();
    env.register_proposition_predicate_summary(&predicate)
        .expect("public predicate can register");
    let proposition = TypeProposition::NamedPredicate(NamedPredicateProposition {
        predicate: predicate_id.clone(),
        args: vec![int_term()],
    });
    env.add_proposition_obligation(
        proposition.clone(),
        anchor("where DeferredReq<Int>"),
        PropositionCheckingSite::new(
            879_001,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("pub fn exported".into()),
        ),
    );

    let facts = env
        .export_public_proposition_fact_summaries(&module)
        .expect("public requirement summaries should export");

    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].proposition, proposition);
    assert_eq!(facts[0].predicate_dependencies, vec![predicate_id]);
    assert!(matches!(
        &facts[0].outcome,
        Some(PropositionOutcome::Deferred(reason))
            if reason.kind == PropositionDeferredKind::UnsupportedNamedPredicate
    ));
}

#[test]
fn task_879_v4_summary_with_proposition_fact_rejects_before_registering_predicate() {
    let module = module_identity(6, &["pkg", "v4_reject"]);
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4)
        .with_exported_proposition_predicate(predicate_summary(
            &module,
            "ShouldNotRegister",
            CoreVisibility::Public,
        ))
        .with_exported_proposition_fact(named_fact(&module, "ShouldNotRegister", vec![int_term()]));

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("V4 proposition fact payload must reject before partial registration");

    assert!(
        env.lookup_proposition_predicate("ShouldNotRegister")
            .is_none()
    );
    assert!(env.proposition_obligations().is_empty());
    assert_eq!(
        err.code().expect("stable proposition summary code").0,
        "E175"
    );
}
