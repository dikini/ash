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
        .stdout(predicate::str::contains("--include-law-tests"))
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
    fs::write(&file, "fn main() { 0 }\n").unwrap();

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
fn test_metadata_discovery_is_preserved_when_engine_admission_rejects_the_source() {
    let dir = make_test_dir();
    let test_file = dir.path().join("tests/ash/unit/meta_test.ash");
    fs::write(
        &test_file,
        "-- @test name: my_special_test\n-- @test tags: smoke\nfn main() { {}; }",
    )
    .unwrap();

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .code(1);
    let json = parse_json_output(&assert);
    let tests = json["tests"].as_array().unwrap();
    assert_eq!(json["success"], Value::Bool(false));
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0]["name"], "my_special_test");
    assert_eq!(tests[0]["kind"], "unit");
    assert_eq!(tests[0]["outcome"], "error");
    assert!(
        tests[0]["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("admission error:")),
        "the Engine admission error should remain visible: {json:#}"
    );
    assert_eq!(tests[0]["tags"], serde_json::json!(["smoke"]));
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
        "fn checked_contract_target() requires: true { 0 }\n",
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
                && repro["oracle_snapshot"]["execution_route"] == "deferred_before_execution"
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
                .is_some_and(|name| name == "synthesized/contract/bounded")
                && repro["check_summary_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("checked:"))
                && repro["source_artifact_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("source-file:"))
                && repro["oracle_snapshot"]["execution_route"] == "deferred_before_execution"
        }),
        "ordinary function contract modules should use parsed checked snapshot evidence instead of raw-source fallback: {output:#}"
    );
}

#[test]
fn only_synthesized_contract_postcondition_without_a_source_wrapper_defers() {
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
        tests.iter().all(|test| {
            test["source"] == "synthesized:contract"
                && test["outcome"] == "skip"
                && test["message"] == "deferred: source identity is not in the TASK-2035 catalogue"
                && test["repro_artifact"]["oracle_snapshot"]["execution_route"]
                    == "catalogue_rejection"
        }),
        "unlisted postcondition metadata must defer without a local oracle: {output:#}"
    );
    assert!(
        tests.iter().all(|test| {
            test["repro_artifact"]["check_summary_id"] != "raw-source-fallback:no-lowered-summary"
        }),
        "supported checked metadata must not use raw-source fallback rows: {output:#}"
    );
}

#[test]
fn only_synthesized_contract_fail_fast_and_timeout_preserve_deferred_rows() {
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
    let output = parse_json_output(&assert.success());
    let tests = output["tests"].as_array().unwrap();

    assert_eq!(output["failed"], Value::from(0));
    assert_eq!(
        tests.len(),
        2,
        "deferred synthesized rows are not fail-fast failures: {output:#}"
    );
    assert!(
        tests
            .iter()
            .all(|test| test["source"] == "synthesized:contract"),
        "--only-synthesized should keep authored rows out of the fail-fast JSON result: {output:#}"
    );
    assert!(
        tests.iter().all(|test| {
            test["outcome"] == "skip"
                && test["message"] == "deferred: source identity is not in the TASK-2035 catalogue"
        }),
        "unlisted synthesized metadata must defer rather than create a local failure: {output:#}"
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
        .stdout(predicate::str::contains("contract:identity"))
        .stdout(predicate::str::contains("[synthesized:contract]"))
        .stdout(predicate::str::contains("1 skipped"));
}

#[test]
fn only_synthesized_unsupported_live_snapshot_metadata_defers_without_pass_rows() {
    let dir = make_test_dir();
    write_authored_test(
        &dir,
        "unit",
        "unsupported_contract_target",
        "fn unsupported_contract_target() { 0 }\n",
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
                && test["repro_artifact"]["oracle_snapshot"]["execution_route"]
                    == "deferred_before_execution"
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
        "fn raw_fallback_only() requires: true { !!! }\n",
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
                && repro["oracle_snapshot"]["execution_route"] == "deferred_before_execution"
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
        "-- @test name: prop_custom\n-- @test kind: property\nfn main() -> Bool { false }",
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
fn test_authored_test_library_import_surfaces_engine_admission_error() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("tests/ash/unit")).unwrap();
    fs::write(
        dir.path().join("tests/ash/unit/use_test_lib.ash"),
        "use test::assert_true\nfn main() -> Bool { assert_true(true) }",
    )
    .unwrap();

    let assert = ash()
        .arg("test")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .code(1);
    let output = parse_json_output(&assert);

    assert_eq!(output["success"], Value::Bool(false));
    assert_eq!(output["tests"][0]["name"], "use_test_lib");
    assert_eq!(output["tests"][0]["kind"], "unit");
    assert_eq!(output["tests"][0]["outcome"], "error");
    assert!(
        output["tests"][0]["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("admission error:")),
        "the Engine admission error should remain visible: {output:#}"
    );
}

#[test]
fn property_kind_file_retains_metadata_under_engine_admission_error() {
    let dir = make_test_dir();
    write_authored_test(&dir, "property", "property_pass", "fn main() { 0 }\n");

    let assert = ash()
        .arg("test")
        .arg(dir.path().join("tests/ash/property/property_pass.ash"))
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.code(1));

    assert_eq!(output["success"], Value::Bool(false));
    assert_eq!(
        output["tests"][0]["kind"],
        Value::String("property".to_string())
    );
    assert_eq!(
        output["tests"][0]["outcome"],
        Value::String("error".to_string())
    );
    assert!(
        output["tests"][0]["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("admission error:")),
        "the Engine admission error should remain visible: {output:#}"
    );
    assert!(
        output["tests"][0]["seed"].as_u64().is_some(),
        "property test seed should be recorded as a u64: {}",
        output["tests"][0]["seed"]
    );
}

#[test]
fn smallworld_kind_file_retains_metadata_under_engine_admission_error() {
    let dir = make_test_dir();
    write_authored_test(&dir, "smallworld", "smallworld_pass", "fn main() { 0 }\n");

    let assert = ash()
        .arg("test")
        .arg(dir.path().join("tests/ash/smallworld/smallworld_pass.ash"))
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.code(1));

    assert_eq!(output["success"], Value::Bool(false));
    assert_eq!(
        output["tests"][0]["kind"],
        Value::String("smallworld".to_string())
    );
    assert_eq!(
        output["tests"][0]["outcome"],
        Value::String("error".to_string())
    );
    assert!(
        output["tests"][0]["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("admission error:")),
        "the Engine admission error should remain visible: {output:#}"
    );
    assert_eq!(output["tests"][0]["world_index"], 1);
    assert!(
        output["tests"][0].get("repro_artifact").is_none(),
        "authored smallworld compatibility path must not claim metadata world snapshots"
    );
}

#[test]
fn unit_tests_do_not_emit_property_or_smallworld_metadata() {
    let dir = make_test_dir();
    write_authored_test(&dir, "unit", "unit_pass", "fn main() { 0 }\n");

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
    let output = parse_json_output(&assert.code(1));
    let test = &output["tests"][0];
    assert_eq!(output["success"], Value::Bool(false));
    assert_eq!(test["kind"], Value::String("unit".to_string()));
    assert_eq!(test["outcome"], Value::String("error".to_string()));
    assert!(
        test["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("admission error:")),
        "the Engine admission error should remain visible: {output:#}"
    );
    assert_eq!(test["failing_case"], Value::Null);
    assert_eq!(test["world_index"], Value::Null);
}

#[test]
fn only_synthesized_keeps_authored_and_selected_sources_distinct() {
    let dir = make_test_dir();
    write_authored_test(&dir, "unit", "authored_pass", "fn main() { 0 }\n");
    write_authored_test(
        &dir,
        "unit",
        "synthesized_targets",
        "fn contract_target(x: Int) -> Int requires: x > 0 ensures: result > 0 { x }\npolicy ReviewPolicy { allow => true }\nfn obligation_target() { oblige Ticket check Ticket 0 }\n",
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
fn only_synthesized_laws_defers_std_io_path_join_law_without_a_source_wrapper() {
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
                && test["name"]
                    .as_str()
                    .is_some_and(|name| name == "synthesized/join_preserves_absolute/deferred")
        })
        .expect("std/src/io/path.ash should generate a synthesized law row");

    let oracle = &law["repro_artifact"]["oracle_snapshot"];
    assert_eq!(oracle["law"], "join_preserves_absolute");
    assert_eq!(oracle["execution_route"], "deferred_before_execution");
    assert_eq!(
        oracle["reason"],
        "deferred: law metadata has no TASK-2035 source identity"
    );
    assert!(
        law["repro_artifact"]["replay_command"]
            .as_str()
            .is_some_and(|command| command.contains("--only-synthesized laws")),
        "law replay command should re-select synthesized law rows, got {:?}",
        law["repro_artifact"]["replay_command"]
    );
}

#[test]
fn generated_algebra_laws_only_synthesized_defer_without_source_wrappers() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src/algebra/monoid.ash");

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

    let law_rows = tests
        .iter()
        .filter(|test| test["source"] == "synthesized:law")
        .collect::<Vec<_>>();
    assert!(
        !law_rows.is_empty(),
        "--only-synthesized laws should report the selected law metadata: {output:#}"
    );
    assert!(
        law_rows.iter().all(|test| {
            test["outcome"] == "skip"
                && test["message"] == "deferred: law metadata has no TASK-2035 source identity"
        }),
        "unlisted generated law metadata must defer without a local algebra evaluator: {output:#}"
    );
}

#[test]
fn generated_algebra_laws_max_cases_zero_defers_instead_of_passing() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src/algebra/monoid.ash");

    let assert = ash()
        .arg("test")
        .arg(path)
        .arg("--only-synthesized")
        .arg("laws")
        .arg("--max-cases")
        .arg("0")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let law_rows = output["tests"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|test| test["source"] == "synthesized:law")
        .collect::<Vec<_>>();

    assert!(
        !law_rows.is_empty(),
        "zero max-cases should still report selected law metadata: {output:#}"
    );
    assert!(
        law_rows.iter().all(|test| test["outcome"] == "skip"),
        "unlisted law metadata must not be reported as pass: {output:#}"
    );
}

#[test]
fn generated_algebra_laws_applicative_function_laws_defer_without_metadata() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src/algebra/applicative.ash");

    let assert = ash()
        .arg("test")
        .arg(path)
        .arg("--only-synthesized")
        .arg("laws")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let rows = output["tests"].as_array().unwrap();

    for law in ["homomorphism", "interchange", "composition"] {
        assert!(
            rows.iter().any(|test| {
                test["name"]
                    .as_str()
                    .is_some_and(|name| name == format!("synthesized/{law}/deferred"))
                    && test["outcome"] == "skip"
                    && test["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("no TASK-2035 source identity"))
            }),
            "applicative {law} should defer without hardcoded pass: {output:#}"
        );
    }
}

#[test]
fn generated_algebra_laws_monad_function_laws_defer_without_metadata() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src/algebra/monad.ash");

    let assert = ash()
        .arg("test")
        .arg(path)
        .arg("--only-synthesized")
        .arg("laws")
        .arg("--format")
        .arg("json")
        .assert();
    let output = parse_json_output(&assert.success());
    let rows = output["tests"].as_array().unwrap();

    for law in ["left_identity", "right_identity", "associativity"] {
        assert!(
            rows.iter().any(|test| {
                test["name"]
                    .as_str()
                    .is_some_and(|name| name == format!("synthesized/{law}/deferred"))
                    && test["outcome"] == "skip"
                    && test["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("no TASK-2035 source identity"))
            }),
            "monad {law} should defer without hardcoded function-model pass: {output:#}"
        );
    }
}
