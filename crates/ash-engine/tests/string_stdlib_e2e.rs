//! TASK-623: End-to-end tests for std/src/string.ash stdlib import.

#[tokio::test]
async fn string_stdlib_concat_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        "use string::{concat}\nworkflow main { ret concat(\"foo\", \"bar\") }\n",
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
        "use string::{starts_with}\nworkflow main { ret starts_with(\"hello\", \"he\") }\n",
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
        "use string::{ends_with}\nworkflow main { ret ends_with(\"hello\", \"lo\") }\n",
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
        "use string::{is_empty}\nworkflow main { ret is_empty(\"\") }\n",
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
        "use string::{concat, starts_with, ends_with, is_empty}\nworkflow main { ret is_empty(\"\") }\n",
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
        "use string::{starts_with}\nworkflow main { ret starts_with(\"hello\", \"x\") }\n",
    ).expect("write main.ash");
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
        "use string::{ends_with}\nworkflow main { ret ends_with(\"hello\", \"x\") }\n",
    ).expect("write main.ash");
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
        "use string::{is_empty}\nworkflow main { ret is_empty(\"hello\") }\n",
    ).expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::Bool(false));
}
