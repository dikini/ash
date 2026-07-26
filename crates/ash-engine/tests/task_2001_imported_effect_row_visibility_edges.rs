//! TASK-2001 RED: selected imported effect rows preserve visibility boundaries.
//!
//! A public row may be selected by name, but selection must not make a private
//! provider dependency visible.  Similarly, an alias at the import boundary
//! must not hide the provider's canonical row-cycle diagnostic.

use ash_engine::{Engine, EngineError};

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write Ash fixture");
}

#[test]
fn task_2001_selected_public_row_with_private_dependency_fails_opaque_at_load_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
effect alias Hidden = { evidence hidden_proof };
pub effect alias Published = { Hidden };
",
    );
    write_file(
        &caller,
        r"
use provider::{Published}
pub fn takes() -> Int where row { Published } { 0 }
fn main() { 0 }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let error = engine
        .parse_file(&caller)
        .expect_err("a public row with a private dependency must fail before source entry parses");
    let EngineError::Parse(diagnostic) = error else {
        panic!("expected an opaque loader/parse diagnostic for the inaccessible row dependency");
    };
    assert!(
        diagnostic.contains("private-dependency-export-failure")
            && diagnostic.contains("Published")
            && !diagnostic.contains("Hidden"),
        "the loader boundary must identify only the unusable public binding, never its private dependency: {diagnostic}"
    );
}

#[test]
fn task_2001_selected_public_row_cycle_through_import_alias_reports_provider_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub effect alias A = { B };
pub effect alias B = { A };
",
    );
    write_file(
        &caller,
        r"
use provider::{A as X}
pub fn takes() -> Int where row { X } { 0 }
fn main() { 0 }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse_file(&caller).expect("caller parses");
    let error = engine
        .check(&mut entry)
        .expect_err("an import alias must not bypass a public provider row cycle");
    let EngineError::Type(diagnostic) = error else {
        panic!("expected a type diagnostic for the imported effect-row cycle");
    };
    assert!(
        diagnostic.contains("cyclic imported effect-row export 'X -> B -> A -> X'"),
        "the alias-aware diagnostic must retain a meaningful provider cycle path: {diagnostic}"
    );
}
