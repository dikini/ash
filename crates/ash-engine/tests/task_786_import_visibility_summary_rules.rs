//! TASK-786 regression tests for import/pub-use/glob visibility and summary transport.

use ash_core::ast::{TypeBody, Visibility};
use ash_engine::module_loader::load_ordinary_file;

fn imported_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_type_defs
        .iter()
        .map(|type_def| type_def.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn semantic_type_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .map(|ty| ty.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn semantic_constructor_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<&str> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_constructors.iter())
        .map(|constructor| constructor.exported_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn check_file(path: &std::path::Path) -> Result<(), String> {
    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|error| error.to_string())?;
    let mut workflow = engine.parse_file(path).map_err(|error| error.to_string())?;
    engine
        .check(&mut workflow)
        .map_err(|error| error.to_string())
}

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
fn named_callable_import_transports_signature_ordinary_type_semantic_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("domain.ash"),
        "pub type Token = Token { value: String };\npub fn accept(token: Token) -> Int { 0 }\n",
    )
    .expect("write domain");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use domain::{accept}\nworkflow main { ret 0 }\n")
        .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("named callable import succeeds");
    assert!(loaded.imported_callables.contains_key("accept"));
    assert_eq!(semantic_type_names(&loaded), vec!["Token"]);
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
fn glob_import_transports_public_types_constructors_and_workflow_summary_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("flows.ash"),
        r"type Secret = Hidden;
pub type Token = Token { value: String };
pub type Choice = First | Second(Int);

pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write flows");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use flows::*\nworkflow main { ret 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("glob imports public exports");
    let names = imported_type_names(&loaded);
    assert!(
        names.contains(&"Token"),
        "glob should import public struct type"
    );
    assert!(
        names.contains(&"Choice"),
        "glob should expose public enum through constructors"
    );
    assert!(
        !names.contains(&"Secret"),
        "glob must not import private ordinary type"
    );
    let semantic_names = semantic_type_names(&loaded);
    assert!(
        semantic_names.contains(&"Token"),
        "glob should transport public type semantic summary"
    );
    assert!(
        semantic_names.contains(&"Choice"),
        "glob should transport public enum semantic summary"
    );
    assert!(
        !semantic_names.contains(&"Secret"),
        "glob must not transport private ordinary type semantic summary"
    );
    assert!(
        loaded
            .imported_callables
            .get("guarded")
            .and_then(|callable| callable.workflow_summary.as_ref())
            .is_some(),
        "glob-imported workflow callable must retain PublicWorkflowSummary"
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
fn pub_use_preserves_type_shape_aliases_callable_names_and_workflow_summaries() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        r"pub type Token = Token { value: String };
pub fn keep(token: Token) -> Token { token }
pub workflow guarded() -> Workflow<Int> {
    done
}
",
    )
    .expect("write inner");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{Token as PublicToken, keep as preserve, guarded as re_guarded};\n",
    )
    .expect("write outer");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{PublicToken, preserve, re_guarded}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("re-exported aliases import");
    let token = loaded
        .imported_type_defs
        .iter()
        .find(|def| def.name == "PublicToken")
        .expect("aliased re-export uses imported alias name");
    assert_eq!(token.visibility, Visibility::Public);
    assert!(!token.builtin, "public ordinary type should not be opaqued");
    assert!(loaded.imported_callables.contains_key("preserve"));
    let semantic_names = semantic_type_names(&loaded);
    assert!(
        semantic_names.contains(&"PublicToken"),
        "pub-use aliases should be represented in imported semantic summaries"
    );
    assert!(
        !semantic_names.contains(&"Token"),
        "caller imported the alias, not the origin module name"
    );
    let re_guarded = loaded
        .imported_callables
        .get("re_guarded")
        .expect("workflow callable alias imported");
    assert_eq!(re_guarded.exported_name, "re_guarded");
    assert!(
        re_guarded.workflow_summary.is_some(),
        "pub-use workflow alias must preserve summary"
    );

    let alias_user = dir.path().join("alias_user.ash");
    std::fs::write(
        &alias_user,
        "use outer::{PublicToken}\nworkflow main(token: PublicToken) -> Int { ret 0 }\n",
    )
    .expect("write alias user");
    check_file(&alias_user).expect("imported alias name typechecks");

    let callable_alias_user = dir.path().join("callable_alias_user.ash");
    std::fs::write(
        &callable_alias_user,
        "use outer::{preserve}\nworkflow main { ret 0 }\n",
    )
    .expect("write callable alias user");
    let callable_loaded = load_ordinary_file(&callable_alias_user)
        .expect("re-exported callable imports its signature type summary dependency");
    assert!(callable_loaded.imported_callables.contains_key("preserve"));
    assert!(
        semantic_type_names(&callable_loaded).contains(&"PublicToken"),
        "callable-only re-export import should transport the aliased signature type summary"
    );

    let origin_user = dir.path().join("origin_user.ash");
    std::fs::write(
        &origin_user,
        "use outer::{PublicToken}\nworkflow main(token: Token) -> Int { ret 0 }\n",
    )
    .expect("write origin user");
    let err = check_file(&origin_user).expect_err("origin name is not imported through alias");
    assert!(
        err.contains("Token"),
        "origin-name leakage diagnostic should mention Token: {err}"
    );
}

#[test]
fn pub_use_glob_preserves_type_semantic_summaries() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\npub type Other = Other { value: String };\n",
    )
    .expect("write inner");
    std::fs::write(dir.path().join("outer.ash"), "pub use inner::*;\n").expect("write outer");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use outer::{Token}\nworkflow main { ret 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("glob re-export type import succeeds");
    assert_eq!(imported_type_names(&loaded), vec!["Token"]);
    assert_eq!(semantic_type_names(&loaded), vec!["Token"]);
}

#[test]
fn pub_use_preserves_reexported_constructor_semantic_summaries() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Status = Pending | Ready(Int);\npub type Token = Token { value: String };\n",
    )
    .expect("write inner");
    std::fs::write(dir.path().join("outer_glob.ash"), "pub use inner::*;\n")
        .expect("write outer glob");
    std::fs::write(
        dir.path().join("outer_nested.ash"),
        "pub use inner::{Status as PublicStatus, Token as PublicToken};\n",
    )
    .expect("write outer nested");

    let glob_caller = dir.path().join("glob_caller.ash");
    std::fs::write(
        &glob_caller,
        "use outer_glob::{Status}\nworkflow main { ret 0 }\n",
    )
    .expect("write glob caller");
    let glob_loaded = load_ordinary_file(&glob_caller).expect("glob re-export imports");
    assert_eq!(semantic_type_names(&glob_loaded), vec!["Status"]);
    assert_eq!(
        semantic_constructor_names(&glob_loaded),
        vec!["Pending", "Ready"]
    );

    let nested_caller = dir.path().join("nested_caller.ash");
    std::fs::write(
        &nested_caller,
        "use outer_nested::{PublicStatus}\nworkflow main { ret 0 }\n",
    )
    .expect("write nested caller");
    let nested_loaded = load_ordinary_file(&nested_caller).expect("nested alias re-export imports");
    assert_eq!(semantic_type_names(&nested_loaded), vec!["PublicStatus"]);
    assert_eq!(
        semantic_constructor_names(&nested_loaded),
        vec!["Pending", "Ready"]
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
fn child_modules_are_not_implicitly_flattened_into_parent_glob_import() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("child.ash"), "pub type ChildOnly = Int;\n")
        .expect("write child");
    std::fs::write(dir.path().join("parent.ash"), "pub mod child;\n").expect("write parent");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use parent::*\nworkflow main { ret 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("parent glob imports no flattened child types");
    assert!(
        !imported_type_names(&loaded).contains(&"ChildOnly"),
        "pub mod child must not flatten child exports into parent glob import"
    );
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
