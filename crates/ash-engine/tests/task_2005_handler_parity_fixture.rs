//! TASK-2005 RED parity contract for one closed-empty source handler.
//!
//! This fixture is intentionally restricted to the exact `absorb_sleep`
//! source witness.  It calls only the private differential harness; it must
//! neither enter an Engine production route nor create a fallback route.

use ash_engine::differential::{
    CaseComparisonStatus, DifferentialHarness, ObservableDimension, ParityDisposition,
    RelationStatus, RustExecutionTarget,
};
use serde_json::{Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const ABSORB_SLEEP_CASE_ID: &str = "phase202-source-absorb-sleep-handler-parity";
const ABSORB_SLEEP_SOURCE_FINGERPRINT: &str =
    "sha256:005a6c46e25884ca13762b7cd26e836b2756263f378fd297aa0afc006e8acf89";

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/differential/corpus")
}

fn source_fingerprint(source: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(source.as_bytes()))
}

fn source_from_input(input: &JsonValue) -> &str {
    input
        .get("direct_runtime")
        .and_then(JsonValue::as_object)
        .and_then(|runtime| runtime.get("source"))
        .and_then(JsonValue::as_str)
        .expect("handler parity input declares a direct source")
}

#[test]
fn closed_empty_absorb_sleep_compares_private_direct_reference_and_checked_cps_handler_semantics() {
    let harness = DifferentialHarness::load(corpus_root())
        .expect("the exact closed-empty handler parity fixture loads");

    let report = harness.run_case(ABSORB_SLEEP_CASE_ID, RustExecutionTarget::DirectRuntime);

    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 0}},
        }))
    );
    assert_eq!(report.checked_core_cps_relation(), RelationStatus::Passed);
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-EFFECT-HANDLE-001" && owner == "TASK-2005"
    ));
}

#[test]
fn handler_parity_fixture_rejects_a_clause_payload_tamper_against_its_declared_source_fingerprint()
{
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let source_case = corpus_root().join(ABSORB_SLEEP_CASE_ID);
    let copied_case = root.path().join(ABSORB_SLEEP_CASE_ID);
    fs::create_dir_all(&copied_case).expect("copied handler parity fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("handler parity fixture file copied into temporary corpus");
    }

    let case: JsonValue = serde_json::from_slice(
        &fs::read(copied_case.join("case.json")).expect("copied handler parity case read"),
    )
    .expect("copied handler parity case is JSON");
    assert_eq!(case["case_id"], ABSORB_SLEEP_CASE_ID);
    assert_eq!(
        case["source_fingerprint"], ABSORB_SLEEP_SOURCE_FINGERPRINT,
        "the corpus, not this test's source literal, binds the exact handler source"
    );
    assert!(
        case["canonical_rule_ids"]
            .as_array()
            .expect("handler parity case carries canonical rules")
            .iter()
            .any(|rule| rule == "SEM-EFFECT-HANDLE-001")
    );

    let input_path = copied_case.join("input.ir.json");
    let mut input: JsonValue =
        serde_json::from_slice(&fs::read(&input_path).expect("copied handler parity input read"))
            .expect("copied handler parity input is JSON");
    let original_source = source_from_input(&input);
    assert_eq!(
        source_fingerprint(original_source),
        ABSORB_SLEEP_SOURCE_FINGERPRINT,
        "the declared fingerprint binds the exact canonical handler source"
    );

    let altered_source = original_source.replacen("resume(ms)", "ms", 1);
    assert_ne!(
        altered_source, original_source,
        "the control must alter the handler clause payload without renaming the case"
    );
    assert!(
        altered_source
            .contains("fn main() -> Int { handle TestClock::sleep(0) with absorb_sleep }"),
        "the altered source retains the exact root handler application"
    );
    *input
        .get_mut("direct_runtime")
        .and_then(JsonValue::as_object_mut)
        .and_then(|runtime| runtime.get_mut("source"))
        .expect("handler parity input carries a mutable direct source") =
        JsonValue::String(altered_source);
    fs::write(
        &input_path,
        serde_json::to_vec_pretty(&input).expect("altered handler parity input serializes"),
    )
    .expect("altered handler parity input written");

    let altered_input: JsonValue =
        serde_json::from_slice(&fs::read(&input_path).expect("altered handler parity input read"))
            .expect("altered handler parity input is JSON");
    assert_ne!(
        source_fingerprint(source_from_input(&altered_input)),
        ABSORB_SLEEP_SOURCE_FINGERPRINT,
        "the clause payload tamper must invalidate the declared corpus fingerprint"
    );

    let error = DifferentialHarness::load(root.path()).expect_err(
        "the handler parity loader must reject a source payload that no longer matches its declared fingerprint",
    );
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("handler-source fingerprint mismatch")
            && diagnostic.contains(ABSORB_SLEEP_SOURCE_FINGERPRINT),
        "the rejection must attribute the exact declared handler-source fingerprint: {diagnostic}"
    );
}
