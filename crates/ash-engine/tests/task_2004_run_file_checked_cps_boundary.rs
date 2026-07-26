//! TASK-2004 RED contracts for the file-backed checked Core/CPS boundary.

use ash_core::Value;
use ash_engine::Engine;
use ash_interp::ExecError;

fn write_entry(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory creates");
    let path = directory.path().join("main.ash");
    std::fs::write(&path, source).expect("entry source writes");
    (directory, path)
}

#[tokio::test]
async fn run_file_admits_a_supported_literal_through_sealed_checked_cps() {
    let (_directory, path) = write_entry("fn main() -> Int { 42 }");
    let engine = Engine::new().build().expect("engine builds");

    let value = engine
        .run_file(&path)
        .await
        .expect("a supported file entry must execute through sealed checked Core/CPS admission");

    assert_eq!(value, Value::Int(42));
}

#[tokio::test]
async fn run_file_rejects_unsupported_nested_lowering_before_direct_evaluation() {
    let (_directory, path) = write_entry("fn main() -> Int { (1 + 2) + 3 }");
    let engine = Engine::new().build().expect("engine builds");

    let error = engine
        .run_file(&path)
        .await
        .expect_err("unsupported file lowering must reject instead of using the direct evaluator");

    assert!(matches!(
        error,
        ExecError::ExecutionFailed(ref message) if message.contains("checked Core/CPS admission")
    ));
}
