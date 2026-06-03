//! Integration tests for the `ash test` command (Phase 76).
//!
//! TASK-509 through TASK-515: Smoke tests for test runner behavior.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

fn ash() -> Command {
    Command::cargo_bin("ash").unwrap()
}

fn make_test_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let test_dir = dir.path().join("tests/ash/unit");
    fs::create_dir_all(&test_dir).unwrap();
    dir
}

fn write_authored_test(dir: &tempfile::TempDir, kind: &str, name: &str, source: &str) {
    let test_dir = dir.path().join("tests/ash").join(kind);
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join(format!("{name}.ash")), source).unwrap();
}

fn parse_json_output(assert: &assert_cmd::assert::Assert) -> Value {
    serde_json::from_slice(&assert.get_output().stdout).unwrap()
}

// ---------------------------------------------------------------------------
// TASK-509: Runner substrate
// ---------------------------------------------------------------------------

#[test]
fn test_help_output() {
    ash()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run Ash tests"));
}

#[test]
fn test_no_tests_found_is_success() {
    let dir = make_test_dir();
    ash().arg("test").arg(dir.path()).assert().success();
}

#[test]
fn test_json_format_empty_dir() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("ash-test-v1.0"));
}

// ---------------------------------------------------------------------------
// TASK-510: Test isolation and panic capture
// ---------------------------------------------------------------------------

#[test]
fn test_parse_error_does_not_crash_runner() {
    let dir = make_test_dir();
    let bad_file = dir.path().join("tests/ash/unit/bad.ash");
    fs::write(&bad_file, "!!! invalid syntax !!!\n").unwrap();

    // Runner should not crash; it should report the error
    ash().arg("test").arg(dir.path()).assert().code(1); // Non-zero exit (test failed)
}

// ---------------------------------------------------------------------------
// TASK-511: Test library surface
// ---------------------------------------------------------------------------

#[test]
fn test_library_surface_file_exists() {
    // Verify the std::test.ash file exists in the stdlib
    // Use CARGO_MANIFEST_DIR to find the workspace root
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let test_lib = workspace_root.join("std/src/test.ash");
    assert!(
        test_lib.exists(),
        "std/src/test.ash should exist at {:?}",
        test_lib
    );

    let content = fs::read_to_string(&test_lib).unwrap();
    assert!(
        content.contains("assert_true"),
        "test library should define assert_true"
    );
    assert!(
        content.contains("assert_false"),
        "test library should define assert_false"
    );
    assert!(content.contains("fail"), "test library should define fail");
    // Verify no @test metadata (library files shouldn't have test metadata)
    assert!(
        !content.contains("// @test"),
        "library file should not contain @test metadata"
    );
}

#[test]
fn test_library_exports_in_lib_ash() {
    // Verify std/src/lib.ash exports the test library
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let lib_file = workspace_root.join("std/src/lib.ash");
    let content = fs::read_to_string(&lib_file).unwrap();

    assert!(
        content.contains("pub use test::"),
        "lib.ash should export test module"
    );
    assert!(
        content.contains("assert_true"),
        "lib.ash should export assert_true"
    );
}

// ---------------------------------------------------------------------------
// TASK-512: Authored test metadata
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_parsing_via_discovery() {
    let dir = make_test_dir();
    let test_file = dir.path().join("tests/ash/unit/meta_test.ash");
    fs::write(
        &test_file,
        "-- @test name: my_special_test\n-- @test tags: smoke\nworkflow main { done; }",
    )
    .unwrap();

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let json = parse_json_output(&assert);
    let tests = json["tests"].as_array().unwrap();
    assert!(tests.iter().any(|test| test["name"] == "my_special_test"));
}

// ---------------------------------------------------------------------------
// TASK-513: Synthesized tests (opt-in)
// ---------------------------------------------------------------------------

#[test]
fn test_synthesized_tests_not_run_by_default() {
    let dir = make_test_dir();
    let result = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // JSON output should not contain "synthesized" when not requested
    let output = std::str::from_utf8(&result.get_output().stdout).unwrap();
    assert!(!output.contains("synthesized:contract"));
}

#[test]
fn test_include_synthesized_flag_is_accepted() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--include-synthesized")
        .arg("contracts,policies,obligations")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_only_synthesized_flag_is_accepted() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_include_synthesized_contracts_only() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--include-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_include_synthesized_policies_only() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "policy_target",
        "policy MyPolicy { allow => true }\n",
    );
    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("policies")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let output = parse_json_output(&assert);
    let tests = output["tests"].as_array().unwrap();
    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:policy")
    );
    assert!(
        tests
            .iter()
            .filter(|test| test["source"] == "synthesized:policy")
            .all(|test| test["outcome"] == "skip")
    );
}

#[test]
fn test_include_synthesized_obligations_only() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--include-synthesized")
        .arg("obligations")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// TASK-514: Property and small-world
// ---------------------------------------------------------------------------

#[test]
fn test_seed_flag_is_accepted() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--seed")
        .arg("42")
        .assert()
        .success();
}

#[test]
fn test_max_cases_flag_is_accepted() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--max-cases")
        .arg("50")
        .assert()
        .success();
}

#[test]
fn test_max_worlds_flag_is_accepted() {
    let dir = make_test_dir();
    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--max-worlds")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn test_direct_property_directory_runs_tests() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("tests/ash/property")).unwrap();
    fs::write(
        dir.path().join("tests/ash/property/prop.ash"),
        "-- @test name: prop_custom\n-- @test kind: property\nworkflow main() -> Bool { ret false }",
    )
    .unwrap();

    let assert = ash()
        .arg("test")
        .arg(dir.path().join("tests/ash/property"))
        .arg("--seed")
        .arg("42")
        .arg("--max-cases")
        .arg("7")
        .arg("--format")
        .arg("json")
        .assert()
        .code(1);
    let json = parse_json_output(&assert);
    let tests = json["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0]["name"], "prop_custom");
    assert_eq!(tests[0]["kind"], "property");
    assert_eq!(tests[0]["seed"], 42);
    assert_eq!(tests[0]["failing_case"], 1);
}

#[test]
fn test_authored_test_can_use_minimal_test_library() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("tests/ash/unit")).unwrap();
    fs::write(
        dir.path().join("tests/ash/unit/use_test_lib.ash"),
        "use test::assert_true\nworkflow main() -> Bool { ret assert_true(true) }",
    )
    .unwrap();

    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("use_test_lib"));
}

#[test]
fn property_kind_file_executes_successfully() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "property",
        "property_pass",
        "workflow main { ret 0 }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path().join("tests/ash/property/property_pass.ash"))
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());

    assert_eq!(output["success"], Value::Bool(true));
    assert_eq!(
        output["tests"][0]["kind"],
        Value::String("property".to_string())
    );
    assert_eq!(
        output["tests"][0]["outcome"],
        Value::String("pass".to_string())
    );
    assert_eq!(output["tests"][0]["seed"], Value::from(42_u64));
}

#[test]
fn smallworld_kind_file_executes_successfully() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "smallworld",
        "smallworld_pass",
        "workflow main { ret 0 }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path().join("tests/ash/smallworld/smallworld_pass.ash"))
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());

    assert_eq!(output["success"], Value::Bool(true));
    assert_eq!(
        output["tests"][0]["kind"],
        Value::String("smallworld".to_string())
    );
    assert_eq!(
        output["tests"][0]["outcome"],
        Value::String("pass".to_string())
    );
    assert_eq!(output["tests"][0]["world_index"], Value::Null);
    assert!(
        output["tests"][0].get("repro_artifact").is_none(),
        "authored smallworld compatibility path must not claim metadata world snapshots"
    );
}

#[test]
fn unit_tests_do_not_emit_property_or_smallworld_metadata() {
    let dir = make_test_dir();
    write_authored_test(&dir, "unit", "unit_pass", "workflow main { ret 0 }\n");

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--max-cases")
        .arg("50")
        .arg("--max-worlds")
        .arg("7")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let test = &output["tests"][0];
    assert_eq!(test["kind"], Value::String("unit".to_string()));
    assert_eq!(test["failing_case"], Value::Null);
    assert_eq!(test["world_index"], Value::Null);
}

#[test]
fn only_synthesized_keeps_authored_and_selected_sources_distinct() {
    let dir = make_test_dir();
    write_authored_test(&dir, "unit", "authored_pass", "workflow main { ret 0 }\n");
    write_authored_test(
        &dir,
        "unit",
        "synthesized_targets",
        "workflow contract_target requires x > 0 ensures result > 0 { ret 0 }\npolicy ReviewPolicy { allow => true }\nworkflow obligation_target { oblige Ticket check Ticket ret 0 }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts,obligations")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(!tests.is_empty());
    assert!(tests.iter().all(|test| {
        let source = test["source"].as_str().unwrap();
        source == "synthesized:contract" || source == "synthesized:obligation"
    }));
    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:contract")
    );
    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:obligation")
    );
    assert!(!tests.iter().any(|test| test["source"] == "authored"));
}
