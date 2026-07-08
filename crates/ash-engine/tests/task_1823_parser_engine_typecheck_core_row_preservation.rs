//! TASK-1823 parser -> engine/typecheck -> Core callable row preservation.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_engine::{CallableRowRequirementSource, Engine, Workflow};
use ash_parser::surface::{ComputationRow, ComputationRowItem, Definition, FnDef, Type};
use std::path::Path;

fn parse_module(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("source module should parse")
}

fn function_named<'a>(module: &'a ash_parser::surface::ModuleFile, name: &str) -> &'a FnDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing parsed function {name}"))
}

fn inline_return_row<'a>(function: &'a FnDef, expected_return: &str) -> &'a ComputationRow {
    let Type::Fn(params, Some(row), return_type) = function
        .return_type
        .as_ref()
        .unwrap_or_else(|| panic!("{} should have a return type", function.name))
    else {
        panic!("{} should have an inline callable row", function.name);
    };
    assert!(
        params.is_empty(),
        "inline return row should have no fn params"
    );
    assert!(matches!(
        return_type.as_ref(),
        Type::Name(name) if name.as_ref() == expected_return
    ));
    row
}

fn where_row(function: &FnDef) -> &ComputationRow {
    &function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .unwrap_or_else(|| panic!("{} should have a where row", function.name))
        .row
}

fn assert_row_operation(row: &ComputationRow, expected_path: &[&str]) {
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::Operation { path, .. }
                if path.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>()
                    == expected_path
        )
    }));
}

fn assert_row_tail(row: &ComputationRow, expected_tail: &str) {
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::Tail { variable, .. }
                | ComputationRowItem::WholeRow { variable, .. }
                if variable.as_ref() == expected_tail
        )
    }));
}

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

fn checked_workflow_from_source(source: &str) -> Workflow {
    let engine = engine();
    let mut workflow = engine.parse(source).expect("workflow source should parse");
    engine
        .check(&mut workflow)
        .expect("workflow should typecheck");
    workflow
}

fn write(path: &Path, source: &str) {
    std::fs::write(path, source)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn checked_imported_workflow(module_source: &str, import_name: &str) -> Workflow {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();
    let library = dir.join("library.ash");
    let caller = dir.join("caller.ash");

    write(&library, module_source);
    write(
        &caller,
        &format!("use library::{{{import_name}}}\nfn main() -> Int {{ 0 }}\n"),
    );

    let engine = engine();
    let mut workflow = engine
        .parse_file(&caller)
        .expect("caller with import should parse");
    engine
        .check(&mut workflow)
        .expect("caller with import should typecheck");
    workflow
}

fn callable_row<'a>(workflow: &'a Workflow, name: &str) -> &'a CoreRow {
    match workflow
        .core_callable_types
        .get(name)
        .unwrap_or_else(|| panic!("missing Core callable type for {name}"))
    {
        CoreType::Function { row, .. } => row,
        other => panic!("{name} did not lower to a Core function type: {other:?}"),
    }
}

fn string_path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn assert_core_operation(row: &CoreRow, expected_path: &[&str], expected_operation: &str) {
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Operation { path, operation }
                if path == &string_path(expected_path) && operation == expected_operation
        )
    }));
}

fn assert_summary_operation(
    workflow: &Workflow,
    name: &str,
    expected_source: CallableRowRequirementSource,
    expected_path: &[&str],
) {
    let summary = workflow
        .callable_row_requirements
        .get(name)
        .unwrap_or_else(|| panic!("missing callable row summary for {name}"));
    assert_eq!(summary.source, expected_source);
    assert_row_operation(&summary.row, expected_path);
}

#[test]
fn inline_row_survives_parser_engine_check_and_core_lowering() {
    let source = "fn read(path: String) -> {posixfs.read} String { let f = fn() -> String { path }; f() }\nfn main() -> Int { 0 }\n";

    let parsed = parse_module(source);
    let row = inline_return_row(function_named(&parsed, "read"), "String");
    assert_row_operation(row, &["posixfs", "read"]);

    let workflow = checked_workflow_from_source(source);
    assert_summary_operation(
        &workflow,
        "read",
        CallableRowRequirementSource::InlineReturn,
        &["posixfs", "read"],
    );

    let row = callable_row(&workflow, "read");
    assert_eq!(row.tail, None);
    assert_core_operation(row, &["posixfs"], "read");
}

#[test]
fn where_row_survives_parser_engine_check_and_core_lowering() {
    let source = "fn audit(event: String) -> String where row { Audit.record } { event }\nfn main() -> Int { 0 }\n";

    let parsed = parse_module(source);
    let row = where_row(function_named(&parsed, "audit"));
    assert_row_operation(row, &["Audit", "record"]);

    let workflow = checked_workflow_from_source(source);
    assert_summary_operation(
        &workflow,
        "audit",
        CallableRowRequirementSource::WhereRow,
        &["Audit", "record"],
    );

    let row = callable_row(&workflow, "audit");
    assert_eq!(row.tail, None);
    assert_core_operation(row, &["Audit"], "record");
}

#[test]
fn imported_exported_callable_row_survives_module_boundary_and_core_lowering() {
    let library_source =
        "pub fn publish(event: String) -> String where row { Bus.publish } { event }\n";

    let parsed_library = parse_module(library_source);
    let row = where_row(function_named(&parsed_library, "publish"));
    assert_row_operation(row, &["Bus", "publish"]);

    let workflow = checked_imported_workflow(library_source, "publish");
    let imported_signature = workflow
        .imported_fn_signatures
        .get("publish")
        .expect("imported public function signature should be preserved");
    assert_row_operation(where_row(imported_signature), &["Bus", "publish"]);
    assert_summary_operation(
        &workflow,
        "publish",
        CallableRowRequirementSource::WhereRow,
        &["Bus", "publish"],
    );

    let row = callable_row(&workflow, "publish");
    assert_eq!(row.tail, None);
    assert_core_operation(row, &["Bus"], "publish");
}

#[test]
fn rowless_function_keeps_stable_default_row_after_engine_check() {
    let source = "fn pure(path: String) -> String { path }\nfn main() -> Int { 0 }\n";

    let parsed = parse_module(source);
    let function = function_named(&parsed, "pure");
    assert!(matches!(
        function.return_type.as_ref(),
        Some(Type::Name(name)) if name.as_ref() == "String"
    ));
    assert!(
        function
            .proposition_tail
            .as_ref()
            .and_then(|tail| tail.row.as_ref())
            .is_none(),
        "rowless parser fixture should not contain where-row metadata"
    );

    let workflow = checked_workflow_from_source(source);
    assert!(
        !workflow.callable_row_requirements.contains_key("pure"),
        "rowless functions should not fabricate row summaries"
    );
    assert_eq!(callable_row(&workflow, "pure"), &CoreRow::default());
}

#[test]
fn open_row_tail_survives_parser_engine_check_and_core_lowering() {
    let source = "fn read(path: String) -> {posixfs.read | r} String { let f = fn() -> String { path }; f() }\nfn main() -> Int { 0 }\n";

    let parsed = parse_module(source);
    let row = inline_return_row(function_named(&parsed, "read"), "String");
    assert_row_operation(row, &["posixfs", "read"]);
    assert_row_tail(row, "r");

    let workflow = checked_workflow_from_source(source);
    let summary = workflow
        .callable_row_requirements
        .get("read")
        .expect("open inline row should populate row summary");
    assert_eq!(summary.source, CallableRowRequirementSource::InlineReturn);
    assert_row_operation(&summary.row, &["posixfs", "read"]);
    assert_row_tail(&summary.row, "r");

    let row = callable_row(&workflow, "read");
    assert_core_operation(row, &["posixfs"], "read");
    assert_eq!(row.tail.as_deref(), Some("r"));
}
