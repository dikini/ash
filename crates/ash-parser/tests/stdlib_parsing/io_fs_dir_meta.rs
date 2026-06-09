use super::support::*;

#[test]
fn test_io_fs_file_exists() {
    let path = stdlib_src_path().join("io/fs.ash");
    assert!(path.exists(), "io/fs.ash should exist");
}

#[test]
fn test_io_dir_file_exists() {
    let path = stdlib_src_path().join("io/dir.ash");
    assert!(path.exists(), "io/dir.ash should exist");
}

#[test]
fn test_io_meta_file_exists() {
    let path = stdlib_src_path().join("io/meta.ash");
    assert!(path.exists(), "io/meta.ash should exist");
}

#[test]
fn test_io_fs_capability_parses() {
    let content = read_stdlib_file("io/fs.ash");

    assert!(
        content.contains("pub capability Fs"),
        "io/fs.ash should declare Fs capability"
    );
}

#[test]
fn test_io_dir_capability_parses() {
    let content = read_stdlib_file("io/dir.ash");

    assert!(
        content.contains("pub capability Dir"),
        "io/dir.ash should declare Dir capability"
    );
}

#[test]
fn test_io_meta_capability_parses() {
    let content = read_stdlib_file("io/meta.ash");

    assert!(
        content.contains("pub capability Meta"),
        "io/meta.ash should declare Meta capability"
    );
}

#[test]
fn test_io_fs_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::fs;",
        "use io::fs::read;",
        "use io::fs::{read, write};",
        "use io::{read, write_string};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::fs import should parse: {source}");
    }
}

#[test]
fn test_io_dir_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::dir;",
        "use io::dir::create_dir;",
        "use io::dir::{create_dir, remove_dir};",
        "use io::{create_dir_all, read_dir};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::dir import should parse: {source}");
    }
}

#[test]
fn test_io_meta_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::meta;",
        "use io::meta::metadata;",
        "use io::meta::{metadata, is_file};",
        "use io::{is_dir, len, readonly};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::meta import should parse: {source}");
    }
}

#[test]
fn test_io_fs_read_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "read"),
        "io/fs.ash should contain read function"
    );
}

#[test]
fn test_io_fs_read_to_string_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "read_to_string"),
        "io/fs.ash should contain read_to_string function"
    );
}

#[test]
fn test_io_fs_write_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "write"),
        "io/fs.ash should contain write function"
    );
}

#[test]
fn test_io_fs_write_string_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "write_string"),
        "io/fs.ash should contain write_string function"
    );
}

#[test]
fn test_io_fs_append_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "append"),
        "io/fs.ash should contain append function"
    );
}

#[test]
fn test_io_fs_copy_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "copy"),
        "io/fs.ash should contain copy function"
    );
}

#[test]
fn test_io_fs_rename_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "rename"),
        "io/fs.ash should contain rename function"
    );
}

#[test]
fn test_io_fs_remove_file_function_parses() {
    let content = read_stdlib_file("io/fs.ash");
    assert!(
        contains_public_callable(&content, "remove_file"),
        "io/fs.ash should contain remove_file function"
    );
}

#[test]
fn test_io_dir_create_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        contains_public_callable(&content, "create_dir"),
        "io/dir.ash should contain create_dir function"
    );
}

#[test]
fn test_io_dir_create_dir_all_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        contains_public_callable(&content, "create_dir_all"),
        "io/dir.ash should contain create_dir_all function"
    );
}

#[test]
fn test_io_dir_remove_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        contains_public_callable(&content, "remove_dir"),
        "io/dir.ash should contain remove_dir function"
    );
}

#[test]
fn test_io_dir_remove_dir_all_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        contains_public_callable(&content, "remove_dir_all"),
        "io/dir.ash should contain remove_dir_all function"
    );
}

#[test]
fn test_io_dir_read_dir_function_parses() {
    let content = read_stdlib_file("io/dir.ash");
    assert!(
        contains_public_callable(&content, "read_dir"),
        "io/dir.ash should contain read_dir function"
    );
}

#[test]
fn test_io_meta_metadata_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        contains_public_callable(&content, "metadata"),
        "io/meta.ash should contain metadata function"
    );
}

#[test]
fn test_io_meta_is_file_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        contains_public_callable(&content, "is_file"),
        "io/meta.ash should contain is_file function"
    );
}

#[test]
fn test_io_meta_is_dir_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        contains_public_callable(&content, "is_dir"),
        "io/meta.ash should contain is_dir function"
    );
}

#[test]
fn test_io_meta_len_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        contains_public_callable(&content, "len"),
        "io/meta.ash should contain len function"
    );
}

#[test]
fn test_io_meta_readonly_function_parses() {
    let content = read_stdlib_file("io/meta.ash");
    assert!(
        contains_public_callable(&content, "readonly"),
        "io/meta.ash should contain readonly function"
    );
}

#[test]
fn test_io_fs_all_required_functions_exist() {
    let content = read_stdlib_file("io/fs.ash");

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
            contains_public_callable(&content, func),
            "io/fs.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_dir_all_required_functions_exist() {
    let content = read_stdlib_file("io/dir.ash");

    let required_functions = [
        "create_dir",
        "create_dir_all",
        "remove_dir",
        "remove_dir_all",
        "read_dir",
    ];

    for func in &required_functions {
        assert!(
            contains_public_callable(&content, func),
            "io/dir.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_meta_all_required_functions_exist() {
    let content = read_stdlib_file("io/meta.ash");

    let required_functions = ["metadata", "is_file", "is_dir", "len", "readonly"];

    for func in &required_functions {
        assert!(
            contains_public_callable(&content, func),
            "io/meta.ash should contain {} function",
            func
        );
    }
}

// TASK-497: io::buf module parsing tests
// These tests will fail until the buffered helpers module is properly implemented
