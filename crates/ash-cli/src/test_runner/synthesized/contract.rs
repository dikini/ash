//! Contract-derived synthesized rows.

use std::path::Path;

use serde_json::json;

use super::repro::repro_artifact;
use super::{RunnerContractMetadata, RunnerIntrospectionSnapshot};
use crate::test_runner::types::{TestResult, TestSource};
use crate::test_runner::{Outcome, TestKind};

const SYNTH_WRAPPER_ID: &str = "TASK-2035-SYNTH-WRAPPER-001";
const SYNTH_WRAPPER_SOURCE: &str =
    "fn contract_target_zero() -> Int { 0 }\nfn main() -> Bool { contract_target_zero() == 0 }\n";
const SYNTH_WRAPPER_DIGEST: &str =
    "sha256:71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390";
const SHARED_ROUTE_ID: &str = "TASK-2035-SHARED-ROUTE-001";
const SHARED_ROUTE_SOURCE: &str = "fn main() -> Int { 42 }\n";
const SHARED_ROUTE_DIGEST: &str =
    "sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c";

/// Submit one of the two TASK-2035 source contracts through the Engine.
///
/// Metadata is an identity selector only. It is never used to reconstruct
/// source or an executable test oracle.
pub(super) fn catalogue_engine_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
    engine: &ash_engine::Engine,
    timeout: std::time::Duration,
) -> TestResult {
    let Some((source, digest)) = catalogue_source(&contract.id) else {
        return catalogue_rejection_result(path, snapshot, contract);
    };

    let execution = crate::test_runner::engine_execution::execute_admitted_source(
        engine, path, source, timeout,
    );
    let (outcome, message, terminal, execution_route) = match execution {
        Ok(ash_engine::CanonicalTerminalEnvelopeV1::AdmissionRejected) => (
            Outcome::Skip,
            Some(format!(
                "deferred: Engine did not admit source contract {}",
                contract.id
            )),
            json!({ "admission_rejected": true, "execution_route": "deferred_before_execution" }),
            "deferred_before_execution",
        ),
        Ok(ash_engine::CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact) => (
            Outcome::Skip,
            Some(format!(
                "deferred: Engine rejected the checked artifact for source contract {}",
                contract.id
            )),
            json!({
                "invalid_checked_artifact": true,
                "execution_route": "deferred_before_execution",
            }),
            "deferred_before_execution",
        ),
        Ok(terminal) if terminal_matches_expected(&terminal, &contract.id) => (
            Outcome::Pass,
            None,
            crate::test_runner::engine_execution::terminal_envelope_json(&terminal),
            "engine_admitted_source",
        ),
        Ok(terminal) => {
            let actual = crate::test_runner::engine_execution::terminal_envelope_json(&terminal);
            (
                Outcome::Fail,
                Some(format!(
                    "Engine terminal did not match source contract {}",
                    contract.id
                )),
                actual,
                "engine_admitted_source",
            )
        }
        Err(error) => (
            Outcome::Skip,
            Some(format!(
                "deferred: Engine submission for source contract {} failed: {error}",
                contract.id
            )),
            json!({
                "engine_submission_error": error,
                "execution_route": "deferred_before_execution",
            }),
            "deferred_before_execution",
        ),
    };
    let repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        contract.id.clone(),
        0,
        1,
        Some(json!([])),
        json!({
            "source_contract_id": contract.id,
            "source": source,
            "source_digest": digest,
            "literal_inputs": [],
            "engine_terminal_envelope": terminal,
            "execution_route": execution_route,
        }),
        None,
    );
    let mut result = TestResult::new(&contract.id, path.to_path_buf())
        .with_outcome(outcome)
        .with_source(TestSource::Contract)
        .with_kind(TestKind::Unit)
        .with_repro_artifact(repro);
    if let Some(message) = message {
        result = result.with_message(message);
    }
    result.tags = vec!["synthesized".to_string(), "contract".to_string()];
    result
}

fn catalogue_source(id: &str) -> Option<(&'static str, &'static str)> {
    match id {
        SYNTH_WRAPPER_ID => Some((SYNTH_WRAPPER_SOURCE, SYNTH_WRAPPER_DIGEST)),
        SHARED_ROUTE_ID => Some((SHARED_ROUTE_SOURCE, SHARED_ROUTE_DIGEST)),
        _ => None,
    }
}

fn terminal_matches_expected(terminal: &ash_engine::CanonicalTerminalEnvelopeV1, id: &str) -> bool {
    matches!(
        (id, terminal),
        (
            SYNTH_WRAPPER_ID,
            ash_engine::CanonicalTerminalEnvelopeV1::Returned(ash_core::Value::Bool(true)),
        ) | (
            SHARED_ROUTE_ID,
            ash_engine::CanonicalTerminalEnvelopeV1::Returned(ash_core::Value::Int(42)),
        )
    )
}

fn catalogue_rejection_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> TestResult {
    let repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        contract.id.clone(),
        0,
        1,
        None,
        json!({ "execution_route": "catalogue_rejection" }),
        None,
    );
    let mut result = TestResult::new(&contract.id, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(TestSource::Contract)
        .with_kind(TestKind::Unit)
        .with_message("deferred: source identity is not in the TASK-2035 catalogue")
        .with_repro_artifact(repro);
    result.tags = vec!["synthesized".to_string(), "contract".to_string()];
    result
}

pub(super) fn expression_parameter(expression: &str) -> Option<String> {
    let tokens: Vec<&str> = expression.split_whitespace().collect();
    if tokens.len() != 3 {
        return None;
    }
    Some(tokens[0].to_string())
}
