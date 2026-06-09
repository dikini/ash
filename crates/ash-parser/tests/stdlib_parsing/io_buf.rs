use super::support::*;

#[test]
fn test_io_buf_file_exists() {
    let path = stdlib_src_path().join("io/buf.ash");
    assert!(path.exists(), "io/buf.ash should exist");
}

#[test]
fn test_io_buf_read_to_end_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        contains_public_callable(&content, "read_to_end"),
        "io/buf.ash should contain read_to_end function"
    );
}

#[test]
fn test_io_buf_read_to_string_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        contains_public_callable(&content, "read_to_string"),
        "io/buf.ash should contain read_to_string function"
    );
}

#[test]
fn test_io_buf_write_all_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        contains_public_callable(&content, "write_all"),
        "io/buf.ash should contain write_all function"
    );
}

#[test]
fn test_io_buf_lines_function_parses() {
    let content = read_stdlib_file("io/buf.ash");
    assert!(
        contains_public_callable(&content, "lines"),
        "io/buf.ash should contain lines function"
    );
}

#[test]
fn test_io_buf_all_required_functions_exist() {
    let content = read_stdlib_file("io/buf.ash");

    let required_functions = ["read_to_end", "read_to_string", "write_all", "lines"];

    for func in &required_functions {
        assert!(
            contains_public_callable(&content, func),
            "io/buf.ash should contain {} function",
            func
        );
    }
}
