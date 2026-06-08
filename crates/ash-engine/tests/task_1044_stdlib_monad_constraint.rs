#![allow(missing_docs)]

use std::path::PathBuf;

use ash_parser::surface::Definition;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

#[test]
fn stdlib_monad_surface_requires_applicative_evidence() {
    let path = std_src_path("algebra/monad.ash");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let module = ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("stdlib monad should parse: {errors:?}"));

    let monad = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == "Monad" => {
                Some(interface)
            }
            _ => None,
        })
        .expect("Monad interface should be present");

    assert_eq!(monad.evidence_constraints.len(), 1);
    let constraint = &monad.evidence_constraints[0];
    assert!(
        matches!(&constraint.subject, ash_parser::surface::Type::Name(name) if name.as_ref() == "M")
    );
    assert!(
        matches!(&constraint.interface, ash_parser::surface::Type::Name(name) if name.as_ref() == "Applicative")
    );
    assert!(source.contains("unit(A) -> M<A>"), "{source}");
    assert!(source.contains("bind(M<A>, A -> M<B>) -> M<B>"), "{source}");
}

#[test]
fn stdlib_monad_implementations_discharge_applicative_requirement() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in ["algebra/monad.ash", "option.ash", "result.ash"] {
        let result = engine
            .check_module_file(&std_src_path(relative))
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors after Monad requires Applicative: {:?}",
            result.errors
        );
    }
}
