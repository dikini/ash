#![allow(missing_docs)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use ash_parser::surface::Definition;

const CLOSED_ADMISSION_ENTRY_RESULT_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge currently accepts atomic, atom-only binary primitives, atomic-not, variable-let, and boolean-if entry results";

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn public_functions(relative: &str) -> BTreeSet<String> {
    let path = std_src_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let source_without_imports = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");
    ash_parser::parse_surface_file(&source_without_imports)
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
        ("algebra/functor.ash", "map(F<A>, (A) -> B) -> F<B>"),
        ("algebra/applicative.ash", "pure(A) -> F<A>"),
        (
            "algebra/applicative.ash",
            "apply(F<(A) -> B>, F<A>) -> F<B>",
        ),
        ("algebra/monad.ash", "unit(A) -> M<A>"),
        ("algebra/monad.ash", "bind(M<A>, (A) -> M<B>) -> M<B>"),
        ("algebra/comonad.ash", "extract(W<A>) -> A"),
        ("algebra/comonad.ash", "extend(W<A>, (W<A>) -> B) -> W<B>"),
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
        "string.ash",
        "list.ash",
        "option.ash",
        "result.ash",
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
async fn carrier_modules_typecheck_final_surface_monoid_helpers_then_reject_closed_admission() {
    let project = tempfile::tempdir().expect("project");
    let string_main = project.path().join("string_main.ash");
    std::fs::write(
        &string_main,
        "use string::{concat}\n\nfn main() -> String { concat(\"ok\", \"!\") }\n",
    )
    .expect("write string example");
    let list_main = project.path().join("list_main.ash");
    std::fs::write(
        &list_main,
        "use list::{concat}\n\nfn main() -> List<Int> { concat([1], [2]) }\n",
    )
    .expect("write list example");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let mut string_application = engine
        .parse_file(&string_main)
        .expect("string monoid helper source should parse");
    assert!(
        string_application
            .imported_builtin_signatures
            .contains_key("concat"),
        "string::concat's public builtin signature must cross the module boundary"
    );
    engine
        .check(&mut string_application)
        .expect("string monoid helper source should typecheck against string::concat");

    let mut list_application = engine
        .parse_file(&list_main)
        .expect("list monoid helper source should parse");
    assert!(
        list_application
            .imported_fn_signatures
            .contains_key("concat"),
        "list::concat's public function signature must cross the module boundary"
    );
    engine
        .check(&mut list_application)
        .expect("list monoid helper source should typecheck against list::concat");

    let string_error = engine
        .run_file(&string_main)
        .await
        .expect_err("string monoid helper lacks validated typed lowering");
    assert_eq!(
        string_error.to_string(),
        CLOSED_ADMISSION_ENTRY_RESULT_ERROR,
        "string helper calls must reject at the exact checked Core/CPS admission boundary"
    );

    let list_error = engine
        .run_file(&list_main)
        .await
        .expect_err("list monoid helper lacks validated typed lowering");
    assert_eq!(
        list_error.to_string(),
        CLOSED_ADMISSION_ENTRY_RESULT_ERROR,
        "list helper calls must reject at the exact checked Core/CPS admission boundary"
    );
}
