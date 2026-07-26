//! TASK-2001 RED: selected source imports must transport effect-row and handler metadata.

use ash_core::semantic_summary::{
    EffectRowAuthority, EffectRowExportClassification, ValueExportKind,
};
use ash_engine::module_loader::load_ordinary_file;
use ash_typeck::TypeEnv;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn task_2001_named_import_transports_public_effect_alias_and_handler_marker() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub effect alias Audit = { evidence audit_log };
pub handler audit_handler(comp: Unit) -> Unit { comp }
",
    );
    write_file(
        &caller,
        r"
use provider::{Audit, audit_handler}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller)
        .expect("named imports must retain public effect-row and handler summary metadata");
    let summary = loaded
        .imported_semantic_summaries
        .iter()
        .find(|summary| {
            !summary.exported_effect_rows.is_empty() || !summary.exported_values.is_empty()
        })
        .expect("named source import must transport a semantic summary");

    assert!(
        summary.exported_effect_rows.iter().any(|row| {
            row.exported_name == "Audit"
                && row.classification == EffectRowExportClassification::TransparentAlias
        }),
        "selected public effect alias metadata is missing: {:?}",
        summary.exported_effect_rows
    );
    assert!(
        summary.exported_values.iter().any(|value| {
            value.exported_name == "audit_handler" && value.kind == ValueExportKind::Handler
        }),
        "selected public handler marker is missing: {:?}",
        summary.exported_values
    );
    let expected_audit_items = summary
        .exported_effect_rows
        .iter()
        .find(|row| row.exported_name == "Audit")
        .expect("the selected alias summary must be present")
        .row_items
        .clone();
    assert_eq!(expected_audit_items.len(), 1);
    assert!(expected_audit_items[0].text.contains("audit_log"));

    let mut type_env = TypeEnv::with_builtin_types();
    type_env
        .register_module_semantic_summaries(&loaded.imported_semantic_summaries)
        .expect("source-loaded public summaries must register transactionally");
    let audit = type_env
        .lookup_effect_row_export("Audit")
        .expect("the named alias must be visible to later type checking");
    assert_eq!(audit.authority, EffectRowAuthority::NonGranting);
    assert_eq!(
        type_env
            .expand_effect_row_export("Audit")
            .expect("the named alias must retain its source-order row metadata"),
        expected_audit_items
    );
    type_env
        .require_handler_callable("audit_handler")
        .expect("the imported handler marker must satisfy handler-only admission");
}

#[test]
fn task_2001_pub_use_reexports_effect_rows_and_handler_markers_without_authority() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub effect group Audit = { evidence audit_log };
pub handler audit_handler(comp: Unit) -> Unit { comp }
",
    );
    write_file(&facade, "pub use provider::{Audit, audit_handler};\n");
    write_file(
        &caller,
        "use facade::{Audit, audit_handler}\nfn main() { 0 }\n",
    );

    let loaded =
        load_ordinary_file(&caller).expect("named imports through a public facade must load");
    let mut type_env = TypeEnv::with_builtin_types();
    type_env
        .register_module_semantic_summaries(&loaded.imported_semantic_summaries)
        .expect("re-exported metadata must register without authority side effects");

    let audit = type_env
        .lookup_effect_row_export("Audit")
        .expect("publicly re-exported row must remain source-visible");
    assert_eq!(audit.authority, EffectRowAuthority::NonGranting);
    assert_eq!(
        audit.classification,
        EffectRowExportClassification::DiagnosticGroup
    );
    assert!(
        !type_env.has_capability_binding("Audit"),
        "a row re-export describes requirements and must not install capability authority"
    );
    assert!(
        type_env.require_handler_callable("audit_handler").is_ok(),
        "the exported handler declaration marker may remain available without installing a handler"
    );
}
