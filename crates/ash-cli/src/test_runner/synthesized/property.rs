//! Generated property synthesized rows.

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::repro::{deferred_result_with_kind, repro_artifact};
use super::{RunnerIntrospectionSnapshot, TypeGeneratorDescriptor, TypeGeneratorSource};
use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult, TestSource};

pub(super) fn generated_property_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_cases: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();
    let mut generated_count = 0;

    for descriptor in &snapshot.generators {
        if descriptor.unsupported_reason.is_some()
            || descriptor.source == TypeGeneratorSource::Unsupported
            || descriptor.exact_values.is_empty()
        {
            results.push(deferred_property_result(path, snapshot, descriptor, seed));
            continue;
        }

        if !is_supported_property_generator(descriptor) {
            results.push(deferred_property_result(path, snapshot, descriptor, seed));
            continue;
        }

        for value in descriptor.exact_values.iter().take(
            max_cases
                .map(|limit| limit.saturating_sub(generated_count))
                .unwrap_or(usize::MAX),
        ) {
            generated_count += 1;
            let case_index = generated_count;
            let case_id = format!("synthesized/property/{}/case-{}", descriptor.id, case_index);
            let Some(property_holds) = property_holds_from_generated_value(value) else {
                results.push(deferred_result_with_kind(
                    path,
                    TestSource::Contract,
                    TestKind::Property,
                    case_id,
                    "deferred: generated property value lacks supported metadata oracle",
                    property_repro_artifact(
                        path,
                        snapshot,
                        descriptor,
                        seed,
                        case_index,
                        value,
                        json!({
                            "kind": "metadata_property_holds",
                            "supported": false,
                        }),
                        max_cases.unwrap_or(descriptor.exact_values.len()),
                    ),
                ));
                continue;
            };

            let outcome = if property_holds {
                Outcome::Pass
            } else {
                Outcome::Fail
            };
            let mut result = TestResult::new(&case_id, path.to_path_buf())
                .with_outcome(outcome)
                .with_source(TestSource::Contract)
                .with_kind(TestKind::Property)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_repro_artifact(property_repro_artifact(
                    path,
                    snapshot,
                    descriptor,
                    seed,
                    case_index,
                    value,
                    json!({
                        "kind": "metadata_property_holds",
                        "expected": true,
                        "actual": property_holds,
                    }),
                    max_cases.unwrap_or(descriptor.exact_values.len()),
                ));
            if !property_holds {
                result = result
                    .with_failing_case(case_index)
                    .with_message("generated property oracle failed");
            }
            result.tags = vec!["synthesized".to_string(), "property".to_string()];
            results.push(result);

            if max_cases == Some(generated_count) {
                break;
            }
        }

        if max_cases == Some(generated_count) {
            break;
        }
    }

    results
}

fn is_supported_property_generator(descriptor: &TypeGeneratorDescriptor) -> bool {
    matches!(
        descriptor.source,
        TypeGeneratorSource::AuthoredExamples
            | TypeGeneratorSource::FiniteDomain
            | TypeGeneratorSource::ContractValid
            | TypeGeneratorSource::ContractInvalidNearby
    )
}

fn property_holds_from_generated_value(value: &Value) -> Option<bool> {
    value.get("property_holds").and_then(Value::as_bool)
}

fn deferred_property_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    descriptor: &TypeGeneratorDescriptor,
    seed: u64,
) -> TestResult {
    let reason = descriptor
        .unsupported_reason
        .clone()
        .unwrap_or_else(|| "generator is not an exact supported finite descriptor".to_string());
    let case_id = format!("synthesized/property/{}/deferred", descriptor.id);
    deferred_result_with_kind(
        path,
        TestSource::Contract,
        TestKind::Property,
        case_id,
        format!("deferred: {reason}"),
        ReproArtifact {
            replay_command: format!(
                "ash test {} --only-synthesized contracts --seed {}",
                path.display(),
                seed
            ),
            ..repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("property:{}:deferred", descriptor.id),
                seed,
                1,
                Some(json!({
                    "descriptor_id": descriptor.id,
                    "target_type": descriptor.target_type,
                    "source": descriptor.source,
                    "exact_value_count": descriptor.exact_values.len(),
                })),
                json!({
                    "kind": "metadata_property_holds",
                    "supported": false,
                    "reason": reason,
                }),
                None,
            )
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn property_repro_artifact(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    descriptor: &TypeGeneratorDescriptor,
    seed: u64,
    case_index: usize,
    value: &Value,
    oracle_snapshot: Value,
    replay_max_cases: usize,
) -> ReproArtifact {
    ReproArtifact {
        replay_command: format!(
            "ash test {} --only-synthesized contracts --seed {} --max-cases {}",
            path.display(),
            seed,
            replay_max_cases
        ),
        ..repro_artifact(
            path,
            snapshot.source_artifact_id.clone(),
            snapshot.check_summary_id.clone(),
            format!("synthesized/property/{}/case-{}", descriptor.id, case_index),
            seed,
            case_index,
            Some(json!({
                "descriptor_id": descriptor.id,
                "target_type": descriptor.target_type,
                "source": descriptor.source,
                "value": value,
            })),
            oracle_snapshot,
            None,
        )
    }
}
