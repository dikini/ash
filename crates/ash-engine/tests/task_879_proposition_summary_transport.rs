//! TASK-879: engine preserves public proposition facts through module-summary transport.

use ash_core::semantic_summary::SummaryVersion;
use ash_core::type_ir::{PropositionDeferredKind, PropositionOutcome, TypeProposition};
use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

const fn provider_with_public_fn_requirement() -> &'static str {
    r"
pub prop PublicReq<T: Int>;
pub fn needs(x: Int) -> Int where PublicReq<Int> { x }
"
}

#[test]
fn task_879_named_import_transports_public_fn_proposition_requirement_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, provider_with_public_fn_requirement());
    write_file(
        &caller,
        r"use provider::{needs}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("named import should load provider");
    let facts = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_proposition_facts.iter())
        .collect::<Vec<_>>();

    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .any(|summary| summary.version == SummaryVersion::SPEC064_PROPOSITIONS_V5),
        "proposition transport must upgrade imported semantic summaries to V5"
    );
    assert_eq!(
        facts.len(),
        1,
        "expected exactly one transported fact: {facts:?}"
    );
    assert!(matches!(
        &facts[0].proposition,
        TypeProposition::NamedPredicate(named) if named.predicate.name.as_str() == "PublicReq"
    ));
    assert!(matches!(
        &facts[0].outcome,
        Some(PropositionOutcome::Deferred(reason))
            if reason.kind == PropositionDeferredKind::UnsupportedNamedPredicate
    ));
}

#[test]
fn task_879_named_import_transports_public_interface_bound_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub interface PublicIface {}
pub fn needs_iface(x: Int) -> Int where Int: PublicIface { x }
",
    );
    write_file(
        &caller,
        r"use provider::{needs_iface}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("named import should load public interface-bound summary");
    let proposition_summary = loaded
        .imported_semantic_summaries
        .iter()
        .find(|summary| !summary.exported_proposition_facts.is_empty())
        .expect("proposition summary should be imported");

    assert!(
        proposition_summary
            .interface_identities
            .iter()
            .any(|identity| identity.name.as_str() == "PublicIface"),
        "public interface-bound proposition transport must include interface identity metadata: {proposition_summary:?}"
    );
}

#[test]
fn task_879_named_import_transports_proposition_term_supporting_type_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub type Extra = Extra;
pub prop PublicReq<T: Type>;
pub fn needs(x: Int) -> Int where PublicReq<Extra> { x }
",
    );
    write_file(
        &caller,
        r"use provider::{needs}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("named callable import should include proposition-supporting type summaries");
    let proposition_summary = loaded
        .imported_semantic_summaries
        .iter()
        .find(|summary| !summary.exported_proposition_facts.is_empty())
        .expect("transported proposition fact summary should be present");

    assert!(
        proposition_summary
            .exported_types
            .iter()
            .any(|ty| ty.exported_name.as_str() == "Extra"),
        "proposition summary must carry supporting public type metadata: {proposition_summary:?}"
    );
}

#[test]
fn task_879_glob_import_and_pub_use_preserve_proposition_fact_payload_once() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, provider_with_public_fn_requirement());
    write_file(&facade, "pub use provider::*;\n");
    write_file(
        &caller,
        r"use facade::*
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("pub-use glob import should load");
    let facts = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_proposition_facts.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        facts.len(),
        1,
        "pub-use/glob transport should preserve exactly one proposition fact payload: {facts:?}"
    );
    assert!(matches!(
        &facts[0].proposition,
        TypeProposition::NamedPredicate(named) if named.predicate.name.as_str() == "PublicReq"
    ));
}
