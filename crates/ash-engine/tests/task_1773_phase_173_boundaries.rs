//! TASK-1773 closeout regressions for Phase 173 engine/module-loader boundaries.

use ash_engine::Engine;
use ash_engine::module_loader::load_ordinary_file;

fn write_pair(
    provider_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, caller)
}

#[tokio::test]
async fn imported_macro_summary_does_not_create_runtime_callable_binding() {
    let (_dir, caller) = write_pair(
        "\npub macro answer() => 1;\n",
        "\nuse provider::{answer}\nworkflow main() -> Int { ret answer(); }\n",
    );

    let loaded = load_ordinary_file(&caller).expect("caller metadata loads");
    assert_eq!(loaded.imported_macro_summaries.len(), 1);
    assert!(
        !loaded.imported_callables.contains_key("answer"),
        "macro summary import must not create a callable binding"
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .run_file(&caller)
        .await
        .expect_err("ordinary call to imported macro name must not execute as a callable");
    let message = err.to_string();
    assert!(
        message.contains("undefined")
            || message.contains("unknown function")
            || message.contains("not found"),
        "unexpected error: {message}"
    );
}

#[tokio::test]
async fn imported_macro_summary_does_not_transport_private_template_helpers() {
    let (_dir, caller) = write_pair(
        "\npub macro secret_inc(x: Int) => secret_add(x, 1);\nfn secret_add(x: Int, y: Int) -> Int { x + y }\n",
        "\nuse provider::{secret_inc}\nworkflow main() -> Int { ret secret_inc!(1); }\n",
    );

    let loaded = load_ordinary_file(&caller).expect("caller metadata loads");
    assert_eq!(loaded.imported_macro_summaries.len(), 1);
    assert!(
        loaded.imported_callables.is_empty(),
        "macro summaries must not leak private helper callables into the caller"
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .run_file(&caller)
        .await
        .expect_err("macro helper dependencies must not be silently transported");
    let message = err.to_string();
    assert!(
        message.contains("secret_add")
            || message.contains("secret_inc")
            || message.contains("unexpanded macro invocation carrier"),
        "unexpected error: {message}"
    );
}
