use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path
}

fn read_file(path: impl AsRef<std::path::Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn prelude_exposes_the_canonical_adt_helper_surface() {
    let content = read_file(workspace_root().join("std/src/prelude.ash"));

    assert!(
        content.contains(
            "pub use option::{is_some, is_none, unwrap, unwrap_or, map, and, or, ok_or};",
        ),
        "prelude should expose the full canonical Option helper surface"
    );
    assert!(
        content.contains(
            "pub use result::{is_ok, is_err, unwrap as unwrap_res, unwrap_err, unwrap_or as unwrap_or_res, map as map_res, map_err, and_then, ok, err as err_opt};",
        ),
        "prelude should expose the full canonical Result helper surface"
    );
}

#[test]
fn examples_readme_describes_the_canonical_adt_helper_surface() {
    let content = read_file(workspace_root().join("examples/README.md"));

    assert!(
        content.contains("Option helper surface"),
        "examples README should call out the canonical Option helper surface"
    );
    assert!(
        content.contains("Result helper surface"),
        "examples README should call out the canonical Result helper surface"
    );
    assert!(
        content.contains("ok_or"),
        "examples README should mention the canonical ADT helper set"
    );
    assert!(
        content.contains("and_then"),
        "examples README should mention the canonical ADT helper set"
    );
}
