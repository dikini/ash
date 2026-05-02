//! TASK-785 regression tests for ModuleFile-backed ordinary type metadata transport.

use ash_engine::Engine;
use ash_engine::module_loader::{check_importable_module_file, load_ordinary_file};

#[test]
fn check_module_file_counts_multiline_type_from_modulefile_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("domain.ash");

    std::fs::write(
        &module,
        r"pub type Person = Person {
    name: String,
    age: Int
};
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("ModuleFile type metadata should check");

    assert_eq!(result.type_count, 1);
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn load_ordinary_file_imports_type_export_from_modulefile_metadata() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("domain.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub type Person = Person {
    name: String,
    age: Int
};
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use domain::{Person}
workflow main { ret 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports public type");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|type_def| type_def.name == "Person"),
        "ModuleFile-backed export collection should import Person"
    );
}

#[test]
fn ordinary_type_summary_path_preserves_imported_workflow_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub type Token = Token { value: String };

pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use flows::{Token, guarded}
workflow main { ret 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports type and workflow");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|type_def| type_def.name == "Token"),
        "ordinary type metadata should still be transported"
    );
    let callable = loaded
        .imported_callables
        .get("guarded")
        .expect("guarded callable imported");
    assert!(
        callable.workflow_summary.is_some(),
        "workflow summary must survive ordinary type summary collection"
    );
}

#[test]
fn malformed_type_declaration_does_not_fall_back_to_type_snippets() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("bad.ash");

    std::fs::write(
        &module,
        r"pub type Broken = ;
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine.check_module_file(&module);

    assert!(
        result.is_err(),
        "ModuleFile-backed type collection must not silently recover via ordinary type snippet scanning"
    );
}

#[test]
fn check_module_file_rejects_public_callable_signature_exposing_private_ordinary_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");

    std::fs::write(
        &module,
        r"type Secret = Int;
pub fn leak(x: Secret) -> Int { 0 }
pub workflow leak_flow(x: Secret) -> Workflow<Int> {
    done
}
pub workflow leak_ret(x: Secret) -> Int { ret 0 }
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module parses but reports export visibility errors");

    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("leak") && error.contains("Secret")),
        "pub fn signature should reject private ordinary type exposure: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("leak_flow") && error.contains("Secret")),
        "pub workflow signature should reject private ordinary type exposure: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("leak_ret") && error.contains("Secret")),
        "pub workflow ret-form signature should reject private ordinary type exposure: {:?}",
        result.errors
    );
}

#[test]
fn importable_export_collection_rejects_public_callable_private_type_leak() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");
    std::fs::write(
        &module,
        r"type Secret = Int;
pub fn leak(x: Secret) -> Int { 0 }
",
    )
    .expect("write module");

    let err = check_importable_module_file(&module)
        .expect_err("importable module check should reject public callable private type leak");
    let msg = err.to_string();
    assert!(
        msg.contains("leak") && msg.contains("Secret"),
        "importable module diagnostic should mention leak and Secret: {msg}"
    );

    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use leaky::{leak}\nworkflow main { ret 0 }\n").expect("write caller");
    let err = load_ordinary_file(&caller)
        .expect_err("export collection should reject public callable private type leak");
    let msg = err.to_string();
    assert!(
        msg.contains("leak") && msg.contains("Secret"),
        "export collection diagnostic should mention leak and Secret: {msg}"
    );
}

#[test]
fn importable_export_collection_rejects_public_representation_private_type_leak() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky_repr.ash");
    std::fs::write(
        &module,
        r"type Secret = Secret { value: Int };
pub type Public = Public { secret: Secret };
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module parses but reports representation visibility errors");
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("Public") && error.contains("Secret")),
        "check_module_file should reject public representation exposing private ordinary type: {:?}",
        result.errors
    );

    let err = check_importable_module_file(&module).expect_err(
        "importable module check should reject public representation private type leak",
    );
    let msg = err.to_string();
    assert!(
        msg.contains("Public") && msg.contains("Secret"),
        "importable module diagnostic should mention Public and Secret: {msg}"
    );

    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use leaky_repr::{Public}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");
    let err = load_ordinary_file(&caller)
        .expect_err("export collection should reject public representation private type leak");
    let msg = err.to_string();
    assert!(
        msg.contains("Public") && msg.contains("Secret"),
        "export collection diagnostic should mention Public and Secret: {msg}"
    );
}
