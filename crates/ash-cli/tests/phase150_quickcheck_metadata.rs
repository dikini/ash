use std::path::PathBuf;
use std::process::Command;

use serde_json::json;

use ash_cli::test_runner::metadata::TestMetadata;
use ash_cli::test_runner::quickcheck::{domain_for_param_with_strategy, strategy_descriptor};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_fixture(relative: &str) -> serde_json::Value {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(["test", relative, "--format", "json"])
        .output()
        .expect("ash test fixture should launch");
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "fixture did not emit valid JSON: {error}\nstatus={}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn run_fixture_with_args(relative: &str, extra_args: &[&str]) -> (serde_json::Value, String) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(["test", relative, "--format", "json"])
        .args(extra_args)
        .output()
        .expect("ash test fixture should launch");
    let json = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "fixture did not emit valid JSON: {error}\nstatus={}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (json, String::from_utf8_lossy(&output.stderr).into_owned())
}

fn assert_generated_metadata_deferred(output: &serde_json::Value) {
    assert_eq!(output["success"], true);
    assert_eq!(output["passed"], 0);
    assert_eq!(output["failed"], 0);
    assert_eq!(output["skipped"], 1);

    let test = &output["tests"][0];
    assert_eq!(test["outcome"], "skip");
    assert_eq!(
        test["message"],
        "deferred: generated property metadata has no TASK-2035 source identity"
    );
    assert!(
        test.get("repro_artifact").is_none(),
        "deferred generated metadata must not claim local generator execution: {test:#}"
    );
}

#[test]
fn parses_quickcheck_strategy_overrides_from_metadata() {
    let source = r#"
-- @test name: sorted_lists
-- @test kind: property
-- @test params: xs: List<Int>, x: Int
-- @test strategy xs: test::quickcheck::sorted_int_lists
-- @test strategy x: test::quickcheck::positive_ints
-- @test property: x >= 0
fn main() -> Bool { true }
"#;
    let meta = TestMetadata::parse_from_source(source);
    assert_eq!(meta.quickcheck_strategies.len(), 2);
    assert_eq!(meta.quickcheck_strategies[0].binding, "xs");
    assert_eq!(
        meta.quickcheck_strategies[0].strategy_path,
        "test::quickcheck::sorted_int_lists"
    );
    assert_eq!(
        meta.quickcheck_strategy_for("x"),
        Some("test::quickcheck::positive_ints")
    );
}

#[test]
fn quickcheck_strategy_override_controls_generated_domain() {
    let domain =
        domain_for_param_with_strategy("xs: List<Int>", Some("test::quickcheck::sorted_int_lists"))
            .expect("sorted list strategy should materialize");
    assert_eq!(domain.binding, "xs");
    assert_eq!(domain.type_name, "List<Int>");
    assert_eq!(domain.values[0], json!([]));
    assert_eq!(domain.values[1], json!([-1]));
    assert_eq!(domain.values[2], json!([-1, 0]));
    assert_eq!(
        domain.descriptor.id,
        "strategy:xs:test::quickcheck::list::sorted_ints"
    );
}

#[test]
fn arbitrary_default_strategy_is_distinct_from_explicit_override() {
    let descriptor = strategy_descriptor("x", "Int", None).unwrap();
    assert_eq!(
        descriptor.strategy_id,
        "test::quickcheck::arbitrary::arbitrary<Int>"
    );
    assert_eq!(
        descriptor.law_coherence,
        "ordinary Strategy<A> gen/shrink selected from in-scope evidence"
    );

    let override_descriptor =
        strategy_descriptor("x", "Int", Some("test::quickcheck::positive_ints")).unwrap();
    assert_eq!(
        override_descriptor.strategy_id,
        "test::quickcheck::int::positive"
    );
    assert_eq!(
        override_descriptor.domain_role,
        "explicit_strategy_override"
    );
}

#[test]
fn quickcheck_metadata_defers_without_generator_execution_or_repro_snapshot() {
    let output = run_fixture(
        "fixtures/phase150-quickcheck/tests/ash/property/quickcheck_positive_int_override_pass.ash",
    );
    assert_generated_metadata_deferred(&output);
}

#[test]
fn quickcheck_strategy_override_metadata_defers_without_validation_execution() {
    for fixture in [
        "fixtures/phase150-quickcheck/tests/ash/property/quickcheck_unknown_strategy_binding_fails_closed.ash",
        "fixtures/phase150-quickcheck/tests/ash/property/quickcheck_duplicate_strategy_binding_fails_closed.ash",
    ] {
        let output = run_fixture(fixture);
        assert_generated_metadata_deferred(&output);
    }
}

#[test]
fn quickcheck_v1_metadata_cases_defer_without_generator_execution() {
    let (output, stderr) = run_fixture_with_args(
        "fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_canonical_positive_source_cases.ash",
        &["--max-cases", "99", "--seed", "123"],
    );
    assert_generated_metadata_deferred(&output);
    assert_eq!(stderr, "");
    assert_eq!(output["tests"][0]["seed"], 123);

    let (seed_override_output, seed_override_stderr) = run_fixture_with_args(
        "fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_source_seed_cli_override.ash",
        &["--seed", "123"],
    );
    assert_generated_metadata_deferred(&seed_override_output);
    assert!(seed_override_stderr.contains("source-pinned QuickCheck seed 7"));
    assert!(seed_override_stderr.contains("overridden by external seed 123"));
    assert_eq!(seed_override_output["tests"][0]["seed"], 123);

    let default_output = run_fixture(
        "fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_default_arbitrary_bool.ash",
    );
    assert_generated_metadata_deferred(&default_output);

    let missing_evidence_output = run_fixture(
        "fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_missing_arbitrary_import_fails_closed.ash",
    );
    assert_generated_metadata_deferred(&missing_evidence_output);

    let sorted_output = run_fixture(
        "fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_canonical_sorted_list.ash",
    );
    assert_generated_metadata_deferred(&sorted_output);
}
