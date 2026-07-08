//! TASK-1022 RED tests for importable pure stdlib algebra instance surfaces.

use std::collections::BTreeSet;
use std::path::PathBuf;

use ash_parser::surface::{Definition, Type as SurfaceType};

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn parse_std_module(relative: &str) -> ash_parser::surface::ModuleFile {
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
}

fn imported_interface_names(
    loaded: &ash_engine::module_loader::LoadedOrdinaryFile,
) -> BTreeSet<String> {
    loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .map(|identity| identity.name.clone())
        .collect()
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Name(name) => name.to_string(),
        SurfaceType::Hole { .. } => "_".to_string(),
        SurfaceType::Constructor { name, args } => {
            let args = args
                .iter()
                .map(surface_type_name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{args}>")
        }
        SurfaceType::List(item) => format!("List<{}>", surface_type_name(item)),
        other => format!("{other:?}"),
    }
}

fn public_impl_heads(relative: &str) -> BTreeSet<String> {
    parse_std_module(relative)
        .definitions
        .into_iter()
        .filter_map(|definition| match definition {
            Definition::Impl(implementation)
                if matches!(
                    implementation.visibility,
                    ash_parser::surface::Visibility::Public
                ) =>
            {
                let args = implementation
                    .type_args
                    .iter()
                    .map(surface_type_name)
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("{}<{args}>", implementation.interface))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn pure_algebra_instances_engine_imports_real_stdlib_algebra_surfaces() {
    let project = tempfile::tempdir().expect("project");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        r"use algebra::semigroup::{Semigroup}
use algebra::monoid::{Monoid}
use algebra::functor::{Functor}
use algebra::applicative::{Applicative}
use algebra::monad::{Monad}
use string::{concat}
use list::{map}
fn main() { 0 }
",
    )
    .expect("main");

    let loaded = ash_engine::module_loader::load_ordinary_file(&main)
        .expect("explicit stdlib algebra and pure helper imports should resolve");
    let names = imported_interface_names(&loaded);

    for expected in ["Semigroup", "Monoid", "Functor", "Applicative", "Monad"] {
        assert!(
            names.contains(expected),
            "expected imported interface {expected}; imported names: {names:?}"
        );
    }
}

#[test]
fn pure_algebra_instances_engine_stdlib_files_contain_public_pure_impl_heads() {
    let expected = [
        ("string.ash", "Semigroup<String>"),
        ("list.ash", "Semigroup<List<A>>"),
        ("string.ash", "Monoid<String>"),
        ("list.ash", "Monoid<List<A>>"),
        ("option.ash", "Functor<Option>"),
        ("result.ash", "Functor<Result<_, E>>"),
        ("list.ash", "Functor<List>"),
        ("option.ash", "Applicative<Option>"),
        ("result.ash", "Applicative<Result<_, E>>"),
        ("option.ash", "Monad<Option>"),
        ("result.ash", "Monad<Result<_, E>>"),
    ];

    for (relative, expected_head) in expected {
        let heads = public_impl_heads(relative);
        assert!(
            heads.contains(expected_head),
            "expected {relative} to expose public impl {expected_head}; found {heads:?}"
        );
    }
}

#[test]
fn pure_algebra_instances_engine_stdlib_files_parse_and_check_after_impls() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in [
        "algebra/semigroup.ash",
        "algebra/monoid.ash",
        "algebra/functor.ash",
        "algebra/applicative.ash",
        "algebra/monad.ash",
        "option.ash",
        "result.ash",
        "list.ash",
        "string.ash",
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
