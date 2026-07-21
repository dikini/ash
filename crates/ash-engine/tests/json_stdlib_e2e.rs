//! TASK-597: End-to-end tests for std/src/json.ash stdlib import.

fn json_main_source(imports: &str, body: &str) -> String {
    format!("use json::{{{imports}}}\nfn main() -> String {{ {body} }}\n")
}

#[tokio::test]
async fn json_stdlib_parse_number_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("parse", "parse(\"42\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("42".to_string()));
}

#[tokio::test]
async fn json_stdlib_parse_bool_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("parse", "parse(\"true\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("true".to_string()));
}

#[tokio::test]
async fn json_stdlib_parse_array_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("parse", "parse(\"[1, 2, 3]\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    // parse validates and returns the input string as-is
    assert_eq!(result, ash_core::Value::String("[1, 2, 3]".to_string()));
}

#[tokio::test]
async fn json_stdlib_parse_null_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("parse", "parse(\"null\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("null".to_string()));
}

#[tokio::test]
async fn json_stdlib_parse_invalid_returns_error() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("parse", "parse(\"{invalid}\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await;
    assert!(result.is_err(), "Expected error for invalid JSON");
}

#[tokio::test]
async fn json_stdlib_stringify_array_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("stringify", "stringify(\"[1, 2, 3]\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("[1,2,3]".to_string()));
}

#[tokio::test]
async fn json_stdlib_stringify_pretty_array_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source("stringify_pretty", "stringify_pretty(\"[1,2]\")"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    let expected = "[\n  1,\n  2\n]";
    assert_eq!(result, ash_core::Value::String(expected.to_string()));
}

#[tokio::test]
async fn json_stdlib_all_three_functions_importable() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        json_main_source(
            "parse, stringify, stringify_pretty",
            "stringify(parse(\"42\"))",
        ),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("42".to_string()));
}
