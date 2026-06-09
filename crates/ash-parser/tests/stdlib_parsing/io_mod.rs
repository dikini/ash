use super::support::*;

#[test]
fn test_io_mod_file_exists() {
    let path = stdlib_src_path().join("io/mod.ash");
    assert!(path.exists(), "io/mod.ash should exist");
}

#[test]
fn test_io_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use io::path;",
        "use io::path::PathBuf;",
        "use io::{Error, ErrorKind};",
        "use io::path::{PathBuf, from_string};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "io import should parse: {source}");
    }
}

#[test]
fn test_io_mod_exports_path() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod path;"),
        "io/mod.ash should declare path module"
    );
    assert!(
        content.contains("pub use path::"),
        "io/mod.ash should re-export from path module"
    );
}

#[test]
fn test_io_mod_exports_error_types() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("pub type Error"),
        "io/mod.ash should export Error type"
    );
    assert!(
        content.contains("pub type ErrorKind"),
        "io/mod.ash should export ErrorKind type"
    );
    assert!(
        content.contains("io::Result<T> remains deferred"),
        "io/mod.ash should document deferred io::Result<T> alias"
    );
}

#[test]
fn test_lib_exports_io() {
    let content = read_stdlib_file("lib.ash");

    assert!(
        content.contains("pub use io::"),
        "lib.ash should export from io module"
    );
}

#[test]
fn test_io_mod_exports_stdio() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod stdio;"),
        "io/mod.ash should declare stdio module"
    );
    assert!(
        content.contains("pub use stdio::"),
        "io/mod.ash should re-export from stdio module"
    );
}

#[test]
fn test_io_mod_exports_fs() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod fs;"),
        "io/mod.ash should declare fs module"
    );
    assert!(
        content.contains("pub use fs::"),
        "io/mod.ash should re-export from fs module"
    );
}

#[test]
fn test_io_mod_exports_dir() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod dir;"),
        "io/mod.ash should declare dir module"
    );
    assert!(
        content.contains("pub use dir::"),
        "io/mod.ash should re-export from dir module"
    );
}

#[test]
fn test_io_mod_exports_meta() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod meta;"),
        "io/mod.ash should declare meta module"
    );
    assert!(
        content.contains("pub use meta::"),
        "io/mod.ash should re-export from meta module"
    );
}

#[test]
fn test_io_mod_exports_buf() {
    let content = read_stdlib_file("io/mod.ash");

    assert!(
        content.contains("mod buf;"),
        "io/mod.ash should declare buf module"
    );
    assert!(
        content.contains("pub use buf::"),
        "io/mod.ash should re-export from buf module"
    );
}
