//! TASK-2038 contract for `ash test` submission through the sole Engine executor.
//!
//! The two supported rows are the complete TASK-2035 source catalogue.  This
//! test intentionally supplies no arbitrary source, generator, or feature
//! slice.  It exercises the public suite boundary so the implementation cannot
//! satisfy the controls with a test-only executor.

use ash_cli::test_runner::{
    Outcome, TestKind,
    executor::{SuiteConfig, SynthesizedSources, run_suite},
    metadata::TestMetadata,
    property::execute_property_test,
    synthesized::{
        IntrospectionUnsupportedReason, RunnerContractMetadata, RunnerIntrospectionSnapshot,
        TypeGeneratorDescriptor, TypeGeneratorSource, synthesize_from_snapshot,
    },
    types::{ReproArtifact, TestResult, TestSuiteResult},
};
use ash_core::Value as AshValue;
use ash_engine::{CanonicalTerminalEnvelopeV1, Engine};
use proptest::prelude::*;
use serde_json::json;
use std::{fs, path::PathBuf, time::Duration};

const SYNTH_WRAPPER_ID: &str = "TASK-2035-SYNTH-WRAPPER-001";
const SHARED_ROUTE_ID: &str = "TASK-2035-SHARED-ROUTE-001";
const SYNTH_WRAPPER_SOURCE: &str =
    "fn contract_target_zero() -> Int { 0 }\nfn main() -> Bool { contract_target_zero() == 0 }\n";
const SYNTH_WRAPPER_DIGEST: &str =
    "sha256:71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390";
const SHARED_ROUTE_SOURCE: &str = "fn main() -> Int { 42 }\n";
const SHARED_ROUTE_DIGEST: &str =
    "sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c";

const DEFERRED_CASES: [(&str, &str); 7] = [
    (
        "test:contract_postcondition_without_executable_target_metadata",
        "deferred: contract metadata lacks executable postcondition target metadata",
    ),
    (
        "test:contract_postcondition_without_structured_oracle_metadata",
        "deferred: contract postcondition metadata is not executable",
    ),
    (
        "test:contract_postcondition_with_unsupported_target_kind_defers",
        "deferred: unsupported contract target kind runtime_callable",
    ),
    (
        "test:contract_postcondition_with_missing_setup_defers",
        "deferred: contract target execution setup is missing",
    ),
    (
        "test:contract_postcondition_explicit_finite_setup_defers",
        "deferred: explicit finite setup is not executable for pure target slice",
    ),
    (
        "test:contract_postcondition_unsupported_body_defers",
        "deferred: contract target body is not executable",
    ),
    (
        "test:contract_postcondition_missing_exact_input_defers",
        "deferred: contract postcondition oracle lacks exact valid input representatives",
    ),
];

fn task_2038_path() -> PathBuf {
    PathBuf::from("task-2038-canonical-engine-catalogue.ash")
}

fn source_contract(id: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match id {
        SYNTH_WRAPPER_ID => (
            SYNTH_WRAPPER_ID,
            SYNTH_WRAPPER_SOURCE,
            SYNTH_WRAPPER_DIGEST,
            "contract_target_zero",
        ),
        SHARED_ROUTE_ID => (
            SHARED_ROUTE_ID,
            SHARED_ROUTE_SOURCE,
            SHARED_ROUTE_DIGEST,
            "main",
        ),
        _ => panic!("test helper accepts only the two TASK-2035 source identities"),
    }
}

fn supported_contract(id: &str) -> RunnerContractMetadata {
    let (id, _source, _digest, callable_name) = source_contract(id);
    RunnerContractMetadata {
        id: id.to_string(),
        callable_name: callable_name.to_string(),
        callable_kind: "pure_function".to_string(),
        param_names: Vec::new(),
        param_types: Vec::new(),
        return_type: Some(
            if id == SYNTH_WRAPPER_ID {
                "Bool"
            } else {
                "Int"
            }
            .to_string(),
        ),
        ..RunnerContractMetadata::default()
    }
}

fn deferred_metadata() -> Vec<IntrospectionUnsupportedReason> {
    DEFERRED_CASES
        .iter()
        .map(
            |(case_id, required_result)| IntrospectionUnsupportedReason {
                source_kind: "contract".to_string(),
                target_name: (*case_id).to_string(),
                reason: (*required_result).to_string(),
            },
        )
        .collect()
}

fn catalogue_snapshot() -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: "ash-synthesized-v1.0".to_string(),
        module_identity: "task-2038-canonical-engine-catalogue".to_string(),
        source_artifact_id: "task-2035-source-catalogue-v1".to_string(),
        check_summary_id: "task-2035-source-catalogue-checked-v1".to_string(),
        contracts: vec![
            supported_contract(SYNTH_WRAPPER_ID),
            supported_contract(SHARED_ROUTE_ID),
        ],
        unsupported: deferred_metadata(),
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn unlisted_snapshot() -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: "ash-synthesized-v1.0".to_string(),
        module_identity: "task-2038-unlisted-source".to_string(),
        source_artifact_id: "task-2035-source-catalogue-v1-mutated".to_string(),
        check_summary_id: "task-2035-source-catalogue-checked-v1-mutated".to_string(),
        contracts: vec![RunnerContractMetadata {
            id: "TASK-2035-SYNTH-WRAPPER-001-mutated".to_string(),
            callable_name: "contract_target_zero".to_string(),
            callable_kind: "pure_function".to_string(),
            return_type: Some("Bool".to_string()),
            ..RunnerContractMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn run_catalogue(snapshot: RunnerIntrospectionSnapshot) -> TestSuiteResult {
    run_suite(&SuiteConfig {
        root: task_2038_path(),
        include_synthesized: true,
        only_synthesized: true,
        synthesized_sources: SynthesizedSources {
            contracts: true,
            policies: false,
            obligations: false,
            laws: false,
        },
        synthesized_snapshots: vec![(task_2038_path(), snapshot)],
        ..SuiteConfig::default()
    })
}

fn normalized_terminal_envelope(terminal: CanonicalTerminalEnvelopeV1) -> serde_json::Value {
    match terminal {
        CanonicalTerminalEnvelopeV1::Returned(AshValue::Bool(value)) => {
            json!({ "returned": { "Bool": value } })
        }
        CanonicalTerminalEnvelopeV1::Returned(AshValue::Int(value)) => {
            json!({ "returned": { "Int": value } })
        }
        CanonicalTerminalEnvelopeV1::Returned(value) => {
            json!({ "returned": { "display": value.to_string() } })
        }
        CanonicalTerminalEnvelopeV1::Trapped(reason) => json!({ "trapped": reason }),
        CanonicalTerminalEnvelopeV1::AdmissionRejected => json!({ "admission_rejected": true }),
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => {
            json!({ "invalid_checked_artifact": true })
        }
        CanonicalTerminalEnvelopeV1::TimedOut => json!({ "timed_out": true }),
        CanonicalTerminalEnvelopeV1::Cancelled => json!({ "cancelled": true }),
    }
}

fn independently_execute_admitted_source(
    path: &std::path::Path,
    source: &str,
) -> serde_json::Value {
    let engine = Engine::new().build().expect("test Engine builds");
    let mut entry = engine
        .parse_file_source(path, source)
        .expect("catalogued source parses through the public Engine boundary");
    let admitted = engine
        .admit_program(&mut entry)
        .expect("catalogued source admits through the public Engine boundary");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, Some(Duration::from_secs(30)))
        .expect("issuing Engine creates the admitted request");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime builds");
    let terminal = runtime
        .block_on(engine.execute_admitted_program(&request))
        .expect("Engine terminalizes the admitted request");
    normalized_terminal_envelope(terminal)
}

fn metadata_property_snapshot() -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: "ash-synthesized-v1.0".to_string(),
        module_identity: "task-2038-legacy-metadata-property".to_string(),
        source_artifact_id: "task-2038-legacy-metadata-property-source".to_string(),
        check_summary_id: "task-2038-legacy-metadata-property-check".to_string(),
        generators: vec![TypeGeneratorDescriptor {
            id: "task-2038-local-property".to_string(),
            target_type: "Int".to_string(),
            source: TypeGeneratorSource::AuthoredExamples,
            exact_values: vec![json!({ "input": 1, "property_holds": true })],
            ..TypeGeneratorDescriptor::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn altered_catalogue_snapshot() -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: "ash-synthesized-v1.0".to_string(),
        module_identity: "task-2038-altered-catalogue-source".to_string(),
        source_artifact_id: "task-2038-altered-shared-route-source".to_string(),
        check_summary_id: "task-2038-altered-shared-route-check".to_string(),
        contracts: vec![supported_contract(SHARED_ROUTE_ID)],
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn repro_for_case<'a>(
    suite: &'a TestSuiteResult,
    case_id: &str,
) -> (&'a TestResult, &'a ReproArtifact) {
    let result = suite
        .tests
        .iter()
        .find(|result| {
            result
                .repro_artifact
                .as_ref()
                .is_some_and(|artifact| artifact.case_id == case_id)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing TASK-2038 catalogue result {case_id}: {:#?}",
                suite.tests
            )
        });
    let repro = result
        .repro_artifact
        .as_ref()
        .expect("catalogued synthesized result has repro linkage");
    (result, repro)
}

fn assert_supported_engine_result(suite: &TestSuiteResult, source_id: &str) {
    let (id, source, digest, _callable_name) = source_contract(source_id);
    let (result, repro) = repro_for_case(suite, id);
    let expected_terminal = if id == SYNTH_WRAPPER_ID {
        json!({"returned": {"Bool": true}})
    } else {
        json!({"returned": {"Int": 42}})
    };

    assert_eq!(
        result.outcome,
        Outcome::Pass,
        "{id} must pass only from Engine terminalization"
    );
    assert_eq!(repro.oracle_snapshot["source_contract_id"], json!(id));
    assert_eq!(repro.oracle_snapshot["source"], json!(source));
    assert_eq!(repro.oracle_snapshot["source_digest"], json!(digest));
    assert_eq!(repro.oracle_snapshot["literal_inputs"], json!([]));
    assert_eq!(
        repro.oracle_snapshot["engine_terminal_envelope"],
        expected_terminal
    );
    assert_eq!(
        repro.oracle_snapshot["execution_route"],
        json!("engine_admitted_source")
    );
}

#[test]
fn selected_contract_wrapper_reaches_engine_with_exact_source_repro_and_bool_true() {
    let suite = run_catalogue(catalogue_snapshot());

    assert_supported_engine_result(&suite, SYNTH_WRAPPER_ID);
}

#[test]
fn shared_int_42_route_observes_the_normalized_engine_envelope() {
    let suite = run_catalogue(catalogue_snapshot());

    assert_supported_engine_result(&suite, SHARED_ROUTE_ID);

    let (_, source, _, _) = source_contract(SHARED_ROUTE_ID);
    let independently_observed_terminal =
        independently_execute_admitted_source(&task_2038_path(), source);
    let (_, repro) = repro_for_case(&suite, SHARED_ROUTE_ID);
    assert_eq!(
        repro.oracle_snapshot["engine_terminal_envelope"], independently_observed_terminal,
        "the test client must preserve the terminal observed from a separate public Engine submission"
    );
}

#[test]
fn each_catalogued_unsupported_shape_has_its_exact_deferred_observation() {
    let suite = run_catalogue(catalogue_snapshot());

    for (case_id, required_result) in DEFERRED_CASES {
        let (result, repro) = repro_for_case(&suite, case_id);
        assert_eq!(result.outcome, Outcome::Skip, "{case_id} must defer");
        assert_eq!(result.message.as_deref(), Some(required_result));
        assert_eq!(
            repro.oracle_snapshot["execution_route"],
            json!("deferred_before_execution")
        );
    }
}

#[test]
fn changed_or_unlisted_source_identity_defers_without_an_evaluator_fallback() {
    let suite = run_catalogue(unlisted_snapshot());
    let (result, repro) = repro_for_case(&suite, "TASK-2035-SYNTH-WRAPPER-001-mutated");

    assert_eq!(result.outcome, Outcome::Skip);
    assert_eq!(
        result.message.as_deref(),
        Some("deferred: source identity is not in the TASK-2035 catalogue")
    );
    assert_eq!(
        repro.oracle_snapshot["execution_route"],
        json!("catalogue_rejection")
    );
    assert!(
        !repro.oracle_snapshot.to_string().contains("ash_runtime")
            && !repro.oracle_snapshot.to_string().contains("CoreExpr")
            && !repro.oracle_snapshot.to_string().contains("CPS")
            && !repro.oracle_snapshot.to_string().contains("differential"),
        "a rejected source identity may not report an AST, Core, CPS, or differential fallback"
    );
}

#[test]
fn legacy_synthesized_metadata_api_defers_instead_of_reporting_local_pass_or_fail() {
    let results = synthesize_from_snapshot(&task_2038_path(), &metadata_property_snapshot());

    assert!(
        !results.is_empty(),
        "the public compatibility API must surface an explicit deferred row rather than silently dropping metadata"
    );
    assert!(
        results.iter().all(|result| {
            result.outcome == Outcome::Skip
                && result
                    .message
                    .as_deref()
                    .is_some_and(|message| message.starts_with("deferred:"))
        }),
        "generated metadata must not report local pass/fail observations: {results:#?}"
    );
}

#[test]
fn engine_backed_suite_defers_generated_property_metadata_with_repro() {
    let suite = run_catalogue(metadata_property_snapshot());
    let result = suite
        .tests
        .iter()
        .find(|result| {
            result.source == ash_cli::test_runner::TestSource::Contract
                && result.kind == TestKind::Property
        })
        .unwrap_or_else(|| {
            panic!(
                "the Engine-backed suite must surface generated property metadata as deferred: {:#?}",
                suite.tests
            )
        });

    assert_eq!(result.outcome, Outcome::Skip);
    assert_eq!(
        result.message.as_deref(),
        Some("deferred: generated property metadata has no TASK-2035 source identity")
    );
    let repro = result
        .repro_artifact
        .as_ref()
        .expect("the deferred generated property row has repro linkage");
    assert_eq!(repro.case_id, "property:task-2038-local-property");
    assert_eq!(
        repro.oracle_snapshot["execution_route"],
        json!("deferred_before_execution")
    );
    assert_eq!(
        repro.oracle_snapshot["descriptor_id"],
        json!("task-2038-local-property")
    );
    assert!(
        repro.generated_input_snapshot.is_none(),
        "the Engine-backed route may not claim local generator execution"
    );
}

#[test]
fn altered_catalogue_shape_rejects_at_engine_admission_and_the_compatibility_api_defers() {
    let altered_source = "fn main() { 1 }\n";
    let engine = Engine::new().build().expect("test Engine builds");
    let mut entry = engine
        .parse_file_source(task_2038_path(), altered_source)
        .expect("an altered source shape still parses before Engine admission");
    let admission_error = engine
        .admit_program(&mut entry)
        .expect_err("only the exact TASK-2035 shared source shape receives Engine admission");
    assert!(
        !admission_error.to_string().is_empty(),
        "the Engine must report its admission rejection rather than choose a client fallback"
    );

    let results = synthesize_from_snapshot(&task_2038_path(), &altered_catalogue_snapshot());
    assert!(
        !results.is_empty(),
        "a listed identity paired with a non-exact supplied source must be surfaced as deferred"
    );
    assert!(
        results.iter().all(|result| {
            result.outcome == Outcome::Skip
                && result
                    .message
                    .as_deref()
                    .is_some_and(|message| message.starts_with("deferred:"))
        }),
        "the compatibility API must defer altered catalogue shapes without pass/fail fallback: {results:#?}"
    );
}

fn test_runner_source(relative_path: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_runner")
            .join(relative_path),
    )
    .unwrap_or_default()
}

#[test]
fn selected_test_execution_paths_retain_no_client_local_evaluator_calls() {
    for (path, forbidden_symbols) in [
        ("executor.rs", &["engine.execute("][..]),
        (
            "synthesized.rs",
            &[
                "generated_property_results(",
                "algebra_law_profile_results(",
                "law_property_results(",
                "smallworld_results(",
                "law_smallworld_results(",
                "execute_synthesized_case",
            ][..],
        ),
        (
            "synthesized/contract.rs",
            &[
                "evaluate_core_expression",
                "evaluate_simple_bool_expression",
            ][..],
        ),
        ("property.rs", &["evaluate_simple_bool_expression"][..]),
    ] {
        let source = test_runner_source(path);
        assert!(
            !source.is_empty(),
            "the local evaluator guard must read {path} rather than silently treating a missing file as clean"
        );
        for symbol in forbidden_symbols {
            assert!(
                !source.contains(symbol),
                "the retained test-runner source {path} must not retain local evaluator symbol {symbol}"
            );
        }
    }

    for path in [
        "synthesized/eval.rs",
        "synthesized/execution.rs",
        "synthesized/law.rs",
        "synthesized/obligation.rs",
        "synthesized/policy.rs",
        "synthesized/property.rs",
        "synthesized/smallworld.rs",
    ] {
        assert!(
            test_runner_source(path).is_empty(),
            "the retired local evaluator path {path} must stay absent"
        );
    }
}

#[test]
fn unlisted_generated_property_defers_instead_of_locally_passing_an_oracle() {
    let engine = Engine::new().build().expect("test Engine builds");
    let metadata = TestMetadata {
        name: Some("task-2038-unlisted-property".to_string()),
        property: Some("1 == 1".to_string()),
        generated_params: vec!["value: Int".to_string()],
        ..TestMetadata::default()
    };

    let result = execute_property_test(
        &task_2038_path(),
        &metadata,
        &engine,
        0,
        "task-2038-unlisted-property",
        1,
        Duration::from_secs(1),
    );

    assert_eq!(result.outcome, Outcome::Skip);
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| message.starts_with("deferred:")),
        "an unlisted generated property must report a deferred observation"
    );
}

#[tokio::test]
async fn engine_backed_synthesized_route_is_safe_inside_an_existing_tokio_runtime() {
    let suite = run_catalogue(catalogue_snapshot());

    assert_supported_engine_result(&suite, SYNTH_WRAPPER_ID);
    assert_supported_engine_result(&suite, SHARED_ROUTE_ID);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn selected_catalogue_property_preserves_identity_terminal_and_engine_route(
        source_id in prop_oneof![Just(SYNTH_WRAPPER_ID), Just(SHARED_ROUTE_ID)],
    ) {
        let suite = run_catalogue(catalogue_snapshot());
        assert_supported_engine_result(&suite, source_id);

        let (_, source, digest, _) = source_contract(source_id);
        prop_assert!(matches!(source, SYNTH_WRAPPER_SOURCE | SHARED_ROUTE_SOURCE));
        prop_assert!(matches!(digest, SYNTH_WRAPPER_DIGEST | SHARED_ROUTE_DIGEST));
    }
}
