//! TASK-1772 regressions for imported inferred macro type summaries.

use ash_engine::Engine;

fn write_pair(
    provider_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, caller)
}

#[test]
fn imported_macro_with_inferred_signature_checks_argument_before_typecheck() {
    let (_dir, caller) = write_pair(
        "\npub macro id_int(x: Int) => x;\n",
        "\nuse provider::{id_int}\nfn use_macro() -> Int { id_int!(\"bad\") }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("imported inferred signature rejects mismatch");
    let message = err.to_string();
    assert!(
        message.contains("macro `id_int` typed signature mismatch at argument 1 at call site"),
        "unexpected error: {message}"
    );
}

#[test]
fn imported_literal_macro_with_inferred_result_summary_expands_successfully() {
    let (_dir, caller) = write_pair(
        "\npub macro answer() => 1;\n",
        "\nuse provider::{answer}\nfn use_macro() -> Int { answer!() }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&caller)
        .expect("imported inferred macro summary expands and checks");
}
