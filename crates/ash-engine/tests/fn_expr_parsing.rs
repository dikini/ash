//! TASK-1582: Test fn expression parsing in function arguments.

#![allow(clippy::needless_raw_string_hashes, clippy::uninlined_format_args)]

use ash_engine::Engine;
use std::fs;

fn build_engine() -> Engine {
    Engine::new().build().expect("engine should build")
}

#[tokio::test]
async fn test_fn_expr_in_function_argument() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let main_path = tmp_dir.path().join("main.ash");

    fs::write(
        &main_path,
        r#"
use list::{map}

workflow main() -> Bool {
    let list = [1, 2, 3]
    let mapped = map(list, fn(x) { x + 1 })
    ret mapped == [2, 3, 4]
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&main_path).await;

    assert!(
        result.is_ok(),
        "fn expression in argument should execute: {:?}",
        result
    );
    assert_eq!(result.unwrap(), ash_core::Value::Bool(true));
}
