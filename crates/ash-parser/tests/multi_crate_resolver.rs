//! Multi-crate resolver integration tests
//!
//! These tests verify that the ModuleResolver can:
//! - Parse crate root metadata and resolve dependencies
//! - Load multiple crates into a single crate-aware graph
//! - Detect duplicate aliases, duplicate crate names, and dependency cycles

use ash_parser::resolver::{Fs, ModuleResolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Mock file system for testing.
struct MockFs {
    files: HashMap<PathBuf, String>,
}

impl MockFs {
    /// Create an empty mock file system.
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Add a file to the mock file system (builder pattern).
    fn with_file(mut self, path: impl AsRef<Path>, content: impl Into<String>) -> Self {
        self.files
            .insert(path.as_ref().to_path_buf(), content.into());
        self
    }
}

impl Fs for MockFs {
    fn read_file(&self, path: &Path) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }
}

// ========================================================================
// Basic Dependency Loading Tests
// ========================================================================

#[test]
fn test_resolve_root_with_one_dependency_crate() {
    // Test: Main crate depends on a utility crate
    let fs = MockFs::new()
        .with_file(
            "main/main.ash",
            r#"crate main_app;

dependency util from "../util/util.ash";

workflow Main {}
"#,
        )
        .with_file(
            "util/util.ash",
            r#"crate util_lib;

capability Helper: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("main/main.ash").unwrap();

    // Should have 2 crates
    assert_eq!(graph.crates.len(), 2);

    // Check that the main crate has the dependency registered
    let root_id = graph.root.unwrap();
    let main_crate_id = graph.crate_id_for_module(root_id).unwrap();
    let main_crate = graph.get_crate(main_crate_id).unwrap();
    assert_eq!(main_crate.name, "main_app");
    assert_eq!(main_crate.dependencies.len(), 1);
    assert!(main_crate.dependencies.contains_key("util"));

    // Check that the util crate exists
    let util_crate_id = main_crate.dependencies.get("util").copied().unwrap();
    let util_crate = graph.get_crate(util_crate_id).unwrap();
    assert_eq!(util_crate.name, "util_lib");
}

// ========================================================================
// Error Cases
// ========================================================================

#[test]
fn test_reject_duplicate_dependency_alias() {
    // Test: Same crate declares two dependencies with the same alias
    let fs = MockFs::new()
        .with_file(
            "main/main.ash",
            r#"crate main_app;

dependency util from "../util1/util.ash";
dependency util from "../util2/util.ash";

workflow Main {}
"#,
        )
        .with_file(
            "util1/util.ash",
            r#"crate util1_lib;

capability Helper1: observe();
"#,
        )
        .with_file(
            "util2/util.ash",
            r#"crate util2_lib;

capability Helper2: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let result = resolver.resolve_crate("main/main.ash");

    assert!(result.is_err(), "Should reject duplicate dependency alias");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("duplicate") || err_str.contains("alias"),
        "Error should mention duplicate alias: {}",
        err_str
    );
}

#[test]
fn test_reject_duplicate_crate_name() {
    // Test: Two crates with the same name should be rejected
    let fs = MockFs::new()
        .with_file(
            "main/main.ash",
            r#"crate my_app;

dependency util from "../util/util.ash";

workflow Main {}
"#,
        )
        .with_file(
            "util/util.ash",
            r#"crate my_app;

capability Helper: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let result = resolver.resolve_crate("main/main.ash");

    assert!(result.is_err(), "Should reject duplicate crate name");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("duplicate") || err_str.contains("crate"),
        "Error should mention duplicate crate: {}",
        err_str
    );
}

#[test]
fn test_detect_dependency_cycle() {
    // Test: A depends on B, B depends on A
    let fs = MockFs::new()
        .with_file(
            "crate_a/main.ash",
            r#"crate crate_a;

dependency b from "../crate_b/main.ash";

workflow Main {}
"#,
        )
        .with_file(
            "crate_b/main.ash",
            r#"crate crate_b;

dependency a from "../crate_a/main.ash";

capability Helper: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let result = resolver.resolve_crate("crate_a/main.ash");

    assert!(result.is_err(), "Should detect dependency cycle");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("cycle") || err_str.contains("circular"),
        "Error should mention cycle: {}",
        err_str
    );
}

#[test]
fn test_missing_dependency_root_file_errors() {
    // Test: Dependency path doesn't exist
    let fs = MockFs::new().with_file(
        "main/main.ash",
        r#"crate main_app;

dependency util from "../util/nonexistent.ash";

workflow Main {}
"#,
    );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let result = resolver.resolve_crate("main/main.ash");

    assert!(
        result.is_err(),
        "Should error when dependency root file doesn't exist"
    );
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("not found") || err_str.contains("dependency"),
        "Error should mention not found or dependency: {}",
        err_str
    );
}

// ========================================================================
// Complex Scenarios
// ========================================================================

#[test]
fn test_resolve_with_multiple_dependencies() {
    // Test: Main crate depends on multiple other crates
    let fs = MockFs::new()
        .with_file(
            "main/main.ash",
            r#"crate main_app;

dependency util from "../util/util.ash";
dependency core from "../core/core.ash";
dependency config from "../config/config.ash";

workflow Main {}
"#,
        )
        .with_file(
            "util/util.ash",
            r#"crate util_lib;

capability Helper: observe();
"#,
        )
        .with_file(
            "core/core.ash",
            r#"crate core_lib;

capability CoreCap: observe();
"#,
        )
        .with_file(
            "config/config.ash",
            r#"crate config_lib;

capability ConfigCap: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("main/main.ash").unwrap();

    // Should have 4 crates
    assert_eq!(graph.crates.len(), 4);

    // Check that main has all dependencies registered
    let root_id = graph.root.unwrap();
    let main_crate_id = graph.crate_id_for_module(root_id).unwrap();
    let main_crate = graph.get_crate(main_crate_id).unwrap();
    assert_eq!(main_crate.dependencies.len(), 3);
    assert!(main_crate.dependencies.contains_key("util"));
    assert!(main_crate.dependencies.contains_key("core"));
    assert!(main_crate.dependencies.contains_key("config"));
}

#[test]
fn test_resolve_transitive_dependencies() {
    // Test: A depends on B, B depends on C (transitive loading)
    let fs = MockFs::new()
        .with_file(
            "crate_a/main.ash",
            r#"crate crate_a;

dependency b from "../crate_b/main.ash";

workflow Main {}
"#,
        )
        .with_file(
            "crate_b/main.ash",
            r#"crate crate_b;

dependency c from "../crate_c/main.ash";

capability BCap: observe();
"#,
        )
        .with_file(
            "crate_c/main.ash",
            r#"crate crate_c;

capability CCap: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("crate_a/main.ash").unwrap();

    // Should have 3 crates
    assert_eq!(graph.crates.len(), 3);

    // Verify all crate names
    let crate_names: Vec<_> = graph.crates.values().map(|c| c.name.clone()).collect();
    assert!(crate_names.contains(&"crate_a".to_string()));
    assert!(crate_names.contains(&"crate_b".to_string()));
    assert!(crate_names.contains(&"crate_c".to_string()));
}

#[test]
fn test_shared_dependency_not_duplicated() {
    // Test: A depends on B and C, both B and C depend on D
    // D should only be loaded once
    let fs = MockFs::new()
        .with_file(
            "crate_a/main.ash",
            r#"crate crate_a;

dependency b from "../crate_b/main.ash";
dependency c from "../crate_c/main.ash";

workflow Main {}
"#,
        )
        .with_file(
            "crate_b/main.ash",
            r#"crate crate_b;

dependency d from "../crate_d/main.ash";

capability BCap: observe();
"#,
        )
        .with_file(
            "crate_c/main.ash",
            r#"crate crate_c;

dependency d from "../crate_d/main.ash";

capability CCap: observe();
"#,
        )
        .with_file(
            "crate_d/main.ash",
            r#"crate crate_d;

capability DCap: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("crate_a/main.ash").unwrap();

    // Should have 4 crates (not 5, since D is shared)
    assert_eq!(graph.crates.len(), 4);

    // Check that both B and C point to the same D crate
    let crate_b = graph.crates.values().find(|c| c.name == "crate_b").unwrap();
    let crate_c = graph.crates.values().find(|c| c.name == "crate_c").unwrap();

    let d_from_b = crate_b.dependencies.get("d").copied().unwrap();
    let d_from_c = crate_c.dependencies.get("d").copied().unwrap();

    assert_eq!(
        d_from_b, d_from_c,
        "B and C should reference the same D crate"
    );
}

#[test]
fn test_crate_without_dependencies() {
    // Test: Crate with no dependencies should still work
    let fs = MockFs::new().with_file(
        "main/main.ash",
        r#"crate standalone;

workflow Main {}
"#,
    );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("main/main.ash").unwrap();

    // Should have 1 crate
    assert_eq!(graph.crates.len(), 1);

    let root_id = graph.root.unwrap();
    let crate_id = graph.crate_id_for_module(root_id).unwrap();
    let crate_info = graph.get_crate(crate_id).unwrap();
    assert_eq!(crate_info.name, "standalone");
    assert!(crate_info.dependencies.is_empty());
}

#[test]
fn test_dependency_with_modules() {
    // Test: Dependency crate has its own modules
    let fs = MockFs::new()
        .with_file(
            "main/main.ash",
            r#"crate main_app;

dependency util from "../util/util.ash";

workflow Main {}
"#,
        )
        .with_file(
            "util/util.ash",
            r#"crate util_lib;

mod helpers;

capability MainUtil: observe();
"#,
        )
        .with_file(
            "util/helpers.ash",
            r#"capability Helper: observe();
"#,
        );
    let resolver = ModuleResolver::with_fs(Box::new(fs));

    let graph = resolver.resolve_crate("main/main.ash").unwrap();

    // Should have 2 crates
    assert_eq!(graph.crates.len(), 2);

    // Should have 3 modules (main, util root, util/helpers)
    assert_eq!(graph.nodes.len(), 3);
}
