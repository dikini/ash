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
fn algebra_combinators_stdlib_surfaces_expose_honest_helpers() {
    let expected = [
        ("algebra/functor.ash", "map_option"),
        ("algebra/functor.ash", "map_result"),
        ("algebra/functor.ash", "map_list"),
        ("algebra/applicative.ash", "pure_option"),
        ("algebra/applicative.ash", "apply_option"),
        ("algebra/applicative.ash", "pure_result"),
        ("algebra/applicative.ash", "apply_result"),
        ("algebra/monad.ash", "unit_option"),
        ("algebra/monad.ash", "bind_option"),
        ("algebra/monad.ash", "unit_result"),
        ("algebra/monad.ash", "bind_result"),
        ("algebra/monoid.ash", "concat_string"),
        ("algebra/monoid.ash", "concat_list"),
    ];

    for (relative, function_name) in expected {
        let functions = public_functions(relative);
        assert!(
            functions.contains(function_name),
            "expected {relative} to expose {function_name}; found {functions:?}"
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
async fn algebra_combinators_execute_final_surface_monoid_helpers() {
    let project = tempfile::tempdir().expect("project");
    let string_main = project.path().join("string_main.ash");
    std::fs::write(
        &string_main,
        "use algebra::monoid::{concat_string}\n\nworkflow main { ret concat_string(\"ok\", \"!\") }\n",
    )
    .expect("write string example");
    let list_main = project.path().join("list_main.ash");
    std::fs::write(
        &list_main,
        "use algebra::monoid::{concat_list}\n\nworkflow main { ret concat_list([1], [2]) }\n",
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
