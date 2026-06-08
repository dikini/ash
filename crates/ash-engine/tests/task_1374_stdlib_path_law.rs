#![allow(missing_docs)]

use std::path::PathBuf;

use ash_parser::surface::Definition;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

#[test]
fn std_io_path_declares_join_preserves_absolute_law() {
    let path = std_src_path("io/path.ash");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let module = ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{} should parse: {errors:?}\n{source}", path.display()));

    assert!(
        module.definitions.iter().any(|definition| matches!(
            definition,
            Definition::Law(law) if law.name.as_ref() == "join_preserves_absolute"
        )),
        "std/src/io/path.ash should declare module law join_preserves_absolute"
    );
}

#[test]
fn std_io_path_law_checks_through_engine() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");
    let path = std_src_path("io/path.ash");

    let result = engine
        .check_module_file(&path)
        .unwrap_or_else(|error| panic!("{} should parse/check: {error}", path.display()));

    assert!(
        result.errors.is_empty(),
        "std/src/io/path.ash should not report module errors: {:?}",
        result.errors
    );
}
