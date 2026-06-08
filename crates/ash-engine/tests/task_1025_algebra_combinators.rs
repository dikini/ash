#![allow(missing_docs)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use ash_core::Value;
use ash_parser::surface::Definition;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn public_functions(relative: &str) -> BTreeSet<String> {
    let path = std_src_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}"))
        .definitions
        .into_iter()
        .filter_map(|definition| match definition {
            Definition::Function(function)
                if matches!(function.visibility, ash_parser::surface::Visibility::Public) =>
            {
                Some(function.name.to_string())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn algebra_interface_modules_do_not_publish_concrete_carrier_helpers() {
    for relative in [
        "algebra/functor.ash",
        "algebra/applicative.ash",
        "algebra/monad.ash",
        "algebra/monoid.ash",
        "algebra/kleisli.ash",
    ] {
        let functions = public_functions(relative);
        assert!(
            functions.is_empty(),
            "expected {relative} to expose interfaces only, not concrete helper functions; found {functions:?}"
        );
    }
}

#[test]
fn algebra_interface_method_signatures_are_generic_not_int_placeholders() {
    let expected = [
        ("algebra/functor.ash", "map(F<A>, A -> B) -> F<B>"),
        ("algebra/applicative.ash", "pure(A) -> F<A>"),
        ("algebra/applicative.ash", "apply(F<A -> B>, F<A>) -> F<B>"),
        ("algebra/monad.ash", "unit(A) -> M<A>"),
        ("algebra/monad.ash", "bind(M<A>, A -> M<B>) -> M<B>"),
        ("algebra/comonad.ash", "extract(W<A>) -> A"),
        ("algebra/comonad.ash", "extend(W<A>, W<A> -> B) -> W<B>"),
    ];

    for (relative, signature) in expected {
        let path = std_src_path(relative);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        assert!(
            source.contains(signature),
            "expected {relative} to contain generic signature {signature}; source:\n{source}"
        );
    }
}

#[test]
fn algebra_combinators_stdlib_modules_parse_and_check() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in [
        "algebra/functor.ash",
        "algebra/applicative.ash",
        "algebra/monad.ash",
        "algebra/monoid.ash",
        "act.ash",
    ] {
        let result = engine
            .check_module_file(&std_src_path(relative))
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors: {:?}",
            result.errors
        );
    }
}

#[tokio::test]
async fn carrier_modules_execute_final_surface_monoid_helpers() {
    let project = tempfile::tempdir().expect("project");
    let string_main = project.path().join("string_main.ash");
    std::fs::write(
        &string_main,
        "use string::{concat}\n\nworkflow main { ret concat(\"ok\", \"!\") }\n",
    )
    .expect("write string example");
    let list_main = project.path().join("list_main.ash");
    std::fs::write(
        &list_main,
        "use list::{concat}\n\nworkflow main { ret concat([1], [2]) }\n",
    )
    .expect("write list example");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let string_value = engine
        .run_file(&string_main)
        .await
        .expect("string monoid helper example should execute");
    assert_eq!(string_value, Value::String("ok!".to_string()));

    let list_value = engine
        .run_file(&list_main)
        .await
        .expect("list monoid helper example should execute");
    assert_eq!(
        list_value,
        Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))
    );
}
