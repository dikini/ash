//! TASK-1911 parser/engine/typecheck fixtures for process concurrency rows.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_engine::{CallableRowRequirementSource, Engine, Workflow};
use ash_parser::surface::{ComputationRowItem, Definition, FnDef};
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

fn where_row(function: &FnDef) -> &ash_parser::surface::ComputationRow {
    &function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .unwrap_or_else(|| panic!("{} should have a where row", function.name))
        .row
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

    let engine = Engine::new().build().expect("engine builds");
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

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[test]
fn imported_process_channel_rows_survive_parser_engine_typecheck_and_core_boundary() {
    let library_source = r"
        pub fn coordinate(job: Int) -> Int
        where
            row {
                process spawn,
                process join,
                channel jobs,
                channel results
            }
        {
            job
        }
    ";

    let parsed_library = parse_module(library_source);
    let parsed_row = where_row(function_named(&parsed_library, "coordinate"));
    assert!(parsed_row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::Process {
                keyword,
                operation: Some(operation),
                ..
            } if keyword.as_ref() == "process" && operation.as_ref() == "spawn"
        )
    }));
    assert!(parsed_row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::Channel {
            mode: None,
            path,
            ..
        } if path.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>() == ["jobs"]
        )
    }));

    let workflow = checked_imported_workflow(library_source, "coordinate");
    let summary = workflow
        .callable_row_requirements
        .get("coordinate")
        .expect("imported callable row summary exists");
    assert_eq!(summary.source, CallableRowRequirementSource::WhereRow);
    assert!(summary.row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::Process {
                keyword,
                operation: Some(operation),
                ..
            } if keyword.as_ref() == "process" && operation.as_ref() == "join"
        )
    }));

    let core_row = callable_row(&workflow, "coordinate");
    assert!(core_row.items.contains(&CoreRowItem::Process {
        operation: "spawn".into(),
    }));
    assert!(core_row.items.contains(&CoreRowItem::Process {
        operation: "join".into(),
    }));
    assert!(core_row.items.contains(&CoreRowItem::Channel {
        path: path(&["jobs"]),
        mode: "send".into(),
        payload_type: Box::new(CoreType::Base("Unit".into())),
    }));
    assert!(core_row.items.contains(&CoreRowItem::Channel {
        path: path(&["results"]),
        mode: "send".into(),
        payload_type: Box::new(CoreType::Base("Unit".into())),
    }));
}
