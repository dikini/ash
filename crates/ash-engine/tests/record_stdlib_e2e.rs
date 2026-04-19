//! TASK-626: End-to-end tests for std/src/record.ash stdlib import.

#[tokio::test]
async fn record_stdlib_keys_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "use record::{keys}\nworkflow main { let r = record(\"a\", 1, \"b\", 2)\nret keys(r) }\n",
    ).expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::List(_)),
        "keys() should return a List, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_values_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "use record::{values}\nworkflow main { let r = record(\"a\", 1, \"b\", 2)\nret values(r) }\n",
    ).expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::List(_)),
        "values() should return a List, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_record_constructor_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "use record::{record}\nworkflow main { ret record(\"x\", 42) }\n",
    ).expect("write main.ash");
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
        "use record::{keys, values, record}\nworkflow main { ret record(\"k\", 1) }\n",
    ).expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert!(
        matches!(result, ash_core::Value::Record(_)),
        "expected Record, got: {result:?}"
    );
}
