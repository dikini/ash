//! TASK-1844 parser -> typecheck -> engine fixture for Core computation conformance.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_engine::{CallableRowRequirementSource, Engine, Workflow};
use ash_parser::surface::{BlockStmt, Definition, Expr, FnDef};

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

fn assert_core_operation(row: &CoreRow, expected_path: &[&str], expected_operation: &str) {
    let expected_path: Vec<String> = expected_path
        .iter()
        .map(|part| (*part).to_owned())
        .collect();
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Capability { path, operation }
                if path == &expected_path && operation == expected_operation
        )
    }));
}

fn expr_contains_do_block(expr: &Expr) -> bool {
    match expr {
        Expr::DoBlock { .. } => true,
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            statements.iter().any(|stmt| match stmt {
                BlockStmt::Let { expr, .. } | BlockStmt::Expr { expr, .. } => {
                    expr_contains_do_block(expr)
                }
            }) || tail_expr
                .as_ref()
                .is_some_and(|tail| expr_contains_do_block(tail))
        }
        _ => false,
    }
}

#[test]
fn fn_with_target_ambient_do_preserves_row_as_requirement_metadata() {
    let source = "fn read(path: String) -> String where row { PosixFs.read } { do { out <- path; return out } }\nworkflow main { ret 0 }\n";

    let parsed = parse_module(source);
    let function = function_named(&parsed, "read");
    assert!(
        expr_contains_do_block(&function.body),
        "parser should keep direct-style do body on the function"
    );

    let workflow = checked_workflow_from_source(source);
    let summary = workflow
        .callable_row_requirements
        .get("read")
        .expect("where-row summary should be preserved");
    assert_eq!(summary.source, CallableRowRequirementSource::WhereRow);

    let row = callable_row(&workflow, "read");
    assert_eq!(row.tail, None);
    assert_core_operation(row, &["PosixFs"], "read");
}
