//! Property and small-world test execution.
//!
//! TASK-514: Bounded, seeded property tests and bounded small-world execution.
//!
//! Both modes are bounded and reproducible:
//! - Property tests: seeded, bounded case count, reports failing case index
//! - Small-world tests: bounded world count/depth, reports world index

use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::test_runner::metadata::TestMetadata;
use crate::test_runner::synthesized::eval::evaluate_simple_bool_expression;
use crate::test_runner::synthesized::value_generation::{
    generated_cases, generated_domain_for_param, shrink_bindings,
};
use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult, TestSource};

/// Default number of property test cases.
pub const DEFAULT_MAX_CASES: usize = 100;

/// Default number of small-world worlds.
pub const DEFAULT_MAX_WORLDS: usize = 50;

/// Execute a property test.
///
/// Property tests run the test body multiple times with different inputs,
/// controlled by a seed for reproducibility. The runner controls case count.
///
/// V1: Property tests are executed by interpreting the `.ash` file which
/// contains the property logic. The file is run once; future iterations will
/// support automatic input generation.
pub fn execute_property_test(
    path: &Path,
    meta: &TestMetadata,
    _engine: &ash_engine::Engine,
    seed: u64,
    max_cases: usize,
    timeout: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    let start = Instant::now();

    if meta.property.is_some() || !meta.generated_params.is_empty() {
        return execute_generated_property_metadata(path, meta, seed, max_cases, start.elapsed());
    }

    let (outcome, message, failing_case) =
        run_property_inner(path, _engine, seed, max_cases, timeout);

    let mut result = TestResult::new(&name, path.to_path_buf())
        .with_outcome(outcome)
        .with_source(TestSource::Authored)
        .with_kind(TestKind::Property)
        .with_duration(start.elapsed())
        .with_seed(seed);
    if let Some(failing_case) = failing_case {
        result = result.with_failing_case(failing_case);
    }
    if let Some(message) = message {
        result = result.with_message(message);
    }
    result
}

fn execute_generated_property_metadata(
    path: &Path,
    meta: &TestMetadata,
    seed: u64,
    max_cases: usize,
    duration: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    let Some(property) = meta.property.as_deref() else {
        return generated_property_error(
            name,
            path,
            seed,
            "invalid generated property test: @test property is required when @test params is present",
        );
    };

    let domains = match meta
        .generated_params
        .iter()
        .map(|param| generated_domain_for_param(param))
        .collect::<Option<Vec<_>>>()
    {
        Some(domains) => domains,
        None => {
            return generated_property_error(
                name,
                path,
                seed,
                "invalid generated property test: unsupported @test params type domain",
            );
        }
    };
    let cases = generated_cases(&domains, max_cases);
    if cases.is_empty() {
        return generated_property_error(
            name,
            path,
            seed,
            "invalid generated property test: max_cases produced no generated inputs",
        );
    }

    for case in cases {
        let outcome = match evaluate_simple_bool_expression(property, &case.bindings) {
            Ok(true) => Outcome::Pass,
            Ok(false) => Outcome::Fail,
            Err(error) => {
                return generated_property_error(
                    name,
                    path,
                    seed,
                    &format!("invalid generated property oracle: {error}"),
                );
            }
        };
        if outcome == Outcome::Fail {
            let shrunk = shrink_bindings(&case.bindings, |candidate| {
                evaluate_simple_bool_expression(property, candidate) == Ok(false)
            });
            let snapshot = json!({
                "bindings": case.bindings,
                "generators": case.generators,
                "shrunk_counterexample": shrunk.bindings,
                "shrink_trace": shrunk.trace,
            });
            let mut result = TestResult::new(name, path.to_path_buf())
                .with_outcome(Outcome::Fail)
                .with_source(TestSource::Authored)
                .with_kind(TestKind::Property)
                .with_duration(duration)
                .with_seed(seed)
                .with_failing_case(case.case_index)
                .with_message(format!(
                    "generated property counterexample at seed {seed}, case {}: {}; shrunk: {}",
                    case.case_index, snapshot["bindings"], snapshot["shrunk_counterexample"]
                ));
            result.repro_artifact = Some(property_repro_artifact(
                path,
                seed,
                case.case_index,
                property,
                Some(snapshot),
            ));
            return result;
        }
    }

    let mut result = TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Pass)
        .with_source(TestSource::Authored)
        .with_kind(TestKind::Property)
        .with_duration(duration)
        .with_seed(seed)
        .with_message(format!(
            "generated property passed {max_cases} bounded cases from @test params"
        ));
    result.repro_artifact = Some(property_repro_artifact(
        path, seed, max_cases, property, None,
    ));
    result
}

fn generated_property_error(name: String, path: &Path, seed: u64, message: &str) -> TestResult {
    let mut result = TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Error)
        .with_source(TestSource::Authored)
        .with_kind(TestKind::Property)
        .with_seed(seed)
        .with_message(message.to_string());
    result.repro_artifact = Some(property_repro_artifact(path, seed, 1, "<invalid>", None));
    result
}

fn property_repro_artifact(
    path: &Path,
    seed: u64,
    case_index: usize,
    property: &str,
    generated_input_snapshot: Option<Value>,
) -> ReproArtifact {
    ReproArtifact {
        runner_schema_version: "ash-property-generation-v1.0".to_string(),
        source_artifact_id: path.display().to_string(),
        check_summary_id: "authored-property-metadata".to_string(),
        case_id: format!("authored-property:{}:{case_index}", path.display()),
        seed,
        case_index,
        world_index: None,
        generated_input_snapshot,
        world_snapshot: None,
        oracle_snapshot: json!({
            "source": "authored_property",
            "property": property,
            "expected": true,
        }),
        replay_command: format!(
            "ASH_UNDER_TEST=${{ASH_UNDER_TEST:?set Ash candidate binary}}; \\\"$ASH_UNDER_TEST\\\" test {} --seed {seed} --max-cases {case_index}",
            path.display()
        ),
    }
}

fn run_property_inner(
    path: &Path,
    _engine: &ash_engine::Engine,
    _seed: u64,
    max_cases: usize,
    timeout: Duration,
) -> (Outcome, Option<String>, Option<usize>) {
    let total_cases = max_cases.max(1);

    for case_index in 1..=total_cases {
        let path = path.to_path_buf();
        let (outcome, message) =
            crate::test_runner::executor::run_operation_with_timeout(timeout, move || {
                let engine = match ash_engine::Engine::new().with_stdio_capabilities().build() {
                    Ok(engine) => engine,
                    Err(e) => {
                        return (
                            Outcome::Error,
                            Some(format!("failed to build test engine: {e}")),
                        );
                    }
                };

                let mut workflow = match engine.parse_file(&path) {
                    Ok(w) => w,
                    Err(e) => return (Outcome::Error, Some(format!("parse error: {e}"))),
                };

                if let Err(e) = engine.check(&mut workflow) {
                    return (Outcome::Error, Some(format!("type error: {e}")));
                }

                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => return (Outcome::Error, Some(format!("runtime error: {e}"))),
                };

                let result = rt.block_on(async move {
                    tokio::time::timeout(timeout, engine.execute(&workflow)).await
                });
                match result {
                    Err(_) => (
                        Outcome::Error,
                        Some(format!("test timed out after {}ms", timeout.as_millis())),
                    ),
                    Ok(Ok(ash_core::Value::Bool(false))) => {
                        (Outcome::Fail, Some("test returned false".to_string()))
                    }
                    Ok(Ok(_)) => (Outcome::Pass, None),
                    Ok(Err(e)) => {
                        let msg = format!("{e}");
                        if msg.contains("assert") {
                            (Outcome::Fail, Some(msg))
                        } else {
                            (Outcome::Error, Some(msg))
                        }
                    }
                }
            });

        if outcome.is_failure() {
            return (outcome, message, Some(case_index));
        }
    }

    (Outcome::Pass, None, None)
}

/// Execute a small-world test.
///
/// Small-world tests explore a bounded state space with bounded depth.
/// The runner controls world count and reports which world failed.
///
/// V1: Single execution with world-limit reporting.
pub fn execute_smallworld_test(
    path: &Path,
    meta: &TestMetadata,
    _engine: &ash_engine::Engine,
    max_worlds: usize,
    timeout: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    let start = Instant::now();

    let (outcome, message, world_idx) = run_smallworld_inner(path, _engine, max_worlds, timeout);

    let mut result = TestResult::new(&name, path.to_path_buf())
        .with_outcome(outcome)
        .with_source(TestSource::Authored)
        .with_kind(TestKind::SmallWorld)
        .with_duration(start.elapsed());
    result.world_index = world_idx;
    if let Some(msg) = message {
        result = result.with_message(msg);
    }
    result
}

fn run_smallworld_inner(
    path: &Path,
    _engine: &ash_engine::Engine,
    max_worlds: usize,
    timeout: Duration,
) -> (Outcome, Option<String>, Option<usize>) {
    let total_worlds = max_worlds.max(1);

    for world_index in 1..=total_worlds {
        let path = path.to_path_buf();
        let (outcome, message) =
            crate::test_runner::executor::run_operation_with_timeout(timeout, move || {
                let engine = match ash_engine::Engine::new().with_stdio_capabilities().build() {
                    Ok(engine) => engine,
                    Err(e) => {
                        return (
                            Outcome::Error,
                            Some(format!("failed to build test engine: {e}")),
                        );
                    }
                };

                let mut workflow = match engine.parse_file(&path) {
                    Ok(w) => w,
                    Err(e) => return (Outcome::Error, Some(format!("parse error: {e}"))),
                };

                if let Err(e) = engine.check(&mut workflow) {
                    return (Outcome::Error, Some(format!("type error: {e}")));
                }

                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => return (Outcome::Error, Some(format!("runtime error: {e}"))),
                };

                let result = rt.block_on(async move {
                    tokio::time::timeout(timeout, engine.execute(&workflow)).await
                });
                match result {
                    Err(_) => (
                        Outcome::Error,
                        Some(format!("test timed out after {}ms", timeout.as_millis())),
                    ),
                    Ok(Ok(ash_core::Value::Bool(false))) => {
                        (Outcome::Fail, Some("test returned false".to_string()))
                    }
                    Ok(Ok(_)) => (Outcome::Pass, None),
                    Ok(Err(e)) => {
                        let msg = format!("{e}");
                        if msg.contains("assert") {
                            (Outcome::Fail, Some(msg))
                        } else {
                            (Outcome::Error, Some(msg))
                        }
                    }
                }
            });

        if outcome.is_failure() {
            return (outcome, message, Some(world_index));
        }
    }

    (Outcome::Pass, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Property and small-world tests require actual .ash files and the engine,
    // so we test the configuration/bound defaults here.

    #[test]
    fn default_max_cases() {
        assert_eq!(DEFAULT_MAX_CASES, 100);
    }

    #[test]
    fn default_max_worlds() {
        assert_eq!(DEFAULT_MAX_WORLDS, 50);
    }
}
