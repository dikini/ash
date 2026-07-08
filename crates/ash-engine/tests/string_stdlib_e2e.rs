//! TASK-623: End-to-end tests for std/src/string.ash stdlib import.

fn string_main_source(imports: &str, return_type: &str, body: &str) -> String {
    format!("use string::{{{imports}}}\nfn main() -> {return_type} {{ {body} }}\n")
}

#[tokio::test]
async fn string_stdlib_concat_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source("concat", "String", "concat(\"foo\", \"bar\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("foobar".to_string()));
}

#[tokio::test]
async fn string_stdlib_starts_with_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source("starts_with", "Bool", "starts_with(\"hello\", \"he\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_ends_with_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source("ends_with", "Bool", "ends_with(\"hello\", \"lo\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_is_empty_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source("is_empty", "Bool", "is_empty(\"\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_all_four_functions_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source(
            "concat, starts_with, ends_with, is_empty",
            "Bool",
            "is_empty(\"\")",
        ),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(true));
}

#[tokio::test]
async fn string_stdlib_starts_with_false_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        string_main_source("starts_with", "Bool", "starts_with(\"hello\", \"x\")"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(false));
}

#[tokio::test]
async fn string_stdlib_ends_with_false_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        string_main_source("ends_with", "Bool", "ends_with(\"hello\", \"x\")"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(false));
}

#[tokio::test]
async fn string_stdlib_is_empty_false_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        string_main_source("is_empty", "Bool", "is_empty(\"hello\")"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(false));
}
