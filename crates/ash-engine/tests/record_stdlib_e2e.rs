//! TASK-626: End-to-end tests for std/src/record.ash stdlib import.

fn record_main_source(imports: &str, body: &str) -> String {
    format!("use record::{{{imports}}}\nworkflow main {{ {body} }}\n")
}

#[tokio::test]
async fn record_stdlib_keys_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        record_main_source("keys", "ret keys(record(\"a\", 1, \"b\", 2))"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    // keys() should return the field names as strings.
    let items = result
        .list_to_vec()
        .unwrap_or_else(|| panic!("keys() should return a List, got: {result:?}"));
    let mut keys: Vec<String> = items
        .iter()
        .filter_map(|v| {
            if let ash_core::Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect();
    keys.sort();
    assert_eq!(keys, vec!["a", "b"], "keys should be [\"a\", \"b\"]");
}

#[tokio::test]
async fn record_stdlib_values_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        record_main_source("values", "ret values(record(\"a\", 1, \"b\", 2))"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    // values() returns heterogeneous values; verify it's a non-empty list.
    assert!(
        result.list_to_vec().is_some_and(|items| !items.is_empty()),
        "values() should return a non-empty List, got: {result:?}"
    );
}

#[tokio::test]
async fn record_stdlib_record_constructor_e2e() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        record_main_source("record", "ret record(\"x\", 42)"),
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
        record_main_source("keys, values, record", "ret record(\"k\", 1)"),
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
