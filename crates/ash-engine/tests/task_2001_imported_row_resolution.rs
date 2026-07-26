//! TASK-2001 RED: imported effect-row names participate in row validation.
//!
//! An imported alias remains a requirement description.  Resolving it must
//! expose invalid row content to the existing row validator, never turn the
//! alias into a capability grant or silently treat its name as a row variable.

use ash_engine::{Engine, EngineError, module_loader::load_ordinary_file};

#[test]
fn task_2001_imported_effect_alias_expands_for_row_validation_without_authority() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(&provider, "pub effect alias Audit = { requires_proof };\n")
        .expect("write provider");
    let caller_source = "use provider::{Audit}\nfn takes() -> Int where row { Audit, evidence audit_log } { 0 }\nfn main() { 0 }\n";
    ash_parser::parse_surface_file(
        caller_source
            .strip_prefix("use provider::{Audit}\n")
            .unwrap(),
    )
    .expect("the source row spelling must be accepted by the parser");
    std::fs::write(&caller, caller_source).expect("write caller");
    let loaded = load_ordinary_file(&caller).expect("the named import loads its row summary");
    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| &summary.exported_effect_rows)
            .any(|row| row.exported_name == "Audit" && row.row_items[0].text == "requires_proof"),
        "the imported alias must retain its row item for checking"
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse_file(&caller)
        .expect("the source import and alias-bearing row parse");
    let error = engine
        .check(&mut entry)
        .expect_err("the imported alias must expand before row validation");

    let EngineError::Type(diagnostic) = error else {
        panic!("expected a type-check diagnostic for the imported row alias");
    };
    assert!(
        diagnostic.contains("unsupported row item family 'requires'"),
        "the alias must expose its invalid requirement item without granting authority: {diagnostic}"
    );
}

#[test]
fn task_2001_imported_effect_group_expands_for_row_validation_without_authority() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(&provider, "pub effect group Audit = { requires_proof };\n")
        .expect("write provider");
    let caller_source = "use provider::{Audit}\nfn takes() -> Int where row { group Audit, evidence audit_log } { 0 }\nfn main() { 0 }\n";
    ash_parser::parse_surface_file(
        caller_source
            .strip_prefix("use provider::{Audit}\n")
            .unwrap(),
    )
    .expect("the source row spelling must be accepted by the parser");
    std::fs::write(&caller, caller_source).expect("write caller");
    let loaded = load_ordinary_file(&caller).expect("the named import loads its row summary");
    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| &summary.exported_effect_rows)
            .any(|row| row.exported_name == "Audit" && row.row_items[0].text == "requires_proof"),
        "the imported group must retain its row item for checking"
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse_file(&caller)
        .expect("the source import and group-bearing row parse");
    let error = engine
        .check(&mut entry)
        .expect_err("the imported diagnostic group must expand before row validation");

    let EngineError::Type(diagnostic) = error else {
        panic!("expected a type-check diagnostic for the imported row group");
    };
    assert!(
        diagnostic.contains("unsupported row item family 'requires'"),
        "the group must expose its invalid requirement item without granting authority: {diagnostic}"
    );
}
