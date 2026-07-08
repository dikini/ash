//! TASK-880: engine hands imported proposition facts to `TypeEnv` checking.

use ash_engine::Engine;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn task_880_engine_rejects_public_v5_deferred_propositions_before_provider_publication() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub prop PublicReq<T: Int>;
pub fn needs(x: Int) -> Int where PublicReq<Int> { x }
",
    );
    write_file(
        &caller,
        r"use provider::{needs}
fn main() { 0 }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .parse_file(&caller)
        .expect_err("deferred public provider proposition must fail before summary publication");
    let message = err.to_string();
    assert!(
        message.contains("proposition")
            && message.contains("provider.ash")
            && (message.contains("deferred") || message.contains("UnsupportedNamedPredicate"))
            && message.contains("PublicReq"),
        "expected provider-publication proposition discharge diagnostic, got {message}"
    );
}

#[test]
fn task_880_engine_check_accepts_satisfied_imported_v5_proposition_requirement_transport_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub fn needs(x: Int) -> Int where Int == Int { x }
",
    );
    write_file(
        &caller,
        r"use provider::{needs}
fn main() { 0 }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine
        .parse_file(&caller)
        .expect("caller with satisfied proposition import parses");

    engine
        .check(&mut workflow)
        .expect("engine should hand summaries to TypeEnv but not solve propositions itself");
}
