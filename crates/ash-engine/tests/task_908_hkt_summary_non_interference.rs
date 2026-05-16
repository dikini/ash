//! TASK-908 HKT summary non-interference regression tests.

use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

fn semantic_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .map(|ty| ty.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn interface_identity_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .map(|identity| identity.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

#[test]
fn hkt_interface_summary_transport_preserves_public_types_without_private_interface_leakage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub type Token = Token { value: Int };

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
        .expect("public HKT interface metadata should transport through ordinary module imports");

    let type_names = semantic_type_names(&loaded);
    assert!(
        type_names.contains(&"Token"),
        "unrelated public type summary behavior should be preserved: {type_names:?}"
    );

    let interface_names = interface_identity_names(&loaded);
    assert!(
        interface_names.contains(&"Monad"),
        "public HKT interface identity should be transported: {interface_names:?}"
    );
    assert!(
        !interface_names.contains(&"PrivateMonad"),
        "private HKT interface identity must not leak through summary transport: {interface_names:?}"
    );
}
