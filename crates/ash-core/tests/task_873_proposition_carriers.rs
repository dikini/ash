//! TASK-873: core proposition carriers and V5 semantic-summary contract.
//!
//! The RED pass for this test target intentionally names the TASK-873 public
//! carriers before production code exists.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::{
    CanonicalTypeExpr, DomainConstructorId, InterfaceBoundProposition, InterfaceIdentityId,
    ModuleIdentity, ModuleSemanticSummary, ModuleSemanticSummaryValidationError,
    ModuleSourceOrigin, ModuleSummaryRef, NormalTypeExpr, PropositionBoundary,
    PropositionDeferredKind, PropositionDeferredReason, PropositionEvidence,
    PropositionEvidenceRule, PropositionFactRole, PropositionFactSummary, PropositionOutcome,
    PropositionPredicateId, PropositionPredicateParamSummary, PropositionPredicateSummary,
    PropositionRefutation, PropositionRefutationReason, PropositionTypeComparisonEvidence,
    SealedDomainId, SourceAnchor, SourceOrigin, SummaryVersion, TypeDisequalityProposition,
    TypeEqualityProposition, TypeProposition, TypePropositionTerm,
};

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn module_identity(module_id: usize, name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(873)),
        ModuleId(module_id),
        vec!["task873".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-873 {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-873 proposition carrier test".to_string(),
        },
        Some(Span { start: 10, end: 20 }),
        label,
    )
}

fn type_list_domain() -> SealedDomainId {
    SealedDomainId::new(module_identity(1, "type_list"), "TypeList")
}

fn nil_constructor() -> DomainConstructorId {
    DomainConstructorId::new(type_list_domain(), "Nil")
}

fn cons_constructor() -> DomainConstructorId {
    DomainConstructorId::new(type_list_domain(), "Cons")
}

fn var_term(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(name.to_string()))
}

fn nil_term() -> TypePropositionTerm {
    TypePropositionTerm::DomainConstructorApp {
        constructor: nil_constructor(),
        domain: type_list_domain(),
        args: Vec::new(),
        kind: Kind::Type,
    }
}

fn cons_term() -> TypePropositionTerm {
    TypePropositionTerm::DomainConstructorApp {
        constructor: cons_constructor(),
        domain: type_list_domain(),
        args: vec![var_term("A"), var_term("T")],
        kind: Kind::Type,
    }
}

fn predicate_id(name: &str) -> PropositionPredicateId {
    PropositionPredicateId::new(module_identity(2, "predicates"), name)
}

fn non_empty_predicate() -> PropositionPredicateSummary {
    PropositionPredicateSummary {
        id: predicate_id("NonEmpty"),
        exported_name: "NonEmpty".to_string(),
        visibility: Visibility::Public,
        params: vec![PropositionPredicateParamSummary {
            name: "Xs".to_string(),
            ty: CanonicalTypeExpr::Primitive("TypeList".to_string()),
            kind: Kind::Type,
            source_anchor: anchor("Xs: TypeList"),
        }],
        source_anchor: anchor("prop NonEmpty<Xs: TypeList>"),
    }
}

fn named_proposition() -> TypeProposition {
    TypeProposition::NamedPredicate(ash_core::NamedPredicateProposition {
        predicate: predicate_id("NonEmpty"),
        args: vec![cons_term()],
    })
}

#[test]
fn task_873_proposition_term_carries_sealed_domain_constructor_apps_without_nominal_encoding() {
    let cons = cons_term();
    let same = cons_term();
    let nil = nil_term();

    assert_eq!(cons, same);
    assert_eq!(hash_of(&cons), hash_of(&same));
    assert_ne!(cons, nil);

    match &cons {
        TypePropositionTerm::DomainConstructorApp {
            constructor,
            domain,
            args,
            kind,
        } => {
            assert_eq!(constructor, &cons_constructor());
            assert_eq!(domain, &type_list_domain());
            assert_eq!(args, &vec![var_term("A"), var_term("T")]);
            assert_eq!(kind, &Kind::Type);
        }
        TypePropositionTerm::Canonical(_) => {
            panic!("Cons<A, T> must not be encoded as CanonicalTypeExpr")
        }
    }

    let json = serde_json::to_string_pretty(&cons).expect("term serializes");
    assert!(json.contains("DomainConstructorApp"));
    assert!(json.contains("Cons"));
    assert!(!json.contains("NominalApp"));
    let decoded: TypePropositionTerm = serde_json::from_str(&json).expect("term deserializes");
    assert_eq!(decoded, cons);
}

#[test]
fn task_873_all_four_core_proposition_forms_are_typed_hashable_and_serializable() {
    let equality = TypeProposition::Equality(TypeEqualityProposition {
        lhs: var_term("Zs"),
        rhs: cons_term(),
    });
    let disequality = TypeProposition::Disequality(TypeDisequalityProposition {
        lhs: cons_term(),
        rhs: nil_term(),
    });
    let interface = TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject: var_term("T"),
        interface: InterfaceIdentityId::new(module_identity(3, "interfaces"), "Iterator"),
        interface_args: vec![var_term("A")],
    });
    let named = named_proposition();

    let propositions = vec![equality, disequality, interface, named];
    for proposition in propositions {
        assert_eq!(proposition, proposition.clone());
        assert_eq!(hash_of(&proposition), hash_of(&proposition.clone()));
        let json = serde_json::to_string(&proposition).expect("proposition serializes");
        let decoded: TypeProposition =
            serde_json::from_str(&json).expect("proposition deserializes");
        assert_eq!(decoded, proposition);
    }
}

#[test]
fn task_873_boundary_outcome_evidence_refutation_and_deferred_reasons_are_structural() {
    let proposition = TypeProposition::Disequality(TypeDisequalityProposition {
        lhs: cons_term(),
        rhs: nil_term(),
    });
    let normalized_terms = PropositionTypeComparisonEvidence {
        lhs: NormalTypeExpr::DomainConstructorApp {
            constructor: cons_constructor(),
            domain: type_list_domain(),
            args: vec![
                NormalTypeExpr::Var("A".to_string()),
                NormalTypeExpr::Var("T".to_string()),
            ],
            kind: Kind::Type,
        },
        rhs: NormalTypeExpr::DomainConstructorApp {
            constructor: nil_constructor(),
            domain: type_list_domain(),
            args: Vec::new(),
            kind: Kind::Type,
        },
    };
    let boundary = PropositionBoundary::ImportedSummary(ModuleSummaryRef {
        module: module_identity(4, "dependency"),
        version: SummaryVersion::SPEC064_PROPOSITIONS_V5,
    });

    let satisfied = PropositionOutcome::Satisfied(PropositionEvidence {
        proposition: proposition.clone(),
        normalized_terms: Some(normalized_terms.clone()),
        rule: PropositionEvidenceRule::SealedDomainConstructorDisjointness,
        source_anchor: Some(anchor("Cons<A, T> != Nil")),
        boundary: boundary.clone(),
    });
    let refuted = PropositionOutcome::Refuted(PropositionRefutation {
        proposition: proposition.clone(),
        normalized_terms: Some(normalized_terms.clone()),
        reason: PropositionRefutationReason::DefinitionalEquality,
        source_anchor: Some(anchor("refuted disequality")),
        boundary: PropositionBoundary::Local,
    });
    let deferred = PropositionOutcome::Deferred(PropositionDeferredReason {
        proposition,
        kind: PropositionDeferredKind::RequiresTypeFunctionInversion,
        source_anchor: Some(anchor("Append<Xs, Ys> == Zs")),
        no_inversion_boundary: true,
    });

    for outcome in [satisfied, refuted, deferred] {
        assert_eq!(outcome, outcome.clone());
        assert_eq!(hash_of(&outcome), hash_of(&outcome.clone()));
        let json = serde_json::to_string(&outcome).expect("outcome serializes");
        let decoded: PropositionOutcome =
            serde_json::from_str(&json).expect("outcome deserializes");
        assert_eq!(decoded, outcome);
    }
}

#[test]
fn task_873_predicate_identity_and_source_anchor_carriers_are_public_and_typed() {
    let predicate = non_empty_predicate();
    let same = non_empty_predicate();
    let other = PropositionPredicateSummary {
        id: predicate_id("Sorted"),
        exported_name: "Sorted".to_string(),
        ..non_empty_predicate()
    };

    assert_eq!(predicate, same);
    assert_eq!(hash_of(&predicate), hash_of(&same));
    assert_ne!(predicate, other);
    assert_eq!(predicate.params[0].kind, Kind::Type);
    assert_eq!(predicate.source_anchor.label, "prop NonEmpty<Xs: TypeList>");

    let json = serde_json::to_string_pretty(&predicate).expect("predicate summary serializes");
    assert!(json.contains("NonEmpty"));
    assert!(json.contains("source_anchor"));
    let decoded: PropositionPredicateSummary =
        serde_json::from_str(&json).expect("predicate summary deserializes");
    assert_eq!(decoded, predicate);
}

#[test]
fn task_873_v5_semantic_summary_accepts_proposition_facts_and_cache_key_changes() {
    let module = module_identity(5, "v5");
    let fact = PropositionFactSummary {
        proposition: named_proposition(),
        role: PropositionFactRole::Requirement,
        source_anchor: anchor("where NonEmpty<Cons<A, T>>"),
        predicate_dependencies: vec![predicate_id("NonEmpty")],
        dependency_summary_refs: Vec::new(),
        outcome: None,
    };

    let empty = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5);
    let with_fact = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_fact(fact.clone());

    with_fact
        .validate_summary_version_contract()
        .expect("V5 summaries may carry proposition facts");
    assert_eq!(with_fact.exported_proposition_facts, vec![fact]);
    assert_ne!(empty.semantic_cache_key(), with_fact.semantic_cache_key());

    let json = serde_json::to_string(&with_fact).expect("summary serializes");
    let decoded: ModuleSemanticSummary = serde_json::from_str(&json).expect("summary deserializes");
    assert_eq!(
        decoded.exported_proposition_facts,
        with_fact.exported_proposition_facts
    );
}

#[test]
fn task_873_v1_through_v4_summaries_with_proposition_facts_are_rejected_before_registration() {
    for version in [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
    ] {
        let summary = ModuleSemanticSummary::new(module_identity(version.0 as usize, "legacy"))
            .with_version(version)
            .with_exported_proposition_fact(PropositionFactSummary {
                proposition: named_proposition(),
                role: PropositionFactRole::Requirement,
                source_anchor: anchor("legacy proposition fact"),
                predicate_dependencies: vec![predicate_id("NonEmpty")],
                dependency_summary_refs: Vec::new(),
                outcome: None,
            });

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 { version })
        );
    }
}
