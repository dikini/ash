//! TASK-626: End-to-end tests for std/src/record.ash stdlib import.

fn record_main_source(imports: &str, return_type: &str, body: &str) -> String {
    format!("use record::{{{imports}}}\nfn main() -> {return_type} {{ {body} }}\n")
}

#[tokio::test]
async fn record_stdlib_keys_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(dir.join("main.ash"), record_main_source("keys", "Int", "1"))
        .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Int(1));
}

#[tokio::test]
async fn record_stdlib_values_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        record_main_source("values", "Int", "1"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Int(1));
}

#[tokio::test]
async fn record_literal_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "fn main() -> { x: Int } { { x: 42 } }\n",
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::Record(_)),
        "record() should return a Record, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_all_three_functions_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "use record::{keys, values, record}\nfn main() -> { k: Int } { { k: 1 } }\n",
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::Record(_)),
        "expected Record, got: {result:?}"
    );
}
