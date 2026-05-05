//! TASK-813: Sealed-domain registration diagnostics and non-interference tests
//! for ash-typeck.
//!
//! Negative cases (rejection of invalid constructs at the TypeEnv boundary) and
//! non-interference with existing Phase 109/110 type-lookup behavior.

use ash_core::Kind;
use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, StructuralFieldStatus, SummaryVersion,
};
use ash_typeck::TypeEnv;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn module_identity(id: u32, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id as usize),
        path.iter().map(|s| (*s).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-813-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-813-test".into(),
        },
        None,
        label,
    )
}

fn make_domain_summary(
    module: &ModuleIdentity,
    name: &str,
    visibility: CoreVisibility,
    constructors: Vec<DomainConstructorSummary>,
) -> SealedDomainSummary {
    let domain_id = SealedDomainId::new(module.clone(), name);
    let mut summary = SealedDomainSummary::new(domain_id, name, visibility, anchor(name));
    for ctor in constructors {
        summary = summary.with_constructor(ctor);
    }
    summary
}

fn unit_ctor(domain_id: &SealedDomainId, name: &str) -> DomainConstructorSummary {
    DomainConstructorSummary::new(
        DomainConstructorId::new(domain_id.clone(), name),
        name,
        vec![],
        anchor(name),
    )
}

fn fielded_ctor(
    domain_id: &SealedDomainId,
    name: &str,
    fields: Vec<DomainFieldSummary>,
) -> DomainConstructorSummary {
    DomainConstructorSummary::new(
        DomainConstructorId::new(domain_id.clone(), name),
        name,
        fields,
        anchor(name),
    )
}

fn make_v2_summary_with_domain(
    module: &ModuleIdentity,
    domain: SealedDomainSummary,
) -> ModuleSemanticSummary {
    let mut summary =
        ModuleSemanticSummary::new(module.clone()).with_exported_sealed_domain(domain);
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary
}

fn make_v2_summary_with_domains(
    module: &ModuleIdentity,
    domains: Vec<SealedDomainSummary>,
) -> ModuleSemanticSummary {
    let mut summary = ModuleSemanticSummary::new(module.clone());
    for domain in domains {
        summary = summary.with_exported_sealed_domain(domain);
    }
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary
}

// ---------------------------------------------------------------------------
// Negative cases: rejection of invalid constructs
// ---------------------------------------------------------------------------

#[test]
fn reject_unsupported_summary_version() {
    let module = module_identity(1, &["test"]);
    let mut summary = ModuleSemanticSummary::new(module);
    summary.version = SummaryVersion(99); // unsupported version

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "unsupported summary version should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("unsupported module semantic summary version"),
        "error should mention unsupported version, got: {err_msg}"
    );
    assert!(
        err_msg.contains("99"),
        "error should mention the invalid version number, got: {err_msg}"
    );
}

#[test]
fn reject_malformed_domain_constructor_id_mismatch() {
    // A constructor whose DomainConstructorId references a different domain
    // than the enclosing domain should be rejected.
    let module = module_identity(2, &["test"]);
    let _enclosing_domain_id = SealedDomainId::new(module.clone(), "Color");
    let wrong_domain_id = SealedDomainId::new(module.clone(), "Shape");

    // Constructor id references Shape but enclosing domain is Color.
    let bad_ctor = DomainConstructorSummary::new(
        DomainConstructorId::new(wrong_domain_id, "X"),
        "X",
        vec![],
        anchor("X"),
    );
    let domain = make_domain_summary(&module, "Color", CoreVisibility::Public, vec![bad_ctor]);
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "constructor with mismatched domain id should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("references a different domain"),
        "error should mention mismatched domain reference, got: {err_msg}"
    );
}

#[test]
fn reject_unknown_field_domain_reference() {
    // A field referencing an unregistered domain should be rejected.
    let module = module_identity(3, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Foo");

    // The field references a domain "Bar" which is not registered anywhere.
    let unknown_domain_id = SealedDomainId::new(module.clone(), "Bar");
    let field = DomainFieldSummary::constrained_to("x", &domain_id, unknown_domain_id);
    let ctor = fielded_ctor(&domain_id, "MkFoo", vec![field]);
    let domain = make_domain_summary(&module, "Foo", CoreVisibility::Public, vec![ctor]);
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "field referencing an unknown sealed domain should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("unknown sealed domain"),
        "error should mention unknown domain reference, got: {err_msg}"
    );
}

#[test]
fn reject_multiple_structural_self_domain_fields() {
    // A constructor with 2+ StructuralSelfDomain fields should be rejected.
    let module = module_identity(4, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Tree");

    // Both fields reference the enclosing domain (self-referencing).
    let field_a = DomainFieldSummary::constrained_to("left", &domain_id, domain_id.clone());
    let field_b = DomainFieldSummary::constrained_to("right", &domain_id, domain_id.clone());

    assert_eq!(
        field_a.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert_eq!(
        field_b.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );

    let ctor = fielded_ctor(&domain_id, "Branch", vec![field_a, field_b]);
    let domain = make_domain_summary(&module, "Tree", CoreVisibility::Public, vec![ctor]);
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "constructor with multiple structural self-domain fields should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("structural self-domain fields"),
        "error should mention structural self-domain constraint, got: {err_msg}"
    );
    assert!(
        err_msg.contains("at most one is permitted"),
        "error should mention 'at most one' constraint, got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Non-interference: ordinary type lookup preserved
// ---------------------------------------------------------------------------

#[test]
fn non_interference_ordinary_type_lookup_preserved() {
    // Register both ordinary types and sealed domains, verify ordinary
    // lookup methods work unchanged.
    use ash_core::semantic_summary::{
        RepresentationExposure, TypeDeclId, TypeDeclSummary, TypeRepresentationSummary,
    };

    let module = module_identity(10, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Dir");
    let domain = make_domain_summary(
        &module,
        "Dir",
        CoreVisibility::Public,
        vec![
            unit_ctor(&domain_id, "North"),
            unit_ctor(&domain_id, "South"),
        ],
    );

    let type_id = TypeDeclId::ordinary(module.clone(), "Status");
    let type_summary = TypeDeclSummary::new(
        type_id,
        "Status",
        CoreVisibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("Status"),
    );

    let mut summary = ModuleSemanticSummary::new(module)
        .with_exported_type(type_summary)
        .with_exported_sealed_domain(domain);
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("mixed registration should succeed");

    // Ordinary type lookup still works.
    assert!(
        env.lookup_type("Status").is_some(),
        "ordinary type 'Status' should still be found via lookup_type"
    );

    // Sealed domain lookup works independently.
    assert!(
        env.lookup_sealed_domain("Dir").is_some(),
        "sealed domain 'Dir' should be found via lookup_sealed_domain"
    );

    // Sealed domain names don't pollute ordinary type lookup.
    assert!(
        env.lookup_type("Dir").is_none(),
        "sealed domain names must not appear in ordinary lookup_type"
    );

    // Ordinary type names don't pollute sealed domain lookup.
    assert!(
        env.lookup_sealed_domain("Status").is_none(),
        "ordinary type names must not appear in lookup_sealed_domain"
    );

    // Constructor lookup still works for ordinary types only.
    assert!(
        env.lookup_constructor("Status").is_none(),
        "type names should not be in constructor lookup"
    );

    // Sealed-domain constructors are NOT in the ordinary constructor map.
    assert!(
        env.lookup_constructor("North").is_none(),
        "sealed domain constructors must not appear in ordinary lookup_constructor"
    );
}

#[test]
fn v1_only_summary_still_works_after_v2_code_path() {
    // After registering a V2 summary, verify a V1 summary still works.
    let module_v2 = module_identity(20, &["v2_mod"]);
    let domain_id = SealedDomainId::new(module_v2.clone(), "Tag");
    let domain = make_domain_summary(
        &module_v2,
        "Tag",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "A")],
    );
    let summary_v2 = make_v2_summary_with_domain(&module_v2, domain);

    let module_v1 = module_identity(21, &["v1_mod"]);
    let summary_v1 = ModuleSemanticSummary::new(module_v1);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary_v2)
        .expect("V2 registration should succeed");
    env.register_module_semantic_summary(&summary_v1)
        .expect("V1 registration should still succeed after V2 code path");

    assert!(env.lookup_sealed_domain("Tag").is_some());
    assert!(env.lookup_sealed_domain("v1_mod").is_none());
}

#[test]
fn reject_v1_summary_with_exported_sealed_domains() {
    let module = module_identity(101, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "LegacyLeak");
    let domain = make_domain_summary(
        &module,
        "LegacyLeak",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "Mk")],
    );
    let summary = ModuleSemanticSummary::new(module).with_exported_sealed_domain(domain);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "V1 summaries must not carry sealed domains"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("V1") && err_msg.contains("sealed domain"),
        "got: {err_msg}"
    );
}

#[test]
fn reject_mutual_recursive_same_summary_domains() {
    let module = module_identity(102, &["test"]);
    let domain_a_id = SealedDomainId::new(module.clone(), "A");
    let domain_b_id = SealedDomainId::new(module.clone(), "B");
    let a_field = DomainFieldSummary::constrained_to("b", &domain_a_id, domain_b_id.clone());
    let b_field = DomainFieldSummary::constrained_to("a", &domain_b_id, domain_a_id.clone());
    let domain_a = make_domain_summary(
        &module,
        "A",
        CoreVisibility::Public,
        vec![fielded_ctor(&domain_a_id, "MkA", vec![a_field])],
    );
    let domain_b = make_domain_summary(
        &module,
        "B",
        CoreVisibility::Public,
        vec![fielded_ctor(&domain_b_id, "MkB", vec![b_field])],
    );
    let summary = make_v2_summary_with_domains(&module, vec![domain_a, domain_b]);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "same-summary A <-> B recursion should reject"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("mutual") || err_msg.contains("cycle"),
        "got: {err_msg}"
    );
}

#[test]
fn reject_field_structural_status_mismatches_and_non_type_kind() {
    let module = module_identity(103, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Tree");
    let other_id = SealedDomainId::new(module.clone(), "Leaf");

    let mut self_marked_non_structural =
        DomainFieldSummary::constrained_to("bad_self", &domain_id, domain_id.clone());
    self_marked_non_structural.structural_status = StructuralFieldStatus::NonStructural;
    let mut other_marked_structural =
        DomainFieldSummary::constrained_to("bad_other", &domain_id, other_id.clone());
    other_marked_structural.structural_status = StructuralFieldStatus::StructuralSelfDomain;
    let mut unconstrained_marked_structural = DomainFieldSummary::unconstrained("bad_none");
    unconstrained_marked_structural.structural_status = StructuralFieldStatus::StructuralSelfDomain;
    let mut non_type_kind = DomainFieldSummary::unconstrained("bad_kind");
    non_type_kind.kind = Kind::Arrow(Box::new(Kind::Type), Box::new(Kind::Type));

    for (label, field) in [
        (
            "self constraint marked NonStructural",
            self_marked_non_structural,
        ),
        (
            "other-domain constraint marked StructuralSelfDomain",
            other_marked_structural,
        ),
        (
            "no constraint marked StructuralSelfDomain",
            unconstrained_marked_structural,
        ),
        ("non-Type field kind", non_type_kind),
    ] {
        let domain = make_domain_summary(
            &module,
            "Tree",
            CoreVisibility::Public,
            vec![fielded_ctor(&domain_id, "MkTree", vec![field])],
        );
        let other = make_domain_summary(
            &module,
            "Leaf",
            CoreVisibility::Public,
            vec![unit_ctor(&other_id, "MkLeaf")],
        );
        let summary = make_v2_summary_with_domains(&module, vec![domain, other]);
        let mut env = TypeEnv::new();
        assert!(
            env.register_module_semantic_summary(&summary).is_err(),
            "{label} should be rejected"
        );
    }
}

#[test]
fn reject_constructor_id_name_exported_name_mismatch() {
    let module = module_identity(104, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Color");
    let bad_ctor = DomainConstructorSummary::new(
        DomainConstructorId::new(domain_id.clone(), "Red"),
        "Blue",
        vec![],
        anchor("Blue"),
    );
    let domain = make_domain_summary(&module, "Color", CoreVisibility::Public, vec![bad_ctor]);
    let summary = make_v2_summary_with_domain(&module, domain);
    let mut env = TypeEnv::new();
    assert!(
        env.register_module_semantic_summary(&summary).is_err(),
        "constructor id.name/exported_name mismatch should reject"
    );
}

#[test]
fn positive_cross_domain_one_way_reference_passes() {
    let module = module_identity(105, &["test"]);
    let domain_a_id = SealedDomainId::new(module.clone(), "A");
    let domain_b_id = SealedDomainId::new(module.clone(), "B");
    let a_field = DomainFieldSummary::constrained_to("b", &domain_a_id, domain_b_id.clone());
    let domain_a = make_domain_summary(
        &module,
        "A",
        CoreVisibility::Public,
        vec![fielded_ctor(&domain_a_id, "MkA", vec![a_field])],
    );
    let domain_b = make_domain_summary(
        &module,
        "B",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_b_id, "MkB")],
    );
    let summary = make_v2_summary_with_domains(&module, vec![domain_a, domain_b]);
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("one-way same-summary cross-domain reference should pass");
}

#[test]
fn sealed_domain_names_are_listed_separately() {
    let module = module_identity(30, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Color");
    let domain = make_domain_summary(
        &module,
        "Color",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "Red")],
    );
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("registration should succeed");

    let names: Vec<&str> = env.sealed_domain_names().collect();
    assert!(
        names.contains(&"Color"),
        "sealed_domain_names should list 'Color'"
    );
    assert_eq!(names.len(), 1, "should have exactly one sealed domain name");
}
