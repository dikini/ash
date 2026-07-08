#![allow(missing_docs)]

use std::path::PathBuf;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

#[test]
fn quickcheck_canonical_stdlib_modules_parse_and_check() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in [
        "test/quickcheck/mod.ash",
        "test/quickcheck/context.ash",
        "test/quickcheck/strategy.ash",
        "test/quickcheck/arbitrary.ash",
        "test/quickcheck/int.ash",
        "test/quickcheck/bool.ash",
        "test/quickcheck/string.ash",
        "test/quickcheck/list.ash",
        "test/quickcheck/combinator.ash",
        "test/quickcheck/prelude.ash",
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

#[test]
fn quickcheck_prelude_and_canonical_submodule_imports_resolve() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r"
use test::quickcheck::prelude::{GenContext, Strategy, Arbitrary}
use test::quickcheck::context::{size, choose_int}
use test::quickcheck::int::{ints, positive}
use test::quickcheck::bool::{bools}
use test::quickcheck::string::{strings}
use test::quickcheck::list::{list_of}
use test::quickcheck::combinator::{map, one_of, recursive, recursive_with, recursive_config, default_recursive_config}

fn main() -> Bool { true }
",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let mut workflow = engine.parse_file(&source_path).expect("parse main.ash");
    engine.check(&mut workflow).expect("typecheck main.ash");
}

#[test]
fn quickcheck_root_aliases_resolve_as_alpha_convenience_surface() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r"
use test::quickcheck::{GenContext, Strategy, Arbitrary, ints, bools, strings, list_of, map, one_of, recursive, recursive_with}

fn main() -> Bool { true }
",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let mut workflow = engine.parse_file(&source_path).expect("parse main.ash");
    engine.check(&mut workflow).expect("typecheck main.ash");
}
#[test]
fn recursive_combinator_manual_strategy_fails_closed_until_type_metadata_lands() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r"
use test::quickcheck::strategy::{Strategy, GenContext, no_shrink}
use test::quickcheck::combinator::{recursive}

fn base_gen(ctx: GenContext) -> Int {
    1
}

fn identity_step(strategy: Strategy<Int>) -> Strategy<Int> {
    strategy
}

fn main() -> Strategy<Int> {
    recursive(Strategy { gen: base_gen, shrink: no_shrink }, identity_step)
}
",
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let mut workflow = engine.parse_file(&source_path).expect("parse main.ash");
    let err = engine
        .check(&mut workflow)
        .expect_err("manual recursive strategy should fail closed");

    assert!(
        err.to_string().contains("Type mismatch in field 'gen'"),
        "expected manual Strategy metadata blocker diagnostic, got: {err}"
    );
}
