//! TASK-2025: module-loader provider binding and sanitizer regressions.
//!
//! These controls exercise the public loader boundary only.  They deliberately
//! make no claim about provider installation, admission, or runtime dispatch.

use ash_core::semantic_summary::{
    EffectRowBindingExposure, EffectRowClosureStatus, EffectRowExportSummary, SummaryVersion,
};
use ash_engine::module_loader::load_ordinary_file;
use ash_typeck::TypeEnv;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write Ash fixture");
}

fn imported_row<'a>(
    loaded: &'a ash_engine::module_loader::LoadedOrdinaryFile,
    visible_name: &str,
) -> &'a EffectRowExportSummary {
    loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .find(|row| row.binding.visible_name == visible_name)
        .unwrap_or_else(|| panic!("missing imported provider binding '{visible_name}'"))
}

fn closure_digest(
    loaded: &ash_engine::module_loader::LoadedOrdinaryFile,
    visible_name: &str,
) -> String {
    imported_row(loaded, visible_name)
        .closure_metadata
        .as_ref()
        .unwrap_or_else(|| panic!("missing V8 closure metadata for '{visible_name}'"))
        .public_closure_digest
        .clone()
}

#[test]
fn named_glob_and_public_facade_imports_preserve_one_provider_with_distinct_bindings() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let named_caller = dir.path().join("named_caller.ash");
    let glob_caller = dir.path().join("glob_caller.ash");
    let facade = dir.path().join("facade.ash");
    let facade_caller = dir.path().join("facade_caller.ash");

    write_file(
        &provider,
        "pub effect alias Audit = { evidence audit_log };\n",
    );
    write_file(
        &named_caller,
        "use provider::{Audit as NamedAudit}\nfn main() { 0 }\n",
    );
    write_file(&glob_caller, "use provider::*\nfn main() { 0 }\n");
    write_file(&facade, "pub use provider::{Audit as FacadeAudit};\n");
    write_file(
        &facade_caller,
        "use facade::{FacadeAudit}\nfn main() { 0 }\n",
    );

    let named = load_ordinary_file(&named_caller).expect("named import loads");
    let glob = load_ordinary_file(&glob_caller).expect("glob import loads");
    let facade = load_ordinary_file(&facade_caller).expect("public facade import loads");

    for loaded in [&named, &glob, &facade] {
        assert!(
            loaded.imported_semantic_summaries.iter().any(|summary| {
                summary.version == SummaryVersion::STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8
                    && !summary.exported_effect_rows.is_empty()
            }),
            "every import path must transport the V8 structural provider-binding contract"
        );
    }

    let named_row = imported_row(&named, "NamedAudit");
    let glob_row = imported_row(&glob, "Audit");
    let facade_row = imported_row(&facade, "FacadeAudit");

    assert_eq!(named_row.provider, glob_row.provider);
    assert_eq!(named_row.provider, facade_row.provider);
    assert_eq!(named_row.binding.provider, named_row.provider);
    assert_eq!(glob_row.binding.provider, glob_row.provider);
    assert_eq!(facade_row.binding.provider, facade_row.provider);
    assert_eq!(
        named_row.binding.exposure,
        EffectRowBindingExposure::NamedImport
    );
    assert_eq!(
        glob_row.binding.exposure,
        EffectRowBindingExposure::GlobImport
    );
    assert_eq!(
        facade_row.binding.exposure,
        EffectRowBindingExposure::PublicReExport,
        "a facade-selected binding retains its public re-export exposure rather than becoming a provider owned by the facade or final caller"
    );
    assert_eq!(named_row.binding.visible_name, "NamedAudit");
    assert_eq!(glob_row.binding.visible_name, "Audit");
    assert_eq!(facade_row.binding.visible_name, "FacadeAudit");
    assert_eq!(
        named_row.binding.closure_status,
        EffectRowClosureStatus::Complete
    );
    assert_eq!(
        glob_row.binding.closure_status,
        EffectRowClosureStatus::Complete
    );
    assert_eq!(
        facade_row.binding.closure_status,
        EffectRowClosureStatus::Complete
    );
}

#[test]
fn inaccessible_private_row_dependency_rejects_before_loader_summary_transport_without_leaking() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    let private_token = "TASK2025_PRIVATE_ROW_DO_NOT_LEAK";

    write_file(
        &provider,
        &format!(
            "effect alias {private_token} = {{ evidence hidden }};\npub effect alias Published = {{ {private_token} }};\n"
        ),
    );
    write_file(&caller, "use provider::{Published}\nfn main() { 0 }\n");

    let error = load_ordinary_file(&caller).expect_err(
        "a private dependency must reject before an imported summary or cache entry exists",
    );
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("private-dependency-export-failure")
            || diagnostic.contains("inaccessible"),
        "the loader must report the opaque public-boundary classification: {diagnostic}"
    );
    assert!(
        !diagnostic.contains(private_token),
        "the loader diagnostic must not disclose the provider-private row token: {diagnostic}"
    );
}

#[test]
fn qualified_symbolic_operation_row_item_is_not_treated_as_a_named_row_dependency() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");

    write_file(&provider, "pub effect alias Audit = { PosixFs::read };\n");
    write_file(&caller, "use provider::{Audit}\nfn main() { 0 }\n");

    let loaded = load_ordinary_file(&caller).expect(
        "a qualified symbolic operation remains raw row content, not an inaccessible alias",
    );
    let row = imported_row(&loaded, "Audit");
    assert_eq!(row.binding.closure_status, EffectRowClosureStatus::Complete);
    assert_eq!(row.row_items.len(), 1);
    assert_eq!(row.row_items[0].text, "PosixFs::read");
}

#[test]
fn sanitizer_transports_the_complete_public_closure_in_provider_source_order() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");

    write_file(
        &provider,
        "pub effect alias First = { evidence first };\npub effect alias Audit = { First };\npub effect alias Unselected = { evidence ignored };\n",
    );
    write_file(&caller, "use provider::{Audit}\nfn main() { 0 }\n");

    let loaded = load_ordinary_file(&caller).expect("the complete public closure loads");
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .map(|row| row.binding.visible_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec!["First", "Audit"],
        "closure selection must finish by preserving provider declaration order, not root-first traversal or unrelated exports"
    );
}

#[test]
fn sanitizer_closure_digest_changes_for_root_public_row_content_under_the_same_provider_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&caller, "use provider::{Audit}\nfn main() { 0 }\n");

    write_file(
        &provider,
        "pub effect alias Audit = { evidence original_audit };\n",
    );
    let first = load_ordinary_file(&caller).expect("first public closure loads");
    let first_row = imported_row(&first, "Audit");
    let first_provider = first_row.provider.clone();
    let first_digest = closure_digest(&first, "Audit");

    write_file(
        &provider,
        "pub effect alias Audit = { evidence changed_audit };\n",
    );
    let second = load_ordinary_file(&caller).expect("changed public closure loads");
    let second_row = imported_row(&second, "Audit");

    assert_eq!(first_provider, second_row.provider);
    assert_ne!(first_digest, closure_digest(&second, "Audit"));
    assert_ne!(
        first
            .imported_semantic_summaries
            .iter()
            .find(|summary| !summary.exported_effect_rows.is_empty())
            .expect("first selected summary")
            .semantic_cache_key(),
        second
            .imported_semantic_summaries
            .iter()
            .find(|summary| !summary.exported_effect_rows.is_empty())
            .expect("second selected summary")
            .semantic_cache_key(),
        "a cache key must change with the public root row contract"
    );
}

#[test]
fn sanitizer_closure_digest_changes_for_transitive_public_row_content_under_the_same_provider_identities()
 {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&caller, "use provider::{Audit}\nfn main() { 0 }\n");

    write_file(
        &provider,
        "pub effect alias Dependency = { evidence original_dependency };\npub effect alias Audit = { Dependency };\n",
    );
    let first = load_ordinary_file(&caller).expect("first transitive public closure loads");
    let first_rows = first
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .collect::<Vec<_>>();
    let first_providers = first_rows
        .iter()
        .map(|row| row.provider.clone())
        .collect::<Vec<_>>();
    let first_digest = closure_digest(&first, "Audit");

    write_file(
        &provider,
        "pub effect alias Dependency = { evidence changed_dependency };\npub effect alias Audit = { Dependency };\n",
    );
    let second = load_ordinary_file(&caller).expect("changed transitive public closure loads");
    let second_rows = second
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .collect::<Vec<_>>();

    assert_eq!(
        first_providers,
        second_rows
            .iter()
            .map(|row| row.provider.clone())
            .collect::<Vec<_>>()
    );
    assert_ne!(first_digest, closure_digest(&second, "Audit"));
}

#[test]
fn sanitizer_closure_digest_changes_for_public_row_classification_and_is_stable_for_an_equivalent_closure()
 {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&caller, "use provider::{Audit}\nfn main() { 0 }\n");

    write_file(
        &provider,
        "pub effect alias Audit = { evidence audit_log };\n",
    );
    let alias_first = load_ordinary_file(&caller).expect("alias closure loads");
    let alias_second = load_ordinary_file(&caller).expect("equivalent alias closure loads");
    let alias_row = imported_row(&alias_first, "Audit");
    assert_eq!(
        alias_row.provider,
        imported_row(&alias_second, "Audit").provider
    );
    assert_eq!(
        closure_digest(&alias_first, "Audit"),
        closure_digest(&alias_second, "Audit"),
        "equivalent public closures must produce a stable digest"
    );

    write_file(
        &provider,
        "pub effect group Audit = { evidence audit_log };\n",
    );
    let group = load_ordinary_file(&caller).expect("group closure loads");
    let group_row = imported_row(&group, "Audit");

    assert_eq!(alias_row.provider, group_row.provider);
    assert_ne!(alias_row.classification, group_row.classification);
    assert_ne!(
        closure_digest(&alias_first, "Audit"),
        closure_digest(&group, "Audit")
    );
}

fn conflicting_visible_binding_diagnostic(imports: &[&str]) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first.ash");
    let second = dir.path().join("second.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &first,
        "pub effect alias Audit = { evidence first_proof };\n",
    );
    write_file(
        &second,
        "pub effect alias Audit = { evidence second_proof };\n",
    );
    write_file(
        &caller,
        &format!("{}\nfn main() {{ 0 }}\n", imports.join("\n")),
    );

    load_ordinary_file(&caller)
        .expect_err("two providers may not publish the same caller-visible effect-row binding")
        .to_string()
}

#[test]
fn two_provider_bindings_for_one_visible_name_reject_in_either_source_order_before_publication() {
    let first_then_second = conflicting_visible_binding_diagnostic(&[
        "use first::{Audit as Shared}",
        "use second::{Audit as Shared}",
    ]);
    let second_then_first = conflicting_visible_binding_diagnostic(&[
        "use second::{Audit as Shared}",
        "use first::{Audit as Shared}",
    ]);

    assert!(
        first_then_second.contains("import-order-conflict")
            || first_then_second.contains("conflict"),
        "the first source order must reject before a caller summary can publish: {first_then_second}"
    );
    assert_eq!(
        first_then_second, second_then_first,
        "the conflict class must not depend on which provider binding appeared first"
    );
}

#[test]
fn duplicate_visible_binding_to_the_same_provider_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        "pub effect alias Audit = { evidence audit_proof };\n",
    );
    write_file(
        &caller,
        "use provider::{Audit as Shared}\nuse provider::{Audit as Shared}\nfn main() { 0 }\n",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("repeating an identical provider binding must not be an order-sensitive conflict");
    let shared_rows = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .filter(|row| row.binding.visible_name == "Shared")
        .collect::<Vec<_>>();
    assert_eq!(shared_rows.len(), 1);
    assert_eq!(shared_rows[0].binding.provider, shared_rows[0].provider);
}

#[test]
fn same_provider_binding_and_closure_through_two_facades_is_idempotent_despite_facade_anchors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let first_facade = dir.path().join("first_facade.ash");
    let second_facade = dir.path().join("second_facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        "pub effect alias Audit = { evidence audit_proof };\n",
    );
    write_file(&first_facade, "pub use provider::{Audit as Shared};\n");
    write_file(&second_facade, "pub use provider::{Audit as Shared};\n");
    write_file(
        &caller,
        "use first_facade::{Shared}\nuse second_facade::{Shared}\nfn main() { 0 }\n",
    );

    let loaded = load_ordinary_file(&caller).expect(
        "the same provider/binding/closure remains idempotent even when separate facades supply distinct local anchors",
    );
    let shared_rows = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .filter(|row| row.binding.visible_name == "Shared")
        .collect::<Vec<_>>();
    assert_eq!(
        shared_rows.len(),
        1,
        "facade-local summary IDs or anchors must not manufacture duplicate provider bindings"
    );
    assert_eq!(shared_rows[0].provider.module.path, vec!["provider"]);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summaries(&loaded.imported_semantic_summaries)
        .expect("equivalent facade transports must register idempotently as one visible binding");
    assert_eq!(
        env.lookup_effect_row_export("Shared")
            .expect("the idempotent binding remains available")
            .provider,
        shared_rows[0].provider
    );
}

fn conflicting_facade_binding_diagnostic(facade_imports: &[&str]) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first.ash");
    let second = dir.path().join("second.ash");
    let first_facade = dir.path().join("first_facade.ash");
    let second_facade = dir.path().join("second_facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &first,
        "pub effect alias Audit = { evidence first_proof };\n",
    );
    write_file(
        &second,
        "pub effect alias Audit = { evidence second_proof };\n",
    );
    write_file(&first_facade, "pub use first::{Audit as Shared};\n");
    write_file(&second_facade, "pub use second::{Audit as Shared};\n");
    write_file(
        &caller,
        &format!("{}\nfn main() {{ 0 }}\n", facade_imports.join("\n")),
    );

    load_ordinary_file(&caller)
        .expect_err("facades may not publish one visible binding for distinct providers")
        .to_string()
}

#[test]
fn conflicting_facade_public_reexports_reject_in_either_source_order_before_publication() {
    let first_then_second = conflicting_facade_binding_diagnostic(&[
        "use first_facade::{Shared}",
        "use second_facade::{Shared}",
    ]);
    let second_then_first = conflicting_facade_binding_diagnostic(&[
        "use second_facade::{Shared}",
        "use first_facade::{Shared}",
    ]);

    assert!(
        first_then_second.contains("import-order-conflict")
            || first_then_second.contains("conflict"),
        "conflicting facade re-exports must reject before caller-summary publication: {first_then_second}"
    );
    assert_eq!(
        first_then_second, second_then_first,
        "facade source order must not affect the public conflict classification"
    );
}

#[test]
fn selected_closure_cannot_collide_with_its_root_alias_before_summary_publication() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    let closure_detail = "TASK2025_CLOSURE_CONTENT_NOT_DIAGNOSTIC";
    write_file(
        &provider,
        &format!(
            "pub effect alias Dependency = {{ evidence {closure_detail} }};\npub effect group Audit = {{ Dependency }};\n"
        ),
    );
    write_file(
        &caller,
        "use provider::{Audit as Dependency}\nfn main() { 0 }\n",
    );

    let diagnostic = load_ordinary_file(&caller)
        .expect_err(
            "a selected closure must reject when its transported dependency collides with the root alias before a public summary/cache entry exists",
        )
        .to_string();
    assert!(
        diagnostic.contains("import-order-conflict") || diagnostic.contains("conflict"),
        "the loader must classify the self-collision generically: {diagnostic}"
    );
    assert!(
        !diagnostic.contains(closure_detail),
        "the public boundary must not reveal selected-closure row content: {diagnostic}"
    );
}
