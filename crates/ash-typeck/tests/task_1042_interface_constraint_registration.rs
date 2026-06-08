use ash_parser::surface::{Definition, ImplDef, InterfaceDef, Type as SurfaceType};
use ash_typeck::TypeEnv;
use ash_typeck::error::TypeEnvError;

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
fn interface_registration_stores_interface_owned_evidence_constraints() {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where M: Applicative {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("required evidence interface registers first");
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("constrained interface should register");

    let monad = env
        .lookup_interface("Monad")
        .expect("Monad interface should be registered");
    assert_eq!(monad.evidence_constraints.len(), 1);
    assert_eq!(monad.evidence_constraints[0].subject_param, "M");
    assert_eq!(
        monad.evidence_constraints[0].required_interface,
        "Applicative"
    );
}

#[test]
fn interface_constraint_subject_must_name_same_interface_parameter() {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where N: Applicative {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("required evidence interface registers first");
    let err = env
        .register_interface(&interface_named(&module, "Monad"))
        .expect_err("constraint subject must be a Monad parameter");
    let message = err.to_string();

    assert!(message.contains("N"), "{message}");
    assert!(message.contains("Monad"), "{message}");
    assert!(message.contains("interface parameter"), "{message}");
    assert!(env.lookup_interface("Monad").is_none());
}

#[test]
fn interface_constraint_requires_known_interface_with_compatible_kind() {
    let unknown_module = parse(
        r#"
        interface Monad<M : * -> *> where M: Applicative {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_interface(&interface_named(&unknown_module, "Monad"))
        .expect_err("unknown required evidence interface should be rejected");
    let message = err.to_string();
    assert!(message.contains("Applicative"), "{message}");
    assert!(message.contains("required evidence"), "{message}");

    let wrong_kind_module = parse(
        r#"
        interface Eq<T> {}
        interface Monad<M : * -> *> where M: Eq {}
        "#,
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&wrong_kind_module, "Eq"))
        .expect("Eq<T> should register");
    let err = env
        .register_interface(&interface_named(&wrong_kind_module, "Monad"))
        .expect_err("required evidence subject kind must match target interface parameter kind");
    let message = err.to_string();
    assert!(message.contains("Eq"), "{message}");
    assert!(message.contains("kind"), "{message}");
    assert!(message.contains("* -> *"), "{message}");
}

#[test]
fn concrete_impl_requires_registered_required_evidence() {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where M: Applicative {}
        impl Monad<Option> {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("Applicative interface should register");
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("Monad interface should register");

    let err = env
        .register_impl(&impl_named(&module, "Monad", 0))
        .expect_err("Monad<Option> requires Applicative<Option> evidence first");
    let message = err.to_string();

    assert!(message.contains("Monad<Option>"), "{message}");
    assert!(message.contains("requires"), "{message}");
    assert!(message.contains("Applicative<Option>"), "{message}");
    match &err {
        TypeEnvError::InvalidDefinition(_, span) => {
            assert_ne!(*span, ash_parser::token::Span::default());
        }
        other => panic!("expected invalid definition error, got {other:?}"),
    }
    assert!(env.impl_schemes().is_empty());
    assert!(
        env.resolve_interface_evidence("Monad", &[SurfaceType::Name("Option".into())])
            .is_err(),
        "failed impl registration must leave no Monad<Option> evidence"
    );
}

#[test]
fn concrete_impl_registers_when_required_evidence_is_available() {
    let module = parse(
        r#"
        interface Applicative<F : * -> *> {}
        interface Monad<M : * -> *> where M: Applicative {}
        impl Applicative<Option> {}
        impl Monad<Option> {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("Applicative interface should register");
    env.register_interface(&interface_named(&module, "Monad"))
        .expect("Monad interface should register");
    env.register_impl(&impl_named(&module, "Applicative", 0))
        .expect("Applicative<Option> evidence should register");
    env.register_impl(&impl_named(&module, "Monad", 0))
        .expect("Monad<Option> should register after required Applicative<Option> evidence");

    env.resolve_interface_evidence("Applicative", &[SurfaceType::Name("Option".into())])
        .expect("Applicative<Option> evidence remains available");
    env.resolve_interface_evidence("Monad", &[SurfaceType::Name("Option".into())])
        .expect("Monad<Option> evidence should be available after successful registration");
}

#[test]
fn interface_constraints_remain_distinct_from_impl_where_bounds() {
    let module = parse(
        r#"
        interface Eq<T> {}
        interface Show<T> where T: Eq {}
        impl <T> Show<T> where T: Eq {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Eq"))
        .expect("Eq interface should register");
    env.register_interface(&interface_named(&module, "Show"))
        .expect("Show interface should register");
    env.register_impl(&impl_named(&module, "Show", 0))
        .expect("generic impl where-bound remains an impl-scheme constraint");

    let show = env
        .lookup_interface("Show")
        .expect("Show interface should remain registered");
    assert_eq!(show.evidence_constraints.len(), 1);
    let scheme = env
        .impl_schemes()
        .iter()
        .find(|scheme| scheme.interface == "Show")
        .expect("generic Show impl scheme should be registered");
    assert_eq!(scheme.where_bounds.len(), 1);
}

#[test]
fn direct_interface_evidence_constraint_cycle_is_rejected() {
    let module = parse(
        r#"
        interface Recursive<T> where T: Recursive {}
        "#,
    );

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_interface(&interface_named(&module, "Recursive"))
        .expect_err("direct interface evidence constraint cycle should be rejected");
    let message = err.to_string();

    assert!(message.contains("cycle"), "{message}");
    assert!(message.contains("Recursive"), "{message}");
    assert!(env.lookup_interface("Recursive").is_none());
}
