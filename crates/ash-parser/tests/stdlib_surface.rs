use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use ash_parser::ModuleResolver;

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

fn read_stdlib_file(path: &str) -> String {
    read_file(workspace_root().join("std/src").join(path))
}

fn normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn runtime_mod_path() -> PathBuf {
    workspace_root().join("std/src/runtime/mod.ash")
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

#[test]
fn runtime_stdlib_surface_is_exposed() {
    let runtime_error = read_stdlib_file("runtime/error.ash");
    let runtime_error_normalized = normalize_whitespace(&runtime_error);
    let runtime_args = read_stdlib_file("runtime/args.ash");
    let runtime_supervisor = read_stdlib_file("runtime/supervisor.ash");
    let lib = read_stdlib_file("lib.ash");

    assert!(
        runtime_error.contains("pub type RuntimeError"),
        "runtime/error.ash should declare RuntimeError"
    );
    assert!(
        runtime_error_normalized.contains("pub type RuntimeError = RuntimeError {"),
        "runtime/error.ash should expose RuntimeError with the canonical single-variant ADT syntax"
    );
    assert!(
        !runtime_error_normalized.contains("pub type RuntimeError = {"),
        "runtime/error.ash should not expose RuntimeError as a plain record alias"
    );
    assert!(
        runtime_args.contains("pub capability Args"),
        "runtime/args.ash should declare Args"
    );
    assert!(
        runtime_supervisor.contains("use super::error::RuntimeError;"),
        "runtime/supervisor.ash should import RuntimeError from its sibling module"
    );
    assert!(
        runtime_supervisor.contains("use super::args::Args;"),
        "runtime/supervisor.ash should import Args from its sibling module"
    );
    assert!(
        runtime_supervisor.contains("pub workflow system_supervisor(args: cap Args) -> Int {"),
        "runtime/supervisor.ash should expose the canonical system_supervisor scaffold"
    );
    assert!(
        runtime_supervisor.contains("0"),
        "runtime/supervisor.ash should keep the minimal placeholder exit-code body"
    );
    let runtime_mod = read_stdlib_file("runtime/mod.ash");
    assert!(
        runtime_mod.contains("pub use supervisor::{system_supervisor};"),
        "runtime/mod.ash should re-export system_supervisor"
    );
    assert!(
        lib.contains("pub use runtime::{RuntimeError, Args};"),
        "lib.ash should expose RuntimeError and Args from runtime"
    );
    assert!(
        lib.contains("pub use runtime::supervisor::{system_supervisor};"),
        "lib.ash should expose system_supervisor from runtime"
    );
}

#[test]
fn runtime_module_tree_resolves_as_real_file_modules() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(runtime_mod_path())
        .expect("runtime/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("runtime module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("runtime module root should exist");

    assert_eq!(root_node.name, "runtime");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert_eq!(
        child_names,
        BTreeSet::from([
            "args".to_string(),
            "error".to_string(),
            "supervisor".to_string(),
        ]),
        "runtime/mod.ash should declare file-based child modules for args, error, and supervisor"
    );
}
