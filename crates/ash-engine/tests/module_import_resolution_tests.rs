//! Tests for ordinary module import resolution.

use ash_engine::module_loader::load_ordinary_file;

const HELPER_GIT_URL: &str = "file:///tmp/helper";
const HELPER_GIT_DIGEST: &str = "520d384526df63a4";

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

#[tokio::test]
async fn task_972_fetched_cache_dependency_roots_are_visible_to_module_loader() {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let dep = tempfile::tempdir().expect("git dep");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let commit = write_fetched_helper_checkout(cache.path(), dep.path());
    write_locked_helper_project(project.path(), &commit);
    let main = src.join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let cache_root = cache.path().to_path_buf();
    let loaded = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&main) },
    )
    .await
    .expect("locked fetched-cache dependency root should resolve without root env vars");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "HelperToken")
    );
}

#[tokio::test]
async fn task_972_missing_fetched_cache_checkout_fails_closed() {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let dep = tempfile::tempdir().expect("git dep");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let commit = init_helper_git_dep(dep.path());
    write_locked_helper_project(project.path(), &commit);
    let main = src.join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let cache_root = cache.path().to_path_buf();
    let err = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&main) },
    )
    .await
    .expect_err("missing locked checkout must fail closed");
    assert!(
        err.to_string()
            .contains("locked package 'helper' is missing from fetched cache"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn task_972_missing_fetched_cache_checkout_does_not_block_self_import() {
    let project = tempfile::tempdir().expect("project");
    let cache = tempfile::tempdir().expect("xdg cache");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    write_locked_helper_project(project.path(), "0123456789abcdef0123456789abcdef01234567");
    std::fs::write(src.join("local.ash"), "pub type Local = Local;\n").expect("local");
    let main = src.join("main.ash");
    std::fs::write(&main, "use self::local::Local\nworkflow main { ret 0 }\n").expect("main");

    let cache_root = cache.path().to_path_buf();
    let loaded = temp_env::async_with_vars(
        [
            ("XDG_CACHE_HOME", Some(cache_root.as_os_str())),
            ("ASH_DEP_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_DEPENDENCY_ROOTS", None::<&std::ffi::OsStr>),
            ("ASH_LIBRARY_PATH", None::<&std::ffi::OsStr>),
        ],
        async { load_ordinary_file(&main) },
    )
    .await
    .expect("local self import should not require locked fetched-cache dependency roots");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "Local")
    );
}

#[test]
fn task_972_explicit_cache_shaped_dependency_root_is_not_locked_by_path_shape() {
    let project = tempfile::tempdir().expect("project");
    let checkout = project
        .path()
        .join("checkouts/helper-0000000000000000/not-a-locked-commit");
    std::fs::create_dir_all(&checkout).expect("checkout");
    std::fs::write(
        checkout.join("mod.ash"),
        "pub type HelperToken = HelperToken { value: Int };\n",
    )
    .expect("helper module");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        "use helper::{HelperToken}\nworkflow main() -> HelperToken { ret HelperToken { value: 7 }; }\n",
    )
    .expect("main");

    let err = ash_engine::module_loader::with_module_roots(vec![checkout], None, || {
        load_ordinary_file(&main)
    })
    .expect_err("explicit cache-shaped roots must not be package-bound by path shape");

    assert!(
        err.to_string().contains("module 'helper' not found"),
        "unexpected error: {err}"
    );
}

fn write_locked_helper_project(root: &std::path::Path, commit: &str) {
    std::fs::write(root.join("ash.toml"), "[package]\nname = \"app\"\n").expect("manifest");
    std::fs::write(
        root.join("ash.lock"),
        format!(
            "[[package]]\nname = \"helper\"\ngit = \"{HELPER_GIT_URL}\"\ncommit = \"{commit}\"\n",
        ),
    )
    .expect("lock");
}

fn write_fetched_helper_checkout(cache: &std::path::Path, dep: &std::path::Path) -> String {
    let commit = init_helper_git_dep(dep);
    let checkout = cache
        .join("ash/git/checkouts")
        .join(format!("helper-{HELPER_GIT_DIGEST}"))
        .join(&commit);
    run_git(
        std::path::Path::new("."),
        &[
            "clone",
            dep.to_str().expect("utf8"),
            checkout.to_str().expect("utf8"),
        ],
    );
    commit
}

fn init_helper_git_dep(dep: &std::path::Path) -> String {
    std::fs::write(
        dep.join("mod.ash"),
        "pub type HelperToken = HelperToken { value: Int };\n",
    )
    .expect("helper module");
    run_git(dep, &["init"]);
    run_git(dep, &["config", "user.email", "ash@example.invalid"]);
    run_git(dep, &["config", "user.name", "Ash Test"]);
    run_git(dep, &["add", "."]);
    run_git(dep, &["commit", "-m", "initial"]);
    git_output(dep, &["rev-parse", "HEAD"]).trim().to_string()
}

fn run_git(dir: &std::path::Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

fn git_output(dir: &std::path::Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgSign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
    assert!(output.status.success(), "git {args:?}");
    String::from_utf8(output.stdout).expect("git stdout")
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
