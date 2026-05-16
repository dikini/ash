//! TASK-910 summary transport and HKT non-interference matrix tests.

use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

fn imported_interface_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .map(|identity| identity.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn imported_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .map(|ty| ty.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

#[test]
fn summary_transport_preserves_public_hkt_interfaces_without_private_or_evidence_leakage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub type Token = Token { value: Int };

pub interface Functor<F : * -> *> {}
pub interface Applicative<F : * -> *> {}
pub interface Monad<M : * -> *> {}
impl Monad<Option> {}

interface PrivateMonad<M : * -> *> {}
impl PrivateMonad<Option> {}
",
    );
    write_file(
        &caller,
        r"use provider::*
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("public HKT interface metadata should transport through module summaries");

    let type_names = imported_type_names(&loaded);
    assert!(
        type_names.contains(&"Token"),
        "unrelated public type summary behavior should be preserved: {type_names:?}"
    );

    let interface_names = imported_interface_names(&loaded);
    for expected in ["Functor", "Applicative", "Monad"] {
        assert!(
            interface_names.contains(&expected),
            "public HKT interface identity {expected} should be transported: {interface_names:?}"
        );
    }
    assert!(
        !interface_names.contains(&"PrivateMonad"),
        "private HKT interface identity must not leak through summary transport: {interface_names:?}"
    );

    let interface_count = interface_names.len();
    let mut deduped = interface_names.clone();
    deduped.dedup();
    assert_eq!(
        interface_count,
        deduped.len(),
        "impl evidence must not create duplicate interface identity summaries: {interface_names:?}"
    );
}
