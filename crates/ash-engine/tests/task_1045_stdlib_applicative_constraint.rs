#![allow(missing_docs)]

use std::path::PathBuf;

use ash_parser::surface::{Definition, ImplDef, InterfaceDef};
use ash_typeck::TypeEnv;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    let source_without_imports = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");
    ash_parser::parse_surface_file(&source_without_imports)
        .unwrap_or_else(|errors| panic!("module file should parse: {errors:?}\n{source}"))
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

#[test]
fn stdlib_applicative_surface_requires_functor_evidence() {
    let path = std_src_path("algebra/applicative.ash");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let module = parse(&source);

    let applicative = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == "Applicative" => {
                Some(interface)
            }
            _ => None,
        })
        .expect("Applicative interface should be present");

    assert_eq!(applicative.evidence_constraints.len(), 1);
    let constraint = &applicative.evidence_constraints[0];
    assert!(
        matches!(&constraint.subject, ash_parser::surface::Type::Name(name) if name.as_ref() == "F")
    );
    assert!(
        matches!(&constraint.interface, ash_parser::surface::Type::Name(name) if name.as_ref() == "Functor")
    );
    assert!(source.contains("pure(A) -> F<A>"), "{source}");
    assert!(
        source.contains("apply(F<(A) -> B>, F<A>) -> F<B>"),
        "{source}"
    );
}

#[test]
fn stdlib_applicative_implementations_discharge_functor_requirement() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in ["algebra/applicative.ash", "option.ash", "result.ash"] {
        let result = engine
            .check_module_file(&std_src_path(relative))
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors after Applicative requires Functor: {:?}",
            result.errors
        );
    }
}

#[test]
fn applicative_impl_without_functor_required_evidence_is_rejected() {
    let module = parse(
        r"
        interface Functor<F : * -> *> {}
        interface Applicative<F : * -> *> where F: Functor {}
        impl Applicative<Option> {}
        ",
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Functor"))
        .expect("Functor should register");
    env.register_interface(&interface_named(&module, "Applicative"))
        .expect("Applicative should register");

    let err = env
        .register_impl(&impl_named(&module, "Applicative"))
        .expect_err("Applicative<Option> must require Functor<Option> evidence");
    let message = err.to_string();
    assert!(message.contains("Applicative"), "{message}");
    assert!(message.contains("Functor"), "{message}");
}
