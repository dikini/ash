//! Law evidence derived from authored Ash tests.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::json;

use super::repro::repro_artifact;
use super::{LawEvidenceStatus, LawTestEvidence, RunnerIntrospectionSnapshot, RunnerLawMetadata};
use crate::test_runner::types::{Outcome, TestKind, TestResult, TestSource};

/// Resolve `by test "..."` law evidence against executed authored tests.
pub(crate) fn authored_law_test_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    authored_tests: &BTreeMap<String, TestResult>,
) -> Vec<TestResult> {
    snapshot
        .laws
        .iter()
        .filter_map(|law| match &law.test_evidence {
            Some(LawTestEvidence::Authored { test_name }) => Some(authored_law_test_result(
                path,
                snapshot,
                law,
                test_name,
                authored_tests.get(test_name),
            )),
            _ => None,
        })
        .collect()
}

fn authored_law_test_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    test_name: &str,
    authored_result: Option<&TestResult>,
) -> TestResult {
    let case_id = format!("synthesized/law/{}/by-test-authored", law.name);
    let (outcome, status, message) = match authored_result {
        Some(result) if result.outcome == Outcome::Pass => (
            Outcome::Pass,
            LawEvidenceStatus::Satisfied,
            format!(
                "law {} satisfied by authored Ash test '{}'",
                law.name, test_name
            ),
        ),
        Some(result) if result.outcome == Outcome::Skip || result.outcome == Outcome::Xfail => (
            Outcome::Error,
            LawEvidenceStatus::InvalidEvidence,
            format!(
                "invalid law test evidence: authored Ash test '{}' did not run to pass ({})",
                test_name, result.outcome
            ),
        ),
        Some(result) => (
            Outcome::Fail,
            LawEvidenceStatus::Broken,
            format!(
                "law {} broken: authored Ash test '{}' reported {}{}",
                law.name,
                test_name,
                result.outcome,
                result
                    .message
                    .as_deref()
                    .map(|message| format!(": {message}"))
                    .unwrap_or_default()
            ),
        ),
        None => (
            Outcome::Error,
            LawEvidenceStatus::InvalidEvidence,
            format!(
                "invalid law test evidence: by test target '{}' was not discovered as an Ash authored test",
                test_name
            ),
        ),
    };

    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("law:{}:by-test-authored", law.id),
        0,
        1,
        None,
        json!({
            "source": "law",
            "law": law.name,
            "proof_evidence_family": "test",
            "test_mode": "authored",
            "evidence_status": evidence_status_name(status),
            "delegated_test": test_name,
            "proposition": law.proposition,
            "authored_test_outcome": authored_result.map(|result| result.outcome.to_string()),
            "authored_test_path": authored_result.map(|result| result.path.display().to_string()),
        }),
        None,
    );
    repro.replay_command = format!(
        "ASH_UNDER_TEST=${{ASH_UNDER_TEST:?set Ash candidate binary}}; \"$ASH_UNDER_TEST\" test {} --include-synthesized laws",
        path.display()
    );

    let mut result = TestResult::new(case_id, path.to_path_buf())
        .with_outcome(outcome)
        .with_source(TestSource::Law)
        .with_kind(TestKind::Unit)
        .with_duration(Duration::ZERO)
        .with_message(message)
        .with_repro_artifact(repro);
    result.evidence_family = Some("test".to_string());
    result.test_mode = Some("authored".to_string());
    result.evidence_status = Some(evidence_status_name(status).to_string());
    result.tags = vec![
        "synthesized".to_string(),
        "law".to_string(),
        "by-test".to_string(),
        "authored".to_string(),
    ];
    result
}

fn evidence_status_name(status: LawEvidenceStatus) -> &'static str {
    match status {
        LawEvidenceStatus::Satisfied => "satisfied",
        LawEvidenceStatus::Broken => "broken",
        LawEvidenceStatus::InvalidEvidence => "invalid_evidence",
        LawEvidenceStatus::Deferred => "deferred",
        LawEvidenceStatus::Untested => "untested",
    }
}
