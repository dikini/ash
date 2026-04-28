//! TASK-743 CLI configuration-surface conformance tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under crates/ash-cli")
        .to_path_buf()
}

fn ash_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ash"))
}

fn module_file(name: &str) -> PathBuf {
    let path = repo_root()
        .join("target")
        .join("task-743-fixtures")
        .join(name);
    fs::create_dir_all(path.parent().expect("fixture has parent")).expect("fixture dir exists");
    fs::write(
        &path,
        r#"
pub capability interface KeyValue:
    observe get(key: String) returns String
  | execute put(key: String, value: String) returns Unit;

pub resource type WorkflowKV {
    namespace: String
}

pub capability impl MockInternalKV for KeyValue
    requires resource store: WorkflowKV
    requires config fixture: String
{
    observe get(key: String) returns String { "mock" }
    execute put(key: String, value: String) returns Unit { () }
}
"#,
    )
    .expect("fixture writes");
    path
}

#[test]
fn run_dry_run_accepts_known_capability_configuration() {
    let path = module_file("known-capability.ash");
    let output = Command::new(ash_bin())
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg("--dry-run")
        .arg("--capability-impl")
        .arg("kv=MockInternalKV")
        .arg(&path)
        .output()
        .expect("ash command runs");

    assert!(
        output.status.success(),
        "stderr:\n{}\nstdout:\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn run_rejects_malformed_capability_impl_flag() {
    let path = module_file("malformed-capability.ash");
    let output = Command::new(ash_bin())
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg("--dry-run")
        .arg("--capability-impl")
        .arg("missing_equals")
        .arg(&path)
        .output()
        .expect("ash command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains("--capability-impl"), "stderr: {stderr}");
    assert!(stderr.contains("NAME=NAME"), "stderr: {stderr}");
}

#[test]
fn run_rejects_unknown_implementation_name() {
    let path = module_file("unknown-implementation.ash");
    let output = Command::new(ash_bin())
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg("--dry-run")
        .arg("--capability-impl")
        .arg("kv=MissingKV")
        .arg(&path)
        .output()
        .expect("ash command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(
        stderr.contains("unknown capability implementation 'MissingKV'"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_unknown_resource_initializer_target() {
    let path = module_file("unknown-resource.ash");
    let output = Command::new(ash_bin())
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg("--dry-run")
        .arg("--resource-init")
        .arg("MissingResource=memory")
        .arg(&path)
        .output()
        .expect("ash command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(
        stderr.contains("unknown resource initializer target 'MissingResource'"),
        "stderr: {stderr}"
    );
}
