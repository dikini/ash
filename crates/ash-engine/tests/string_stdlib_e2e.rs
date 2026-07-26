//! TASK-623: `std/src/string.ash` import/typecheck coverage under closed admission.
//!
//! TASK-2014 Path B retains these representative string-call fixtures at the parser/typechecker
//! boundary until their source forms have validated production typed lowering. Each must reject at
//! checked Core/CPS admission rather than revive the removed direct-evaluator fallback.

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn string_main_source(imports: &str, return_type: &str, body: &str) -> String {
    format!("use string::{{{imports}}}\nfn main() -> {return_type} {{ {body} }}\n")
}

async fn assert_string_stdlib_source_rejects_without_typed_lowering(
    imports: &str,
    return_type: &str,
    body: &str,
) {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("main.ash"),
        string_main_source(imports, return_type, body),
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
        "string stdlib source must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn string_stdlib_concat_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "concat",
        "String",
        "concat(\"foo\", \"bar\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_starts_with_true_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "starts_with",
        "Bool",
        "starts_with(\"hello\", \"he\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_ends_with_true_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "ends_with",
        "Bool",
        "ends_with(\"hello\", \"lo\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_is_empty_true_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "is_empty",
        "Bool",
        "is_empty(\"\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_all_four_functions_import_parse_typecheck_and_fail_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "concat, starts_with, ends_with, is_empty",
        "Bool",
        "is_empty(\"\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_starts_with_false_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "starts_with",
        "Bool",
        "starts_with(\"hello\", \"x\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_ends_with_false_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "ends_with",
        "Bool",
        "ends_with(\"hello\", \"x\")",
    )
    .await;
}

#[tokio::test]
async fn string_stdlib_is_empty_false_parses_typechecks_and_fails_closed_at_execution() {
    assert_string_stdlib_source_rejects_without_typed_lowering(
        "is_empty",
        "Bool",
        "is_empty(\"hello\")",
    )
    .await;
}
