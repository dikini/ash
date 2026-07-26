//! TASK-2013 RED contracts for lexical scopes inside handler applications.

use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{checked_handler_application_facts_for_test, type_check_program};

const DEVICE_PREFIX: &str = r#"
interface Device<T> { read(Int) -> Int }
type TestDevice = SystemDevice(Int);
impl Device<TestDevice> { read(value) = value }
"#;

const HANDLER: &str = r#"
handler h(comp: () -> { TestDevice::read } Int) -> Int {
  on comp {
    TestDevice::read(value, resume) => value,
    done(value) => value,
  }
}
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

fn assert_one_application(source: &str) {
    let program = parse_program(source);
    let checked = type_check_program(&program)
        .expect("a lexical binder visible at the handle site must typecheck its operand");
    assert_eq!(
        checked_handler_application_facts_for_test(&checked).len(),
        1,
        "the accepted lexical-scope application publishes exactly one source fact"
    );
}

#[test]
fn task_2013_handle_operand_resolves_anonymous_function_parameter() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           let closure = fn(x: Int) -> Int {{ handle TestDevice::read(x) with h }};\
           0\
         }}"
    ));
}

#[test]
fn task_2013_handle_operand_resolves_match_arm_pattern_binder() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           match 1 {{ value => handle TestDevice::read(value) with h }}\
         }}"
    ));
}

#[test]
fn task_2013_handle_operand_resolves_if_let_pattern_binder() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           if let value = 1 then {{ handle TestDevice::read(value) with h }} else {{ 0 }}\
         }}"
    ));
}

#[test]
fn task_2013_handle_operand_resolves_lets_inside_the_handled_block() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           handle {{ let x = 1; TestDevice::read(x) }} with h\
         }}"
    ));
}
