//! TASK-2013 RED contract for checked source-handler lowering into Core/CPS.
//!
//! This is an inspection-only bridge.  It must reject typed facts its narrow
//! one-clause Core carrier cannot represent, rather than erase them or create
//! a provider frame or executable engine path.

use ash_core::{
    core_ash::{CoreExpr, CoreMultiplicity, CoreRow, CoreType},
    core_ash_lower::CoreLoweringContext,
    core_ash_typecheck::{CoreTypeCheckEnv, type_check_and_lower_core_program},
    core_ash_validate::{RawCoreProgram, validate_core_program},
    cps::ContRef,
};
use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{
    checked_handler_application_facts_for_test, lower_checked_handler_application_to_core,
    type_check_program,
};

const CLOCK_PREFIX: &str = r#"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
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

fn echo_sleep_program(done_body: &str, clause_body: &str) -> Program {
    parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler echo_sleep(comp: () -> {{ TestClock::sleep }} Int) -> Int {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => {clause_body},\n\
             done(value) => {done_body},\n\
           }}\n\
         }}\n\
         fn main() -> Int {{ handle TestClock::sleep(0) with echo_sleep }}"
    ))
}

fn resume_sleep_program(handler_result: &str, done_body: &str, clause_body: &str) -> Program {
    parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler resume_sleep(comp: () -> {{ TestClock::sleep }} Int) -> {handler_result} {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => {clause_body},\n\
             done(value) => {done_body},\n\
           }}\n\
         }}\n\
         fn main() -> {handler_result} {{ handle TestClock::sleep(0) with resume_sleep }}"
    ))
}

fn assert_private_core_bridge_preserves_multishot(program: &Program) {
    let checked_source = type_check_program(program)
        .expect("the canonical source handler must first typecheck as immutable thunk evidence");
    let core = lower_checked_handler_application_to_core(program, &checked_source, "main")
        .expect("the private Core inspection bridge must retain MultiShotPure source evidence");
    assert!(
        matches!(
            core,
            CoreExpr::Handle {
                clause: ash_core::core_ash::CoreHandlerClause {
                    resume: ash_core::core_ash::CoreParam {
                        ty: CoreType::Cont {
                            multiplicity: CoreMultiplicity::MultiShotPure,
                            ..
                        },
                        ..
                    },
                    ..
                },
                ..
            }
        ),
        "the Core boundary must preserve MultiShotPure continuation evidence"
    );
}

fn assert_private_core_bridge_rejects(program: &Program, expected: &str) {
    let checked_source = type_check_program(program)
        .expect("the generalized source handler must typecheck before its private Core boundary");
    let error = lower_checked_handler_application_to_core(program, &checked_source, "main")
        .expect_err(
            "the private Core inspection bridge must reject generalized typed handler facts",
        );
    assert!(
        error.to_string().contains(expected),
        "the Core boundary must state the rejected generalized fact `{expected}`: {error}"
    );
}

#[test]
fn task_2013_checked_source_handler_preserves_a_multishot_fact_for_core() {
    let program = echo_sleep_program("value", "ms");
    assert_private_core_bridge_preserves_multishot(&program);
}

#[test]
fn task_2013_private_core_bridge_preserves_closed_empty_multishot_resume_through_cps() {
    let program = echo_sleep_program("value", "ms");
    let checked_source = type_check_program(&program)
        .expect("closed-empty echo handler must first typecheck as source evidence");

    let core = lower_checked_handler_application_to_core(&program, &checked_source, "main")
        .expect("the narrow inspection bridge must retain, not reject, MultiShotPure evidence");
    let CoreExpr::Handle { clause, .. } = &core else {
        panic!("narrow handler bridge must produce a Core Handle");
    };
    assert!(matches!(
        clause.resume.ty,
        CoreType::Cont {
            multiplicity: CoreMultiplicity::MultiShotPure,
            ..
        }
    ));

    let mut environment = CoreTypeCheckEnv::default();
    assert!(environment.operations_mut().insert(clause.op.clone()));
    let validated = validate_core_program(RawCoreProgram::new(core))
        .expect("the multishot handler inspection Core must validate");
    type_check_and_lower_core_program(
        validated,
        &environment,
        CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default()),
    )
    .expect("the multishot handler inspection Core must type-check and lower to CPS");
}

#[test]
fn task_2013_private_core_bridge_rejects_arbitrary_done_without_synthesizing_a_return_clause() {
    let program = echo_sleep_program("0", "ms");
    assert_private_core_bridge_rejects(&program, "done clause");
}

#[test]
fn task_2013_private_core_bridge_rejects_grouped_open_residual_without_erasing_or_admission() {
    let program = parse_program(&format!(
        "{CLOCK_PREFIX}\
         effect group OpenClockGroup = {{ TestClock::sleep | rest }};\
         handler forward_sleep(comp: () -> {{ OpenClockGroup }} Int) -> Int {{\
           on comp {{\
             TestClock::sleep(milliseconds, resume) => milliseconds,\
             done(value) => value,\
           }}\
         }}\
         fn main(computation: () -> {{ OpenClockGroup }} Int) -> Int {{\
           handle computation() with forward_sleep\
         }}"
    ));

    let checked = type_check_program(&program).expect(
        "the grouped open-row handler application must typecheck and publish immutable source evidence before inspection",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1, "the typed source handle publishes one fact");
    assert_eq!(facts[0].input_row.tail.as_deref(), Some("rest"));
    assert_eq!(
        facts[0].output_row.tail.as_deref(),
        Some("rest"),
        "the residual open tail must remain observable at the source-fact boundary"
    );

    let error = lower_checked_handler_application_to_core(&program, &checked, "main")
        .expect_err("the narrow Core inspection bridge must not erase or admit an open residual");
    assert_eq!(
        error.to_string(),
        "Type error: private Core handler bridge rejects nonempty or open output row",
        "the exact inspection boundary must reject before it can construct a Core/frame/runtime artifact"
    );
}

#[test]
fn task_2013_resume_invocation_preserves_multishot_in_the_narrow_core_inspection_boundary() {
    let program = echo_sleep_program("value", "resume(ms)");
    assert_private_core_bridge_preserves_multishot(&program);
}

#[test]
fn task_2013_single_resume_with_declared_result_preserves_multishot_core_lowering() {
    let program = resume_sleep_program("Int", "value", "resume(ms)");
    assert_private_core_bridge_preserves_multishot(&program);
}

#[test]
fn task_2013_resume_payload_must_match_the_declared_operation_result_type() {
    let program = resume_sleep_program("Int", "value", "resume(null)");

    let error = type_check_program(&program).expect_err(
        "resuming TestClock::sleep with Null instead of its declared Int result must reject",
    );
    let text = error.to_string();
    assert!(
        text.contains("resume") && text.contains("Int"),
        "wrong resume-payload diagnostic must identify the binder and declared result type: {text}"
    );
}

#[test]
fn task_2013_closed_empty_resume_may_be_repeated_before_the_core_boundary() {
    let program = resume_sleep_program("Int", "value", "{ resume(ms); resume(ms) }");
    assert_private_core_bridge_rejects(&program, "identity operation clause body");
}

#[test]
fn task_2013_malformed_resume_calls_do_not_receive_the_affine_duplicate_diagnostic() {
    let program = resume_sleep_program("Int", "value", "{ resume(); resume(ms) }");

    let error = type_check_program(&program).expect_err(
        "a malformed nested resume call must remain in ordinary checking rather than affine classification",
    );
    let text = error.to_string();
    assert!(
        text.contains("resume") && !text.contains("affine"),
        "malformed resume calls must fail closed through ordinary checking: {text}"
    );
}

#[test]
fn task_2013_resume_must_be_the_entire_clause_body() {
    let program = resume_sleep_program("Int", "value", "{ let resumed = resume(ms); resumed }");
    let error = type_check_program(&program)
        .expect_err("a nested resume must not be accepted as the direct resume form");
    assert!(
        error.to_string().contains("resume"),
        "nested-resume diagnostic must retain the fail-closed direct-form boundary: {error}"
    );
}

#[test]
fn task_2013_handle_with_requires_handler_input_to_match_handled_expression() {
    let mismatched = parse_program(&format!(
        "{CLOCK_PREFIX}\
         handler wrong_input(comp: () -> {{ TestClock::sleep }} Null) -> Int {{\n\
           on comp {{\n\
             TestClock::sleep(ms, resume) => resume(ms),\n\
             done(value) => value,\n\
           }}\n\
         }}\n\
         fn main() -> Int {{ handle TestClock::sleep(0) with wrong_input }}"
    ));
    let error = type_check_program(&mismatched)
        .expect_err("a handler accepting Null must not handle an Int computation");
    let text = error.to_string();
    assert!(
        text.contains("handler 'wrong_input' input computation mismatch")
            && text.contains("Null")
            && text.contains("Int"),
        "handler-input mismatch must identify the normalized implicit thunk and result types: {text}"
    );

    let matching = resume_sleep_program("Int", "value", "resume(ms)");
    type_check_program(&matching)
        .expect("a handler accepting the handled Int result remains a matching control");
}

#[test]
fn task_2013_handle_with_rejects_a_normalized_row_mismatch_after_implicit_thunk_inference() {
    let program = parse_program(
        "interface Clock<T> { sleep(Int) -> Int wake(Int) -> Int }\n\
         type TestClock = SystemClock(Int);\n\
         impl Clock<TestClock> { sleep(milliseconds) = milliseconds wake(milliseconds) = milliseconds }\n\
         handler wake_only(comp: () -> { TestClock::wake } Int) -> Int {\n\
           on comp {\n\
             TestClock::wake(ms, resume) => resume(ms),\n\
             done(value) => value,\n\
           }\n\
         }\n\
         fn main() -> Int { handle TestClock::sleep(0) with wake_only }",
    );

    let error = type_check_program(&program).expect_err(
        "a handler input thunk must match the handled expression's normalized operation row",
    );
    let text = error.to_string();
    assert!(
        text.contains("handler 'wake_only' input computation mismatch")
            && text.contains("TestClock::Clock::wake")
            && text.contains("TestClock::Clock::sleep"),
        "row mismatch must name the handler and both normalized operation requirements: {text}"
    );
}

#[test]
fn task_2013_private_core_bridge_preserves_a_closed_empty_multishot_source_fact() {
    let program = echo_sleep_program("value", "resume(ms)");
    assert_private_core_bridge_preserves_multishot(&program);
}

#[test]
fn task_2013_private_core_bridge_rejects_multi_clause_source_facts_before_selecting_a_clause() {
    let program = parse_program(
        "interface Device<T> { read(Int) -> Int write(Int) -> Int }\n\
         type TestDevice = SystemDevice(Int);\n\
         impl Device<TestDevice> { read(value) = value write(value) = value }\n\
         handler both(comp: () -> { TestDevice::read, TestDevice::write } Int) -> Int {\n\
           on comp {\n\
             TestDevice::read(value, read_resume) => read_resume(value),\n\
             TestDevice::write(value, write_resume) => write_resume(value),\n\
             done(result) => result,\n\
           }\n\
         }\n\
         fn main() -> Int { handle { TestDevice::read(0); TestDevice::write(0) } with both }",
    );
    assert_private_core_bridge_rejects(&program, "exactly one operation clause");
}

#[test]
fn task_2013_private_core_bridge_rejects_nonempty_output_rows_without_erasing_them() {
    let program = echo_sleep_program("TestClock::sleep(value)", "resume(ms)");
    assert_private_core_bridge_rejects(&program, "output row");
}
