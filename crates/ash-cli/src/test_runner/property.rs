//! Property and small-world test execution.
//!
//! TASK-514: Bounded, seeded property tests and bounded small-world execution.
//!
//! Both modes are bounded and reproducible:
//! - Property tests: seeded, bounded case count, reports failing case index
//! - Small-world tests: bounded world count/depth, reports world index

use std::path::Path;
use std::time::{Duration, Instant};

use crate::test_runner::metadata::TestMetadata;
use crate::test_runner::types::{Outcome, TestKind, TestResult, TestSource};

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
    engine: &ash_engine::Engine,
    seed: u64,
    _seed_source: &str,
    max_cases: usize,
    timeout: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    let start = Instant::now();

    if meta.property.is_some() || !meta.generated_params.is_empty() {
        return execute_generated_property_metadata(path, meta, seed, start.elapsed());
    }

    let (outcome, message, failing_case) =
        run_property_inner(path, engine, seed, max_cases, timeout);

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
    duration: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(TestSource::Authored)
        .with_kind(TestKind::Property)
        .with_duration(duration)
        .with_seed(seed)
        .with_message("deferred: generated property metadata has no TASK-2035 source identity")
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

                let source = match std::fs::read_to_string(&path) {
                    Ok(source) => source,
                    Err(error) => return (Outcome::Error, Some(format!("source error: {error}"))),
                };
                match crate::test_runner::engine_execution::execute_admitted_source(
                    &engine, &path, &source, timeout,
                ) {
                    Ok(terminal) => {
                        crate::test_runner::engine_execution::classify_authored_terminal(&terminal)
                    }
                    Err(message) => (Outcome::Error, Some(message)),
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

                let source = match std::fs::read_to_string(&path) {
                    Ok(source) => source,
                    Err(error) => return (Outcome::Error, Some(format!("source error: {error}"))),
                };
                match crate::test_runner::engine_execution::execute_admitted_source(
                    &engine, &path, &source, timeout,
                ) {
                    Ok(terminal) => {
                        crate::test_runner::engine_execution::classify_authored_terminal(&terminal)
                    }
                    Err(message) => (Outcome::Error, Some(message)),
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
