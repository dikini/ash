//! TASK-1814 engine/module-boundary evidence for Phase 177 rows.

use ash_parser::surface::{ComputationRowItem, Type as SurfaceType};
use ash_typeck::type_env::TypeEnv;
use std::path::Path;

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn imported_workflow(module_source: &str, import_name: &str) -> ash_engine::Workflow {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");

    write(&library, module_source);
    write(
        &caller,
        &format!("use library::{{{import_name}}}\nfn main() -> Int {{ 0 }}\n"),
    );

    ash_engine::Engine::new()
        .build()
        .expect("engine builds")
        .parse_file(&caller)
        .expect("caller with import should parse")
}

#[test]
fn imported_public_signature_preserves_source_row_but_typeck_conversion_remains_rowless() {
    let workflow = imported_workflow(
        "pub fn accepts(reader: (Int) -> {PosixFs::read} Int) -> Int { 0 }\n",
        "accepts",
    );
    let signature = workflow
        .imported_fn_signatures
        .get("accepts")
        .expect("ordinary pub fn signature should be imported");

    let SurfaceType::Fn(_params, Some(row), ret) = &signature.params[0].ty else {
        panic!(
            "imported signature should preserve row-bearing callable parameter, got {:?}",
            signature.params[0].ty
        );
    };
    assert!(matches!(ret.as_ref(), SurfaceType::Name(name) if name.as_ref() == "Int"));
    assert_eq!(row.items.len(), 1);
    assert!(matches!(
        &row.items[0],
        ComputationRowItem::Operation { path, .. }
            if path
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect::<Vec<_>>() == ["PosixFs", "read"]
    ));

    let typeck_signature = ash_typeck::fn_signature_type(&TypeEnv::with_builtin_types(), signature)
        .expect("imported pub fn signature should convert to typeck Type::Fn");
    assert_eq!(
        typeck_signature.to_string(),
        "((Int) -> Int) -> Int",
        "TASK-1814 records this as the current validation-only source-to-typeck row boundary"
    );
    assert!(
        workflow.imported_workflow_summaries.is_empty(),
        "row requirements must not fabricate workflow admission summaries"
    );
}
