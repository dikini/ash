//! TASK-626: `std/src/record.ash` import/typecheck coverage under closed admission.
//!
//! TASK-2014 Path B keeps these record fixtures at the parser/typechecker boundary until their
//! source forms have validated production typed lowering. They must reject at the checked
//! Core/CPS admission boundary rather than revive the removed direct-evaluator fallback.

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn record_main_source(imports: &str, return_type: &str, body: &str) -> String {
    format!("use record::{{{imports}}}\nfn main() -> {return_type} {{ {body} }}\n")
}

#[tokio::test]
async fn record_stdlib_keys_imports_parse_typecheck_and_fail_closed_at_execution() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(dir.join("main.ash"), record_main_source("keys", "Int", "1"))
        .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "record stdlib source must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn record_stdlib_values_imports_parse_typecheck_and_fail_closed_at_execution() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        record_main_source("values", "Int", "1"),
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "record stdlib source must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn record_literal_parses_typechecks_and_fails_closed_without_typed_lowering() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "fn main() -> { x: Int } { { x: 42 } }\n",
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "record literal source must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn record_stdlib_all_three_functions_import_parse_typecheck_and_fail_closed_at_execution() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();
    std::fs::write(
        dir.join("main.ash"),
        "use record::{keys, values, record}\nfn main() -> { k: Int } { { k: 1 } }\n",
    )
    .expect("write main.ash");
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut application).expect("typecheck");
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "record stdlib source must expose the exact canonical closed-admission error"
    );
}
