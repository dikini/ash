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

fn check_module_file(path: &std::path::Path) -> Result<(), String> {
    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|error| error.to_string())?;
    let result = engine
        .check_module_file(path)
        .map_err(|error| error.to_string())?;
    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(result.errors.join("\n"))
    }
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
fn named_type_import_transports_public_representation_dependencies_without_legacy_type_leaks() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("chat.ash"),
        r"pub type Role = System | User;
pub type ToolCall = ToolCall { id: String };
pub type Message = Message {
    sender: Role,
    tool_calls: Option<List<ToolCall>>
};
",
    )
    .expect("write chat module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use chat::{Message}\nworkflow main(message: Message) -> Int { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("named dependent type import succeeds");
    assert_eq!(
        imported_type_names(&loaded),
        vec!["Message"],
        "legacy TypeDef fallback should expose only the explicitly imported type"
    );
    assert_eq!(
        semantic_type_names(&loaded),
        vec![
            "$ash_dependency$Role",
            "$ash_dependency$ToolCall",
            "Message",
        ],
        "semantic summaries should carry the selected type plus hidden representation dependencies"
    );
    check_file(&caller).expect("dependent representation summaries should typecheck");

    let leaked_dependency_type_user = dir.path().join("leaked_dependency_type_user.ash");
    std::fs::write(
        &leaked_dependency_type_user,
        "use chat::{Message}\nworkflow main(r: Role) -> Int { ret 0 }\n",
    )
    .expect("write leaked dependency type user");
    let err = check_file(&leaked_dependency_type_user)
        .expect_err("representation dependencies must not be source-visible annotations");
    assert!(
        err.contains("Role"),
        "diagnostic should mention hidden dependency type Role: {err}"
    );

    let leaked_constructor_user = dir.path().join("leaked_constructor_user.ash");
    std::fs::write(
        &leaked_constructor_user,
        "use chat::{Message}\nworkflow main(message: Message) -> Role { ret System; }\n",
    )
    .expect("write leaked constructor user");
    let err = check_file(&leaked_constructor_user)
        .expect_err("representation dependencies must not expose constructors as values");
    assert!(
        err.contains("System") || err.contains("Role"),
        "diagnostic should mention hidden dependency constructor/type: {err}"
    );
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
        "use inner::{Node as PublicNode}\nworkflow main(node: PublicNode) -> Int { ret 0 }\n",
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
        "use outer::{PublicNode}\nworkflow main(node: PublicNode) -> Int { ret 0 }\n",
    )
    .expect("write reexported caller");
    check_file(&reexported).expect("pub-use aliased self-recursive type should check");
}

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
        "use outer::{PublicToken}\nworkflow main() -> PublicToken { ret PublicToken { value: 1 }; }\n",
    )
    .expect("write alias constructor user");
    let loaded = load_ordinary_file(&alias_constructor_user).expect("aliased type import loads");
    assert_eq!(semantic_type_names(&loaded), vec!["PublicToken"]);
    assert_eq!(semantic_constructor_names(&loaded), vec!["PublicToken"]);
    check_file(&alias_constructor_user).expect("aliased struct constructor should be visible");

    let origin_constructor_user = dir.path().join("origin_constructor_user.ash");
    std::fs::write(
        &origin_constructor_user,
        "use outer::{PublicToken}\nworkflow main() -> PublicToken { ret Token { value: 1 }; }\n",
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
        "use outer::{shout}\nworkflow main() -> String { ret shout(\"hey\"); }\n",
    )
    .expect("write caller");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse");
    engine.check(&mut workflow).expect("typecheck");
    let result = engine.execute(&workflow).await.expect("execute");
    assert_eq!(result, ash_core::Value::String("HEY".to_string()));
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
        "use inner::{A as PublicA}\nworkflow main(a: PublicA) -> Int { ret 0 }\n",
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
fn representation_dependency_import_does_not_strip_existing_enum_constructors() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("chat.ash"),
        "pub type Role = System | User;\npub type Message = Message { role: Role };\n",
    )
    .expect("write chat module");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &caller,
        "use chat::{System, Message}\nworkflow main(message: Message) -> Role { ret System; }\n",
    )
    .expect("write caller");

    check_file(&caller).expect("Message dependency summary should not de-expose Role constructors");
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
        "use outer::{PublicA}\nworkflow main(a: PublicA) -> Int { ret 0 }\n",
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
        "legacy fallback representation should use dependency alias: {fallback_debug}"
    );
    assert!(
        !fallback_debug.contains("Named(\"B\")"),
        "legacy fallback representation must not leak origin dependency name: {fallback_debug}"
    );
    check_file(&caller).expect("aliased dependency summary should typecheck");
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

#[test]
fn public_callable_signature_allows_builtin_carrier_types() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("carrier.ash");
    std::fs::write(
        &module,
        "pub builtin fn read() -> Bytes;\npub builtin fn await<A>(handle: P<A>) -> Proc<A>;\n",
    )
    .expect("write carrier module");

    check_module_file(&module).expect("builtin carrier public signatures should be accepted");
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
