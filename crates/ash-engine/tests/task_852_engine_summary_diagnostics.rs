//! TASK-852: engine-facing summary diagnostics for private-opacity failures.

use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn engine_reports_private_type_function_export_failure_with_source_context() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }

type fn Secret(xs: TypeList) -> TypeList {
    case Secret<xs> = xs;
}

pub type fn Public(xs: TypeList) -> TypeList {
    case Public<xs> = Secret<xs>;
}
",
    );
    write_file(
        &caller,
        r"use provider::{Public}
fn main() { 0 }
",
    );

    let err = load_ordinary_file(&caller).expect_err("private dependency blocks export");
    let message = err.to_string();

    assert!(
        message.contains("private-dependency-export-failure"),
        "diagnostic family missing: {message}"
    );
    assert!(
        message.contains("public type function 'Public' depends on private type function 'Secret'"),
        "specific private dependency missing: {message}"
    );
    assert!(
        message.contains("provider.ash") && message.contains("span"),
        "source file/span context missing: {message}"
    );
}

#[test]
fn engine_reports_private_ordinary_type_export_failure_with_source_context() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
type Secret = Int;
pub sealed type domain TypeList { Nil; }

pub type fn Public(xs: TypeList) -> Secret {
    case Public<xs> = Secret;
}
",
    );
    write_file(
        &caller,
        r"use provider::{Public}
fn main() { 0 }
",
    );

    let err = load_ordinary_file(&caller).expect_err("private ordinary type blocks export");
    let message = err.to_string();

    assert!(
        message.contains("private-dependency-export-failure"),
        "diagnostic family missing: {message}"
    );
    assert!(
        message.contains("private ordinary type 'Secret'"),
        "specific private ordinary type dependency missing: {message}"
    );
    assert!(
        message.contains("provider.ash") && message.contains("span"),
        "source file/span context missing: {message}"
    );
}

#[test]
fn engine_documents_mvp_rejects_private_dependencies_before_downstream_reduction() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
sealed type domain SecretList { Nil; }

pub type fn Public(xs: SecretList) -> SecretList {
    case Public<xs> = xs;
}
",
    );
    write_file(
        &caller,
        r"use provider::{Public}
fn main() { 0 }
",
    );

    let err = load_ordinary_file(&caller).expect_err("private domain blocks export");
    let message = err.to_string();

    assert!(
        message.contains("private-dependency-export-failure"),
        "diagnostic family missing: {message}"
    );
    assert!(
        message.contains("before downstream reduction")
            || message.contains("before downstream use"),
        "MVP rejection-before-use note missing: {message}"
    );
}
