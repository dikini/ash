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
    ash_parser::parse_surface_file(source)
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
fn stdlib_monoid_surface_requires_semigroup_evidence() {
    let path = std_src_path("algebra/monoid.ash");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let module = parse(&source);

    let monoid = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == "Monoid" => {
                Some(interface)
            }
            _ => None,
        })
        .expect("Monoid interface should be present");

    assert_eq!(monoid.evidence_constraints.len(), 1);
    let constraint = &monoid.evidence_constraints[0];
    assert!(
        matches!(&constraint.subject, ash_parser::surface::Type::Name(name) if name.as_ref() == "A")
    );
    assert!(
        matches!(&constraint.interface, ash_parser::surface::Type::Name(name) if name.as_ref() == "Semigroup")
    );
}

#[test]
fn stdlib_monoid_implementations_discharge_semigroup_requirement() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in ["algebra/monoid.ash", "string.ash", "list.ash"] {
        let result = engine
            .check_module_file(&std_src_path(relative))
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors after Monoid requires Semigroup: {:?}",
            result.errors
        );
    }
}

#[test]
fn monoid_impl_without_semigroup_required_evidence_is_rejected() {
    let module = parse(
        r"
        interface Semigroup<A> {}
        interface Monoid<A> where A: Semigroup {}
        impl Monoid<String> {}
        ",
    );
    let mut env = TypeEnv::with_builtin_types();
    env.register_interface(&interface_named(&module, "Semigroup"))
        .expect("Semigroup should register");
    env.register_interface(&interface_named(&module, "Monoid"))
        .expect("Monoid should register");

    let err = env
        .register_impl(&impl_named(&module, "Monoid"))
        .expect_err("Monoid<String> must require Semigroup<String> evidence");
    let message = err.to_string();
    assert!(message.contains("Monoid"), "{message}");
    assert!(message.contains("Semigroup"), "{message}");
}
