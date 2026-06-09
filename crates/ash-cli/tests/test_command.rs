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
        .stdout(predicate::str::contains("Run Ash tests"))
        .stdout(predicate::str::contains("--skip-law-tests"))
        .stdout(predicate::str::contains("--skip-law-test"));
}

#[test]
fn check_help_exposes_proof_fuel_flag() {
    ash()
        .arg("check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--proof-fuel"));
}

#[test]
fn check_proof_fuel_flag_accepts_explicit_value() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("proof-fuel.ash");
    fs::write(&file, "workflow main { ret 0 }\n").unwrap();

    ash()
        .arg("check")
        .arg("--proof-fuel")
        .arg("0")
        .arg(&file)
        .assert()
        .success();
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
fn only_synthesized_contract_source_uses_live_checked_snapshot_when_available() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "checked_contract_target",
        "workflow checked_contract_target requires: true { ret 0 }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:contract"),
        "contract synthesis should emit a selected structured row: {output:#}"
    );
    assert!(
        tests.iter().all(|test| test["outcome"] == "skip"),
        "TASK-1012 must not invent executable contract passes from ordinary source: {output:#}"
    );
    assert!(
        tests.iter().any(|test| {
            let repro = &test["repro_artifact"];
            repro["check_summary_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("checked:"))
                && repro["check_summary_id"] != "raw-source-fallback:no-lowered-summary"
                && repro["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("source-file:"))
                && repro["oracle_snapshot"]["snapshot_source"] == "live_checked_snapshot"
        }),
        "ordinary CLI source should use live checked snapshot evidence instead of raw-source fallback: {output:#}"
    );
}

#[test]
fn only_synthesized_function_contract_module_uses_live_checked_snapshot() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "checked_fn_contract_target",
        "fn bounded(n: Int) -> Int requires: n >= 0 { n }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:contract"),
        "function contract source should emit a selected synthesized row: {output:#}"
    );
    assert!(
        tests.iter().all(|test| test["outcome"] == "skip"),
        "TASK-1012 must defer function contract execution until TASK-1013: {output:#}"
    );
    assert!(
        tests.iter().any(|test| {
            let repro = &test["repro_artifact"];
            test["name"]
                .as_str()
                .is_some_and(|name| name.contains("/bounded/"))
                && repro["check_summary_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("checked:"))
                && repro["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("source-file:"))
                && repro["oracle_snapshot"]["snapshot_source"] == "live_checked_snapshot"
        }),
        "ordinary function contract modules should use parsed checked snapshot evidence instead of raw-source fallback: {output:#}"
    );
}

#[test]
fn only_synthesized_contract_postcondition_executes_supported_pure_function_metadata() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "checked_fn_postcondition_target",
        "fn identity(n: Int) -> Int\n    requires: n >= 0\n    ensures: result == n\n{\n    n\n}\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(
        tests.iter().any(|test| {
            test["source"] == "synthesized:contract"
                && test["outcome"] == "pass"
                && test["name"]
                    .as_str()
                    .is_some_and(|name| name.contains("/identity/ensures"))
                && test["repro_artifact"]["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("source-file:"))
                && test["repro_artifact"]["check_summary_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("checked:"))
                && test["repro_artifact"]["generated_input_snapshot"]["bindings"]["n"].is_i64()
                && test["repro_artifact"]["oracle_snapshot"]["target_output"].is_i64()
                && test["repro_artifact"]["oracle_snapshot"]["ensures"] == "result == n"
                && test["repro_artifact"]["oracle_snapshot"]["target_execution"]["substrate"]
                    == "ash_interp_core_expr"
        }),
        "supported pure function postconditions should execute with input/output repro context: {output:#}"
    );
    assert!(
        tests.iter().all(|test| {
            test["repro_artifact"]["check_summary_id"] != "raw-source-fallback:no-lowered-summary"
        }),
        "supported checked metadata must not use raw-source fallback rows: {output:#}"
    );
}

#[test]
fn only_synthesized_contract_fail_fast_and_timeout_apply_to_json_synthesized_rows() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "fail_fast_synthesized_contracts",
        "fn bad(n: Int) -> Int\n    requires: n >= 0\n    ensures: result == n\n{\n    1\n}\nfn good(n: Int) -> Int\n    requires: n >= 0\n    ensures: result == n\n{\n    n\n}\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--fail-fast")
        .arg("--timeout")
        .arg("30000")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.code(1));
    let tests = output["tests"].as_array().unwrap();

    assert_eq!(output["failed"], Value::from(1));
    assert_eq!(
        tests.len(),
        3,
        "--fail-fast should stop synthesized JSON output after the first failing synthesized case: {output:#}"
    );
    assert!(
        tests
            .iter()
            .all(|test| test["source"] == "synthesized:contract"),
        "--only-synthesized should keep authored rows out of the fail-fast JSON result: {output:#}"
    );
    assert_eq!(tests[2]["outcome"], Value::String("fail".to_string()));
    assert!(
        tests[2]["message"]
            .as_str()
            .is_some_and(|message| message.contains("postcondition failed")),
        "the stopped row should be the synthesized postcondition failure: {output:#}"
    );
}

#[test]
fn only_synthesized_contract_human_output_accepts_generation_and_world_controls() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "human_synthesized_contracts",
        "fn identity(n: Int) -> Int\n    requires: n >= 0\n    ensures: result == n\n{\n    n\n}\n",
    );

    ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--seed")
        .arg("123")
        .arg("--max-cases")
        .arg("2")
        .arg("--max-worlds")
        .arg("2")
        .arg("--timeout")
        .arg("30000")
        .arg("--format")
        .arg("human")
        .assert()
        .success()
        .stdout(predicate::str::contains("synthesized/contract/identity"))
        .stdout(predicate::str::contains("[synthesized:contract]"))
        .stdout(predicate::str::contains("PASSED 3 tests"));
}

#[test]
fn only_synthesized_unsupported_live_snapshot_metadata_defers_without_pass_rows() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "unsupported_contract_target",
        "workflow unsupported_contract_target { ret 0 }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(
        tests
            .iter()
            .any(|test| test["source"] == "synthesized:contract"),
        "contract synthesis should emit explicit unsupported metadata rows: {output:#}"
    );
    assert!(
        tests.iter().all(|test| test["outcome"] == "skip"),
        "unsupported live metadata must defer instead of passing: {output:#}"
    );
    assert!(
        tests.iter().any(|test| {
            test["message"]
                .as_str()
                .is_some_and(|message| message.contains("deferred"))
                && test["repro_artifact"]["oracle_snapshot"]["snapshot_source"]
                    == "live_checked_snapshot"
        }),
        "unsupported structured rows should carry explicit deferred live-snapshot evidence: {output:#}"
    );
}

#[test]
fn raw_fallback_is_applied_per_file_in_mixed_live_snapshot_suite() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "checked_fn_contract_target",
        "fn bounded(n: Int) -> Int requires: n >= 0 { n }\n",
    );
    write_authored_test(
        &dir,
        "unit",
        "raw_fallback_only",
        "workflow raw_fallback_only requires: true { !!! }\n",
    );

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--only-synthesized")
        .arg("contracts")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert!(
        tests.iter().any(|test| {
            let repro = &test["repro_artifact"];
            test["source"] == "synthesized:contract"
                && test["outcome"] == "skip"
                && repro["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.contains("checked_fn_contract_target.ash"))
                && repro["check_summary_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("checked:"))
                && repro["oracle_snapshot"]["snapshot_source"] == "live_checked_snapshot"
        }),
        "mixed suite should retain the live checked snapshot row for the good file: {output:#}"
    );
    assert!(
        tests.iter().any(|test| {
            let repro = &test["repro_artifact"];
            test["source"] == "synthesized:contract"
                && test["outcome"] == "skip"
                && repro["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.contains("raw_fallback_only.ash"))
                && repro["check_summary_id"] == "raw-source-fallback:no-lowered-summary"
                && repro["oracle_snapshot"]["fallback"] == "raw_source_pattern"
        }),
        "mixed suite should use raw-source fallback for only the file whose live snapshot failed: {output:#}"
    );
    assert!(
        tests.iter().all(|test| test["outcome"] != "pass"),
        "raw-source fallback rows must stay deferred and never pass: {output:#}"
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

#[test]
fn only_synthesized_laws_generates_std_io_path_join_law_row() {
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std/src/io/path.ash");

    let assert = ash()
        .arg("test")
        .arg(path)
        .arg("--only-synthesized")
        .arg("laws")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    let law = tests
        .iter()
        .find(|test| {
            test["source"] == "synthesized:law"
                && test["name"].as_str().is_some_and(|name| {
                    name.starts_with("synthesized/law/join_preserves_absolute/")
                })
        })
        .expect("std/src/io/path.ash should generate a synthesized law row");

    let oracle = &law["repro_artifact"]["oracle_snapshot"];
    assert_eq!(oracle["law"], "join_preserves_absolute");
    assert_eq!(
        oracle["params"],
        serde_json::json!(["base: PathBuf", "child: String"])
    );
    assert_eq!(
        oracle["proposition"],
        "preserves_absolute_after_join(base, child)"
    );
    assert!(
        law["repro_artifact"]["replay_command"]
            .as_str()
            .is_some_and(|command| command.contains("--only-synthesized laws")),
        "law replay command should re-select synthesized law rows, got {:?}",
        law["repro_artifact"]["replay_command"]
    );
}
