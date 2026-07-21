//! TASK-849: engine transport/reconciliation for public type-computation summaries.

use ash_core::semantic_summary::SummaryVersion;
use ash_engine::Engine;
use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

const fn public_list_module_with_helpers() -> &'static str {
    r"
pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }
pub sealed type domain Unused { UnusedCase; }

pub type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
pub type fn Sibling(xs: TypeList) -> TypeList { case Sibling<xs> = xs; }
"
}

const fn public_list_module_with_ordinary_type_dependency() -> &'static str {
    r"
pub type Token = Int;
pub sealed type domain TypeList { Nil; }

pub type fn TokenResult(xs: TypeList) -> Type { case TokenResult<Nil> = Token; }
"
}

#[test]
fn selected_type_function_import_hides_ordinary_dependency_source_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        public_list_module_with_ordinary_type_dependency(),
    );
    write_file(
        &caller,
        r"use provider::{TokenResult}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("selected type fn import loads");
    let type_names = loaded
        .imported_type_defs
        .iter()
        .map(|ty| ty.name.as_str())
        .collect::<Vec<_>>();
    assert!(
        !type_names.contains(&"Token"),
        "ordinary dependency type name must not become source-visible from selected type-function import: {type_names:?}"
    );
    let summary_type_names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .map(|ty| ty.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(summary_type_names.contains(&"$ash_dependency$Token"));
    assert!(!summary_type_names.contains(&"Token"));
}

#[test]
fn aliased_reexport_retains_summary_when_original_name_is_reexported_too() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &facade,
        "pub use provider::{UseHelper};\npub use provider::{UseHelper as AliasUseHelper};\n",
    );
    write_file(
        &caller,
        r"use facade::{AliasUseHelper}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("aliased re-export import loads");
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|type_function| type_function.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(
        names.contains(&"AliasUseHelper"),
        "aliased re-export selected import must transport a matching semantic summary: {names:?}"
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(&caller).expect("caller parses");
    engine
        .check(&mut application)
        .expect("aliased re-export imported head is backed by transported summary");
}

#[test]
fn named_type_function_import_transports_selected_head_and_public_helper_closure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &caller,
        r"use provider::{UseHelper}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("named public type fn import loads");
    let summary = loaded
        .imported_semantic_summaries
        .iter()
        .find(|summary| !summary.exported_type_functions.is_empty())
        .expect("public computation summary is transported");

    assert_eq!(summary.version, SummaryVersion::SPEC062_TYPE_COMPUTATION_V3);
    let names = summary
        .exported_type_functions
        .iter()
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(
        names.contains(&"UseHelper"),
        "selected head missing: {names:?}"
    );
    assert!(
        names.contains(&"$ash_dependency$Helper"),
        "helper closure head missing: {names:?}"
    );
    assert!(
        !names.contains(&"Sibling"),
        "unrelated sibling public type fn must not leak into named import closure: {names:?}"
    );
    let domains = summary
        .exported_sealed_domains
        .iter()
        .map(|domain| domain.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(domains.contains(&"TypeList"));
    assert!(
        !domains.contains(&"Unused"),
        "unrelated public sealed domain must not leak into named import closure: {domains:?}"
    );
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .all(|ty| ty.name != "Helper"),
        "helper type-function head must not become an ordinary source-visible type"
    );
    assert_eq!(
        loaded
            .imported_type_function_heads
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["UseHelper"],
        "only the selected type function should become source-visible"
    );
}

#[test]
fn glob_type_function_import_transports_all_public_computation_heads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &caller,
        r"use provider::*
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("glob public type fn import loads");
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();

    assert_eq!(names, vec!["Helper", "Sibling", "UseHelper"]);
    let mut visible = loaded
        .imported_type_function_heads
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    visible.sort_unstable();
    assert_eq!(visible, vec!["Helper", "Sibling", "UseHelper"]);
}

#[test]
fn pub_use_glob_reexport_preserves_type_function_summary_transport() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(&facade, "pub use provider::*;\n");
    write_file(
        &caller,
        r"use facade::{UseHelper}
fn main() { 0 }
",
    );

    let loaded =
        load_ordinary_file(&caller).expect("named re-exported public type fn import loads");
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();

    assert!(
        names.contains(&"UseHelper"),
        "selected re-export missing: {names:?}"
    );
    assert!(
        names.contains(&"$ash_dependency$Helper"),
        "helper closure missing through re-export: {names:?}"
    );
    assert!(
        !names.contains(&"Sibling"),
        "unrelated sibling must not leak through re-exported named import: {names:?}"
    );
    let domains = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_sealed_domains.iter())
        .map(|domain| domain.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(
        domains.contains(&"TypeList"),
        "sealed-domain dependency missing through re-export: {domains:?}"
    );
    assert!(
        !domains.contains(&"Unused"),
        "unrelated sealed-domain dependency must not leak through re-exported named import: {domains:?}"
    );
    assert_eq!(
        loaded
            .imported_type_function_heads
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["UseHelper"]
    );
}

#[test]
fn pub_use_glob_rejects_duplicate_type_function_exports() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = dir.path().join("a.ash");
    let b = dir.path().join("b.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &a,
        r"
pub sealed type domain DA { A; }
pub type fn F(x: DA) -> DA { case F<x> = x; }
",
    );
    write_file(
        &b,
        r"
pub sealed type domain DB { B; }
pub type fn F(x: DB) -> DB { case F<x> = x; }
",
    );
    write_file(&facade, "pub use a::*;\npub use b::*;\n");
    write_file(
        &caller,
        r"use facade::{F}
fn main() { 0 }
",
    );

    let error =
        load_ordinary_file(&caller).expect_err("duplicate type-function re-export rejected");
    assert!(
        error
            .to_string()
            .contains("duplicate exported type function 'F'"),
        "unexpected error: {error}"
    );
}

#[test]
fn named_type_function_import_can_select_multiple_explicit_heads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &caller,
        r"use provider::{UseHelper, Sibling}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("explicit sibling import loads");
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"UseHelper"));
    assert!(names.contains(&"$ash_dependency$Helper"));
    assert!(names.contains(&"Sibling"));

    let mut visible = loaded
        .imported_type_function_heads
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    visible.sort_unstable();
    assert_eq!(visible, vec!["Sibling", "UseHelper"]);
}

#[test]
fn dependency_helper_head_does_not_become_source_visible_import_selection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &caller,
        r"use provider::{UseHelper}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("named public type fn import loads");
    assert!(loaded.imported_callables.is_empty());
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .all(|ty| ty.name != "Helper"),
        "dependency helper should be transported only in computation summaries, not source-visible ordinary imports"
    );
}

#[test]
fn aliased_type_function_import_exposes_alias_without_helper_leakage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(
        &caller,
        r"use provider::{UseHelper as AliasUseHelper}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("aliased public type fn import loads");
    assert_eq!(
        loaded
            .imported_type_function_heads
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["AliasUseHelper"]
    );
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"AliasUseHelper"));
    assert!(names.contains(&"$ash_dependency$Helper"));
    assert!(!names.contains(&"UseHelper"));
    assert!(!names.contains(&"Helper"));
}

#[test]
fn selected_reexport_chain_keeps_dependency_helper_metadata_name_stable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let facade2 = dir.path().join("facade2.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, public_list_module_with_helpers());
    write_file(&facade, "pub use provider::{UseHelper};\n");
    write_file(&facade2, "pub use facade::{UseHelper};\n");
    write_file(
        &caller,
        r"use facade2::{UseHelper}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("chained selected re-export loads");
    let names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|tf| tf.exported_name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"UseHelper"));
    assert!(names.contains(&"$ash_dependency$Helper"));
    assert!(
        !names.contains(&"$ash_dependency$$ash_dependency$Helper"),
        "dependency helper metadata prefix must not accumulate across re-export chains: {names:?}"
    );
}
