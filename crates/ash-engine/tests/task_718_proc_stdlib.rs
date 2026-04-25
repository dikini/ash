//! TASK-718: End-to-end tests for the initial std/src/proc.ash library surface.

fn proc_main_source(imports: &str, body: &str) -> String {
    format!("use proc::{{{imports}}}\nworkflow main {{ {body} }}\n")
}

#[tokio::test]
async fn proc_stdlib_unit_import_typechecks_and_returns_proc_value() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        proc_main_source("unit", "ret unit(42)"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine
        .check(&mut workflow)
        .expect("typecheck Proc unit import");
    let result = engine.execute(&workflow).await.expect("execute");
    let ash_core::Value::Closure { params, .. } = result else {
        panic!("expected Proc runtime closure from proc::unit, got {result:?}");
    };
    assert_eq!(params, vec![("__proc_env".to_string(), None)]);
}

#[tokio::test]
async fn proc_stdlib_then_import_typechecks_without_process_handles() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        proc_main_source("unit, then", "ret then(unit(0), unit(\"ok\"))"),
    )
    .expect("write main.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine
        .check(&mut workflow)
        .expect("typecheck Proc unit/bind/then imports");
    let result = engine.execute(&workflow).await.expect("execute");
    let ash_core::Value::Closure { params, .. } = result else {
        panic!("expected Proc runtime closure from proc::then, got {result:?}");
    };
    assert_eq!(params, vec![("__proc_env".to_string(), None)]);
}
