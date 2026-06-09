use super::support::*;

#[test]
fn callable_reexport_signature_aliases_are_order_independent_across_pub_use_statements() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\npub fn keep(token: Token) -> Token { token }\n",
    )
    .expect("write inner module");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{keep as preserve};\npub use inner::{Token as PublicToken};\n",
    )
    .expect("write outer module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{preserve, PublicToken}\nworkflow main(token: PublicToken) -> PublicToken { ret preserve(token); }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("callable alias re-export imports");
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    check_file(&caller)
        .expect("callable re-export should use final alias map independent of pub-use order");
}

#[test]
fn callable_reexport_signature_alias_rewrite_ignores_masking_local_type_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\npub fn keep(token: Token) -> Token { token }\n",
    )
    .expect("write inner module");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub type Token = Token { local: String };\npub use inner::{Token as PublicToken, keep as preserve};\n",
    )
    .expect("write outer module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{preserve, PublicToken}\nworkflow main(token: PublicToken) -> PublicToken { ret preserve(token); }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("masked callable alias re-export imports");
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    check_file(&caller).expect(
        "callable re-export signature should target PublicToken, not the masking outer Token",
    );
}

#[test]
fn callable_reexport_transports_signature_summaries_without_reexporting_types() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\npub fn take_a(token: Token) -> Token { token }\n",
    )
    .expect("write inner module");
    std::fs::write(dir.path().join("outer.ash"), "pub use inner::{take_a};\n")
        .expect("write outer module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&caller, "use outer::{take_a}\nworkflow main { ret 0 }\n")
        .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("callable-only re-export imports");
    assert_eq!(semantic_type_names(&loaded), vec!["Token"]);
    check_file(&caller).expect("callable signature dependency summary should typecheck");
}

#[test]
fn callable_reexport_alias_rewrite_does_not_cross_same_name_type_identities() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("a.ash"),
        "pub type Token = Token { a: Int };\npub fn take_a(token: Token) -> Token { token }\n",
    )
    .expect("write a module");
    std::fs::write(
        dir.path().join("b.ash"),
        "pub type Token = Token { b: String };\n",
    )
    .expect("write b module");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use a::{take_a};\npub use b::{Token as PublicToken};\n",
    )
    .expect("write outer module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{take_a, PublicToken}\nworkflow main(token: PublicToken) -> PublicToken { ret take_a(token); }\n",
    )
    .expect("write caller");

    let err = check_file(&caller).expect_err("unrelated same-name aliases must remain distinct");
    assert!(
        err.contains("type mismatch") || err.contains("Cannot unify"),
        "expected nominal mismatch diagnostic, got: {err}"
    );
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
        !imported_type_names(&callable_loaded).contains(&"Token"),
        "callable-only alias import must not leak the origin type name through legacy TypeDef fallback"
    );
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
fn callable_only_reexport_alias_does_not_make_origin_type_visible() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: String };\npub fn keep(token: Token) -> Token { token }\n",
    )
    .expect("write inner");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{keep as preserve, Token as PublicToken};\n",
    )
    .expect("write outer");

    let callable_only = dir.path().join("callable_only.ash");
    std::fs::write(
        &callable_only,
        "use outer::{preserve}\nworkflow main(token: PublicToken) -> Int { ret 0 }\n",
    )
    .expect("write callable-only caller");
    let loaded = load_ordinary_file(&callable_only)
        .expect("callable-only import should transport aliased signature summary");
    assert!(loaded.imported_callables.contains_key("preserve"));
    assert!(
        imported_type_names(&loaded).is_empty(),
        "callable-only alias import should rely on semantic summaries instead of legacy TypeDef fallback"
    );
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    check_file(&callable_only).expect("aliased signature type should be typecheck-visible");

    let origin_user = dir.path().join("origin_user.ash");
    std::fs::write(
        &origin_user,
        "use outer::{preserve}\nworkflow main(token: Token) -> Int { ret 0 }\n",
    )
    .expect("write origin user");
    let err = check_file(&origin_user).expect_err("callable-only import must not expose Token");
    assert!(
        err.contains("Token"),
        "origin-name diagnostic should mention Token: {err}"
    );
}

#[test]
fn public_callable_signature_accepts_reexported_type_alias_import() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: Int };\n",
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
        "use outer::{PublicToken}\npub fn expose(x: PublicToken) -> Option<PublicToken> { None }\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    check_module_file(&caller).expect(
        "re-exported public type aliases and prelude Option are valid public signature names",
    );
}
