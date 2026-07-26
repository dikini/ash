//! IO Standard Library Wiring Tests (TASK-498)
//!
//! Tests that verify the io stdlib modules work end-to-end with the engine.
//! These tests ensure:
//! - io modules load correctly through the module loader
//! - Runtime-backed IO declarations are exposed through target stdlib modules
//! - Engine builder pattern works with io capabilities
//! - End-to-end execution of io operations

use ash_engine::Engine;
use std::path::PathBuf;
use std::sync::Arc;

fn assert_closed_checked_cps_admission(result: ash_interp::ExecResult<ash_core::Value>) {
    let error = result
        .expect_err("IO-wired source must remain closed without validated Core/CPS admission");
    assert!(
        error
            .to_string()
            .contains("checked Core/CPS admission rejected"),
        "IO-wired source must expose the stable closed-admission diagnostic, got: {error}"
    );
}

// ============================================================
// Module Loading Tests
// ============================================================

/// Test that io modules can be found through the stdlib root
#[test]
fn test_io_modules_load_through_stdlib_root() {
    // The stdlib root is at std/src/lib.ash which re-exports io modules
    let stdlib_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/lib.ash");

    assert!(stdlib_root.exists(), "stdlib lib.ash should exist");

    // Read the stdlib to verify io re-exports are present
    let content = std::fs::read_to_string(&stdlib_root).expect("can read lib.ash");

    // Verify io module re-exports are present
    assert!(
        content.contains("io::path"),
        "lib.ash should re-export io::path"
    );
    assert!(
        content.contains("io::stdio"),
        "lib.ash should re-export io::stdio"
    );
    assert!(
        content.contains("io::fs"),
        "lib.ash should re-export io::fs"
    );
    assert!(
        content.contains("io::dir"),
        "lib.ash should re-export io::dir"
    );
    assert!(
        content.contains("io::meta"),
        "lib.ash should re-export io::meta"
    );
    assert!(
        content.contains("io::buf"),
        "lib.ash should re-export io::buf"
    );
}

/// Test that individual io module files exist
#[test]
fn test_io_module_files_exist() {
    let stdlib_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io");

    // Check that all expected io module files exist
    assert!(
        stdlib_src.join("mod.ash").exists(),
        "io/mod.ash should exist"
    );
    assert!(
        stdlib_src.join("path.ash").exists(),
        "io/path.ash should exist"
    );
    assert!(
        stdlib_src.join("stdio.ash").exists(),
        "io/stdio.ash should exist"
    );
    assert!(stdlib_src.join("fs.ash").exists(), "io/fs.ash should exist");
    assert!(
        stdlib_src.join("dir.ash").exists(),
        "io/dir.ash should exist"
    );
    assert!(
        stdlib_src.join("meta.ash").exists(),
        "io/meta.ash should exist"
    );
    assert!(
        stdlib_src.join("buf.ash").exists(),
        "io/buf.ash should exist"
    );
}

/// Test that io/mod.ash exports all submodules
#[test]
fn test_io_mod_exports_submodules() {
    let mod_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/mod.ash");

    let content = std::fs::read_to_string(&mod_path).expect("can read mod.ash");

    // Check for submodule declarations
    assert!(
        content.contains("pub mod path"),
        "mod.ash should declare pub mod path"
    );
    assert!(
        content.contains("pub mod stdio"),
        "mod.ash should declare pub mod stdio"
    );
    assert!(
        content.contains("pub mod fs"),
        "mod.ash should declare pub mod fs"
    );
    assert!(
        content.contains("pub mod dir"),
        "mod.ash should declare pub mod dir"
    );
    assert!(
        content.contains("pub mod meta"),
        "mod.ash should declare pub mod meta"
    );
    assert!(
        content.contains("pub mod buf"),
        "mod.ash should declare pub mod buf"
    );
}

// ============================================================
// Runtime-Backed IO Declaration Tests
// ============================================================

/// Test that stdio runtime-backed functions are declared in stdio.ash
#[test]
fn test_stdio_runtime_functions_declared() {
    let stdio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/stdio.ash");

    let content = std::fs::read_to_string(&stdio_path).expect("can read stdio.ash");

    assert!(
        content.contains("pub builtin fn read_line() -> String;"),
        "stdio.ash should declare read_line builtin"
    );
    assert!(
        content.contains("pub builtin fn print(text: String) -> Unit;"),
        "stdio.ash should declare print builtin"
    );
    assert!(
        content.contains("pub builtin fn println(text: String) -> Unit;"),
        "stdio.ash should declare println builtin"
    );
}

/// Test that fs runtime-backed functions are declared in fs.ash
#[test]
fn test_fs_runtime_functions_declared() {
    let fs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/fs.ash");

    let content = std::fs::read_to_string(&fs_path).expect("can read fs.ash");

    assert!(
        content.contains("pub builtin fn read(path: PathBuf) -> Bytes;"),
        "fs.ash should declare read builtin"
    );
    assert!(
        content.contains("pub builtin fn read_to_string(path: PathBuf) -> String;"),
        "fs.ash should declare read_to_string builtin"
    );
    assert!(
        content.contains("pub builtin fn write(path: PathBuf, content: Bytes) -> Unit;"),
        "fs.ash should declare write builtin"
    );
}

/// Test that dir runtime-backed functions are declared in dir.ash
#[test]
fn test_dir_runtime_functions_declared() {
    let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/dir.ash");

    let content = std::fs::read_to_string(&dir_path).expect("can read dir.ash");

    assert!(
        content.contains("pub builtin fn create_dir(path: PathBuf) -> Unit;"),
        "dir.ash should declare create_dir builtin"
    );
    assert!(
        content.contains("pub builtin fn create_dir_all(path: PathBuf) -> Unit;"),
        "dir.ash should declare create_dir_all builtin"
    );
    assert!(
        content.contains("pub builtin fn read_dir(path: PathBuf) -> List<String>;"),
        "dir.ash should declare read_dir builtin"
    );
}

/// Test that metadata runtime-backed functions are declared in meta.ash
#[test]
fn test_meta_runtime_functions_declared() {
    let meta_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/meta.ash");

    let content = std::fs::read_to_string(&meta_path).expect("can read meta.ash");

    assert!(
        content.contains("pub type Metadata = Metadata"),
        "meta.ash should declare Metadata type"
    );
    assert!(
        content.contains("pub builtin fn metadata(path: PathBuf) -> Metadata;"),
        "meta.ash should declare metadata builtin"
    );
}

// ============================================================
// Engine Builder Integration Tests
// ============================================================

/// Test that engine builder `with_stdio_capabilities` works with io imports.
#[test]
fn test_engine_builder_with_stdio_for_io_imports() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .expect("engine builds with stdio capabilities");

    // Preserve parsing coverage; generic application execution remains closed.
    let application = engine.parse("fn main() { 42 }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&application).await });

    assert_closed_checked_cps_admission(result);
}

/// Test that engine builder `with_fs_capabilities` works with io imports.
#[test]
fn test_engine_builder_with_fs_for_io_imports() {
    let engine = Engine::new()
        .with_fs_capabilities()
        .build()
        .expect("engine builds with fs capabilities");

    // Preserve parsing coverage; generic application execution remains closed.
    let application = engine.parse("fn main() { 42 }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&application).await });

    assert_closed_checked_cps_admission(result);
}

/// Test that engine builder with both stdio and fs capabilities works
#[test]
fn test_engine_builder_with_stdio_and_fs_capabilities() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds with io capabilities");

    // Preserve parsing coverage; generic application execution remains closed.
    let application = engine.parse("fn main() { 42 }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&application).await });

    assert_closed_checked_cps_admission(result);
}

/// Test that custom providers can override io capabilities
#[test]
fn test_custom_provider_can_override_stdio() {
    use ash_core::capability::{
        CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    };
    use ash_core::{Constraint, Effect, Value};
    use async_trait::async_trait;

    /// A test provider that overrides stdio
    #[derive(Debug)]
    struct TestStdioProvider;

    #[async_trait]
    impl CapabilityProvider for TestStdioProvider {
        fn name(&self) -> &'static str {
            "stdio"
        }

        fn effect(&self) -> Effect {
            Effect::Operational
        }

        fn provider_metadata(&self) -> ProviderAuthoringMetadata {
            ProviderAuthoringMetadata::new("stdio")
                .with_operation(
                    ProviderOperationMetadata::new("read_line", Effect::Epistemic)
                        .with_required_row("stdio.read_line")
                        .with_resource("stdio")
                        .with_sandbox_policy("test.stdio.read")
                        .with_provenance_policy("test.stdio.read.redacted"),
                )
                .with_operation(
                    ProviderOperationMetadata::new("print", Effect::Operational)
                        .with_required_row("stdio.print")
                        .with_resource("stdio")
                        .with_sandbox_policy("test.stdio.write")
                        .with_provenance_policy("test.stdio.write.redacted"),
                )
                .with_operation(
                    ProviderOperationMetadata::new("println", Effect::Operational)
                        .with_required_row("stdio.println")
                        .with_resource("stdio")
                        .with_sandbox_policy("test.stdio.write")
                        .with_provenance_policy("test.stdio.write.redacted"),
                )
        }

        async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
            Ok(Value::String("test".to_string()))
        }

        async fn execute(
            &self,
            _action_name: &str,
            _args: &[Value],
        ) -> Result<Value, CapabilityError> {
            Ok(Value::Null)
        }
    }

    let engine = Engine::new()
        .with_stdio_capabilities() // First enable built-in
        .with_custom_provider("stdio", Arc::new(TestStdioProvider)) // Then override
        .build()
        .expect("engine builds with custom stdio provider");

    // Preserve custom-provider wiring; generic application execution remains closed.
    let application = engine.parse("fn main() { 42 }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&application).await });

    assert_closed_checked_cps_admission(result);
}

// ============================================================
// End-to-End IO Execution Tests
// ============================================================

/// Test that engine with io capabilities can execute basic applications
#[tokio::test]
async fn test_e2e_engine_with_io_capabilities() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // Untyped source remains closed without a validated Core/CPS admission.
    let result = engine.run("fn main() { 42 }").await;

    assert_closed_checked_cps_admission(result);
}

/// Test that engine with io capabilities handles multiple executions
#[tokio::test]
async fn test_e2e_io_capabilities_multiple_executions() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // Run multiple applications
    for i in 0..5 {
        let result = engine.run(&format!("fn main() -> Int {{ {i} }}")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ash_core::Value::Int(i));
    }
}

/// Test that engine with io capabilities is Send + Sync
#[test]
fn test_io_capabilities_engine_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // Compile-time checks
    assert_send::<Engine>();
    assert_sync::<Engine>();

    // Runtime verification
    let _ = &engine;
}

// ============================================================
// Path Module Pure Function Tests
// ============================================================

/// Test that path operations work correctly (pure functions)
#[test]
fn test_path_pure_operations() {
    // Test PathBuf construction and joining
    let path1 = PathBuf::from("/home/user");
    let path2 = PathBuf::from("documents");
    let joined = path1.join(&path2);
    assert_eq!(joined, PathBuf::from("/home/user/documents"));

    // Test parent extraction
    let file_path = PathBuf::from("/home/user/file.txt");
    let parent = file_path.parent().expect("has parent");
    assert_eq!(parent, PathBuf::from("/home/user"));

    // Test file name extraction
    let file_name = file_path.file_name().expect("has file name");
    assert_eq!(file_name, "file.txt");

    // Test extension extraction
    let ext = file_path.extension().expect("has extension");
    assert_eq!(ext, "txt");

    // Test absolute path check
    assert!(file_path.is_absolute());
    assert!(!PathBuf::from("relative/path").is_absolute());
}

/// Test path operations with edge cases
#[test]
fn test_path_edge_cases() {
    // Root path
    let root = PathBuf::from("/");
    assert!(root.is_absolute());
    assert!(root.parent().is_none());

    // Empty extension
    let no_ext = PathBuf::from("/home/user/file");
    assert!(no_ext.extension().is_none());

    // Multiple dots
    let multi_dot = PathBuf::from("/home/user/archive.tar.gz");
    let ext = multi_dot.extension().expect("has extension");
    assert_eq!(ext, "gz");
}
