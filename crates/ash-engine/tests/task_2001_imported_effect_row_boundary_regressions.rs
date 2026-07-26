//! TASK-2001 RED: imported effect rows must preserve provider export boundaries.

use ash_engine::{Engine, EngineError, module_loader::load_ordinary_file};

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write Ash fixture");
}

#[test]
fn task_2001_glob_import_rejects_private_row_dependency_at_loader_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        "effect alias Hidden = { evidence hidden_proof };\npub effect alias Published = { Hidden };\n",
    );
    write_file(
        &caller,
        "use provider::*\npub fn takes() -> Int where row { Published } { 0 }\nfn main() { 0 }\n",
    );

    let error = load_ordinary_file(&caller).expect_err(
        "a glob import must reject a public row with a private dependency at load time",
    );
    let EngineError::Parse(diagnostic) = error else {
        panic!("expected source-loader rejection");
    };
    assert!(
        diagnostic.contains("private-dependency-export-failure")
            && diagnostic.contains("Published")
            && !diagnostic.contains("Hidden"),
        "the loader boundary must reject opaquely without exporting private metadata: {diagnostic}"
    );
}

#[test]
fn task_2001_parse_file_surfaces_opaque_private_row_export_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        "effect alias Hidden = { evidence hidden_proof };\npub effect alias Published = { Hidden };\n",
    );
    write_file(
        &caller,
        "use provider::*\npub fn takes() -> Int where row { Published } { 0 }\nfn main() { 0 }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    let error = engine
        .parse_file(&caller)
        .expect_err("private dependency must be rejected before entry parsing succeeds");
    let EngineError::Parse(diagnostic) = error else {
        panic!("expected parse/load boundary failure");
    };
    assert!(
        diagnostic.contains("private-dependency-export-failure")
            && diagnostic.contains("Published")
            && !diagnostic.contains("Hidden"),
        "the public boundary must use the structured private-dependency failure without leaking the provider-private name: {diagnostic}"
    );
}

#[test]
fn task_2001_facade_conflicting_public_effect_row_exports_reject_instead_of_first_wins() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = dir.path().join("first.ash");
    let second = dir.path().join("second.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &first,
        "pub effect alias Shared = { evidence first_proof };\n",
    );
    write_file(
        &second,
        "pub effect alias Shared = { evidence second_proof };\n",
    );
    write_file(
        &facade,
        "pub use first::{Shared};\npub use second::{Shared};\n",
    );
    write_file(
        &caller,
        "use facade::{Shared}\npub fn takes() -> Int where row { Shared } { 0 }\nfn main() { 0 }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    match engine.parse_file(&caller) {
        Err(EngineError::Type(diagnostic) | EngineError::Parse(diagnostic)) => assert!(
            diagnostic.contains("import-order-conflict") || diagnostic.contains("conflict"),
            "conflicting facade exports require a deterministic conflict diagnostic: {diagnostic}"
        ),
        Err(other) => panic!("expected an import conflict, got {other:?}"),
        Ok(mut entry) => {
            let error = engine
                .check(&mut entry)
                .expect_err("conflicting public effect-row exports must not be first-wins");
            let diagnostic = match error {
                EngineError::Type(diagnostic) | EngineError::Parse(diagnostic) => diagnostic,
                other => panic!("expected type/import conflict, got {other:?}"),
            };
            assert!(
                diagnostic.contains("import-order-conflict") || diagnostic.contains("conflict"),
                "conflicting facade exports require a deterministic conflict diagnostic: {diagnostic}"
            );
        }
    }
}
