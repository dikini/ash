//! TASK-2001 RED: selected public effect-row imports retain their public closure.
//!
//! An imported row is a non-granting requirement description, but selecting a
//! public head must not make the public rows it names disappear.  Otherwise a
//! caller can accidentally bypass row expansion and its cycle diagnostics.

use ash_engine::{Engine, EngineError, module_loader::load_ordinary_file};

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write Ash fixture");
}

fn imported_effect_row_names(
    loaded: &ash_engine::module_loader::LoadedOrdinaryFile,
) -> Vec<String> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| &summary.exported_effect_rows)
        .map(|row| row.exported_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

#[test]
fn task_2001_selected_public_effect_row_import_retains_public_dependency_closure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub effect alias A = { B };
pub effect alias B = { evidence audit_log };
",
    );
    write_file(
        &caller,
        r"
use provider::{A}
pub fn takes() -> Int where row { A } { 0 }
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("a selected public row should carry its public row dependencies");
    assert_eq!(
        imported_effect_row_names(&loaded),
        vec!["A", "B"],
        "selecting A must preserve B for later row expansion rather than treating B as raw text"
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse_file(&caller).expect("caller parses");
    engine
        .check(&mut entry)
        .expect("a public non-cyclic dependency closure remains a non-granting row requirement");
}

#[test]
fn task_2001_selected_public_effect_row_cycle_rejects_with_cycle_path() {
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
use provider::{A}
pub fn takes() -> Int where row { A } { 0 }
fn main() { 0 }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse_file(&caller).expect("caller parses");
    let error = engine
        .check(&mut entry)
        .expect_err("a selected public row must not bypass its imported cycle");
    let EngineError::Type(diagnostic) = error else {
        panic!("expected a type diagnostic for the imported effect-row cycle");
    };
    assert!(
        diagnostic.contains("cyclic imported effect-row export 'A -> B -> A'"),
        "the deterministic selected-import diagnostic must include the closure cycle: {diagnostic}"
    );
}
