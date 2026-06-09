use super::support::*;

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
fn glob_import_of_reexport_alias_does_not_leak_origin_type_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\n",
    )
    .expect("write inner");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{Token as PublicToken};\n",
    )
    .expect("write outer");

    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::*\nworkflow main(token: PublicToken) -> Int { ret 0 }\n",
    )
    .expect("write caller");
    let loaded =
        load_ordinary_file(&caller).expect("glob import should use alias-visible type name");
    assert_eq!(imported_type_names(&loaded), vec!["PublicToken"]);
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    check_file(&caller).expect("glob-imported alias should typecheck under alias-visible name");

    let origin_user = dir.path().join("origin_user.ash");
    std::fs::write(
        &origin_user,
        "use outer::*\nworkflow main(token: Token) -> Int { ret 0 }\n",
    )
    .expect("write origin user");
    let err = check_file(&origin_user).expect_err("glob import must not expose origin Token");
    assert!(
        err.contains("Token"),
        "origin-name diagnostic should mention Token: {err}"
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
