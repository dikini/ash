use ash_parser::surface::{Definition, ImplDef, InterfaceDef};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn interface_named(module: &ash_parser::surface::ModuleFile, name: &str) -> InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => {
                Some(interface.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str) -> ImplDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl {name} should be present"))
}

fn option(arg: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![arg],
        kind: Kind::Type,
    }
}

fn constructor_var_app(name: &str, args: Vec<Type>) -> Type {
    Type::ConstructorVariableApp {
        constructor: name.to_string(),
        args,
        kind: Kind::Type,
    }
}

fn register_monad_option_env() -> TypeEnv {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {
            extract(M<Int>) -> Int
            make() -> M<Int>
        }

        impl Monad<Option> {
            extract(value) = 1
            make() = Some { value: 1 }
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register with kinded evidence key metadata");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register explicit Option evidence");
    env
}

fn register_unrelated_generic_monad_option_env() -> TypeEnv {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {
            extract(M<Int>, A) -> A
        }

        impl Monad<Option> {
            extract(value, fallback) = fallback
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register with unrelated method type var");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register explicit Option evidence");
    env
}

#[test]
fn method_lookup_with_option_input_does_not_invert_m_to_find_monad_option() {
    let env = register_monad_option_env();

    let err = env
        .resolve_interface_method_call("Monad", "extract", &[option(Type::Int)])
        .expect_err(
            "Monad<Option> evidence must not be selected by unifying M<Int> with Option<Int>",
        );
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && (message.contains("could not be fully determined")
                || message.contains("does not invert")
                || message.contains("Missing")
                || message.contains("missing")),
        "expected non-inverting evidence lookup diagnostic, got: {message}"
    );
}

#[test]
fn method_lookup_with_expected_option_output_does_not_select_monad_option() {
    let env = register_monad_option_env();

    let err = env
        .resolve_interface_method_call("Monad", "make", &[])
        .expect_err("Monad<Option> evidence must not be selected from expected Option<Int> output");
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && (message.contains("could not be fully determined")
                || message.contains("expected output")
                || message.contains("Missing")
                || message.contains("missing")),
        "expected output-independent evidence lookup diagnostic, got: {message}"
    );
}

#[test]
fn method_lookup_with_constructor_variable_argument_does_not_select_registered_option_evidence() {
    let env = register_monad_option_env();
    let open_arg = constructor_var_app("M", vec![Type::Int]);

    let err = env
        .resolve_interface_method_call("Monad", "extract", &[open_arg])
        .expect_err("registered Monad<Option> evidence must not be found from open M<Int>");
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && (message.contains("constructor-variable")
                || message.contains("does not invert")
                || message.contains("could not be fully determined")
                || message.contains("Missing")
                || message.contains("missing")),
        "expected no constructor-variable inversion during evidence lookup, got: {message}"
    );
}

#[test]
fn unrelated_method_generic_does_not_enable_constructor_variable_inversion() {
    let env = register_unrelated_generic_monad_option_env();

    let err = env
        .resolve_interface_method_call("Monad", "extract", &[option(Type::Int), Type::String])
        .expect_err(
            "unrelated generic A must not allow selecting Monad<Option> by inverting M<Int>",
        );
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && (message.contains("constructor-variable")
                || message.contains("does not invert")
                || message.contains("could not be fully determined")
                || message.contains("Missing")
                || message.contains("missing")),
        "expected unrelated method generic to preserve no-inversion boundary, got: {message}"
    );
}
