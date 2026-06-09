use super::support::*;

#[test]
fn test_io_stdio_file_exists() {
    let path = stdlib_src_path().join("io/stdio.ash");
    assert!(path.exists(), "io/stdio.ash should exist");
}

#[test]
fn test_io_stdio_read_line_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    // Check that read_line function exists and is public
    assert!(
        contains_public_callable(&content, "read_line"),
        "io/stdio.ash should contain read_line function"
    );
}

#[test]
fn test_io_stdio_print_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        contains_public_callable(&content, "print"),
        "io/stdio.ash should contain print function"
    );
}

#[test]
fn test_io_stdio_println_function_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        contains_public_callable(&content, "println"),
        "io/stdio.ash should contain println function"
    );
}

#[test]
fn test_io_stdio_capability_parses() {
    let content = read_stdlib_file("io/stdio.ash");

    assert!(
        content.contains("pub capability Stdio"),
        "io/stdio.ash should declare Stdio capability"
    );
}

#[test]
fn test_io_stdio_all_required_functions_exist() {
    let content = read_stdlib_file("io/stdio.ash");

    let required_functions = ["read_line", "print", "println"];

    for func in &required_functions {
        assert!(
            contains_public_callable(&content, func),
            "io/stdio.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_io_stdio_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::stdio;",
        "use io::stdio::read_line;",
        "use io::stdio::{print, println};",
        "use io::{read_line, print, println};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io::stdio import should parse: {source}");
    }
}

// TASK-496: io::fs, io::dir, io::meta module parsing tests
// These tests will fail until the filesystem modules are properly implemented
