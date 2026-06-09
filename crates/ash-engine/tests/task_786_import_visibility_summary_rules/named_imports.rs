use super::support::*;

#[test]
fn named_import_scopes_semantic_summary_to_selected_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Token = Token { value: String };\npub type Other = Other { value: String };\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use domain::{Token}\nworkflow main { ret 0 }\n")
        .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("named type import succeeds");
    assert_eq!(imported_type_names(&loaded), vec!["Token"]);
    assert_eq!(semantic_type_names(&loaded), vec!["Token"]);
}

#[test]
fn constructor_only_import_does_not_expose_sibling_constructors() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Status = Pending | Ready;\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use domain::{Ready}\nworkflow main { ret Pending; }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("constructor import loads");
    assert_eq!(semantic_constructor_names(&loaded), vec!["Ready"]);
    let err = check_file(&caller).expect_err("sibling constructor must not be visible");
    assert!(
        err.contains("Pending"),
        "diagnostic should mention unimported sibling constructor: {err}"
    );

    std::fs::write(dir.path().join("outer.ash"), "pub use domain::{Ready};\n")
        .expect("write outer");
    let reexport_user = dir.path().join("reexport_user.ash");
    std::fs::write(
        &reexport_user,
        "use outer::{Ready}\nworkflow main { ret Pending; }\n",
    )
    .expect("write reexport user");
    let reexport_loaded = load_ordinary_file(&reexport_user).expect("constructor re-export loads");
    assert_eq!(semantic_constructor_names(&reexport_loaded), vec!["Ready"]);
    let err = check_file(&reexport_user)
        .expect_err("re-exported constructor must not expose sibling constructor");
    assert!(
        err.contains("Pending"),
        "diagnostic should mention unimported re-export sibling constructor: {err}"
    );
}

#[test]
fn duplicate_visible_named_imports_from_distinct_modules_keep_distinct_semantic_identities() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("left.ash"), "pub type Token = LeftToken;\n")
        .expect("write left");
    std::fs::write(
        dir.path().join("right.ash"),
        "pub type Token = RightToken;\n",
    )
    .expect("write right");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use left::{Token}\nuse right::{Token}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("both visible imports are collected");
    assert_eq!(semantic_type_names(&loaded), vec!["Token", "Token"]);
}

#[test]
fn explicit_builtin_opaque_type_identity_remains_importable() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("runtime.ash"),
        "builtin type RuntimeHandle;\npub builtin type PublicHandle;\n",
    )
    .expect("write runtime");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use runtime::{RuntimeHandle, PublicHandle}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("explicit builtin opaque identities import");
    let runtime_handle = loaded
        .imported_type_defs
        .iter()
        .find(|def| def.name == "RuntimeHandle")
        .expect("private builtin identity imported");
    assert!(runtime_handle.builtin);
    assert!(matches!(runtime_handle.body, TypeBody::Struct(ref fields) if fields.is_empty()));
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "PublicHandle" && def.builtin),
        "public builtin identity should also import"
    );
}

#[test]
fn constructor_only_import_exposes_public_enum_parent_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Status = Pending | Ready(Int);\ntype Hidden = Concealed;\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use domain::{Ready}\nworkflow main { ret 0 }\n")
        .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("constructor import succeeds");
    assert_eq!(imported_type_names(&loaded), vec!["Status"]);
}

#[test]
fn import_order_independent_for_named_type_and_callable_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        r"pub type Token = Token { value: String };
pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write domain");

    for (file, imports) in [
        (
            "type_first.ash",
            "use domain::{Token}\nuse domain::{guarded}\n",
        ),
        (
            "callable_first.ash",
            "use domain::{guarded}\nuse domain::{Token}\n",
        ),
    ] {
        let caller = dir.path().join(file);
        std::fs::write(&caller, format!("{imports}workflow main {{ ret 0 }}\n"))
            .expect("write caller");
        let loaded = load_ordinary_file(&caller).expect("imports are order independent");
        assert!(imported_type_names(&loaded).contains(&"Token"));
        assert!(
            loaded
                .imported_callables
                .get("guarded")
                .and_then(|callable| callable.workflow_summary.as_ref())
                .is_some(),
            "workflow summary should survive regardless of import order"
        );
    }
}

#[test]
fn separate_named_constructor_imports_accumulate_with_parent_type_import() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Status = Pending | Ready(Int);\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use domain::{Status}\nuse domain::{Pending}\nuse domain::{Ready}\nworkflow main() -> Status { ret Pending; }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("separate imports load");
    assert_eq!(semantic_type_names(&loaded), vec!["Status"]);
    assert_eq!(
        semantic_constructor_names(&loaded),
        vec!["Pending", "Ready"]
    );
    check_file(&caller).expect("later constructor imports must not erase earlier constructors");
}
