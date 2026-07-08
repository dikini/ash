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
fn main() { 0 }
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
fn ordinary_type_summary_path_preserves_imported_callable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("flows.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        r"pub type Token = Token { value: String };

pub fn guarded() -> Int { 0 }
",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        r"use flows::{Token, guarded}
fn main() { 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports type and callable");

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
    assert_eq!(callable.exported_name, "guarded");
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
fn check_module_file_allows_public_callable_signature_private_type_as_opaque() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");

    std::fs::write(
        &module,
        r"type Secret = Int;
pub fn leak(x: Secret) -> Int { 0 }
pub fn leak_flow(x: Secret) -> Int { 0 }
pub fn leak_ret(x: Secret) -> Int { 0 }
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module parses and exposes private signature types opaquely");

    assert!(
        result.errors.is_empty(),
        "Phase 154 should permit private callable-signature types as opaque nameable identities: {:?}",
        result.errors
    );
}

#[test]
fn importable_export_collection_exports_public_callable_private_type_as_opaque() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");
    std::fs::write(
        &module,
        r"type Secret = Int;
pub fn leak(x: Secret) -> Int { 0 }
",
    )
    .expect("write module");

    check_importable_module_file(&module)
        .expect("Phase 154 permits private callable-signature types as opaque exports");

    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use leaky::{leak}\nfn main() { 0 }\n").expect("write caller");
    let loaded = load_ordinary_file(&caller)
        .expect("export collection should import callable plus opaque signature type");
    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .any(|summary| summary
                .exported_types
                .iter()
                .any(|ty| ty.exported_name == "Secret"
                    && matches!(
                        ty.representation,
                        ash_core::semantic_summary::TypeRepresentationSummary::Opaque { .. }
                    ))),
        "callable signature should transport Secret as an opaque nameable type"
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
    std::fs::write(&caller, "use leaky_repr::{Public}\nfn main() { 0 }\n").expect("write caller");
    let err = load_ordinary_file(&caller)
        .expect_err("export collection should reject public representation private type leak");
    let msg = err.to_string();
    assert!(
        msg.contains("Public") && msg.contains("Secret"),
        "export collection diagnostic should mention Public and Secret: {msg}"
    );
}
