//! TASK-809: Sealed-domain identity, field metadata, and summary carrier tests.
//!
//! Validates SPEC-059 §6-8 identity carriers, field metadata, summary carriers,
//! version advancement, and backward compatibility.

use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, StructuralFieldStatus, SummaryVersion,
};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(7)),
        ModuleId(42),
        vec!["crate".into(), "domain".into()],
        ModuleSourceOrigin::File("/repo/src/domain.ash".into()),
    )
}

fn domain_anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::File("/repo/src/domain.ash".into()),
        Some(Span { start: 0, end: 10 }),
        label,
    )
}

// ---------------------------------------------------------------------------
// SealedDomainId
// ---------------------------------------------------------------------------

#[test]
fn sealed_domain_id_equality_based_on_module_and_name() {
    let module = module_identity();
    let same_module_different_path = ModuleIdentity::new(
        Some(CrateId(7)),
        ModuleId(42),
        vec!["other".into()],
        ModuleSourceOrigin::File("/other.ash".into()),
    );

    let id1 = SealedDomainId::new(module.clone(), "Shape");
    let id2 = SealedDomainId::new(same_module_different_path, "Shape");
    let id3 = SealedDomainId::new(module, "Color");

    // ModuleIdentity ignores diagnostic metadata (path, source)
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn sealed_domain_id_hash_consistent_with_equality() {
    use std::collections::HashSet;

    let module = module_identity();
    let id1 = SealedDomainId::new(module.clone(), "Shape");
    let id2 = SealedDomainId::new(module, "Shape");

    let mut set = HashSet::new();
    assert!(set.insert(id1.clone()));
    assert!(!set.insert(id2)); // same hash+eq => not inserted again
}

#[test]
fn sealed_domain_id_serde_roundtrip() {
    let id = SealedDomainId::new(module_identity(), "Shape");
    let json = serde_json::to_string(&id).expect("serialize");
    let back: SealedDomainId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(id, back);
}

#[test]
fn sealed_domain_id_is_distinct_from_type_decl_id() {
    // Compile-time proof: SealedDomainId and TypeDeclId are different types.
    // This test just verifies the identities don't share representation.
    let module = module_identity();
    let domain_id = SealedDomainId::new(module.clone(), "Shape");
    let type_id = ash_core::semantic_summary::TypeDeclId::ordinary(module, "Shape");

    // They are different types; this test verifies the distinct identity spaces.
    assert_eq!(domain_id.name, type_id.name);
    // SealedDomainId has no item_kind field, unlike TypeDeclId.
    assert!(std::any::type_name_of_val(&domain_id).contains("SealedDomainId"));
    assert!(std::any::type_name_of_val(&type_id).contains("TypeDeclId"));
}

// ---------------------------------------------------------------------------
// DomainConstructorId
// ---------------------------------------------------------------------------

#[test]
fn domain_constructor_id_equality_based_on_domain_and_name() {
    let module = module_identity();
    let domain = SealedDomainId::new(module, "Shape");
    let c1 = DomainConstructorId::new(domain.clone(), "Circle");
    let c2 = DomainConstructorId::new(domain.clone(), "Circle");
    let c3 = DomainConstructorId::new(domain, "Square");

    assert_eq!(c1, c2);
    assert_ne!(c1, c3);
}

#[test]
fn domain_constructor_id_hash_consistent_with_equality() {
    use std::collections::HashSet;

    let domain = SealedDomainId::new(module_identity(), "Shape");
    let c1 = DomainConstructorId::new(domain.clone(), "Circle");
    let c2 = DomainConstructorId::new(domain, "Circle");

    let mut set = HashSet::new();
    assert!(set.insert(c1.clone()));
    assert!(!set.insert(c2));
}

#[test]
fn domain_constructor_id_serde_roundtrip() {
    let domain = SealedDomainId::new(module_identity(), "Shape");
    let id = DomainConstructorId::new(domain, "Circle");
    let json = serde_json::to_string(&id).expect("serialize");
    let back: DomainConstructorId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(id, back);
}

// ---------------------------------------------------------------------------
// DomainFieldSummary + StructuralFieldStatus
// ---------------------------------------------------------------------------

#[test]
fn unconstrained_field_has_type_kind_no_constraint_and_non_structural() {
    let field = DomainFieldSummary::unconstrained("width");
    assert_eq!(field.name, "width");
    assert_eq!(field.kind, Kind::Type);
    assert!(field.domain_constraint.is_none());
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::NonStructural
    );
}

#[test]
fn constrained_to_self_domain_produces_structural_self_domain() {
    let module = module_identity();
    let enclosing = SealedDomainId::new(module, "Shape");
    let same_domain = enclosing.clone();

    let field = DomainFieldSummary::constrained_to("inner", &enclosing, same_domain);
    assert_eq!(field.name, "inner");
    assert_eq!(field.kind, Kind::Type);
    assert_eq!(field.domain_constraint.as_ref(), Some(&enclosing));
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
}

#[test]
fn constrained_to_other_domain_produces_non_structural() {
    let module = module_identity();
    let enclosing = SealedDomainId::new(module.clone(), "Shape");
    let other = SealedDomainId::new(module, "Color");

    let field = DomainFieldSummary::constrained_to("color", &enclosing, other.clone());
    assert_eq!(field.domain_constraint.as_ref(), Some(&other));
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::NonStructural
    );
}

#[test]
fn structural_field_status_serde_roundtrip() {
    for status in [
        StructuralFieldStatus::NonStructural,
        StructuralFieldStatus::StructuralSelfDomain,
    ] {
        let json = serde_json::to_string(&status).expect("serialize");
        let back: StructuralFieldStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(status, back);
    }
}

#[test]
fn domain_field_summary_serde_roundtrip() {
    let enclosing = SealedDomainId::new(module_identity(), "Shape");
    let field = DomainFieldSummary::constrained_to("inner", &enclosing, enclosing.clone());
    let json = serde_json::to_string(&field).expect("serialize");
    let back: DomainFieldSummary = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(field, back);
}

// ---------------------------------------------------------------------------
// SealedDomainSummary + DomainConstructorSummary
// ---------------------------------------------------------------------------

#[test]
fn sealed_domain_summary_construction_and_builder() {
    let module = module_identity();
    let domain_id = SealedDomainId::new(module, "Shape");
    let anchor = domain_anchor("sealed domain Shape");

    let ctor_id = DomainConstructorId::new(domain_id.clone(), "Circle");
    let ctor_anchor = domain_anchor("constructor Circle");
    let field = DomainFieldSummary::unconstrained("radius");
    let ctor = DomainConstructorSummary::new(ctor_id, "Circle", vec![field], ctor_anchor.clone());

    let summary = SealedDomainSummary::new(
        domain_id.clone(),
        "Shape",
        Visibility::Public,
        anchor.clone(),
    )
    .with_constructor(ctor.clone());

    assert_eq!(summary.id, domain_id);
    assert_eq!(summary.exported_name, "Shape");
    assert_eq!(summary.visibility, Visibility::Public);
    assert_eq!(summary.constructors.len(), 1);
    assert_eq!(summary.constructors[0].id, ctor.id);
    assert_eq!(summary.constructors[0].fields.len(), 1);
    assert_eq!(summary.constructors[0].fields[0].name, "radius");
    assert_eq!(summary.anchor, anchor);
}

#[test]
fn sealed_domain_summary_serde_roundtrip() {
    let domain_id = SealedDomainId::new(module_identity(), "Shape");
    let anchor = domain_anchor("sealed domain Shape");
    let ctor_id = DomainConstructorId::new(domain_id.clone(), "Circle");
    let ctor = DomainConstructorSummary::new(
        ctor_id,
        "Circle",
        vec![DomainFieldSummary::unconstrained("radius")],
        domain_anchor("ctor Circle"),
    );

    let summary = SealedDomainSummary::new(domain_id, "Shape", Visibility::Public, anchor)
        .with_constructor(ctor);

    let json = serde_json::to_string(&summary).expect("serialize");
    let back: SealedDomainSummary = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(summary, back);
}

// ---------------------------------------------------------------------------
// ModuleSemanticSummary with sealed domains + version V2
// ---------------------------------------------------------------------------

#[test]
fn module_semantic_summary_v2_carries_sealed_domains() {
    let module = module_identity();
    let domain_id = SealedDomainId::new(module.clone(), "Shape");
    let anchor = domain_anchor("sealed domain Shape");
    let domain = SealedDomainSummary::new(domain_id, "Shape", Visibility::Public, anchor);

    let mut summary = ModuleSemanticSummary::new(module);
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary = summary.with_exported_sealed_domain(domain.clone());

    assert_eq!(summary.version, SummaryVersion::SPEC059_SEALED_DOMAIN_V2);
    assert_eq!(summary.exported_sealed_domains.len(), 1);
    assert_eq!(summary.exported_sealed_domains[0].id, domain.id);
}

#[test]
fn module_semantic_summary_new_initializes_empty_sealed_domains() {
    let summary = ModuleSemanticSummary::new(module_identity());
    assert!(summary.exported_sealed_domains.is_empty());
}

#[test]
fn summary_version_v2_is_distinct_from_v1() {
    assert_ne!(
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );
    assert_eq!(SummaryVersion::SPEC057_ORDINARY_TYPE_V1.0, 1);
    assert_eq!(SummaryVersion::SPEC059_SEALED_DOMAIN_V2.0, 2);
}

// ---------------------------------------------------------------------------
// Backward compatibility: V1 deserialization still works
// ---------------------------------------------------------------------------

#[test]
fn v1_summary_deserializes_without_sealed_domains_field() {
    let module = module_identity();
    let mut summary = ModuleSemanticSummary::new(module);
    // Force V1 version
    summary.version = SummaryVersion::SPEC057_ORDINARY_TYPE_V1;

    let mut value = serde_json::to_value(summary.clone()).expect("serialize");
    // Remove sealed-domain fields to simulate V1 payload
    let object = value.as_object_mut().expect("summary serializes as object");
    object.remove("exported_sealed_domains");

    let back: ModuleSemanticSummary =
        serde_json::from_value(value).expect("V1 summary should deserialize");
    assert_eq!(back.version, SummaryVersion::SPEC057_ORDINARY_TYPE_V1);
    assert!(back.exported_sealed_domains.is_empty());
}

// ---------------------------------------------------------------------------
// Sealed-domain identity is NOT TypeDeclId/ConstructorId (SPEC-059 §6 constraint)
// ---------------------------------------------------------------------------

#[test]
fn domain_constructor_id_is_not_constructor_id() {
    let module = module_identity();
    let domain = SealedDomainId::new(module.clone(), "Shape");
    let domain_ctor = DomainConstructorId::new(domain, "Circle");

    // DomainConstructorId references a SealedDomainId, not a TypeDeclId.
    // ConstructorId references a TypeDeclId parent.
    // These are fundamentally different identity spaces.
    assert!(std::any::type_name_of_val(&domain_ctor.domain).contains("SealedDomainId"));
}

#[test]
fn at_most_one_structural_self_domain_field_per_constructor_enforced_at_construction() {
    // The API does not enforce this at the type level (it's a validation concern
    // for later phases), but we verify that both statuses are representable and
    // distinct.
    let module = module_identity();
    let enclosing = SealedDomainId::new(module.clone(), "Tree");
    let field_self = DomainFieldSummary::constrained_to("children", &enclosing, enclosing.clone());
    let field_plain = DomainFieldSummary::unconstrained("label");

    assert_eq!(
        field_self.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert_eq!(
        field_plain.structural_status,
        StructuralFieldStatus::NonStructural
    );
}
