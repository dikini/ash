//! Integration tests for sealed-domain summary transport through engine paths.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, StructuralFieldStatus, SummaryVersion,
};

fn test_module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(1),
        vec!["test".into()],
        ModuleSourceOrigin::File("test.ash".into()),
    )
}

fn lower(source: &str) -> ash_parser::lower::LoweredTypeMetadata {
    let module = ash_parser::parse_surface_file(source).expect("should parse");
    ash_parser::lower::lower_module_type_metadata(&module, test_module_identity())
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

// --- Lowering + summary correctness ---

#[test]
fn public_sealed_domain_appears_in_summary() {
    let metadata = lower("pub sealed type domain Color { Red; Green; Blue; }");
    let domain = find_domain(&metadata, "Color").expect("domain should exist");
    assert_eq!(domain.constructors.len(), 3);
    assert_eq!(domain.visibility, CoreVisibility::Public);
}

#[test]
fn summary_version_v2_with_sealed_domains() {
    let metadata = lower("pub sealed type domain D { X; }");
    assert_eq!(
        metadata.summary.version,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    );
}

#[test]
fn summary_version_v1_without_sealed_domains() {
    let metadata = lower("pub type Foo = Bar { x: Int };");
    assert_eq!(
        metadata.summary.version,
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1
    );
    assert!(metadata.summary.exported_sealed_domains.is_empty());
}

#[test]
fn private_domain_has_inherited_visibility() {
    let metadata = lower("sealed type domain Internal { A; B; }");
    let domain = find_domain(&metadata, "Internal").expect("domain should exist");
    assert_eq!(domain.visibility, CoreVisibility::Private);
}

#[test]
fn domain_with_recursive_field_has_structural_status() {
    let metadata = lower("pub sealed type domain Nat { Zero; Succ<pred: Nat>; }");
    let domain = find_domain(&metadata, "Nat").expect("domain should exist");
    let succ = &domain.constructors[1];
    assert_eq!(succ.fields.len(), 1);
    assert_eq!(
        succ.fields[0].structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
    assert!(succ.fields[0].domain_constraint.is_some());
}

#[test]
fn domain_identity_is_module_anchored() {
    let metadata = lower("pub sealed type domain MyDomain { X; }");
    let domain = find_domain(&metadata, "MyDomain").expect("domain should exist");
    assert_eq!(domain.id.name, "MyDomain");
    assert_eq!(domain.id.module.module_id, ModuleId(1));
}

#[test]
fn multiple_sealed_domains_all_lowered() {
    let metadata = lower(
        "pub sealed type domain Color { Red; Blue; }\npub sealed type domain Shape { Circle; Square; }",
    );
    assert_eq!(metadata.summary.exported_sealed_domains.len(), 2);
}

#[test]
fn ordinary_types_alongside_sealed_domains() {
    let metadata = lower(
        "pub type Option<T> = Some { value: T } | None;\npub sealed type domain Dir { North; South; }",
    );
    assert!(
        metadata
            .summary
            .exported_types
            .iter()
            .any(|t| t.exported_name == "Option"),
        "ordinary type should still appear"
    );
    assert!(find_domain(&metadata, "Dir").is_some());
}

#[test]
fn sealed_domain_constructor_fields_preserve_order() {
    let metadata = lower("pub sealed type domain Pair { MkPair<first: Type, second: Type>; }");
    let domain = find_domain(&metadata, "Pair").expect("domain should exist");
    let ctor = &domain.constructors[0];
    assert_eq!(ctor.fields.len(), 2);
    assert_eq!(ctor.fields[0].name, "first");
    assert_eq!(ctor.fields[1].name, "second");
}

// --- Inline-module rejection ---

#[test]
fn inline_module_sealed_domains_do_not_appear_in_file_definitions() {
    let source = r"mod inner {
sealed type domain Bad { X; }
}";
    let result = ash_parser::parse_surface_file(source);
    // Either parse fails (rejection) or sealed domains don't appear at file level
    if let Ok(module) = result {
        // The inline module's sealed domain should NOT appear in the top-level definitions
        assert!(
            module
                .definitions
                .iter()
                .all(|d| !matches!(d, ash_parser::surface::Definition::SealedDomain(_))),
            "inline module sealed domains should not appear in file-level definitions"
        );
    }
}
