//! TASK-879: public proposition summary schema transport contract.

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
        Some(CrateId(879)),
        ModuleId(module_id),
        vec!["task879".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-879 {name}"),
        },
    )
}

fn anchor(label: impl Into<String>) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-879 proposition summary schema".to_string(),
        },
        Some(Span { start: 10, end: 20 }),
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

fn named_proposition(module: &ModuleIdentity, name: &str) -> TypeProposition {
    TypeProposition::NamedPredicate(ash_core::NamedPredicateProposition {
        predicate: predicate_id(module, name),
        args: vec![TypePropositionTerm::Canonical(
            CanonicalTypeExpr::Primitive("Int".to_string()),
        )],
    })
}

fn proposition_fact(module: &ModuleIdentity, name: &str) -> PropositionFactSummary {
    let proposition = named_proposition(module, name);
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
fn task_879_v5_serializes_predicate_identities_facts_deferred_evidence_and_cache_key() {
    let module = module_identity(1, "schema");
    let predicate = predicate_summary(&module, "PublicReq");
    let fact = proposition_fact(&module, "PublicReq");

    let empty = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5);
    let with_payload = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate.clone())
        .with_exported_proposition_fact(fact.clone());

    with_payload
        .validate_summary_version_contract()
        .expect("V5 summaries may carry public proposition payloads");
    assert_eq!(
        with_payload.exported_proposition_predicates,
        vec![predicate]
    );
    assert_eq!(with_payload.exported_proposition_facts, vec![fact]);
    assert_ne!(
        empty.semantic_cache_key(),
        with_payload.semantic_cache_key()
    );

    let json = serde_json::to_string_pretty(&with_payload).expect("summary serializes");
    assert!(json.contains("exported_proposition_predicates"));
    assert!(json.contains("exported_proposition_facts"));
    assert!(json.contains("UnsupportedNamedPredicate"));
    let decoded: ModuleSemanticSummary = serde_json::from_str(&json).expect("summary deserializes");
    assert_eq!(decoded, with_payload);
}

#[test]
fn task_879_v4_or_older_proposition_fact_payloads_are_rejected_by_schema_contract() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
    ] {
        let module = module_identity(version.0 as usize, "pre-v5");
        let summary = ModuleSemanticSummary::new(module.clone())
            .with_version(version)
            .with_exported_proposition_fact(proposition_fact(&module, "LegacyReq"));

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 { version })
        );
    }
}
