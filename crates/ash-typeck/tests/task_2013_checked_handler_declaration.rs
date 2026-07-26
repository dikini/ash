//! TASK-2013 RED contract for checked source-handler declarations.
//!
//! This stage validates declaration-backed clause metadata only.  It neither
//! installs runtime authority nor claims to execute a continuation.

use ash_parser::surface::{Definition, Expr, HandlerClause, Program, ProgramEntry};
use ash_typeck::{CallableDeclarationKind, type_check_program, types::Type};

const CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
"#;

fn parse_program(source: &str) -> Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("TASK-2013 source should parse: {errors:?}"));
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

fn canonical_handler_program() -> Program {
    parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler absorb_sleep(comp: () -> {{ TestClock::sleep }} Null) -> Null {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => null,\n\
             done(value) => value,\n\
           }}\n\
         }}\n\
         fn main() -> Null {{ null }}"
    ))
}

fn handler_on_clauses_mut(program: &mut Program) -> &mut Vec<HandlerClause> {
    let handler = program
        .definitions
        .iter_mut()
        .find_map(|definition| match definition {
            Definition::Handler(handler) if handler.name.as_ref() == "absorb_sleep" => {
                Some(handler)
            }
            _ => None,
        })
        .expect("fixture must define handler absorb_sleep");
    let Expr::On { clauses, .. } = &mut handler.body else {
        panic!("fixture handler must have canonical on body");
    };
    clauses
}

#[test]
fn task_2013_checked_handler_sidecar_keeps_marker_signature_and_declared_clause() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler absorb_sleep(comp: () -> {{ TestClock::sleep }} Null) -> Null {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => null,\n\
             done(value) => value,\n\
           }}\n\
         }}\n\
         fn main() -> Null {{ handle TestClock::sleep(0) with absorb_sleep }}"
    ));

    let result = type_check_program(&program)
        .expect("an unused resume binder must retain a checked declaration sidecar");
    let handler = result
        .checked_handlers
        .get("absorb_sleep")
        .expect("checked handler declaration must be retained for typed Core lowering");

    assert_eq!(handler.callable_kind, CallableDeclarationKind::Handler);
    assert_eq!(
        handler.callable_signature,
        Type::Fn(
            vec![Type::Fn(vec![], Box::new(Type::Null))],
            Box::new(Type::Null)
        )
    );
    assert_eq!(handler.clauses.len(), 1);
    let clause = &handler.clauses[0];
    assert_eq!(clause.operation.impl_type, "TestClock");
    assert_eq!(clause.operation.interface, "Clock");
    assert_eq!(clause.operation.operation, "sleep");
    assert_eq!(clause.operation.params, vec![Type::Int]);
    assert_eq!(clause.operation.result_type, Type::Null);
    assert_eq!(clause.payload_type, Type::Int);
    assert_eq!(clause.resume_name, "resume");
    assert_eq!(clause.done_binding, "value");
    assert_eq!(clause.done_body_type, Type::Null);
}

#[test]
fn task_2013_unknown_clause_operation_rejects_before_any_handler_lowering() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler absorb_sleep(comp: () -> {{ TestClock::sleep }} Null) -> Null {{\n\
           on comp {{\n\
             TestClock::wake(ms, resume) => null,\n\
             done(value) => value,\n\
           }}\n\
         }}\n\
         fn main() -> Null {{ null }}"
    ));

    let error = type_check_program(&program)
        .expect_err("an unknown concrete clause operation must reject during declaration checking");
    assert!(
        error
            .to_string()
            .contains("concrete impl 'TestClock' has no operation 'wake'"),
        "unexpected unknown-clause diagnostic: {error}"
    );
}

#[test]
fn task_2013_done_body_must_match_the_handler_answer_type() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler absorb_sleep(comp: () -> {{ TestClock::sleep }} Null) -> Null {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => null,\n\
             done(value) => 1,\n\
           }}\n\
         }}\n\
         fn main() -> Null {{ null }}"
    ));

    let error = type_check_program(&program)
        .expect_err("a done body with a different answer type must reject");
    assert!(
        error.to_string().contains("done") && error.to_string().contains("Null"),
        "unexpected done-answer diagnostic: {error}"
    );
}

#[test]
fn task_2013_checked_handler_rejects_a_constructed_on_with_no_concrete_operation_before_facts_publish()
 {
    let mut program = canonical_handler_program();
    let clauses = handler_on_clauses_mut(&mut program);
    clauses.retain(|clause| matches!(clause, HandlerClause::Done { .. }));

    let error = type_check_program(&program)
        .expect_err("checker must reject a manually constructed on body with no operation clauses");
    assert!(
        error
            .to_string()
            .contains("missing concrete operation clause"),
        "unexpected missing-operation checker diagnostic: {error}"
    );
}

#[test]
fn task_2013_checked_handler_rejects_a_constructed_duplicate_done_deterministically() {
    let mut program = canonical_handler_program();
    let clauses = handler_on_clauses_mut(&mut program);
    let done = clauses
        .iter()
        .find(|clause| matches!(clause, HandlerClause::Done { .. }))
        .expect("fixture must have one done clause")
        .clone();
    clauses.push(done);

    let error = type_check_program(&program)
        .expect_err("checker must reject a manually constructed duplicate done clause");
    assert!(
        error.to_string().contains("duplicate done clause"),
        "unexpected duplicate-done checker diagnostic: {error}"
    );
}

#[test]
fn task_2013_checked_handler_retains_missing_done_rejection_for_constructed_ast() {
    let mut program = canonical_handler_program();
    let clauses = handler_on_clauses_mut(&mut program);
    clauses.retain(|clause| matches!(clause, HandlerClause::Operation { .. }));

    let error = type_check_program(&program)
        .expect_err("checker must reject a manually constructed on body without done");
    assert!(
        error.to_string().contains("missing done clause"),
        "unexpected missing-done checker diagnostic: {error}"
    );
}

#[test]
fn task_2013_handle_with_rejects_an_ordinary_function_even_when_compatible() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         fn ordinary(comp: () -> {{ TestClock::sleep }} Null) -> Null {{ null }}\n\
         fn main() -> Null {{ handle TestClock::sleep(0) with ordinary }}"
    ));

    let error = type_check_program(&program)
        .expect_err("a compatible ordinary fn must not satisfy handler-only admission");
    assert!(
        error
            .to_string()
            .contains("ordinary function, not a handler"),
        "unexpected ordinary-function handler diagnostic: {error}"
    );
}

#[test]
fn task_2013_ordinary_calls_do_not_implicitly_thunk_computation_arguments() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         fn ordinary(comp: () -> {{ TestClock::sleep }} Null) -> Null {{ null }}\n\
         fn main() -> Null {{ ordinary(TestClock::sleep(0)) }}"
    ));

    let error = type_check_program(&program).expect_err(
        "implicit thunk evidence belongs exclusively to `handle expr with handler`, not ordinary calls",
    );
    assert!(
        error.to_string().contains("ordinary") || error.to_string().contains("() ->"),
        "ordinary calls must continue to check their argument as an ordinary value: {error}"
    );
}

#[test]
fn task_2013_resume_invocation_is_typed_as_the_declared_operation_result() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler resume_sleep(comp: () -> {{ TestClock::sleep }} Null) -> Null {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => resume(null),\n\
             done(value) => value,\n\
           }}\n\
         }}\n\
         fn main() -> Null {{ null }}"
    ));

    let result = type_check_program(&program)
        .expect("a typed one-resume form must not be treated as an ordinary function call");
    assert!(
        result.checked_handlers.contains_key("resume_sleep"),
        "resume typing must remain a declaration-sidecar fact, not runtime authority"
    );
}
