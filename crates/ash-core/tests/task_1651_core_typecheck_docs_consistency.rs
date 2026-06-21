use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn read_reference_page() -> String {
    let path = repo_root().join("docs/reference/core-ash-type-checking.md");
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

#[test]
fn core_typechecking_reference_names_implemented_boundary() {
    let source = read_reference_page();

    for required in [
        "ash-core::core_ash_typecheck",
        "type_check_core_program",
        "type_check_and_lower_core_program",
        "ValidCoreProgram",
        "TypedCoreProgram",
        "CoreTypeCheckEnv",
        "CoreTypeCheckFacts",
        "parse -> validate -> type-check -> lower",
    ] {
        assert!(
            source.contains(required),
            "reference page should document `{required}`"
        );
    }
}

#[test]
fn core_typechecking_reference_names_algorithmic_profile_and_deferred_features() {
    let source = read_reference_page();

    for required in [
        "annotation-led",
        "structural row solving",
        "refinement obligations",
        "discharge metadata",
        "not full Hindley-Milner inference",
        "not proof solving",
        "not typeclass solving",
        "not MultiShotPure",
    ] {
        assert!(
            source.contains(required),
            "reference page should document `{required}`"
        );
    }
}
