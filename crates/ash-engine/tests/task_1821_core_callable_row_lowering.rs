//! TASK-1821 source callable rows lower into Core callable row carriers.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use std::path::Path;

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn local_program_application(source: &str) -> ash_engine::Entry {
    ash_engine::Engine::new()
        .build()
        .expect("engine builds")
        .parse(source)
        .expect("program should parse")
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

fn callable_row<'a>(application: &'a ash_engine::Entry, name: &str) -> &'a CoreRow {
    match application
        .core_callable_types
        .get(name)
        .unwrap_or_else(|| panic!("missing Core callable type for {name}"))
    {
        CoreType::Function { row, .. } => row,
        other => panic!("{name} did not lower to a Core function type: {other:?}"),
    }
}

fn assert_has_operation(row: &CoreRow, path: &[&str], operation: &str) {
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Operation {
                path: actual_path,
                operation: actual_operation,
            } if actual_path == &path.iter().map(|part| (*part).to_owned()).collect::<Vec<_>>()
                && actual_operation == operation
        )
    }));
}

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[test]
fn local_inline_row_lowers_to_core_function_row() {
    let application = local_program_application(
        "fn read(path: String) -> {PosixFs::read} String { path }\nfn main() -> Int { 0 }\n",
    );

    let row = callable_row(&application, "read");

    assert_eq!(row.tail, None);
    assert_has_operation(row, &["PosixFs"], "read");
}

#[test]
fn imported_where_row_lowers_to_core_function_row() {
    let application = imported_application(
        "pub fn audit(x: String) -> String where row { Audit::record } { x }\n",
        "audit",
    );

    let row = callable_row(&application, "audit");

    assert_eq!(row.tail, None);
    assert_has_operation(row, &["Audit"], "record");
}

#[test]
fn open_row_tail_is_preserved_in_core_function_row() {
    let application = local_program_application(
        "fn read(path: String) -> {PosixFs::read | r} String { path }\nfn main() -> Int { 0 }\n",
    );

    let row = callable_row(&application, "read");

    assert_eq!(row.tail.as_deref(), Some("r"));
    assert_has_operation(row, &["PosixFs"], "read");
}

#[test]
fn supported_target_row_families_lower_to_core_row_items() {
    let application = local_program_application(
        "fn guarded(x: String) -> String where row { \
         resource fs read, process spawn, fail IOError, \
         evidence sig, group audit \
         } { x }\nfn main() -> Int { 0 }\n",
    );

    let row = callable_row(&application, "guarded");

    assert!(row.items.contains(&CoreRowItem::Resource {
        path: path(&["fs"]),
        mode: "read".into(),
    }));
    assert!(row.items.contains(&CoreRowItem::Process {
        operation: "spawn".into(),
    }));
    assert!(row.items.contains(&CoreRowItem::Failure {
        ty: Some(Box::new(CoreType::Named("IOError".into()))),
    }));
    assert!(row.items.contains(&CoreRowItem::Evidence {
        path: path(&["sig"]),
    }));
    assert!(row.items.contains(&CoreRowItem::EffectGroupRef {
        path: path(&["audit"]),
    }));
}

#[test]
fn rowless_function_uses_default_core_function_row() {
    let application = local_program_application(
        "fn pure(path: String) -> String { path }\nfn main() -> Int { 0 }\n",
    );

    let row = callable_row(&application, "pure");

    assert_eq!(row, &CoreRow::default());
}

#[test]
fn rowless_function_metadata_does_not_reject_unlowered_surface_type_forms() {
    let application = local_program_application(
        "fn project(x: T::Item) -> T::Item { x }\nfn main() -> Int { 0 }\n",
    );

    let row = callable_row(&application, "project");

    assert_eq!(row, &CoreRow::default());
    assert!(
        !application
            .callable_row_requirements
            .contains_key("project"),
        "rowless compatibility metadata must not fabricate explicit row requirements"
    );
}
