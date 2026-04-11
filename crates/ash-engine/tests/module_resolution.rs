//! Integration tests for ordinary file execution with imports and module roots.
//!
//! These tests cover executable import-backed ordinary workflows, including
//! imported type/constructor usage, pure helper calls, and stdlib re-exports.

use ash_core::Value;
use ash_engine::Engine;
use temp_env::async_with_vars;
use tempfile::tempdir;

fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

#[tokio::test]
async fn ordinary_file_execution_resolves_local_user_module_adts_from_entry_root_tree() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path();
    let app_root = root.join("app");
    let entry = app_root.join("main.ash");
    let helper = app_root.join("models/answer.ash");

    // This module path should resolve relative to the entry file's root tree.
    write(
        &helper,
        r"
        pub type Answer = Answer { value: Int };
        ",
    );
    write(
        &entry,
        r"
        use models::answer::{Answer}

        workflow main() -> Answer { ret Answer { value: 7 }; }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should resolve local module types");

    assert!(matches!(
        result,
        Value::Variant { ref name, ref fields }
            if name == "Answer"
                && matches!(&fields[..], [(field, Value::Int(7))] if field == "value")
    ));
}

#[tokio::test]
async fn ordinary_file_execution_calls_imported_local_helper_workflows() {
    let temp = tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    let entry = app_root.join("main.ash");
    let helper = app_root.join("helpers/math.ash");

    write(
        &helper,
        r"
        workflow seven() -> Int { ret 7; }
        ",
    );
    write(
        &entry,
        r"
        use helpers::math::{seven}

        workflow main() -> Int { ret seven(); }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should call imported helper workflows");

    assert_eq!(result, Value::Int(7));
}

#[tokio::test]
async fn ordinary_file_execution_calls_imported_local_pure_functions() {
    let temp = tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    let entry = app_root.join("main.ash");
    let helper = app_root.join("helpers/math.ash");

    write(
        &helper,
        r"
        pub fn seven() -> Int { 7 }
        ",
    );
    write(
        &entry,
        r"
        use helpers::math::{seven}

        workflow main() -> Int { ret seven(); }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should call imported local pure functions");

    assert_eq!(result, Value::Int(7));
}

#[tokio::test]
async fn ordinary_file_execution_executes_stdlib_imports_from_ordinary_files() {
    let temp = tempdir().expect("tempdir");
    let entry = temp.path().join("main.ash");

    // This intentionally exercises executable stdlib imports from an ordinary
    // workflow file, not just import parsing.
    write(
        &entry,
        r"
        use option::{Option, Some}

        workflow main() -> Option<Int> { ret Some { value: 7 }; }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should resolve stdlib modules");

    assert!(matches!(
        result,
        Value::Variant { ref name, ref fields }
            if name == "Some"
                && matches!(&fields[..], [(field, Value::Int(7))] if field == "value")
    ));
}

#[tokio::test]
async fn ordinary_file_execution_calls_direct_stdlib_helper_functions() {
    let temp = tempdir().expect("tempdir");
    let entry = temp.path().join("main.ash");

    write(
        &entry,
        r"
        use option::{Option, Some, is_some}

        workflow main() -> Bool { ret is_some(Some { value: 7 }); }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should call direct stdlib helper functions");

    assert_eq!(result, Value::Bool(true));
}

#[tokio::test]
async fn ordinary_file_execution_calls_prelude_reexported_helper_functions() {
    let temp = tempdir().expect("tempdir");
    let entry = temp.path().join("main.ash");

    write(
        &entry,
        r"
        use prelude::{is_some}
        use option::{Some}

        workflow main() -> Bool { ret is_some(Some { value: 7 }); }
        ",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .run_file(&entry)
        .await
        .expect("ordinary file execution should call prelude re-exported helper functions");

    assert_eq!(result, Value::Bool(true));
}

#[tokio::test]
async fn ordinary_file_execution_resolves_versioned_library_modules_from_ash_library_path() {
    let temp = tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    let entry = app_root.join("main.ash");
    let lib_root = temp.path().join("libs");
    let vector = lib_root.join("math@1/vector.ash");

    write(
        &vector,
        r"
        pub type Vec2 = Vec2 { x: Int, y: Int };
        ",
    );
    write(
        &entry,
        r"
        use math@1::vector::{Vec2}

        workflow main() -> Vec2 { ret Vec2 { x: 3, y: 4 }; }
        ",
    );

    let result = async_with_vars([("ASH_LIBRARY_PATH", Some(&lib_root))], async {
        let engine = Engine::new().build().expect("engine builds");
        engine
            .run_file(&entry)
            .await
            .expect("ordinary file execution should resolve versioned library modules")
    })
    .await;

    assert!(matches!(
        result,
        Value::Variant { ref name, ref fields }
            if name == "Vec2"
                && matches!(
                    &fields[..],
                    [(x, Value::Int(3)), (y, Value::Int(4))] if x == "x" && y == "y"
                )
    ));
}
