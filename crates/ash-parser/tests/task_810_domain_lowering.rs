//! TASK-810: Focused lowering tests for sealed-domain declarations.
//!
//! These tests verify that `lower_module_type_metadata` correctly processes
//! `Definition::SealedDomain` declarations and produces `SealedDomainSummary`
//! carriers with correct identities, field metadata, structural status,
//! source anchors, and summary version advancement.

use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    DomainConstructorId, SealedDomainId, StructuralFieldStatus, SummaryVersion,
};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn test_module_identity() -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        None,
        ModuleId(1),
        vec!["test".into()],
        ash_core::semantic_summary::ModuleSourceOrigin::File("test.ash".into()),
    )
}

fn lower(source: &str) -> ash_parser::lower::LoweredTypeMetadata {
    let module = parse(source);
    ash_parser::lower::lower_module_type_metadata(&module, test_module_identity())
}

// --- Tests ---

#[test]
fn simple_sealed_domain_produces_correct_summary() {
    let lowered = lower("sealed type domain Boolish { Yes; No; }\n");
    let summary = &lowered.summary;

    assert_eq!(summary.exported_sealed_domains.len(), 1);
    let domain = &summary.exported_sealed_domains[0];

    // Identity
    let expected_id = SealedDomainId::new(test_module_identity(), "Boolish");
    assert_eq!(domain.id, expected_id);
    assert_eq!(domain.exported_name.as_str(), "Boolish");

    // Constructors
    assert_eq!(domain.constructors.len(), 2);
    assert_eq!(domain.constructors[0].exported_name.as_str(), "Yes");
    assert_eq!(domain.constructors[0].fields.len(), 0);
    assert_eq!(domain.constructors[1].exported_name.as_str(), "No");
    assert_eq!(domain.constructors[1].fields.len(), 0);
}

#[test]
fn public_domain_has_correct_visibility() {
    let lowered = lower("pub sealed type domain Shape { Circle; Rect; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];
    assert_eq!(domain.visibility, ash_core::ast::Visibility::Public);
}

#[test]
fn private_domain_has_correct_visibility() {
    let lowered = lower("sealed type domain Priv { A; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];
    assert_eq!(domain.visibility, ash_core::ast::Visibility::Private);
}

#[test]
fn crate_visible_domain_has_correct_visibility() {
    let lowered = lower("pub(crate) sealed type domain Internal { X; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];
    assert_eq!(domain.visibility, ash_core::ast::Visibility::Crate);
}

#[test]
fn self_referencing_field_gets_structural_self_domain() {
    let lowered = lower("sealed type domain Nat { Zero; Succ<pred: Nat>; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];

    // Succ has one field "pred" referencing Nat
    let succ = &domain.constructors[1];
    assert_eq!(succ.exported_name.as_str(), "Succ");
    assert_eq!(succ.fields.len(), 1);

    let field = &succ.fields[0];
    assert_eq!(field.name.as_str(), "pred");
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert!(field.domain_constraint.is_some());
    let constraint = field.domain_constraint.as_ref().unwrap();
    assert_eq!(constraint.name.as_str(), "Nat");
}

#[test]
fn unconstrained_type_field_is_non_structural() {
    let lowered = lower("sealed type domain Pair { MkPair<first: Type, second: Type>; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];

    let ctor = &domain.constructors[0];
    assert_eq!(ctor.fields.len(), 2);

    for field in &ctor.fields {
        assert_eq!(
            field.structural_status,
            StructuralFieldStatus::NonStructural
        );
        assert!(field.domain_constraint.is_none());
    }
}

#[test]
fn cross_domain_reference_is_non_structural() {
    let source = r"sealed type domain Alpha { A; }
sealed type domain Beta { B<alpha: Alpha>; }
";
    let lowered = lower(source);
    assert_eq!(lowered.summary.exported_sealed_domains.len(), 2);

    let beta = &lowered.summary.exported_sealed_domains[1];
    assert_eq!(beta.exported_name.as_str(), "Beta");

    let b_ctor = &beta.constructors[0];
    assert_eq!(b_ctor.exported_name.as_str(), "B");
    assert_eq!(b_ctor.fields.len(), 1);

    let field = &b_ctor.fields[0];
    assert_eq!(field.name.as_str(), "alpha");
    assert_eq!(
        field.structural_status,
        StructuralFieldStatus::NonStructural
    );
    assert!(field.domain_constraint.is_some());
    let constraint = field.domain_constraint.as_ref().unwrap();
    assert_eq!(constraint.name.as_str(), "Alpha");
    // Should be same-module cross-domain reference
    assert_eq!(constraint.module, test_module_identity());
}

#[test]
fn domain_constructor_identities_are_correct() {
    let lowered = lower("sealed type domain Color { Red; Green; Blue; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];

    let domain_id = SealedDomainId::new(test_module_identity(), "Color");
    let expected_ids: Vec<_> = ["Red", "Green", "Blue"]
        .iter()
        .map(|name| DomainConstructorId::new(domain_id.clone(), *name))
        .collect();

    for (ctor, expected) in domain.constructors.iter().zip(expected_ids.iter()) {
        assert_eq!(&ctor.id, expected);
    }
}

#[test]
fn source_anchors_are_populated() {
    let lowered = lower("pub sealed type domain Flag { On; Off; }\n");
    let domain = &lowered.summary.exported_sealed_domains[0];

    // Domain anchor
    assert!(domain.anchor.span.is_some());
    assert!(
        domain.anchor.label.contains("sealed type domain Flag"),
        "domain anchor label should mention sealed domain, got: {}",
        domain.anchor.label,
    );

    // Constructor anchors
    for ctor in &domain.constructors {
        assert!(ctor.anchor.span.is_some());
        assert!(
            ctor.anchor.label.contains("domain constructor"),
            "constructor anchor label should mention constructor, got: {}",
            ctor.anchor.label,
        );
    }
}

#[test]
fn summary_version_advances_when_sealed_domains_present() {
    let lowered = lower("sealed type domain Tag { A; }\n");
    assert_eq!(
        lowered.summary.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        "summary version should advance to V2 when sealed domains are present"
    );
}

#[test]
fn summary_version_remains_v1_without_sealed_domains() {
    let lowered = lower("type Status = Pending | Done;\n");
    assert_eq!(
        lowered.summary.version,
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        "summary version should remain V1 when no sealed domains"
    );
}

#[test]
fn ordinary_type_lowering_still_works_alongside_sealed_domains() {
    let source = r"type Status = Pending | Done;
sealed type domain Tag { A; B; }
";
    let lowered = lower(source);

    // Ordinary type still lowered
    assert_eq!(lowered.type_defs.len(), 1);
    assert_eq!(lowered.type_defs[0].name, "Status");
    assert_eq!(lowered.summary.exported_types.len(), 1);
    assert_eq!(lowered.summary.exported_constructors.len(), 2);

    // Sealed domain also present
    assert_eq!(lowered.summary.exported_sealed_domains.len(), 1);

    // Version should be V2 because sealed domain is present
    assert_eq!(
        lowered.summary.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );
}

#[test]
fn multiple_sealed_domains_are_all_lowered() {
    let source = r"sealed type domain A { X; }
sealed type domain B { Y; Z; }
";
    let lowered = lower(source);

    assert_eq!(lowered.summary.exported_sealed_domains.len(), 2);
    assert_eq!(
        lowered.summary.exported_sealed_domains[0]
            .exported_name
            .as_str(),
        "A"
    );
    assert_eq!(
        lowered.summary.exported_sealed_domains[1]
            .exported_name
            .as_str(),
        "B"
    );
    assert_eq!(
        lowered.summary.exported_sealed_domains[1]
            .constructors
            .len(),
        2
    );
}

#[test]
fn complex_domain_with_mixed_fields() {
    let source = "pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }\n";
    let lowered = lower(source);
    let domain = &lowered.summary.exported_sealed_domains[0];

    assert_eq!(domain.visibility, ash_core::ast::Visibility::Public);
    assert_eq!(domain.constructors.len(), 2);

    // Nil - unit constructor
    let nil = &domain.constructors[0];
    assert_eq!(nil.exported_name.as_str(), "Nil");
    assert!(nil.fields.is_empty());

    // Cons - two fields
    let cons = &domain.constructors[1];
    assert_eq!(cons.exported_name.as_str(), "Cons");
    assert_eq!(cons.fields.len(), 2);

    // head: Type (unconstrained)
    let head = &cons.fields[0];
    assert_eq!(head.name.as_str(), "head");
    assert_eq!(head.structural_status, StructuralFieldStatus::NonStructural);
    assert!(head.domain_constraint.is_none());

    // tail: TypeList (self-referencing, structural)
    let tail = &cons.fields[1];
    assert_eq!(tail.name.as_str(), "tail");
    assert_eq!(
        tail.structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert!(tail.domain_constraint.is_some());
    assert_eq!(
        tail.domain_constraint.as_ref().unwrap().name.as_str(),
        "TypeList"
    );
}
