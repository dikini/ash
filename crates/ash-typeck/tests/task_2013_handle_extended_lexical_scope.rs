//! TASK-2013 RED contracts for lexical scopes carried by nested source forms.

use ash_parser::surface::{Definition, Program, ProgramEntry};
use ash_typeck::{
    Kind, QualifiedName, Type, TypeEnv, checked_handler_application_facts_for_test,
    type_check_program, type_check_program_in_env,
};

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
        .expect("a nested lexical binder visible at the handle site must typecheck its operand");
    assert_eq!(
        checked_handler_application_facts_for_test(&checked).len(),
        1
    );
}

fn env_with_monad_option_evidence() -> TypeEnv {
    let module = ash_parser::parse_surface_file(
        r#"
        interface Monad<M : * -> *> {
            unit(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }
        impl Monad<Option> {
            unit(value) = Some { value: value }
            bind(value, f) = value
        }
        "#,
    )
    .expect("Monad<Option> fixture should parse");
    let interface = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == "Monad" => {
                Some(interface)
            }
            _ => None,
        })
        .expect("fixture defines Monad");
    let implementation = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == "Monad" => {
                Some(implementation)
            }
            _ => None,
        })
        .expect("fixture defines Monad<Option>");
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(interface)
        .expect("register Monad interface");
    env.register_impl(implementation)
        .expect("register Monad<Option> evidence");
    let option_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    env.bind_variable(
        "option::pure",
        Type::Fn(vec![Type::Int], Box::new(option_int)),
    );
    env
}

#[test]
fn task_2013_handle_operand_resolves_with_error_arm_binder() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           with_error {{ fail 1 }} handle {{ value => handle TestDevice::read(value) with h; }}\
         }}"
    ));
}

#[test]
fn task_2013_handle_operand_resolves_do_block_let_binder() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           do {{ let value = 1; return handle TestDevice::read(value) with h }}\
         }}"
    ));
}

#[test]
fn task_2013_do_bind_exposes_monadic_inner_value_to_handle_operand() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() {{\
           do:Option {{\
             value <- option::pure(1);\
             return handle TestDevice::read(value) with h\
           }}\
         }}"
    ));
    let checked = type_check_program_in_env(&env_with_monad_option_evidence(), &program)
        .expect("the do-bind name must have the monadic inner Int type at the handler application");
    assert_eq!(
        checked_handler_application_facts_for_test(&checked).len(),
        1
    );
}

#[test]
fn task_2013_comprehension_qualifier_remains_blocked_without_monad_evidence() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> List<Int> {{\
           [handle TestDevice::read(value) with h | let value = 1]: List\
         }}"
    ));
    let error = type_check_program(&program)
        .expect_err("a source comprehension without Monad<List> evidence must remain unsupported");
    assert!(
        error.to_string().contains("missing Monad evidence"),
        "the nested handler application must not bypass the pre-existing comprehension evidence boundary: {error}"
    );
}

#[test]
fn task_2013_fn_def_annotation_is_not_silently_freshened_for_handler_operands() {
    let program = parse_program(&format!(
        "{DEVICE_PREFIX}{HANDLER}\
         fn main() -> Int {{\
           let closure = fn(value: String) -> Int {{ handle TestDevice::read(value) with h }};\
           0\
         }}"
    ));

    let error = type_check_program(&program).expect_err(
        "a String-annotated handler operand argument must not be treated as an inferred Int",
    );
    assert!(
        error
            .to_string()
            .contains("unsupported-handler-computation-expression")
            || error.to_string().contains("String")
            || error.to_string().contains("argument type mismatch"),
        "the rejection must preserve the typed annotation boundary: {error}"
    );
}

#[test]
fn task_2013_with_error_nominal_pattern_uses_canonical_payload_bindings() {
    assert_one_application(&format!(
        "{DEVICE_PREFIX}\
         type Failure = NetworkFailure {{ value: Int }} | TimeoutFailure;\
         {HANDLER}\
         fn main() -> Int {{\
           with_error {{ fail NetworkFailure {{ value: 1 }} }} handle {{\
             NetworkFailure {{ value: value }} => handle TestDevice::read(value) with h;\
             TimeoutFailure => 0;\
             _ => 0;\
           }}\
         }}"
    ));
}
