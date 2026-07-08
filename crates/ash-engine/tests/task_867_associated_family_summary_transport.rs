//! TASK-867: engine transport/reconciliation for public associated-family summaries.

use ash_core::semantic_summary::SummaryVersion;
use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

const fn public_provider_with_associated_family() -> &'static str {
    r"
pub type Token = Int;
pub interface Iterator<I> { sealed type family Item: Type }
impl<T> Iterator<T> { type Item = T; }
"
}

const fn provider_with_hidden_helper_family() -> &'static str {
    r"
interface Helper<I> { sealed type family Mid: Type }
impl<T> Helper<T> { type Mid = T; }
pub interface Public<I> { sealed type family Out: Type }
impl<T> Public<T> { type Out = <Helper<T>>::Mid; }
"
}

const fn provider_with_transitive_hidden_helper_families() -> &'static str {
    r"
interface Leaf<I> { sealed type family LeafOut: Type }
impl<T> Leaf<T> { type LeafOut = T; }
interface Helper<I> { sealed type family Mid: Type }
impl<T> Helper<T> { type Mid = <Leaf<T>>::LeafOut; }
pub interface Public<I> { sealed type family Out: Type }
impl<T> Public<T> { type Out = <Helper<T>>::Mid; }
"
}

#[test]
fn task_867_glob_import_transports_public_associated_family_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_provider_with_associated_family());
    write_file(
        &caller,
        r"use provider::*
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("glob import should load provider");
    let family_names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_associated_families.iter())
        .map(|family| family.visible_name.as_str())
        .collect::<Vec<_>>();

    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .any(|summary| summary.version == SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4),
        "associated-family transport must upgrade imported semantic summaries to V4"
    );
    assert!(
        family_names.contains(&"Item"),
        "glob import should transport public associated-family summary payloads: {family_names:?}"
    );
}

#[test]
fn task_867_real_export_transports_hidden_helper_family_payload() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, provider_with_hidden_helper_family());
    write_file(
        &caller,
        r"use provider::*
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("glob import should load helper-backed family");
    let family_names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_associated_families.iter())
        .map(|family| family.visible_name.as_str())
        .collect::<Vec<_>>();

    assert!(
        family_names.contains(&"Out"),
        "public family payload should be transported: {family_names:?}"
    );
    assert!(
        family_names.contains(&"$ash_dependency$Mid"),
        "hidden helper family payload must be transported for normalizer availability without source visibility: {family_names:?}"
    );
}

#[test]
fn task_867_named_import_transports_transitive_hidden_helper_family_payloads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, provider_with_transitive_hidden_helper_families());
    write_file(
        &caller,
        r"use provider::{Out}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("named import should load selected family with transitive hidden helpers");
    let family_names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_associated_families.iter())
        .map(|family| family.visible_name.as_str())
        .collect::<Vec<_>>();

    assert!(
        family_names.contains(&"Out"),
        "selected public family payload should be transported: {family_names:?}"
    );
    assert!(
        family_names.contains(&"$ash_dependency$Mid"),
        "selected summary must include direct hidden helper payload: {family_names:?}"
    );
    assert!(
        family_names.contains(&"$ash_dependency$LeafOut"),
        "selected summary must include transitive hidden helper payload: {family_names:?}"
    );
}

#[test]
fn task_867_pub_use_reexport_preserves_associated_family_identity_and_payload() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_provider_with_associated_family());
    write_file(&facade, "pub use provider::*;\n");
    write_file(
        &caller,
        r"use facade::*
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("re-exported glob import should load");
    let families = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_associated_families.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        families.len(),
        1,
        "pub-use transport should preserve exactly one canonical associated-family payload, got {families:?}"
    );
    assert_eq!(families[0].visible_name, "Item");
    assert_eq!(families[0].interface_identity.name.as_str(), "Iterator");
    assert_eq!(families[0].member_identity.name.as_str(), "Item");
}
