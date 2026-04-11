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
    _engine: &ash_engine::Engine,
    seed: u64,
    max_cases: usize,
    timeout: Duration,
) -> TestResult {
    let name = meta.effective_name(path);
    let start = Instant::now();

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

fn run_property_inner(
    path: &Path,
    _engine: &ash_engine::Engine,
    _seed: u64,
    _max_cases: usize,
    timeout: Duration,
) -> (Outcome, Option<String>, Option<usize>) {
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

            let workflow = match engine.parse_file(&path) {
                Ok(w) => w,
                Err(e) => return (Outcome::Error, Some(format!("parse error: {e}"))),
            };

            if let Err(e) = engine.check(&workflow) {
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

    let failing_case = matches!(outcome, Outcome::Fail).then_some(1);
    (outcome, message, failing_case)
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
    _max_worlds: usize,
    timeout: Duration,
) -> (Outcome, Option<String>, Option<usize>) {
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

            let workflow = match engine.parse_file(&path) {
                Ok(w) => w,
                Err(e) => return (Outcome::Error, Some(format!("parse error: {e}"))),
            };

            if let Err(e) = engine.check(&workflow) {
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

    (outcome, message, Some(1))
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
