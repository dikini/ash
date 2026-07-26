//! TASK-2007: Test-runner Core terminology compatibility and clarity.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;

fn ash() -> Command {
    Command::cargo_bin("ash").expect("ash binary should be available to integration tests")
}

#[test]
fn contract_repro_metadata_preserves_substrate_and_names_ash_core_expr_representation() {
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
    let postcondition = report["tests"]
        .as_array()
        .and_then(|tests| {
            tests.iter().find(|test| {
                test["name"]
                    .as_str()
                    .is_some_and(|name| name.contains("/identity/ensures"))
            })
        })
        .expect("contract postcondition result should be present");
    let execution = &postcondition["repro_artifact"]["oracle_snapshot"]["target_execution"];

    assert_eq!(
        execution["substrate"], "ash_interp_core_expr",
        "the existing substrate field is a compatibility contract"
    );
    assert_eq!(
        execution["representation"], "ash_core::Expr",
        "public metadata must distinguish the legacy ash_core::Expr substrate from Core Ash or CPS"
    );
}
