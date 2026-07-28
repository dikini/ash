use super::support::*;

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

#[test]
fn aliased_self_recursive_type_rewrites_self_references_to_visible_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Node = Node { next: Option<Node> };\n",
    )
    .expect("write inner module");
    let direct = dir.path().join("direct.ash");
    std::fs::write(
        &direct,
        "use inner::{Node as PublicNode}\nfn main(node: PublicNode) -> Int { 0 }\n",
    )
    .expect("write direct caller");

    let loaded = load_ordinary_file(&direct).expect("aliased self-recursive type imports");
    assert_eq!(imported_type_names(&loaded), vec!["PublicNode"]);
    let public_node = loaded
        .imported_type_defs
        .iter()
        .find(|type_def| type_def.name == "PublicNode")
        .expect("visible fallback type exists");
    assert!(
        format!("{:?}", public_node.body).contains("PublicNode"),
        "aliased fallback type body should use the visible self name: {public_node:?}"
    );
    check_file(&direct).expect("direct aliased self-recursive type should check");

    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{Node as PublicNode};\n",
    )
    .expect("write outer module");
    let reexported = dir.path().join("reexported.ash");
    std::fs::write(
        &reexported,
        "use outer::{PublicNode}\nfn main(node: PublicNode) -> Int { 0 }\n",
    )
    .expect("write reexported caller");
    check_file(&reexported).expect("pub-use aliased self-recursive type should check");
}

#[test]
fn split_pub_use_type_alias_exposes_alias_constructor_not_origin_constructor() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: Int };\npub fn keep(token: Token) -> Token { token }\n",
    )
    .expect("write inner module");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{keep};\npub use inner::{Token as PublicToken};\n",
    )
    .expect("write outer module");

    let alias_constructor_user = dir.path().join("alias_constructor_user.ash");
    std::fs::write(
        &alias_constructor_user,
        "use outer::{PublicToken}\nfn main() -> PublicToken { PublicToken { value: 1 } }\n",
    )
    .expect("write alias constructor user");
    let loaded = load_ordinary_file(&alias_constructor_user).expect("aliased type import loads");
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    assert_eq!(semantic_constructor_names(&loaded), vec!["PublicToken"]);
    check_file(&alias_constructor_user).expect("aliased struct constructor should be visible");

    let origin_constructor_user = dir.path().join("origin_constructor_user.ash");
    std::fs::write(
        &origin_constructor_user,
        "use outer::{PublicToken}\nfn main() -> PublicToken { Token { value: 1 } }\n",
    )
    .expect("write origin constructor user");
    let err = check_file(&origin_constructor_user)
        .expect_err("origin struct constructor must not leak through aliased pub-use");
    assert!(
        err.contains("Token"),
        "diagnostic should mention hidden origin constructor Token: {err}"
    );
}

#[tokio::test]
async fn builtin_callable_reexport_alias_executes_original_dispatch_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use string::{to_upper as shout};\n",
    )
    .expect("write outer module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{shout}\nfn main() -> String { shout(\"hey\") }\n",
    )
    .expect("write caller");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let error = engine
        .execute(&workflow)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(error, ash_runtime::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "callable reexport source must expose the exact canonical closed-admission error"
    );
}

#[test]
fn public_signature_resolution_sees_same_module_pub_use_type_alias() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type Token = Token { value: Int };\n",
    )
    .expect("write inner module");
    let outer = dir.path().join("outer.ash");
    std::fs::write(
        &outer,
        "pub use inner::{Token as PublicToken};\npub fn expose(x: PublicToken) -> Int { 0 }\n",
    )
    .expect("write outer module");

    check_module_file(&outer).expect("pub fn signatures may name same-module pub-use aliases");
}

#[test]
fn aliased_selected_type_rewrites_dependency_bodies() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type A = A { b: B };\npub type B = B { a: A };\n",
    )
    .expect("write inner module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use inner::{A as PublicA}\nfn main(a: PublicA) -> Int { 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("aliased mutually recursive type imports");
    assert_eq!(imported_type_names(&loaded), vec!["PublicA"]);
    assert_eq!(
        semantic_type_names(&loaded),
        vec!["$ash_dependency$B", "PublicA"]
    );
    let dependency = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_types.iter())
        .find(|ty| ty.exported_name == "$ash_dependency$B")
        .expect("dependency summary exists");
    assert!(
        format!("{:?}", dependency.representation).contains("PublicA"),
        "dependency body should refer to the imported alias: {dependency:?}"
    );
    check_file(&caller).expect("aliased dependency body should check");
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
    std::fs::write(&glob_caller, "use outer_glob::{Status}\nfn main() { 0 }\n")
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
        "use outer_nested::{PublicStatus}\nfn main() { 0 }\n",
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
fn reexport_aliases_rewrite_selected_representation_dependencies() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("inner.ash"),
        "pub type A = A { b: B };\npub type B = B { value: Int };\n",
    )
    .expect("write inner");
    std::fs::write(
        dir.path().join("outer.ash"),
        "pub use inner::{A as PublicA, B as PublicB};\n",
    )
    .expect("write outer");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use outer::{PublicA}\nfn main(a: PublicA) -> Int { 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("aliased dependency import succeeds");
    assert_eq!(imported_type_names(&loaded), vec!["PublicA"]);
    assert_eq!(
        semantic_type_names(&loaded),
        vec!["$ash_dependency$PublicB", "PublicA"]
    );
    let fallback_debug = format!("{:?}", loaded.imported_type_defs);
    assert!(
        fallback_debug.contains("PublicB"),
        "imported type-definition representation should use dependency alias: {fallback_debug}"
    );
    assert!(
        !fallback_debug.contains("Named(\"B\")"),
        "imported type-definition representation must not leak origin dependency name: {fallback_debug}"
    );
    check_file(&caller).expect("aliased dependency summary should typecheck");
}
