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

fn io_mod_path() -> PathBuf {
    workspace_root().join("std/src/io/mod.ash")
}

#[test]
fn prelude_exposes_the_canonical_adt_helper_surface() {
    let content = read_file(workspace_root().join("std/src/prelude.ash"));

    assert!(
        content.contains(
            "pub use option::{is_some, is_none, unwrap, unwrap_or, map, and_opt, or_opt, ok_or};",
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
        runtime_error_normalized.contains("pub type RuntimeError = RuntimeError(Int, String);"),
        "runtime/error.ash should expose RuntimeError with the canonical tuple-variant ADT syntax"
    );
    assert!(
        !runtime_error_normalized.contains("pub type RuntimeError = RuntimeError {"),
        "runtime/error.ash should not expose RuntimeError with record payload syntax"
    );
    assert!(
        runtime_args.contains("pub capability Args"),
        "runtime/args.ash should declare Args"
    );
    assert!(
        runtime_supervisor.contains("use result::{Result, Err};"),
        "runtime/supervisor.ash should import the canonical Result surface"
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
        "runtime/supervisor.ash should expose the canonical system_supervisor contract"
    );
    assert!(
        runtime_supervisor.contains("Result<(), RuntimeError>"),
        "runtime/supervisor.ash should document the runtime-provided terminal Result contract"
    );
    assert!(
        !runtime_supervisor.contains("parser-feasible stand-in"),
        "runtime/supervisor.ash should drop the parser-feasible completion placeholder wording"
    );
    assert!(
        !runtime_supervisor.contains("supervisor_completion"),
        "runtime/supervisor.ash should reject the unresolved supervisor_completion placeholder"
    );
    assert!(
        !runtime_supervisor.contains("let completion="),
        "runtime/supervisor.ash should not bind a fake completion payload"
    );
    assert!(
        runtime_supervisor.contains("if let Err"),
        "runtime/supervisor.ash should shape RuntimeError exit codes through if-let destructuring"
    );
    assert!(
        runtime_supervisor.contains("Err { error: RuntimeError(code, _) }"),
        "runtime/supervisor.ash should keep the nested RuntimeError exit-code destructuring intent"
    );
    assert!(
        runtime_supervisor.contains("then code else 0"),
        "runtime/supervisor.ash should preserve the fallback exit-code shaping intent"
    );
    assert!(
        runtime_supervisor.contains("ret exit_code;"),
        "runtime/supervisor.ash should return the shaped exit code"
    );
    assert!(
        !runtime_supervisor.contains("ret 0;"),
        "runtime/supervisor.ash should reject the old placeholder return body"
    );
    assert!(
        !runtime_supervisor.contains("await"),
        "runtime/supervisor.ash should not invent await syntax"
    );
    assert!(
        runtime_supervisor.contains("TASK-363c wires that bootstrap behavior"),
        "runtime/supervisor.ash should keep the runtime/bootstrap boundary explicit"
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
fn test_stdlib_exposes_minimal_test_surface() {
    let test_module = read_stdlib_file("test.ash");
    let lib = read_stdlib_file("lib.ash");

    assert!(
        test_module.contains("literal panic messages inside function bodies"),
        "test.ash should document why the v1 surface uses fixed panic strings"
    );
    assert!(
        test_module.contains("pub fn assert_true(value: Bool) -> Bool"),
        "test.ash should expose the parseable assert_true signature"
    );
    assert!(
        test_module.contains("pub fn fail() -> Bool"),
        "test.ash should expose the minimal zero-argument fail helper"
    );
    assert!(
        !test_module.contains("panic(message)"),
        "test.ash should not use unsupported panic call syntax"
    );
    assert!(
        !test_module.contains("++"),
        "test.ash should not use unsupported string concatenation syntax"
    );
    assert!(
        lib.contains("pub use test::{"),
        "lib.ash should re-export the std::test surface"
    );
    assert!(
        lib.contains("assert_true"),
        "lib.ash should export std::test helpers"
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

// TASK-494: io module surface tests
// These tests will fail until the io module and io::path are properly implemented

#[test]
fn io_module_tree_resolves_as_directory_module_root() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    assert_eq!(root_node.name, "io");
}

#[test]
fn io_module_has_path_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("path"),
        "io/mod.ash should declare a file-based child module for path"
    );
}

#[test]
fn io_mod_file_exists() {
    let path = workspace_root().join("std/src/io/mod.ash");
    assert!(path.exists(), "io/mod.ash should exist");
}

#[test]
fn io_path_file_exists() {
    let path = workspace_root().join("std/src/io/path.ash");
    assert!(path.exists(), "io/path.ash should exist");
}

#[test]
fn io_module_exposes_error_type() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("pub type Error"),
        "io/mod.ash should declare Error type"
    );
    assert!(
        io_mod.contains("pub type ErrorKind"),
        "io/mod.ash should declare ErrorKind type"
    );
    assert!(
        io_mod.contains("pub type Result<T>"),
        "io/mod.ash should declare Result<T> type alias"
    );
}

#[test]
fn io_path_has_required_functions() {
    let path_content = read_stdlib_file("io/path.ash");

    let required_functions = [
        "from_string",
        "join",
        "parent",
        "file_name",
        "extension",
        "is_absolute",
    ];

    for func in &required_functions {
        assert!(
            path_content.contains(&format!("pub fn {}", func)),
            "io/path.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_module_exports_path_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("pub use path::"),
        "io/mod.ash should re-export items from path module"
    );
}

#[test]
fn lib_exports_io_module() {
    let lib = read_stdlib_file("lib.ash");

    assert!(
        lib.contains("pub use io::"),
        "lib.ash should export items from io module"
    );
}

#[test]
fn io_path_exports_pathbuf_type() {
    let path_content = read_stdlib_file("io/path.ash");

    assert!(
        path_content.contains("pub type PathBuf"),
        "io/path.ash should declare PathBuf type"
    );
}

// TASK-495: io::stdio module surface tests
// These tests will fail until the io::stdio module is properly implemented

#[test]
fn io_stdio_file_exists() {
    let path = workspace_root().join("std/src/io/stdio.ash");
    assert!(path.exists(), "io/stdio.ash should exist");
}

#[test]
fn io_module_has_stdio_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("stdio"),
        "io/mod.ash should declare a file-based child module for stdio"
    );
}

#[test]
fn io_stdio_has_required_functions() {
    let stdio_content = read_stdlib_file("io/stdio.ash");

    let required_functions = ["read_line", "print", "println"];

    for func in &required_functions {
        assert!(
            stdio_content.contains(&format!("pub fn {}", func)),
            "io/stdio.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_stdio_has_stdio_capability() {
    let stdio_content = read_stdlib_file("io/stdio.ash");

    assert!(
        stdio_content.contains("pub capability Stdio"),
        "io/stdio.ash should declare Stdio capability"
    );
}

#[test]
fn io_module_exports_stdio_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("pub mod stdio;"),
        "io/mod.ash should declare stdio submodule"
    );
}

#[test]
fn io_module_exports_stdio_functions() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("pub use stdio::"),
        "io/mod.ash should re-export items from stdio module"
    );
}

#[test]
fn lib_exports_io_stdio() {
    let lib = read_stdlib_file("lib.ash");

    assert!(
        lib.contains("pub use io::stdio::"),
        "lib.ash should export stdio functions from io module"
    );
}

// TASK-496: io::fs, io::dir, io::meta module surface tests
// These tests will fail until the filesystem modules are properly implemented

#[test]
fn io_fs_file_exists() {
    let path = workspace_root().join("std/src/io/fs.ash");
    assert!(path.exists(), "io/fs.ash should exist");
}

#[test]
fn io_dir_file_exists() {
    let path = workspace_root().join("std/src/io/dir.ash");
    assert!(path.exists(), "io/dir.ash should exist");
}

#[test]
fn io_meta_file_exists() {
    let path = workspace_root().join("std/src/io/meta.ash");
    assert!(path.exists(), "io/meta.ash should exist");
}

#[test]
fn io_module_has_fs_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("fs"),
        "io/mod.ash should declare a file-based child module for fs"
    );
}

#[test]
fn io_module_has_dir_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("dir"),
        "io/mod.ash should declare a file-based child module for dir"
    );
}

#[test]
fn io_module_has_meta_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("meta"),
        "io/mod.ash should declare a file-based child module for meta"
    );
}

#[test]
fn io_fs_has_required_functions() {
    let fs_content = read_stdlib_file("io/fs.ash");

    let required_functions = [
        "read",
        "read_to_string",
        "write",
        "write_string",
        "append",
        "copy",
        "rename",
        "remove_file",
    ];

    for func in &required_functions {
        assert!(
            fs_content.contains(&format!("pub fn {}", func)),
            "io/fs.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_dir_has_required_functions() {
    let dir_content = read_stdlib_file("io/dir.ash");

    let required_functions = [
        "create_dir",
        "create_dir_all",
        "remove_dir",
        "remove_dir_all",
        "read_dir",
    ];

    for func in &required_functions {
        assert!(
            dir_content.contains(&format!("pub fn {}", func)),
            "io/dir.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_meta_has_required_functions() {
    let meta_content = read_stdlib_file("io/meta.ash");

    let required_functions = ["metadata", "is_file", "is_dir", "len", "readonly"];

    for func in &required_functions {
        assert!(
            meta_content.contains(&format!("pub fn {}", func)),
            "io/meta.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_fs_has_fs_capability() {
    let fs_content = read_stdlib_file("io/fs.ash");

    assert!(
        fs_content.contains("pub capability Fs"),
        "io/fs.ash should declare Fs capability"
    );
}

#[test]
fn io_dir_has_dir_capability() {
    let dir_content = read_stdlib_file("io/dir.ash");

    assert!(
        dir_content.contains("pub capability Dir"),
        "io/dir.ash should declare Dir capability"
    );
}

#[test]
fn io_meta_has_meta_capability() {
    let meta_content = read_stdlib_file("io/meta.ash");

    assert!(
        meta_content.contains("pub capability Meta"),
        "io/meta.ash should declare Meta capability"
    );
}

#[test]
fn io_module_exports_fs_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("mod fs;"),
        "io/mod.ash should declare fs submodule"
    );
    assert!(
        io_mod.contains("pub use fs::"),
        "io/mod.ash should re-export items from fs module"
    );
}

#[test]
fn io_module_exports_dir_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("mod dir;"),
        "io/mod.ash should declare dir submodule"
    );
    assert!(
        io_mod.contains("pub use dir::"),
        "io/mod.ash should re-export items from dir module"
    );
}

#[test]
fn io_module_exports_meta_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("mod meta;"),
        "io/mod.ash should declare meta submodule"
    );
    assert!(
        io_mod.contains("pub use meta::"),
        "io/mod.ash should re-export items from meta module"
    );
}

#[test]
fn lib_exports_io_fs_dir_meta() {
    let lib = read_stdlib_file("lib.ash");

    assert!(
        lib.contains("pub use io::fs::"),
        "lib.ash should export fs functions from io module"
    );
    assert!(
        lib.contains("pub use io::dir::"),
        "lib.ash should export dir functions from io module"
    );
    assert!(
        lib.contains("pub use io::meta::"),
        "lib.ash should export meta functions from io module"
    );
}

#[test]
fn io_fs_has_read_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn read"),
        "io/fs.ash should contain read function"
    );
}

#[test]
fn io_fs_has_read_to_string_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn read_to_string"),
        "io/fs.ash should contain read_to_string function"
    );
}

#[test]
fn io_fs_has_write_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn write"),
        "io/fs.ash should contain write function"
    );
}

#[test]
fn io_fs_has_write_string_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn write_string"),
        "io/fs.ash should contain write_string function"
    );
}

#[test]
fn io_fs_has_append_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn append"),
        "io/fs.ash should contain append function"
    );
}

#[test]
fn io_fs_has_copy_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn copy"),
        "io/fs.ash should contain copy function"
    );
}

#[test]
fn io_fs_has_rename_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn rename"),
        "io/fs.ash should contain rename function"
    );
}

#[test]
fn io_fs_has_remove_file_function() {
    let fs_content = read_stdlib_file("io/fs.ash");
    assert!(
        fs_content.contains("pub fn remove_file"),
        "io/fs.ash should contain remove_file function"
    );
}

#[test]
fn io_dir_has_create_dir_function() {
    let dir_content = read_stdlib_file("io/dir.ash");
    assert!(
        dir_content.contains("pub fn create_dir"),
        "io/dir.ash should contain create_dir function"
    );
}

#[test]
fn io_dir_has_create_dir_all_function() {
    let dir_content = read_stdlib_file("io/dir.ash");
    assert!(
        dir_content.contains("pub fn create_dir_all"),
        "io/dir.ash should contain create_dir_all function"
    );
}

#[test]
fn io_dir_has_remove_dir_function() {
    let dir_content = read_stdlib_file("io/dir.ash");
    assert!(
        dir_content.contains("pub fn remove_dir"),
        "io/dir.ash should contain remove_dir function"
    );
}

#[test]
fn io_dir_has_remove_dir_all_function() {
    let dir_content = read_stdlib_file("io/dir.ash");
    assert!(
        dir_content.contains("pub fn remove_dir_all"),
        "io/dir.ash should contain remove_dir_all function"
    );
}

#[test]
fn io_dir_has_read_dir_function() {
    let dir_content = read_stdlib_file("io/dir.ash");
    assert!(
        dir_content.contains("pub fn read_dir"),
        "io/dir.ash should contain read_dir function"
    );
}

#[test]
fn io_meta_has_metadata_function() {
    let meta_content = read_stdlib_file("io/meta.ash");
    assert!(
        meta_content.contains("pub fn metadata"),
        "io/meta.ash should contain metadata function"
    );
}

#[test]
fn io_meta_has_is_file_function() {
    let meta_content = read_stdlib_file("io/meta.ash");
    assert!(
        meta_content.contains("pub fn is_file"),
        "io/meta.ash should contain is_file function"
    );
}

#[test]
fn io_meta_has_is_dir_function() {
    let meta_content = read_stdlib_file("io/meta.ash");
    assert!(
        meta_content.contains("pub fn is_dir"),
        "io/meta.ash should contain is_dir function"
    );
}

#[test]
fn io_meta_has_len_function() {
    let meta_content = read_stdlib_file("io/meta.ash");
    assert!(
        meta_content.contains("pub fn len"),
        "io/meta.ash should contain len function"
    );
}

#[test]
fn io_meta_has_readonly_function() {
    let meta_content = read_stdlib_file("io/meta.ash");
    assert!(
        meta_content.contains("pub fn readonly"),
        "io/meta.ash should contain readonly function"
    );
}

// TASK-497: io::buf module surface tests
// These tests will fail until the buffered helpers module is properly implemented

#[test]
fn io_buf_file_exists() {
    let path = workspace_root().join("std/src/io/buf.ash");
    assert!(path.exists(), "io/buf.ash should exist");
}

#[test]
fn io_module_has_buf_child_module() {
    let resolver = ModuleResolver::new();
    let graph = resolver
        .resolve_crate(io_mod_path())
        .expect("io/mod.ash should resolve as a directory module root");

    let root_id = graph.root.expect("io module graph should have a root");
    let root_node = graph
        .get_node(root_id)
        .expect("io module root should exist");

    let child_names: BTreeSet<_> = root_node
        .children
        .iter()
        .map(|&child_id| graph.get_node(child_id).unwrap().name.clone())
        .collect();

    assert!(
        child_names.contains("buf"),
        "io/mod.ash should declare a file-based child module for buf"
    );
}

#[test]
fn io_buf_has_required_functions() {
    let buf_content = read_stdlib_file("io/buf.ash");

    let required_functions = ["read_to_end", "read_to_string", "write_all", "lines"];

    for func in &required_functions {
        assert!(
            buf_content.contains(&format!("pub fn {}", func)),
            "io/buf.ash should contain {} function",
            func
        );
    }
}

#[test]
fn io_module_exports_buf_module() {
    let io_mod = read_stdlib_file("io/mod.ash");

    assert!(
        io_mod.contains("mod buf;"),
        "io/mod.ash should declare buf submodule"
    );
    assert!(
        io_mod.contains("pub use buf::"),
        "io/mod.ash should re-export items from buf module"
    );
}

#[test]
fn lib_exports_io_buf() {
    let lib = read_stdlib_file("lib.ash");

    assert!(
        lib.contains("pub use io::buf::"),
        "lib.ash should export buf functions from io module"
    );
}
