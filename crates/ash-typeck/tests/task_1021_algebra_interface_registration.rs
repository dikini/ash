//! TASK-1021 RED tests for registering the real `std::algebra` interface files.

use std::collections::BTreeSet;
use std::path::PathBuf;

use ash_core::Kind;
use ash_parser::surface::{Definition, InterfaceDef};
use ash_typeck::TypeEnv;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn parse_interface(relative: &str, name: &str) -> InterfaceDef {
    let path = std_src_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let source = strip_import_lines(&source);
    let module = ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"));
    module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => Some(interface),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{relative} should define interface {name}"))
}

fn strip_import_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_stdlib_exports_algebra_namespace_and_children() {
    let lib = std::fs::read_to_string(std_src_path("lib.ash")).expect("read std/src/lib.ash");
    assert!(
        lib.lines().any(|line| line.trim() == "pub mod algebra;"),
        "std/src/lib.ash must expose the algebra namespace with `pub mod algebra;`"
    );

    let algebra_mod =
        std::fs::read_to_string(std_src_path("algebra/mod.ash")).expect("read algebra/mod.ash");
    for child in ["semigroup", "monoid", "functor", "applicative", "monad"] {
        assert!(
            algebra_mod
                .lines()
                .any(|line| line.trim() == format!("pub mod {child};")),
            "std/src/algebra/mod.ash must expose child module `{child}`"
        );
    }
}

#[test]
fn algebra_interface_stdlib_sources_register_with_expected_methods() {
    assert_stdlib_exports_algebra_namespace_and_children();

    let cases = [
        ("algebra/semigroup.ash", "Semigroup", &[("append", 2)][..]),
        (
            "algebra/monoid.ash",
            "Monoid",
            &[("append", 2), ("empty", 0)][..],
        ),
        ("algebra/functor.ash", "Functor", &[("map", 2)][..]),
        (
            "algebra/applicative.ash",
            "Applicative",
            &[("apply", 2), ("pure", 1)][..],
        ),
        (
            "algebra/monad.ash",
            "Monad",
            &[("bind", 2), ("unit", 1)][..],
        ),
    ];

    let mut env = TypeEnv::with_builtin_types();
    for (relative, name, expected_methods) in cases {
        let interface = parse_interface(relative, name);
        env.register_interface(&interface)
            .unwrap_or_else(|error| panic!("{name} should register from {relative}: {error}"));

        let registered = env
            .lookup_interface(name)
            .unwrap_or_else(|| panic!("{name} should be registered"));
        let methods = registered.methods.keys().cloned().collect::<BTreeSet<_>>();
        let expected_method_names = expected_methods
            .iter()
            .map(|(method, _)| method.to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            methods, expected_method_names,
            "{name} should expose the TASK-1021 method surface"
        );
        for (method, expected_param_count) in expected_methods {
            let registered_method = registered
                .methods
                .get(*method)
                .unwrap_or_else(|| panic!("{name} should expose method {method}"));
            assert_eq!(
                registered_method.params.len(),
                *expected_param_count,
                "{name}::{method} should use the TASK-1020 positional method arity"
            );
        }
    }
}

#[test]
fn algebra_interface_constructor_kinded_interfaces_register_unary_carriers() {
    assert_stdlib_exports_algebra_namespace_and_children();

    let mut env = TypeEnv::with_builtin_types();

    for (relative, name) in [
        ("algebra/functor.ash", "Functor"),
        ("algebra/applicative.ash", "Applicative"),
        ("algebra/monad.ash", "Monad"),
    ] {
        let interface = parse_interface(relative, name);
        env.register_interface(&interface)
            .unwrap_or_else(|error| panic!("{name} should register from {relative}: {error}"));

        let registered = env
            .lookup_interface(name)
            .unwrap_or_else(|| panic!("{name} should be registered"));
        assert_eq!(
            registered.type_param_kinds,
            vec![Kind::n_ary(1)],
            "{name} should bind its carrier parameter at kind * -> *"
        );
    }
}
