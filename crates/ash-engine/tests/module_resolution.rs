//! Module resolution integration tests.
//!
//! Tests verify that the module resolver correctly discovers and loads modules
//! from the local file tree, `ASH_LIBRARY_PATH`, and the built-in stdlib root,
//! with proper search precedence, cycle detection, and error reporting.

use ash_engine::Engine;
use tempfile::TempDir;

/// Helper: write `contents` to `path`, creating parent directories as needed.
fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

/// Helper: build a default engine.
fn build_engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

// ── 1. Local sibling module resolution ──────────────────────────────────

/// Two files sit next to each other in the same directory.
/// `main.ash` imports a type from `sibling.ash`.
///
/// Expected: the resolver locates `sibling.ash` relative to the entry file
/// root and the imported type is available for construction.
#[tokio::test]
async fn sibling_module_type_import_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("sibling.ash"),
        "pub type Point = Point { x: Int, y: Int };",
    );
    write(
        &dir.join("main.ash"),
        "\
        use sibling::{Point}\n\
        \n\
        workflow main() -> Point { ret Point { x: 1, y: 2 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    // Validates that the legacy loader resolves this today.
    assert!(
        result.is_ok(),
        "sibling module resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, fields }
                if name == "Point"
                    && fields.len() == 2
                    && fields[0].0 == "x" && fields[0].1 == ash_core::Value::Int(1)
                    && fields[1].0 == "y" && fields[1].1 == ash_core::Value::Int(2)
        ),
        "expected Point {{ x: 1, y: 2 }}, got {:?}",
        value,
    );

}

// ── 2. Nested multi-file modules ───────────────────────────────────────

/// Project layout:
///
/// ```text
/// main.ash
/// foo/mod.ash       -- declares `pub mod bar;`
/// foo/bar.ash       -- defines `pub type Inner`
/// ```
///
/// Expected: `main.ash` can import `foo::bar::Inner` through the nested
/// module tree.  The resolver must follow `pub mod` declarations and load
/// child modules recursively.
#[tokio::test]
async fn nested_pub_mod_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("foo").join("bar.ash"),
        "pub type Inner = Inner { val: Int };",
    );
    write(&dir.join("foo").join("mod.ash"), "pub mod bar;");
    write(
        &dir.join("main.ash"),
        "\
        use foo::bar::{Inner}\n\
        \n\
        workflow main() -> Inner { ret Inner { val: 99 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "nested module resolution: expected successful execution, got: {:?}",
        result.err()
    );
}

// ── 3. Stdlib import resolution ────────────────────────────────────────

/// `main.ash` imports a type from the standard library (e.g. `option`).
///
/// Expected: the resolver falls through to the builtin stdlib root and
/// resolves the module without the user specifying a path.
#[tokio::test]
async fn stdlib_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use option::{Option, Some}\n\
        \n\
        workflow main() -> Option<Int> { ret Some { value: 42 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib module resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, .. } if name == "Some"
        ),
        "expected Some variant, got {:?}",
        value,
    );
}

/// `main.ash` can import the Phase 97 Act stdlib helpers from the builtin
/// stdlib root.
#[tokio::test]
async fn stdlib_act_module_resolution() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use act::{unit, bind, then, guard}\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib act module resolution: expected successful execution, got: {:?}",
        result.err()
    );
}

// ── 4. Missing module error ────────────────────────────────────────────

/// `main.ash` imports from a module `nonexistent` that does not exist on
/// disk or in the stdlib.
///
/// Expected: the resolver returns a clear, actionable error indicating
/// that the module was not found and where it searched.
#[tokio::test]
async fn missing_module_produces_clear_error() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use nonexistent::{Foo}\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(result.is_err(), "missing module should produce an error");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("not found") || msg.contains("module"),
        "error should mention the missing module, got: {msg}",
    );
}

// ── 5. Circular import detection ───────────────────────────────────────

/// Two modules import each other:
///
/// - `a.ash` does `use b::{X}`
/// - `b.ash` does `use a::{Y}`
///
/// Expected: the resolver detects the cycle and returns a deterministic
/// error rather than recursing indefinitely or panicking.
#[tokio::test]
async fn circular_import_detection() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("a.ash"),
        "\
        use b::{X}\n\
        \n\
        pub type Y = Y { v: Int };\n\
        \n\
        workflow main() -> Int { ret 1; }\n\
        ",
    );
    write(
        &dir.join("b.ash"),
        "\
        use a::{Y}\n\
        \n\
        pub type X = X { v: Int };\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("a.ash")).await;

    // The ModuleResolver must detect circular imports and report them
    // with a clear "cyclic" or "circular" message.
    match result {
        Err(ref e) => {
            let msg = format!("{e}");
            assert!(
                msg.to_lowercase().contains("cyclic") || msg.to_lowercase().contains("circular"),
                "circular import should report a cycle, got: {msg}",
            );
        }
        Ok(_) => {
            panic!(
                "circular import should be detected and reported as an error, \
                 but execution succeeded"
            );
        }
    }
}

// ── 6. ASH_LIBRARY_PATH search order ───────────────────────────────────

/// Verify that the resolver searches paths in the following order:
///
///   1. Entry file root tree (local files)
///   2. Directories listed in `ASH_LIBRARY_PATH`
///   3. Built-in stdlib root
///
/// A module with the same name exists both locally and in the library path;
/// the local one must win.
#[tokio::test]
async fn ash_library_path_search_order_local_wins_over_lib() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    // Local module -- value 10
    write(
        &dir.join("mymod.ash"),
        "pub type LocalResult = LocalResult { val: Int };",
    );

    // Library-path module -- value 20 (should NOT be used)
    let lib_dir = temp.path().join("libs");
    write(
        &lib_dir.join("mymod.ash"),
        "pub type LocalResult = LocalResult { val: Int };",
    );

    write(
        &dir.join("main.ash"),
        "\
        use mymod::{LocalResult}\n\
        \n\
        workflow main() -> LocalResult { ret LocalResult { val: 10 }; }\
        ",
    );

    let lib_dir_clone = lib_dir.clone();
    let result = temp_env::async_with_vars(
        [("ASH_LIBRARY_PATH", Some(lib_dir_clone.as_os_str()))],
        async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        },
    )
    .await;

    assert!(
        result.is_ok(),
        "ASH_LIBRARY_PATH search order: local should win, got: {:?}",
        result.err()
    );
}

/// When no local module exists, the resolver falls through to
/// `ASH_LIBRARY_PATH`.
#[tokio::test]
async fn ash_library_path_falls_through_to_lib_dir() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();
    let lib_dir = temp.path().join("libs");

    // Only in library path -- NOT local.
    write(
        &lib_dir.join("extmod.ash"),
        "pub type ExtVal = ExtVal { n: Int };",
    );

    write(
        &dir.join("main.ash"),
        "\
        use extmod::{ExtVal}\n\
        \n\
        workflow main() -> ExtVal { ret ExtVal { n: 5 }; }\n\
        ",
    );

    let lib_dir_clone = lib_dir.clone();
    let result = temp_env::async_with_vars(
        [("ASH_LIBRARY_PATH", Some(lib_dir_clone.as_os_str()))],
        async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        },
    )
    .await;

    assert!(
        result.is_ok(),
        "ASH_LIBRARY_PATH fallback: should resolve from lib dir, got: {:?}",
        result.err()
    );
}

/// When neither a local module nor an `ASH_LIBRARY_PATH` entry exists, the
/// resolver should fall through to the builtin stdlib root.
#[tokio::test]
async fn ash_library_path_falls_through_to_stdlib() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use option::{Option, Some}\n\
        \n\
        workflow main() -> Option<Int> { ret Some { value: 1 }; }\n\
        ",
    );

    let result =
        temp_env::async_with_vars([("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>)], async {
            let engine = build_engine();
            engine.run_file(dir.join("main.ash")).await
        })
        .await;

    assert!(
        result.is_ok(),
        "stdlib fallback: should resolve `option` from builtin root, got: {:?}",
        result.err()
    );
}

// ── 7. Comprehensive stdlib resolution ─────────────────────────────────

/// Verify that multiple stdlib modules resolve together: `result` type,
/// `string` builtin functions, and `list` builtin functions all work
/// when imported through the resolver.
#[tokio::test]
async fn stdlib_result_type_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use result::{Result, Ok, Err}\n\
        \n\
        workflow main() -> Result<Int, String> { ret Ok { value: 42 }; }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib result resolution: expected successful execution, got: {:?}",
        result.err()
    );
    let value = result.expect("checked above");
    assert!(
        matches!(
            &value,
            ash_core::Value::Variant { name, .. } if name == "Ok"
        ),
        "expected Ok variant, got {:?}",
        value,
    );
}

/// Verify that `string` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_string_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use string::{concat}\n\
        \n\
        workflow main() -> String { ret concat(\"hello\", \" world\"); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib string builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(
        result.expect("checked above"),
        ash_core::Value::String("hello world".to_string()),
    );
}

/// Verify that `list` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_list_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use list::{len}\n\
        \n\
        workflow main() -> Int { ret len([1, 2, 3]); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib list builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(result.expect("checked above"), ash_core::Value::Int(3));
}

/// Verify that `predicate` builtin functions resolve through the stdlib root.
#[tokio::test]
async fn stdlib_predicate_builtin_resolves() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
        use predicate::{is_int}\n\
        \n\
        workflow main() -> Bool { ret is_int(42); }\n\
        ",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "stdlib predicate builtin: expected successful execution, got: {:?}",
        result.err()
    );
    assert_eq!(result.expect("checked above"), ash_core::Value::Bool(true));
}
