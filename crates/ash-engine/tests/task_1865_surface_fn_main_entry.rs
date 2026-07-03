//! TASK-1865 regression coverage for target `fn main` entry sources.

use ash_core::Value;
use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_engine::{CallableRowRequirementSource, Engine, Workflow};

fn engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

fn checked_workflow_from_source(source: &str) -> Workflow {
    let engine = engine();
    let mut workflow = engine.parse(source).expect("source should parse");
    engine
        .check(&mut workflow)
        .expect("source should typecheck");
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

#[test]
fn fn_main_source_without_workflow_parses_checks_and_preserves_row_metadata() {
    let source = r"
        fn helper(value: Int) -> Int { value + 1 }

        fn main() -> Int where row { PosixFs.read } {
            do {
                value <- helper(41);
                return value
            }
        }
    ";

    let workflow = checked_workflow_from_source(source);

    let summary = workflow
        .callable_row_requirements
        .get("main")
        .expect("fn main row requirement should be preserved");
    assert_eq!(summary.source, CallableRowRequirementSource::WhereRow);

    let row = callable_row(&workflow, "main");
    assert_eq!(row.tail, None);
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Capability { path, operation }
                if path == &vec!["PosixFs".to_string()] && operation == "read"
        )
    }));
}

#[test]
fn fn_main_source_composes_records_adts_match_calls_and_do_without_workflow() {
    let source = r#"
        type UserPayload = UserPayload { name: String, age: Int };
        type Lookup = Found { age: Int } | Missing;

        fn age_of_record(user: UserPayload) -> Int {
            user.age
        }

        fn score(lookup: Lookup) -> Int {
            match lookup {
                Found { age: age } => age,
                Missing => 0,
            }
        }

        fn main() -> Int {
            do {
                user <- UserPayload { name: "Ada", age: 41 };
                lookup <- Found { age: age_of_record(user) };
                return score(lookup)
            }
        }
    "#;

    let workflow = checked_workflow_from_source(source);
    assert!(
        workflow.core_callable_types.contains_key("main"),
        "fn main should lower as an ordinary Core callable"
    );
    assert!(
        workflow.core_callable_types.contains_key("score"),
        "helper function should lower as an ordinary Core callable"
    );
}

#[test]
fn fn_main_inline_row_source_preserves_row_metadata_without_workflow() {
    let source = r"
        fn main() -> {PosixFs.read} Int {
            do {
                return 7;
            }
        }
    ";

    let workflow = checked_workflow_from_source(source);

    let summary = workflow
        .callable_row_requirements
        .get("main")
        .expect("inline fn main row requirement should be preserved");
    assert_eq!(summary.source, CallableRowRequirementSource::InlineReturn);

    let row = callable_row(&workflow, "main");
    assert!(row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Capability { path, operation }
                if path == &vec!["PosixFs".to_string()] && operation == "read"
        )
    }));
}

#[tokio::test]
async fn fn_main_source_executes_without_workflow_syntax() {
    let result = engine()
        .run(
            r"
            fn main() -> Int {
                do {
                    return 42;
                }
            }
        ",
        )
        .await
        .expect("fn main source should execute");

    assert_eq!(result, Value::Int(42));
}
