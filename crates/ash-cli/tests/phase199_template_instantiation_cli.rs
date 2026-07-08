//! TASK-1947: template instantiation CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use std::fs;

fn ash() -> Command {
    Command::cargo_bin("ash").expect("ash binary exists")
}

const TARGET_ENTRY: &str =
    "use runtime::RuntimeError;\n\nfn main() -> Result<(), RuntimeError> { Ok { value: {} } }\n";
const TARGET_ENTRY_WITH_APP_NAME: &str = "-- app {{app_name}}\nuse runtime::RuntimeError;\n\nfn main() -> Result<(), RuntimeError> { Ok { value: {} } }\n";
const TARGET_ENTRY_WITH_TYPE_ERROR: &str = "use runtime::RuntimeError;\n\nfn main() -> Result<(), RuntimeError> { missing_generated_symbol }\n";

fn manifest_json(content: &str) -> String {
    json!({
        "schema_version": "ash-template-v1",
        "id": "cli-tool",
        "version": "0.1.0",
        "description": "CLI tool template",
        "required_profiles": ["application-default"],
        "providers": [{
            "profile": "application-default",
            "provider": "logging",
            "operations": ["info"]
        }],
        "resources": ["stdout"],
        "evidence_expectations": ["ash check src/main.ash"],
        "parameters": [{
            "name": "app_name",
            "required": true,
            "default": null
        }],
        "files": [{
            "path": "src/main.ash",
            "content": content
        }],
        "generated_checks": [{
            "command": "ash check src/main.ash",
            "file": "src/main.ash"
        }]
    })
    .to_string()
}

#[test]
fn template_instantiate_writes_files_and_runs_checks() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest = dir.path().join("template.json");
    let out = dir.path().join("app");
    fs::write(&manifest, manifest_json(TARGET_ENTRY_WITH_APP_NAME)).expect("write manifest");

    ash()
        .args(["template", "instantiate"])
        .arg("--manifest")
        .arg(&manifest)
        .arg("--out")
        .arg(&out)
        .arg("--param")
        .arg("app_name=demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.ash"));

    let generated = fs::read_to_string(out.join("src/main.ash")).expect("read generated file");
    assert!(generated.contains("-- app demo"));
}

#[test]
fn template_instantiate_refuses_overwrite_by_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest = dir.path().join("template.json");
    let out = dir.path().join("app");
    fs::create_dir_all(out.join("src")).expect("mkdir");
    fs::write(out.join("src/main.ash"), TARGET_ENTRY).expect("existing file");
    fs::write(&manifest, manifest_json(TARGET_ENTRY)).expect("write manifest");

    ash()
        .args(["template", "instantiate"])
        .arg("--manifest")
        .arg(&manifest)
        .arg("--out")
        .arg(&out)
        .arg("--param")
        .arg("app_name=demo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
}

#[test]
fn template_instantiate_requires_declared_parameters() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest = dir.path().join("template.json");
    let out = dir.path().join("app");
    fs::write(&manifest, manifest_json(TARGET_ENTRY)).expect("write manifest");

    ash()
        .args(["template", "instantiate"])
        .arg("--manifest")
        .arg(&manifest)
        .arg("--out")
        .arg(&out)
        .assert()
        .failure()
        .stderr(predicate::str::contains("required template parameter"));
}

#[test]
fn template_instantiate_reports_generated_check_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest = dir.path().join("template.json");
    let out = dir.path().join("app");
    fs::write(&manifest, manifest_json(TARGET_ENTRY_WITH_TYPE_ERROR)).expect("write manifest");

    ash()
        .args(["template", "instantiate"])
        .arg("--manifest")
        .arg(&manifest)
        .arg("--out")
        .arg(&out)
        .arg("--param")
        .arg("app_name=demo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("type error"));
}
