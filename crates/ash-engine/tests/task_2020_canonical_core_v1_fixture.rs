//! TASK-2020: strict canonical-Core V1 fixture boundary.
//!
//! This fixture kind carries Core text locally and reaches only the private
//! checked Core/CPS prototype. It never creates a direct-runtime route.

use ash_engine::differential::{CaseComparisonStatus, DifferentialHarness, RustExecutionTarget};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

fn canonical_manifest(overrides: &str) -> String {
    format!(
        r#"{{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-return-int-7",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
  "core_text": "(lit-int 7)"{overrides}
}}"#
    )
}

fn write_single_canonical_fixture(manifest: &str) -> TempDir {
    let corpus = TempDir::new().expect("temporary corpus directory");
    let case_dir = corpus.path().join("canonical-case");
    fs::create_dir(&case_dir).expect("fixture case directory");
    fs::write(case_dir.join("canonical-core.json"), manifest).expect("fixture manifest");
    corpus
}

fn load_error(manifest: &str) -> String {
    let corpus = write_single_canonical_fixture(manifest);
    DifferentialHarness::load(corpus.path())
        .expect_err("invalid canonical Core V1 fixture must not load")
        .to_string()
}

#[test]
fn v1_literal_core_fixture_runs_only_through_checked_core_cps_and_returns_int_7() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let checked = harness.run_case(
        "canonical-core-v1-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );
    assert_eq!(
        checked.direct_runtime_status(),
        CaseComparisonStatus::Passed
    );
    assert_eq!(
        checked.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );

    let direct = harness.run_case(
        "canonical-core-v1-return-int-7",
        RustExecutionTarget::DirectRuntime,
    );
    assert!(matches!(
        direct.direct_runtime_status(),
        CaseComparisonStatus::Unsupported { .. }
    ));
    assert!(direct.actual_result().is_none());
}

#[test]
fn v1_manifest_rejects_unknown_schema_fields_targets_and_rule_claims_during_load() {
    let cases = [
        (
            "bad schema",
            canonical_manifest("").replace(
                "ash-canonical-core-fixture/v1",
                "ash-canonical-core-fixture/v2",
            ),
        ),
        (
            "unknown field",
            canonical_manifest(",\n  \"unknown\": true"),
        ),
        (
            "noncanonical target",
            canonical_manifest("")
                .replace("rust-checked-core-cps-prototype", "rust-direct-runtime"),
        ),
        (
            "path carrier",
            canonical_manifest(",\n  \"core_file\": \"program.core\""),
        ),
        (
            "input carrier",
            canonical_manifest(",\n  \"input_file\": \"input.json\""),
        ),
        (
            "URL carrier",
            canonical_manifest(",\n  \"url\": \"https://example.invalid/core\""),
        ),
        (
            "unsupported rule",
            r#"{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-return-int-7",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["NOT-A-CANONICAL-RULE"],
  "core_text": "(lit-int 7)"
}"#
            .to_string(),
        ),
        (
            "duplicate rule",
            r#"{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-return-int-7",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["SEM-CPS-RETURN-001", "SEM-CPS-RETURN-001"],
  "core_text": "(lit-int 7)"
}"#
            .to_string(),
        ),
    ];

    for (name, manifest) in cases {
        let corpus = write_single_canonical_fixture(&manifest);
        let error = DifferentialHarness::load(corpus.path())
            .expect_err(name)
            .to_string();
        assert!(
            error.contains("canonical") || error.contains("schema") || error.contains("JSON"),
            "{name} must reject while loading: {error}"
        );
    }
}

#[test]
fn v1_manifest_rejects_malformed_core_before_any_terminal_result_exists() {
    let corpus = write_single_canonical_fixture(
        r#"{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-malformed",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
  "core_text": "(lit-int"
}"#,
    );
    let error = DifferentialHarness::load(corpus.path())
        .expect_err("malformed Core text must fail during fixture load")
        .to_string();
    assert!(
        error.contains("parse") || error.contains("Core") || error.contains("canonical"),
        "malformed Core must identify a corpus/Core phase: {error}"
    );
}

#[test]
fn v1_manifest_rejects_missing_identity_empty_ids_and_nonstring_core_text_during_decode() {
    let missing_target =
        canonical_manifest("").replace("  \"target\": \"rust-checked-core-cps-prototype\",\n", "");
    let empty_case_id = canonical_manifest("").replace("canonical-core-v1-return-int-7", "   ");
    let empty_rule_ids = canonical_manifest("").replace(
        "[\"SEM-CPS-RETURN-001\", \"CONF-IMPLEMENTATION-001\"]",
        "[]",
    );
    let nonstring_core_text = canonical_manifest("").replace("\"(lit-int 7)\"", "7");

    for (name, manifest, expected_phase) in [
        ("missing target", missing_target, "could not parse JSON"),
        ("empty case ID", empty_case_id, "empty case ID"),
        (
            "empty canonical rule IDs",
            empty_rule_ids,
            "must declare non-empty canonical rule IDs",
        ),
        (
            "non-string core text",
            nonstring_core_text,
            "could not parse JSON",
        ),
    ] {
        let error = load_error(&manifest);
        assert!(
            error.contains(expected_phase),
            "{name} must fail at its manifest identity/decode boundary: {error}"
        );
    }
}

#[test]
fn v1_manifest_denies_every_path_or_indirection_shaped_carrier_field() {
    // V1's only executable carrier is the manifest-local `core_text` literal.
    // Each spelling below must be rejected structurally, before Core parsing.
    for (field, value) in [
        ("input_file", "input.json"),
        ("core_file", "program.core"),
        ("path", "./program.core"),
        ("core_path", "/tmp/program.core"),
        ("source_file", "source.ash"),
        ("file", "fixture.core"),
        ("url", "https://example.invalid/core"),
        ("core_url", "https://example.invalid/program.core"),
        ("include", "other.core"),
    ] {
        let manifest = canonical_manifest(&format!(",\n  \"{field}\": \"{value}\""));
        let error = load_error(&manifest);
        assert!(
            error.contains("could not parse JSON") && error.contains(field),
            "{field} must be denied by the closed manifest decoder before Core parsing: {error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn v1_fixture_rejects_symlinked_case_directory_before_manifest_load() {
    use std::os::unix::fs::symlink;

    let corpus = TempDir::new().expect("temporary corpus directory");
    let target = TempDir::new().expect("symlink target directory");
    fs::write(
        target.path().join("canonical-core.json"),
        canonical_manifest(""),
    )
    .expect("target fixture manifest");
    symlink(target.path(), corpus.path().join("canonical-case")).expect("case-directory symlink");

    let error = DifferentialHarness::load(corpus.path())
        .expect_err("canonical Core V1 must not load a symlinked case directory")
        .to_string();
    assert!(
        error.contains("canonical Core V1 case directory must not be a symlink"),
        "the case-directory symlink must reject before manifest loading: {error}"
    );
}

#[cfg(unix)]
#[test]
fn v1_fixture_rejects_symlinked_manifest_before_json_decode() {
    use std::os::unix::fs::symlink;

    let corpus = TempDir::new().expect("temporary corpus directory");
    let case_dir = corpus.path().join("canonical-case");
    fs::create_dir(&case_dir).expect("fixture case directory");
    let target = corpus.path().join("outside-canonical-core.json");
    fs::write(&target, canonical_manifest("")).expect("symlink target manifest");
    symlink(&target, case_dir.join("canonical-core.json")).expect("manifest symlink");

    let error = DifferentialHarness::load(corpus.path())
        .expect_err("canonical Core V1 must not load a symlinked manifest")
        .to_string();
    assert!(
        error.contains("canonical Core V1 manifest must not be a symlink"),
        "the manifest symlink must reject before JSON decoding: {error}"
    );
}

#[cfg(not(unix))]
#[test]
fn v1_fixture_symlink_regressions_are_explicitly_unavailable_without_unix_symlinks() {
    // This is an explicit platform skip: the Unix-only tests above exercise
    // symlink metadata rather than silently omitting the closed-envelope case.
    eprintln!("TASK-2020 symlink regressions require Unix symlink support");
}

#[test]
fn v1_fixed_text_admission_rejects_noncanonical_programs_before_core_pipeline() {
    let cases = [
        (
            "validation",
            canonical_manifest("").replace("(lit-int 7)", "(force result (lit-int 7) (lit-int 7))"),
            "canonical Core validation failed",
        ),
        (
            "typecheck",
            canonical_manifest("").replace("(lit-int 7)", "unbound_value"),
            "canonical Core type check failed: unknown value `unbound_value`",
        ),
        (
            "lowering",
            canonical_manifest("").replace("(lit-int 7)", "(prim add)"),
            "canonical Core lowering failed: Core value cannot lower to CPS",
        ),
    ];

    for (name, manifest, _former_phase) in cases {
        let error = load_error(&manifest);
        assert!(
            error.contains(
                "canonical Core V1 fixture must use the exact fixed text for its admitted control"
            ),
            "{name} must reject at fixed-text admission before Core parsing, validation, type checking, lowering, or terminal comparison: {error}"
        );
    }
}
