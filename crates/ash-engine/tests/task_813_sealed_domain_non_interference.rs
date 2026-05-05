//! TASK-813: Sealed-domain non-interference tests for ash-engine.
//!
//! Verifies that the V2 sealed-domain code path does not interfere with
//! existing Phase 109/110 engine behavior — ordinary types, summaries,
//! and module transport remain unchanged.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, StructuralFieldStatus, SummaryVersion,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_module_identity(id: u32) -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(id as usize),
        vec!["test".into()],
        ModuleSourceOrigin::File(format!("test_{id}.ash")),
    )
}

fn lower(source: &str, identity: ModuleIdentity) -> ash_parser::lower::LoweredTypeMetadata {
    let module = ash_parser::parse_surface_file(source).expect("should parse");
    ash_parser::lower::lower_module_type_metadata(&module, identity)
}

fn find_domain<'a>(
    metadata: &'a ash_parser::lower::LoweredTypeMetadata,
    name: &str,
) -> Option<&'a ash_core::semantic_summary::SealedDomainSummary> {
    metadata
        .summary
        .exported_sealed_domains
        .iter()
        .find(|d| d.exported_name == name)
}

// ---------------------------------------------------------------------------
// Non-interference: ordinary types unchanged by sealed-domain addition
// ---------------------------------------------------------------------------

#[test]
fn ordinary_types_unchanged_by_sealed_domain_addition() {
    // Register a module with only ordinary types.
    let id_a = test_module_identity(100);
    let metadata_a = lower(
        "type Status = Pending | Done;\npub type Pair = MkPair { a: Int, b: Int };",
        id_a,
    );

    // Ordinary types are present, no sealed domains.
    assert_eq!(metadata_a.type_defs.len(), 2);
    assert!(metadata_a.summary.exported_sealed_domains.is_empty());
    assert_eq!(
        metadata_a.summary.version,
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1
    );

    // Now register a module with sealed domains.
    let id_b = test_module_identity(101);
    let metadata_b = lower("pub sealed type domain Color { Red; Green; Blue; }", id_b);

    // Sealed domain is present.
    assert_eq!(metadata_b.summary.exported_sealed_domains.len(), 1);
    assert_eq!(
        metadata_b.summary.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );

    // The first module's ordinary types are still intact (unrelated metadata).
    assert_eq!(metadata_a.type_defs.len(), 2);
    assert!(metadata_a.summary.exported_sealed_domains.is_empty());
}

#[test]
fn sealed_domains_do_not_leak_into_type_definitions() {
    let id = test_module_identity(200);
    let metadata = lower(
        "pub sealed type domain Direction { North; South; East; West; }\ntype Pair = MkPair { x: Int };",
        id,
    );

    // Ordinary type is in type_defs.
    assert_eq!(metadata.type_defs.len(), 1);
    assert_eq!(metadata.type_defs[0].name, "Pair");

    // Sealed domains are NOT in type_defs; they live in summary.exported_sealed_domains.
    assert!(
        !metadata.type_defs.iter().any(|t| t.name == "Direction"),
        "sealed domains must not appear in type_defs"
    );

    // Sealed domain is in the summary.
    assert!(find_domain(&metadata, "Direction").is_some());
}

#[test]
fn v1_summary_unaffected_by_v2_code_path() {
    // A module with no sealed domains must still produce a V1 summary.
    let id = test_module_identity(300);
    let metadata = lower(
        "pub type Result<T, E> = Ok { value: T } | Err { error: E };",
        id,
    );

    assert_eq!(
        metadata.summary.version,
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        "V1 summary must remain unchanged when no sealed domains are present"
    );
    assert!(metadata.summary.exported_sealed_domains.is_empty());

    // Ordinary types and constructors still populated.
    assert_eq!(metadata.summary.exported_types.len(), 1);
    assert_eq!(metadata.summary.exported_constructors.len(), 2);
}

// ---------------------------------------------------------------------------
// Summary correctness with mixed content
// ---------------------------------------------------------------------------

#[test]
fn sealed_domain_summary_fields_match_lowering_output() {
    let id = test_module_identity(400);
    let metadata = lower("pub sealed type domain Nat { Zero; Succ<pred: Nat>; }", id);

    let domain = find_domain(&metadata, "Nat").expect("domain should exist");
    assert_eq!(domain.visibility, CoreVisibility::Public);
    assert_eq!(domain.constructors.len(), 2);

    let zero = &domain.constructors[0];
    assert_eq!(zero.exported_name.as_str(), "Zero");
    assert!(zero.fields.is_empty());

    let succ = &domain.constructors[1];
    assert_eq!(succ.exported_name.as_str(), "Succ");
    assert_eq!(succ.fields.len(), 1);
    assert_eq!(
        succ.fields[0].structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
}

#[test]
fn cross_domain_reference_in_summary() {
    let id = test_module_identity(500);
    let metadata = lower(
        "pub sealed type domain Alpha { A; }\npub sealed type domain Beta { B<alpha: Alpha>; }",
        id,
    );

    let beta = find_domain(&metadata, "Beta").expect("Beta should exist");
    let b_ctor = &beta.constructors[0];
    assert_eq!(b_ctor.fields.len(), 1);
    assert_eq!(
        b_ctor.fields[0].structural_status,
        StructuralFieldStatus::NonStructural,
        "cross-domain reference should be non-structural"
    );
    assert!(b_ctor.fields[0].domain_constraint.is_some());
    assert_eq!(
        b_ctor.fields[0]
            .domain_constraint
            .as_ref()
            .unwrap()
            .name
            .as_str(),
        "Alpha"
    );
}

#[test]
fn ordinary_type_constructors_preserved_with_sealed_domains() {
    let id = test_module_identity(600);
    let metadata = lower(
        "type Bool = True | False;\npub sealed type domain Dir { North; South; }",
        id,
    );

    // Ordinary constructors should still be present
    assert_eq!(metadata.summary.exported_constructors.len(), 2);
    let ctor_names: Vec<_> = metadata
        .summary
        .exported_constructors
        .iter()
        .map(|c| c.exported_name.as_str())
        .collect();
    assert!(ctor_names.contains(&"True"));
    assert!(ctor_names.contains(&"False"));

    // Sealed domain constructors are NOT in exported_constructors
    assert!(
        !ctor_names.contains(&"North"),
        "sealed domain constructors must not appear in ordinary exported_constructors"
    );
}
