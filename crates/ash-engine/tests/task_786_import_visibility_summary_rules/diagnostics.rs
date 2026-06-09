use super::support::*;

#[test]
fn named_import_rejects_private_and_crate_ordinary_types_but_allows_public() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("domain.ash");
    std::fs::write(
        &module,
        "type Secret = Int;\npub(crate) type CrateOnly = Int;\npub type Public = Int;\n",
    )
    .expect("write domain");

    let public_caller = dir.path().join("public_caller.ash");
    std::fs::write(
        &public_caller,
        "use domain::{Public}\nworkflow main { ret 0 }\n",
    )
    .expect("write public caller");
    let public_loaded = load_ordinary_file(&public_caller).expect("public type imports");
    assert_eq!(imported_type_names(&public_loaded), vec!["Public"]);
    assert_eq!(semantic_type_names(&public_loaded), vec!["Public"]);

    for (name, filename) in [
        ("Secret", "secret_caller.ash"),
        ("CrateOnly", "crate_caller.ash"),
    ] {
        let caller = dir.path().join(filename);
        std::fs::write(
            &caller,
            format!("use domain::{{{name}}}\nworkflow main {{ ret 0 }}\n"),
        )
        .expect("write caller");
        let err = load_ordinary_file(&caller).expect_err("non-public ordinary type import fails");
        let msg = err.to_string();
        assert!(
            msg.contains(name) && msg.contains("not found"),
            "diagnostic should mention missing non-public import {name}: {msg}"
        );
    }
}

#[test]
fn module_check_rejects_public_signature_using_imported_private_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("secret.ash"), "type Secret = Int;\n").expect("write secret");
    let leaky = dir.path().join("leaky.ash");
    std::fs::write(
        &leaky,
        "use secret::{Secret}\npub fn leak(x: Secret) -> Int { 0 }\n",
    )
    .expect("write leaky");

    let result = ash_engine::Engine::new()
        .build()
        .expect("engine")
        .check_module_file(&leaky)
        .expect("module file check returns diagnostics");
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.contains("leak") && error.contains("Secret")),
        "expected imported private type leak diagnostic, got {:?}",
        result.errors
    );
}

#[test]
fn public_representation_private_leak_check_ignores_type_parameter_shadowing() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("lib.ash"),
        "type Secret = Int;\npub type Box<Secret> = Box { value: Secret };\n",
    )
    .expect("write lib module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use lib::{Box}\nworkflow main { ret 0 }\n").expect("write caller");

    load_ordinary_file(&caller).expect("generic parameter shadowing should not leak private type");
}

#[test]
fn constructor_alias_imports_and_reexports_are_rejected_explicitly() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Status = Pending | Ready(Int);\n",
    )
    .expect("write domain");

    let import_alias_user = dir.path().join("import_alias_user.ash");
    std::fs::write(
        &import_alias_user,
        "use domain::{Ready as PublicReady}\nworkflow main { ret 0 }\n",
    )
    .expect("write import alias user");
    let err = load_ordinary_file(&import_alias_user)
        .expect_err("constructor aliases should be rejected instead of silently accepted");
    let msg = err.to_string();
    assert!(
        msg.contains("Ready") && msg.contains("alias"),
        "constructor alias diagnostic should mention Ready and alias: {msg}"
    );

    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use domain::{Ready as PublicReady};\n",
    )
    .expect("write outer");
    let reexport_alias_user = dir.path().join("reexport_alias_user.ash");
    std::fs::write(
        &reexport_alias_user,
        "use outer::{PublicReady}\nworkflow main { ret 0 }\n",
    )
    .expect("write reexport alias user");
    let err = load_ordinary_file(&reexport_alias_user)
        .expect_err("constructor re-export aliases should be rejected explicitly");
    let msg = err.to_string();
    assert!(
        msg.contains("Ready") && msg.contains("alias"),
        "constructor re-export alias diagnostic should mention Ready and alias: {msg}"
    );
}

#[test]
fn missing_pub_use_target_is_diagnostic() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("inner.ash"), "pub type Present = Int;\n").expect("write inner");
    std::fs::write(dir.path().join("outer.ash"), "pub use inner::{Missing};\n")
        .expect("write outer");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use outer::{Missing}\nworkflow main { ret 0 }\n")
        .expect("write caller");

    let err = load_ordinary_file(&caller)
        .expect_err("missing pub use target fails during export collection");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing") && msg.contains("pub use"),
        "missing re-export diagnostic should mention target and pub use: {msg}"
    );
}

#[test]
fn bare_non_unit_constructor_import_is_rejected_as_value() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Status = Pending | Ready(Int);\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use domain::{Ready}\nworkflow main -> Status { ret Ready; }\n",
    )
    .expect("write caller");

    let err = check_file(&caller).expect_err("payload constructor requires constructor syntax");
    assert!(
        err.contains("Ready"),
        "diagnostic should mention bare non-unit constructor: {err}"
    );
}

#[test]
fn public_callable_signature_rejects_unresolved_ordinary_type_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "pub fn leak(x: Missing) -> Int { 0 }\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let err = check_module_file(&caller).expect_err("unresolved public signature type must fail");
    assert!(
        err.contains("unresolved ordinary type") && err.contains("Missing"),
        "diagnostic should mention unresolved ordinary type Missing: {err}"
    );
}

#[test]
fn public_callable_signature_rejects_unresolved_imported_ordinary_type_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Present = Int;\npub fn Missing() -> Int { 0 }\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use domain::{Missing as Alias}\npub fn leak(x: Alias) -> Int { 0 }\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let err = check_module_file(&caller)
        .expect_err("unresolved imported public signature type must fail");
    assert!(
        err.contains("unresolved imported ordinary type") && err.contains("Alias"),
        "diagnostic should mention unresolved imported ordinary type Alias: {err}"
    );
}

#[test]
fn public_callable_signature_rejects_callable_reexport_masquerading_as_type_alias() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub fn take_a(x: Int) -> Int { x }\n",
    )
    .expect("write inner");
    std::fs::write(dir.path().join("outer.ash"), "pub use inner::{take_a};\n")
        .expect("write outer");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{take_a as Alias}\npub fn leak(x: Alias) -> Int { 0 }\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let err = check_module_file(&caller)
        .expect_err("callable re-export aliases must not resolve as ordinary types");
    assert!(
        err.contains("unresolved imported ordinary type") && err.contains("Alias"),
        "diagnostic should mention unresolved imported ordinary type Alias: {err}"
    );
}
