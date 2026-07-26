//! TASK-2013 RED contracts for source `handle … with` application evidence.
//!
//! The application fact must point at the *use* of the handler name, and
//! operand inference must use the lexical bindings surrounding that use.

use ash_parser::{
    surface::{Definition, Program, ProgramEntry},
    token::Span,
};
use ash_typeck::{checked_handler_application_facts_for_test, type_check_program};

const DEVICE_PREFIX: &str = r#"
interface Device<T> {
    read(Int) -> Int
    write(Int) -> Int
}
type TestDevice = SystemDevice(Int);
impl Device<TestDevice> {
    read(value) = value
    write(value) = value
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

#[test]
fn task_2013_application_fact_anchors_the_handler_name_use_not_its_declaration() {
    let source = format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => TestDevice::write(value),\
             done(value) => value,\
           }}\
         }}\
         fn main() -> Int {{ handle TestDevice::read(1) with h }}"
    );
    let expected_start = source
        .rfind("with h")
        .expect("fixture contains the handler use")
        + "with ".len();
    let prefix = &source[..expected_start];
    let expected_handler_span = Span::new(
        expected_start,
        expected_start + 1,
        prefix.lines().count(),
        prefix
            .rsplit_once('\n')
            .map_or(prefix.len() + 1, |(_, line)| line.len() + 1),
    );
    let program = parse_program(&source);

    let checked =
        type_check_program(&program).expect("the exactly matched implicit thunk must typecheck");
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0].handler_span, expected_handler_span,
        "application evidence must anchor `h` at its exact use token, never its declaration or full expression"
    );
}

#[test]
fn task_2013_application_inference_uses_surrounding_lexical_let_bindings() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}\
         handler h(comp: () -> {{ TestDevice::read }} Int) -> Int {{\
           on comp {{\
             TestDevice::read(value, resume) => TestDevice::write(value),\
             done(value) => value,\
           }}\
         }}\
         fn main() -> Int {{\
           let x = 1;\
           handle TestDevice::read(x) with h\
         }}"
    ));

    let checked = type_check_program(&program).expect(
        "implicit thunk inference must resolve the operand argument through the surrounding block binding",
    );
    let facts = checked_handler_application_facts_for_test(&checked);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].input_result_type, ash_typeck::types::Type::Int);
    assert_eq!(
        facts[0]
            .input_row
            .items
            .iter()
            .map(|item| item.canonical_key())
            .collect::<Vec<_>>(),
        ["operation:TestDevice::Device::read"],
        "the lexical argument is pure while the concrete operand operation remains in the inferred row"
    );
}
