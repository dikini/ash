//! Deferred synthesized results and repro artifact helpers.

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use super::{RUNNER_SYNTHESIS_SCHEMA_VERSION, RunnerIntrospectionSnapshot};
use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult, TestSource};

pub(super) fn deferred_result(
    path: &Path,
    source: TestSource,
    name: impl Into<String>,
    message: impl Into<String>,
    repro: ReproArtifact,
) -> TestResult {
    TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(source)
        .with_kind(TestKind::Unit)
        .with_duration(Duration::ZERO)
        .with_message(message)
        .with_repro_artifact(repro)
}

pub(super) fn deferred_result_with_kind(
    path: &Path,
    source: TestSource,
    kind: TestKind,
    name: impl Into<String>,
    message: impl Into<String>,
    repro: ReproArtifact,
) -> TestResult {
    let seed = repro.seed;
    TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(source)
        .with_kind(kind)
        .with_duration(Duration::ZERO)
        .with_seed(seed)
        .with_message(message)
        .with_repro_artifact(repro)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn repro_artifact(
    path: &Path,
    source_artifact_id: String,
    check_summary_id: String,
    case_id: String,
    seed: u64,
    case_index: usize,
    generated_input_snapshot: Option<Value>,
    oracle_snapshot: Value,
    world_snapshot: Option<Value>,
) -> ReproArtifact {
    ReproArtifact {
        runner_schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        source_artifact_id,
        check_summary_id,
        case_id,
        seed,
        case_index,
        world_index: None,
        generated_input_snapshot,
        world_snapshot,
        oracle_snapshot,
        replay_command: format!(
            "ash test {} --only-synthesized contracts,obligations",
            path.display()
        ),
    }
}

pub(super) fn fallback_repro(
    path: &Path,
    _source: TestSource,
    case_id: String,
    oracle: Value,
) -> ReproArtifact {
    repro_artifact(
        path,
        format!("source-file:{}", path.display()),
        "raw-source-fallback:no-lowered-summary".to_string(),
        case_id,
        0,
        1,
        None,
        oracle,
        None,
    )
}

pub(super) fn source_from_label(source_kind: &str) -> TestSource {
    match source_kind {
        "contract" | "contracts" => TestSource::Contract,
        "obligation" | "obligations" => TestSource::Obligation,
        "law" | "laws" => TestSource::Law,
        _ => TestSource::Authored,
    }
}

pub(super) fn snapshot_source_label(snapshot: &RunnerIntrospectionSnapshot) -> &'static str {
    if snapshot.check_summary_id.starts_with("checked:") {
        "live_checked_snapshot"
    } else {
        "structured_snapshot"
    }
}
