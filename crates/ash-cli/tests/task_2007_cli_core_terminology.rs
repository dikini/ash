//! TASK-2007: Test-runner raw-source contract deferral and JSON clarity.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;

fn ash() -> Command {
    Command::cargo_bin("ash").expect("ash binary should be available to integration tests")
}

#[test]
fn raw_source_contract_with_ensures_defers_without_legacy_execution_metadata() {
    let root = tempfile::tempdir().expect("temporary test root should be created");
    let tests = root.path().join("tests/ash/unit");
    fs::create_dir_all(&tests).expect("test directory should be created");
    fs::write(
        tests.join("identity_contract.ash"),
        "fn identity(n: Int) -> Int\n    requires: n >= 0\n    ensures: result == n\n{\n    n\n}\n",
    )
    .expect("contract test source should be written");

    let output = ash()
        .arg("test")
        .arg(root.path())
        .args(["--only-synthesized", "contracts", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("test runner should emit JSON");
    let boundary = report["tests"]
        .as_array()
        .and_then(|tests| {
            tests.iter().find(|test| {
                test["name"]
                    .as_str()
                    .is_some_and(|name| name == "contract:identity")
            })
        })
        .expect("raw-source contract result should be present in the CLI JSON report");
    let repro = &boundary["repro_artifact"];
    let oracle = &repro["oracle_snapshot"];

    assert_eq!(
        boundary["outcome"], "skip",
        "an unlowered raw-source contract must defer rather than run locally"
    );
    assert_eq!(
        boundary["message"],
        "deferred: source identity is not in the TASK-2035 catalogue"
    );
    assert_eq!(oracle["execution_route"], "catalogue_rejection");
    assert!(
        oracle.get("target_execution").is_none()
            && !oracle.to_string().contains("ash_interp_core_expr"),
        "the raw-source deferral must not expose legacy target execution metadata"
    );
}
