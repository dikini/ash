//! TASK-1820 row-summary transport across engine/typecheck-facing boundaries.

use ash_engine::CallableRowRequirementSource;
use ash_parser::surface::ComputationRowItem;
use std::path::Path;

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn imported_application(module_source: &str, import_name: &str) -> ash_engine::Entry {
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

fn local_program_application(source: &str) -> ash_engine::Entry {
    ash_engine::Engine::new()
        .build()
        .expect("engine builds")
        .parse(source)
        .expect("program should parse")
}

fn assert_posix_read_row(row: &ash_parser::surface::ComputationRow) {
    assert_eq!(row.items.len(), 1);
    assert!(matches!(
        &row.items[0],
        ComputationRowItem::Operation { path, .. }
            if path.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>()
                == ["PosixFs", "read"]
    ));
}

#[test]
fn local_function_inline_row_is_threaded_into_application_summary() {
    let application = local_program_application(
        "fn read(path: String) -> {PosixFs::read} String { path }\nfn main() -> Int { 0 }\n",
    );

    let summary = application
        .callable_row_requirements
        .get("read")
        .expect("local function row summary should be threaded");

    assert_eq!(summary.source, CallableRowRequirementSource::InlineReturn);
    assert_posix_read_row(&summary.row);
}

#[test]
fn imported_function_where_row_is_threaded_into_application_summary() {
    let application = imported_application(
        "pub fn read(path: String) -> String where row { PosixFs::read } { path }\n",
        "read",
    );

    let summary = application
        .callable_row_requirements
        .get("read")
        .expect("imported function row summary should be threaded");

    assert_eq!(summary.source, CallableRowRequirementSource::WhereRow);
    assert_posix_read_row(&summary.row);
}

#[test]
fn rowless_imported_function_does_not_fabricate_application_row_summary() {
    let application = imported_application("pub fn pure(x: Int) -> Int { x }\n", "pure");

    assert!(
        !application.callable_row_requirements.contains_key("pure"),
        "rowless imports should preserve rowless behavior"
    );
}
