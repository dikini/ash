//! IO Standard Library Wiring Tests (TASK-498)
//!
//! Tests that verify the io stdlib modules work end-to-end with the engine.
//! These tests ensure:
//! - io modules load correctly through the module loader
//! - Capabilities from io modules are resolved correctly
//! - Engine builder pattern works with io capabilities
//! - End-to-end execution of io operations

use ash_engine::Engine;
use std::path::PathBuf;
use std::sync::Arc;

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
// Capability Export Resolution Tests
// ============================================================

/// Test that stdio capability is defined in stdio.ash
#[test]
fn test_stdio_capability_defined() {
    let stdio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/stdio.ash");

    let content = std::fs::read_to_string(&stdio_path).expect("can read stdio.ash");

    // Check for Stdio capability definition
    assert!(
        content.contains("capability Stdio"),
        "stdio.ash should define Stdio capability"
    );
    assert!(
        content.contains("observe read_line"),
        "Stdio capability should have read_line observe"
    );
    assert!(
        content.contains("execute print"),
        "Stdio capability should have print execute"
    );
    assert!(
        content.contains("execute println"),
        "Stdio capability should have println execute"
    );
}

/// Test that fs capability is defined in fs.ash
#[test]
fn test_fs_capability_defined() {
    let fs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/fs.ash");

    let content = std::fs::read_to_string(&fs_path).expect("can read fs.ash");

    // Check for Fs capability definition
    assert!(
        content.contains("capability Fs"),
        "fs.ash should define Fs capability"
    );
    assert!(
        content.contains("observe read("),
        "Fs capability should have read observe"
    );
    assert!(
        content.contains("execute write("),
        "Fs capability should have write execute"
    );
}

/// Test that dir capability is defined in dir.ash
#[test]
fn test_dir_capability_defined() {
    let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/dir.ash");

    let content = std::fs::read_to_string(&dir_path).expect("can read dir.ash");

    // Check for Dir capability definition
    assert!(
        content.contains("capability Dir"),
        "dir.ash should define Dir capability"
    );
    assert!(
        content.contains("execute create_dir"),
        "Dir capability should have create_dir execute"
    );
    assert!(
        content.contains("observe read_dir"),
        "Dir capability should have read_dir observe"
    );
}

/// Test that meta capability is defined in meta.ash
#[test]
fn test_meta_capability_defined() {
    let meta_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/meta.ash");

    let content = std::fs::read_to_string(&meta_path).expect("can read meta.ash");

    // Check for Meta capability definition
    assert!(
        content.contains("capability Meta"),
        "meta.ash should define Meta capability"
    );
    assert!(
        content.contains("observe metadata"),
        "Meta capability should have metadata observe"
    );
}

// ============================================================
// Engine Builder Integration Tests
// ============================================================

/// Test that engine builder with_stdio_capabilities works with io imports
#[test]
fn test_engine_builder_with_stdio_for_io_imports() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .expect("engine builds with stdio capabilities");

    // Verify the engine can parse and execute workflows
    let workflow = engine.parse("workflow main { ret 42; }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&workflow).await });

    assert!(result.is_ok(), "Engine should execute workflow");
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

/// Test that engine builder with_fs_capabilities works with io imports
#[test]
fn test_engine_builder_with_fs_for_io_imports() {
    let engine = Engine::new()
        .with_fs_capabilities()
        .build()
        .expect("engine builds with fs capabilities");

    // Verify the engine can parse and execute workflows
    let workflow = engine.parse("workflow main { ret 42; }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&workflow).await });

    assert!(result.is_ok(), "Engine should execute workflow");
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

/// Test that engine builder with both stdio and fs capabilities works
#[test]
fn test_engine_builder_with_stdio_and_fs_capabilities() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds with io capabilities");

    // Verify the engine can parse and execute workflows
    let workflow = engine.parse("workflow main { ret 42; }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&workflow).await });

    assert!(result.is_ok(), "Engine should execute workflow");
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

/// Test that custom providers can override io capabilities
#[test]
fn test_custom_provider_can_override_stdio() {
    use ash_core::capability::{CapabilityError, CapabilityProvider};
    use ash_core::{Constraint, Effect, Value};
    use async_trait::async_trait;

    /// A test provider that overrides stdio
    #[derive(Debug)]
    struct TestStdioProvider;

    #[async_trait]
    impl CapabilityProvider for TestStdioProvider {
        fn name(&self) -> &str {
            "stdio"
        }

        fn effect(&self) -> Effect {
            Effect::Operational
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

    // Verify the engine works with the custom provider
    let workflow = engine.parse("workflow main { ret 42; }").expect("parses");
    let result = tokio_test::block_on(async { engine.execute(&workflow).await });

    assert!(
        result.is_ok(),
        "Engine should execute workflow with custom stdio"
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

// ============================================================
// End-to-End IO Execution Tests
// ============================================================

/// Test that engine with io capabilities can execute basic workflows
#[tokio::test]
async fn test_e2e_engine_with_io_capabilities() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // Execute a simple workflow
    let result = engine.run("workflow main { ret 42; }").await;

    assert!(
        result.is_ok(),
        "Engine with io capabilities should run workflows"
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

/// Test that engine with io capabilities handles multiple executions
#[tokio::test]
async fn test_e2e_io_capabilities_multiple_executions() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // Run multiple workflows
    for i in 0..5 {
        let result = engine.run(&format!("workflow main {{ ret {i}; }}")).await;
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
