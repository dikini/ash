//! TASK-1771 regressions for typed macro checking across module boundaries.

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
fn imported_typed_macro_argument_mismatch_rejects_in_caller_module() {
    let (_dir, caller) = write_pair(
        "\npub macro inc(x: Int) -> Int => x;\n",
        "\nuse provider::{inc}\nfn use_macro() -> Int { inc!(\"not-int\") }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("imported typed macro mismatch rejects in caller");
    let message = err.to_string();
    assert!(
        message.contains("macro `inc` typed signature mismatch at argument 1 at call site"),
        "unexpected error: {message}"
    );
}

#[test]
fn imported_typed_macro_unknown_argument_rejects_fail_closed_in_caller_module() {
    let (_dir, caller) = write_pair(
        "\npub macro inc(x: Int) -> Int => x;\n",
        "\nuse provider::{inc}\nfn use_macro(n: Int) -> Int { inc!(n) }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("imported typed macro unknown arg rejects in caller");
    let message = err.to_string();
    assert!(
        message.contains("macro `inc` typed signature mismatch at argument 1 at call site"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("got unknown argument type"),
        "unexpected error: {message}"
    );
}

#[test]
fn imported_typed_macro_matching_signature_checks_successfully() {
    let (_dir, caller) = write_pair(
        "\npub macro id_int(x: Int) -> Int => x;\n",
        "\nuse provider::{id_int}\nfn use_macro() -> Int { id_int!(1) }\n",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&caller)
        .expect("matching imported typed macro expands before checking");
}
