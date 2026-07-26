//! TASK-2024 regression contract for a declaration-resolved handler-local row.
//!
//! TASK-2013 supersedes TASK-2024's historical `comp: Int` fixture: handlers
//! now receive a canonical, row-annotated computation input.  The source fact
//! may retain a nonempty `wake` output row. TASK-2026 promotes exactly the
//! canonical `forward_sleep` fixture through the private Core/CPS bridge; all
//! other nonempty rows remain closed.

use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{
    checked_handler_application_facts_for_test, lower_checked_handler_application_to_core,
    type_check_program,
};

const FORWARD_SLEEP_SOURCE: &str = r#"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler forward_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}

fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
"#;

fn parse_program_from(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2024 source should parse: {errors:?}"));
    let entry = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == "main" => {
                Some(ProgramEntry {
                    function: function.name.clone(),
                    span: function.span,
                })
            }
            _ => None,
        })
        .expect("fixture must define fn main");
    Program {
        definitions: module.definitions,
        entry,
    }
}

fn parse_program() -> Program {
    parse_program_from(FORWARD_SLEEP_SOURCE)
}

fn assert_private_core_rejects_nonempty_output_row(source: &str, variant: &str) {
    let program = parse_program_from(source);
    let checked_source = type_check_program(&program).unwrap_or_else(|error| {
        panic!("TASK-2024 {variant} must first be retained as a source fact: {error}")
    });
    let error = lower_checked_handler_application_to_core(&program, &checked_source, "main")
        .expect_err("the narrow Core bridge must reject a nonempty source output row");
    assert!(
        error.to_string().contains("output row"),
        "TASK-2024 {variant} must reject at the output-row boundary: {error}"
    );
}

#[test]
fn task_2024_canonical_forward_sleep_retains_wake_in_source_fact_and_core_lowers_exactly() {
    let program = parse_program();
    let checked_source = type_check_program(&program)
        .expect("the canonical declaration-backed forward_sleep handler should typecheck");
    let facts = checked_handler_application_facts_for_test(&checked_source);
    assert_eq!(
        facts.len(),
        1,
        "the source application retains one immutable fact"
    );
    assert!(
        facts[0]
            .output_row
            .items
            .iter()
            .any(|item| item.canonical_key() == "operation:TestClock::Clock::wake"),
        "the source-only typed application fact retains the clause wake effect"
    );

    let core = lower_checked_handler_application_to_core(&program, &checked_source, "main")
        .expect("TASK-2026 promotes only the canonical forward_sleep row through Core lowering");
    let ash_core::core_ash::CoreExpr::Handle { clause, body } = core else {
        panic!("the promoted fixture must lower to a root Core Handle");
    };
    assert!(matches!(
        body.as_ref(),
        ash_core::core_ash::CoreExpr::Raise { args, .. }
            if matches!(args.as_slice(), [ash_core::core_ash::CoreAtom::LitInt(0)])
    ));
    assert!(matches!(
        clause.body.as_ref(),
        ash_core::core_ash::CoreExpr::Raise { args, .. }
            if matches!(args.as_slice(), [ash_core::core_ash::CoreAtom::Var(name)] if name == "ms")
    ));
    assert_eq!(clause.row.items.len(), 1);
}

#[test]
fn task_2024_distinct_declared_clause_body_operation_is_source_typed_but_not_core_lowered() {
    let source = FORWARD_SLEEP_SOURCE
        .replace(
            "    wake(Int) -> Int\n",
            "    wake(Int) -> Int\n    other(Int) -> Int\n",
        )
        .replace(
            "    wake(milliseconds) = milliseconds\n",
            "    wake(milliseconds) = milliseconds\n    other(milliseconds) = milliseconds\n",
        )
        .replace("TestClock::wake(ms)", "TestClock::other(ms)");

    assert_private_core_rejects_nonempty_output_row(
        &source,
        "distinct declared clause body operation",
    );
}

#[test]
fn task_2024_rejects_a_clause_body_with_non_binder_payload() {
    let source = FORWARD_SLEEP_SOURCE.replace("TestClock::wake(ms)", "TestClock::wake(0)");

    assert_private_core_rejects_nonempty_output_row(&source, "literal clause payload");
}
