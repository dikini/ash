//! TASK-882: SPEC-064 H9/H10 summary acceptance and V1-V4 non-interference.

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::{
    CanonicalTypeExpr, ModuleIdentity, ModuleSemanticSummary, ModuleSemanticSummaryValidationError,
    ModuleSourceOrigin, PropositionDeferredKind, PropositionDeferredReason, PropositionFactRole,
    PropositionFactSummary, PropositionOutcome, PropositionPredicateId,
    PropositionPredicateParamSummary, PropositionPredicateSummary, SourceAnchor, SourceOrigin,
    SummaryVersion, TypeProposition, TypePropositionTerm,
};

fn module_identity(module_id: usize, name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(882)),
        ModuleId(module_id),
        vec!["task882".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-882 summary non-interference {name}"),
        },
    )
}

fn anchor(label: impl Into<String>) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-882 proposition acceptance matrix".to_string(),
        },
        Some(Span { start: 1, end: 9 }),
        label,
    )
}

fn predicate_id(module: &ModuleIdentity, name: &str) -> PropositionPredicateId {
    PropositionPredicateId::new(module.clone(), name)
}

fn predicate_summary(module: &ModuleIdentity, name: &str) -> PropositionPredicateSummary {
    PropositionPredicateSummary {
        id: predicate_id(module, name),
        exported_name: name.to_string(),
        visibility: Visibility::Public,
        params: vec![PropositionPredicateParamSummary {
            name: "T".to_string(),
            ty: CanonicalTypeExpr::Primitive("Int".to_string()),
            kind: Kind::Type,
            source_anchor: anchor("T: Int"),
        }],
        source_anchor: anchor(format!("pub prop {name}<T: Int>;")),
    }
}

fn proposition_fact(module: &ModuleIdentity, name: &str) -> PropositionFactSummary {
    let proposition = TypeProposition::NamedPredicate(ash_core::NamedPredicateProposition {
        predicate: predicate_id(module, name),
        args: vec![TypePropositionTerm::Canonical(
            CanonicalTypeExpr::Primitive("Int".to_string()),
        )],
    });
    PropositionFactSummary {
        proposition: proposition.clone(),
        role: PropositionFactRole::Requirement,
        source_anchor: anchor(format!("where {name}<Int>")),
        predicate_dependencies: vec![predicate_id(module, name)],
        dependency_summary_refs: Vec::new(),
        outcome: Some(PropositionOutcome::Deferred(PropositionDeferredReason {
            proposition,
            kind: PropositionDeferredKind::UnsupportedNamedPredicate,
            source_anchor: Some(anchor("deferred named-predicate evidence")),
            no_inversion_boundary: true,
        })),
    }
}

#[test]
fn task_882_h9_v5_summary_preserves_public_proposition_requirements_without_touching_older_payloads()
 {
    let module = module_identity(1, "v5_acceptance");
    let predicate = predicate_summary(&module, "PublicReq");
    let fact = proposition_fact(&module, "PublicReq");
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate.clone())
        .with_exported_proposition_fact(fact.clone());

    summary
        .validate_summary_version_contract()
        .expect("H9: V5 summaries may carry proposition requirements");
    assert_eq!(summary.exported_proposition_predicates, vec![predicate]);
    assert_eq!(summary.exported_proposition_facts, vec![fact]);
}

#[test]
fn task_882_h10_v4_and_older_summaries_reject_proposition_facts_before_v5_registration() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
    ] {
        let module = module_identity(version.0 as usize, "pre_v5_reject");
        let summary = ModuleSemanticSummary::new(module.clone())
            .with_version(version)
            .with_exported_proposition_fact(proposition_fact(&module, "LegacyReq"));

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 { version }),
            "H10: pre-V5 summary version {version:?} must fail closed for proposition payloads"
        );
    }
}
