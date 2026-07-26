//! TASK-597: `std/src/json.ash` import/typecheck coverage under closed admission.

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn json_main_source(imports: &str, body: &str) -> String {
    format!("use json::{{{imports}}}\nfn main() -> String {{ {body} }}\n")
}

async fn assert_json_stdlib_source_rejects_without_typed_lowering(imports: &str, body: &str) {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let dir = tmp_dir.path();

    std::fs::write(dir.join("main.ash"), json_main_source(imports, body)).expect("write main.ash");

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
        "source must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn json_stdlib_imports_parse_typecheck_and_fail_closed_at_execution() {
    for (imports, body) in [
        ("parse", "parse(\"42\")"),
        ("parse", "parse(\"true\")"),
        ("parse", "parse(\"[1, 2, 3]\")"),
        ("parse", "parse(\"null\")"),
        // Admission precedes JSON host dispatch, so malformed JSON has the same error.
        ("parse", "parse(\"{invalid}\")"),
        ("stringify", "stringify(\"[1, 2, 3]\")"),
        ("stringify_pretty", "stringify_pretty(\"[1,2]\")"),
        (
            "parse, stringify, stringify_pretty",
            "stringify(parse(\"42\"))",
        ),
    ] {
        assert_json_stdlib_source_rejects_without_typed_lowering(imports, body).await;
    }
}
