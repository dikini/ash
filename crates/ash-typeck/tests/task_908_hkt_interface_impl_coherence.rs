use ash_parser::surface::{Definition, ImplDef, InterfaceDef, Type as SurfaceType};
use ash_typeck::TypeEnv;

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

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str, index: usize) -> ImplDef {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .nth(index)
        .unwrap_or_else(|| panic!("impl {name} at index {index} should be present"))
}

#[test]
fn interface_monad_registers_constructor_kinded_parameter() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad<M : * -> *> should register as a constructor-kinded interface");

    let registered = env
        .lookup_interface("Monad")
        .expect("registered Monad interface should be visible");
    assert_eq!(registered.type_params, vec!["M"]);
}

#[test]
fn impl_monad_option_registers_as_unary_constructor_evidence() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Option> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register before impl registration");
    env.register_impl(&implementation)
        .expect("impl Monad<Option> should register because Option has kind * -> *");

    let schemes = env.impl_schemes();
    assert_eq!(schemes.len(), 1);
    assert_eq!(schemes[0].interface, "Monad");

    let evidence = env
        .resolve_interface_evidence("Monad", &[SurfaceType::Name("Option".into())])
        .expect("explicit Monad<Option> evidence lookup should select the registered impl");
    assert_eq!(evidence.interface, "Monad");
}

#[test]
fn impl_monad_int_is_rejected_as_wrong_kind() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Int> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad", 0);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register before checking impl-head kind");
    let err = env
        .register_impl(&implementation)
        .expect_err("impl Monad<Int> must reject Int because Monad expects * -> *");
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && message.contains("Int")
            && (message.contains("kind") || message.contains("* -> *")),
        "expected wrong-kind diagnostic for impl Monad<Int>, got: {message}"
    );
}

#[test]
fn duplicate_monad_option_impl_is_rejected_as_same_head_overlap() {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {}
        impl Monad<Option> {}
        impl Monad<Option> {}
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let first = impl_named(&module, "Monad", 0);
    let duplicate = impl_named(&module, "Monad", 1);

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface)
        .expect("Monad interface should register before impl registration");
    env.register_impl(&first)
        .expect("first impl Monad<Option> should register");
    let err = env
        .register_impl(&duplicate)
        .expect_err("duplicate impl Monad<Option> must be rejected by coherence");
    let message = err.to_string();

    assert!(
        message.contains("Monad")
            && message.contains("Option")
            && (message.contains("duplicate") || message.contains("overlap")),
        "expected duplicate/overlap diagnostic for impl Monad<Option>, got: {message}"
    );
}
