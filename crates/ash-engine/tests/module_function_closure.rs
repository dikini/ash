//! TASK-1580: Test module-level function visibility in closures.

use ash_engine::Engine;
use std::fs;

fn build_engine() -> Engine {
    Engine::new()
        .build()
        .expect("engine should build")
}

#[tokio::test]
async fn test_module_function_in_closure() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let main_path = tmp_dir.path().join("main.ash");

    fs::write(
        &main_path,
        r#"
fn add_one(x: Int) -> Int { x + 1 }
fn mul_two(x: Int) -> Int { x * 2 }

workflow main() -> Bool {
    let f = fn(x) { add_one(mul_two(x)) }
    let result = f(5)
    ret result == 11
}
"#,
    ).expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&main_path).await;

    assert!(result.is_ok(), "module function in closure should execute: {:?}", result);
    assert_eq!(result.unwrap(), ash_core::Value::Bool(true));
}
