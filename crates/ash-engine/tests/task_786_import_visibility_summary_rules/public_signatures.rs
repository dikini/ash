use super::support::*;

#[test]
fn named_type_import_transports_public_representation_dependencies_without_type_leaks() {
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
        "use chat::{Message}\nfn main(message: Message) -> Int { 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("named dependent type import succeeds");
    assert_eq!(
        imported_type_names(&loaded),
        vec!["Message"],
        "imported type-definition cache should expose only the explicitly imported type"
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
        "use chat::{Message}\nfn main(r: Role) -> Int { 0 }\n",
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
        "use chat::{Message}\nfn main(message: Message) -> Role { System }\n",
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
        "use chat::{System, Message}\nfn main(message: Message) -> Role { System }\n",
    )
    .expect("write caller");

    check_file(&caller).expect("Message dependency summary should not de-expose Role constructors");
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
    std::fs::write(&caller, "use domain::{accept}\nfn main() { 0 }\n").expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("named callable import succeeds");
    assert!(loaded.imported_callables.contains_key("accept"));
    assert_eq!(semantic_type_names(&loaded), vec!["Token"]);
}

#[test]
fn public_callable_signature_allows_builtin_carrier_types() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("carrier.ash");
    std::fs::write(
        &module,
        "pub builtin fn read() -> Bytes;\npub builtin fn await<A>(handle: P<A>) -> P<A>;\n",
    )
    .expect("write carrier module");

    check_module_file(&module).expect("builtin carrier public signatures should be accepted");
}
