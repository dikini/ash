//! Tests for ordinary module import resolution.

use ash_engine::module_loader::load_ordinary_file;

#[test]
fn plain_workflow_with_legacy_act_body_is_importable_by_signature() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("dispatch.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &module,
        "pub type Message = Message(String);\npub type ToolDef = ToolDef(String);\npub type ChatResponse = ChatResponse(String);\npub type CompletionParams = CompletionParams(String);\nworkflow complete_with_tools(\n    provider: String,\n    model: String,\n    messages: List<Message>,\n    tools: List<ToolDef>,\n    params: Option<CompletionParams>\n) -> ChatResponse {\n    act execute Llm.chat_with_tools with\n        provider: provider,\n        model: model,\n        messages: messages,\n        tools: tools,\n        params: params\n}\n",
    )
    .expect("write module");
    std::fs::write(
        &caller,
        "use dispatch::{Message, ToolDef, ChatResponse, CompletionParams, complete_with_tools}\nworkflow main { ret 0 }\n",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller)
        .expect("legacy workflow signature export should import without parsing body");
    let callable = loaded
        .imported_callables
        .get("complete_with_tools")
        .expect("complete_with_tools callable should be imported");
    assert_eq!(callable.exported_name, "complete_with_tools");
    assert_eq!(
        callable.params,
        vec!["provider", "model", "messages", "tools", "params"]
    );
}

#[test]
fn task_972_dependency_roots_from_env_are_visible_to_module_loader() {
    let project = tempfile::tempdir().expect("project");
    let dep_root = tempfile::tempdir().expect("dep");
    std::fs::write(dep_root.path().join("dep.ash"), "pub type Dep = Dep;\n").expect("dep");
    let main = project.path().join("main.ash");
    std::fs::write(&main, "use dep::Dep\nworkflow main { ret 0 }\n").expect("main");

    let loaded = ash_engine::module_loader::with_module_roots(
        vec![dep_root.path().to_path_buf()],
        None,
        || load_ordinary_file(&main),
    )
    .expect("dependency root import should resolve");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "Dep")
    );
}

#[test]
fn super_self_and_crate_imports_resolve_relative_to_importing_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("runtime/supervisor")).expect("mkdirs");

    std::fs::write(
        root.join("runtime/error.ash"),
        "pub type RuntimeError = RuntimeError(Int, String);\n",
    )
    .expect("write error");
    std::fs::write(
        root.join("runtime/args.ash"),
        "pub type Args = Args(List<String>);\n",
    )
    .expect("write args");
    std::fs::write(
        root.join("runtime/supervisor/local.ash"),
        "pub type Local = Local;\n",
    )
    .expect("write local");
    std::fs::write(
        root.join("runtime/supervisor/main.ash"),
        "use super::error::RuntimeError\nuse crate::runtime::args::Args\nuse self::local::Local\nworkflow main { ret 0 }\n",
    )
    .expect("write main");

    let loaded = load_ordinary_file(&root.join("runtime/supervisor/main.ash"))
        .expect("relative imports should resolve");
    let names = loaded
        .imported_type_defs
        .iter()
        .map(|def| def.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        names.contains(&"RuntimeError"),
        "super import should resolve"
    );
    assert!(names.contains(&"Args"), "crate import should resolve");
    assert!(names.contains(&"Local"), "self import should resolve");
}
