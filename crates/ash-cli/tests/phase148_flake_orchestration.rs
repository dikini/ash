use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_ash_test(args: &[&str]) -> (std::process::ExitStatus, serde_json::Value, String) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("ash test fixture should launch");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "fixture did not emit valid JSON: {error}\nstatus={}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (
        output.status,
        json,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn retries_classify_pass_after_failure_as_flaky() {
    let (status, output, stderr) = run_ash_test(&[
        "test",
        "fixtures/phase148-flakes",
        "--retries",
        "2",
        "--format",
        "json",
    ]);
    assert!(status.success(), "ash test failed: {stderr}");
    assert_eq!(output["success"], true);
    assert_eq!(output["flake_summary"]["schema_version"], "ash-flake-v1.0");
    assert_eq!(output["flake_summary"]["flaky"], 1);

    let flaky = output["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|test| test["name"] == "flaky_once")
        .expect("flaky_once row should exist");
    assert_eq!(flaky["outcome"], "pass");
    assert_eq!(flaky["flake"]["status"], "flaky");
    assert_eq!(flaky["attempts"].as_array().unwrap().len(), 2);
    assert_eq!(flaky["attempts"][0]["outcome"], "fail");
    assert_eq!(flaky["attempts"][1]["outcome"], "pass");
}

#[test]
fn quarantine_metadata_keeps_failing_test_visible_but_non_failing() {
    let (status, output, stderr) =
        run_ash_test(&["test", "fixtures/phase148-quarantine", "--format", "json"]);
    assert!(status.success(), "ash test failed: {stderr}");
    assert_eq!(output["success"], true);
    assert_eq!(output["skipped"], 1);
    let row = &output["tests"][0];
    assert_eq!(row["name"], "quarantined_failure");
    assert_eq!(row["outcome"], "skip");
    assert_eq!(row["quarantine"]["status"], "quarantined");
    assert_eq!(row["quarantine"]["original_outcome"], "fail");
    assert!(
        row["quarantine"]["reason"]
            .as_str()
            .unwrap()
            .contains("known flaky")
    );
}

#[test]
fn malformed_quarantine_metadata_fails_closed() {
    let (status, output, _stderr) = run_ash_test(&[
        "test",
        "fixtures/phase148-quarantine-malformed",
        "--format",
        "json",
    ]);
    assert!(!status.success(), "malformed quarantine must fail closed");
    assert_eq!(output["success"], false);
    assert_eq!(output["tests"][0]["outcome"], "error");
    assert!(
        output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("malformed quarantine metadata")
    );
}

#[test]
fn local_shard_execution_reports_deterministic_plan() {
    let (status, output, stderr) = run_ash_test(&[
        "test",
        "fixtures/phase148-shards",
        "--shard",
        "1/2",
        "--format",
        "json",
    ]);
    assert!(status.success(), "ash test failed: {stderr}");
    assert_eq!(output["shard"]["schema_version"], "ash-shard-v1.0");
    assert_eq!(output["shard"]["index"], 1);
    assert_eq!(output["shard"]["total"], 2);
    assert_eq!(output["shard"]["selected_count"], 2);
    assert_eq!(output["shard"]["skipped_count"], 2);
    let names: Vec<_> = output["tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["shard_a", "shard_c"]);
    assert!(
        output["tests"]
            .as_array()
            .unwrap()
            .iter()
            .all(|test| test["shard"]["index"] == 1)
    );
}

#[test]
fn merge_results_combines_shards_and_rejects_missing_or_duplicate_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let shard1 = dir.path().join("shard-1.json");
    let shard2 = dir.path().join("shard-2.json");
    write_shard(&shard1, "1/2");
    write_shard(&shard2, "2/2");

    let (status, output, stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        shard1.to_str().unwrap(),
        shard2.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(status.success(), "merge failed: {stderr}");
    assert_eq!(output["merge"]["schema_version"], "ash-merge-v1.0");
    assert_eq!(output["merge"]["shards"], 2);
    assert_eq!(output["total"], 4);
    assert_eq!(output["passed"], 4);

    let (missing_status, missing_output, _stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        shard1.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(!missing_status.success(), "missing shard must fail closed");
    assert!(
        missing_output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("missing shard result")
    );

    let (dup_status, dup_output, _stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        shard1.to_str().unwrap(),
        shard1.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(!dup_status.success(), "duplicate shard must fail closed");
    assert!(
        dup_output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("duplicate shard result")
    );

    let invalid = dir.path().join("invalid-shard.json");
    std::fs::write(
        &invalid,
        r#"{"schema_version":"ash-test-v1.0","success":true,"duration_ms":0,"shard":{"schema_version":"ash-shard-v1.0","index":0,"total":0},"tests":[]}"#,
    )
    .unwrap();
    let (invalid_status, invalid_output, _stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        invalid.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        !invalid_status.success(),
        "invalid shard range must fail closed"
    );
    assert!(
        invalid_output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("invalid shard range")
    );

    let missing_tests = dir.path().join("missing-tests.json");
    std::fs::write(
        &missing_tests,
        r#"{"schema_version":"ash-test-v1.0","success":true,"duration_ms":0,"shard":{"schema_version":"ash-shard-v1.0","index":1,"total":1}}"#,
    )
    .unwrap();
    let (missing_tests_status, missing_tests_output, _stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        missing_tests.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        !missing_tests_status.success(),
        "missing tests array must fail closed"
    );
    assert!(
        missing_tests_output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("missing tests array")
    );

    let duplicate_test_a = dir.path().join("duplicate-test-a.json");
    let duplicate_test_b = dir.path().join("duplicate-test-b.json");
    std::fs::write(
        &duplicate_test_a,
        r#"{"schema_version":"ash-test-v1.0","success":true,"duration_ms":0,"shard":{"schema_version":"ash-shard-v1.0","index":1,"total":2},"tests":[{"name":"same","path":"tests/same.ash","outcome":"pass","source":"authored","kind":"unit","duration_ms":0,"tags":[]}] }"#,
    )
    .unwrap();
    std::fs::write(
        &duplicate_test_b,
        r#"{"schema_version":"ash-test-v1.0","success":true,"duration_ms":0,"shard":{"schema_version":"ash-shard-v1.0","index":2,"total":2},"tests":[{"name":"same","path":"tests/same.ash","outcome":"pass","source":"authored","kind":"unit","duration_ms":0,"tags":[]}] }"#,
    )
    .unwrap();
    let (duplicate_test_status, duplicate_test_output, _stderr) = run_ash_test(&[
        "test",
        "--merge-results",
        duplicate_test_a.to_str().unwrap(),
        duplicate_test_b.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        !duplicate_test_status.success(),
        "duplicate test rows must fail closed"
    );
    assert!(
        duplicate_test_output["tests"][0]["message"]
            .as_str()
            .unwrap()
            .contains("duplicate test row")
    );
}

fn write_shard(path: &Path, shard: &str) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args([
            "test",
            "fixtures/phase148-shards",
            "--shard",
            shard,
            "--format",
            "json",
        ])
        .output()
        .expect("ash shard fixture should launch");
    assert!(
        output.status.success(),
        "shard {shard} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(path, output.stdout).expect("write shard json");
}
