//! Integration tests for sealed-domain summary transport through engine paths.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, StructuralFieldStatus, SummaryVersion,
};
use ash_engine::module_loader::{check_importable_module_file, load_ordinary_file};

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
fn selected_ordinary_type_import_does_not_transport_unrelated_sealed_domains() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("source.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub type Box = MkBox { value: Int };
pub sealed type domain Unrelated { Marker; }
",
    )
    .expect("write module");
    std::fs::write(&caller, "use source::{Box}\nfn main() { 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("selected ordinary type import should load");

    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .all(|summary| summary.exported_sealed_domains.is_empty()),
        "selected ordinary type import must not transport unrelated sealed domains"
    );
}

#[test]
fn selected_ordinary_constructor_import_does_not_transport_unrelated_sealed_domains() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("source.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub type Box = MkBox { value: Int };
pub sealed type domain Unrelated { Marker; }
",
    )
    .expect("write module");
    std::fs::write(&caller, "use source::{MkBox}\nfn main() { 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("selected constructor import should load");

    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .all(|summary| summary.exported_sealed_domains.is_empty()),
        "selected ordinary constructor import must not transport unrelated sealed domains"
    );
}

#[test]
fn public_sealed_domain_referencing_private_domain_is_rejected_at_export_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");

    std::fs::write(
        &module,
        r"sealed type domain Private { Hidden; }
pub sealed type domain Public { Exposes<field: Private>; }
",
    )
    .expect("write module");

    let err = check_importable_module_file(&module)
        .expect_err("public sealed domain must not expose private domain field");
    let msg = err.to_string();
    assert!(
        msg.contains("Public") && msg.contains("Private"),
        "error should name the leaky domains: {msg}"
    );
}

#[test]
fn inline_module_sealed_domains_are_rejected() {
    let source = r"mod inner {
sealed type domain Bad { X; }
}";
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("inline-domain.ash");
    std::fs::write(&path, source).expect("write inline-domain fixture");
    let result = check_importable_module_file(&path);
    assert!(
        result.is_err(),
        "SPEC-059 requires the engine module boundary to reject inline-module sealed-domain declarations"
    );
}
