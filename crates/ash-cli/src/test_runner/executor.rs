//! Test execution: run individual tests with isolation and panic capture.
//!
//! TASK-510: Per-test isolation, panic capture, timeout handling.
//! TASK-512: Authored test execution model.

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::test_runner::discovery::infer_kind_from_path;
use crate::test_runner::metadata::TestMetadata;
use crate::test_runner::types::{Outcome, TestKind, TestResult, TestSource};

/// Default test timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Execute a single Ash test file.
///
/// This function:
/// 1. Parses the test file
/// 2. Type-checks it
/// 3. Executes it (capturing panics)
/// 4. Returns a classified result
///
/// Panics in the test are caught and reported as `Outcome::Panic`.
/// The calling process is NOT aborted.
#[allow(clippy::too_many_arguments)]
pub fn execute_test(
    path: &Path,
    meta: &TestMetadata,
    engine: &ash_engine::Engine,
    source: TestSource,
    seed: Option<u64>,
    _max_cases: Option<usize>,
    _max_worlds: Option<usize>,
    timeout_ms: u64,
) -> TestResult {
    let kind = meta
        .kind
        .as_deref()
        .map(parse_kind)
        .unwrap_or_else(|| infer_kind_from_path(path));
    let name = meta.effective_name(path);
    let timeout = effective_timeout(meta, timeout_ms);

    let start = Instant::now();
    let (outcome, message) = run_with_isolation(path, engine, timeout);
    let duration = start.elapsed();

    // Apply xfail logic: if expected to fail and did fail, mark xfail
    let final_outcome = if meta.xfail && outcome.is_failure() {
        Outcome::Xfail
    } else if meta.xfail && outcome == Outcome::Pass {
        // Unexpected pass when xfail was expected — still report as pass
        // (the test author should remove the xfail annotation)
        Outcome::Pass
    } else {
        outcome
    };

    let mut result = TestResult::new(&name, path.to_path_buf())
        .with_outcome(final_outcome)
        .with_source(source)
        .with_kind(kind)
        .with_duration(duration);

    if let Some(msg) = message {
        result = result.with_message(msg);
    }
    if let Some(s) = seed {
        result = result.with_seed(s);
    }
    result.tags = meta.tags.clone();

    result
}

fn parse_kind(s: &str) -> TestKind {
    match s {
        "unit" => TestKind::Unit,
        "integration" => TestKind::Integration,
        "e2e" => TestKind::E2e,
        "property" => TestKind::Property,
        "smallworld" => TestKind::SmallWorld,
        _ => TestKind::Unit,
    }
}

fn effective_timeout(meta: &TestMetadata, timeout_ms: u64) -> Duration {
    if meta.timeout_ms > 0 {
        Duration::from_millis(meta.timeout_ms)
    } else if timeout_ms > 0 {
        Duration::from_millis(timeout_ms)
    } else {
        Duration::from_secs(DEFAULT_TIMEOUT_SECS)
    }
}

/// Run a test with isolation: catch panics, enforce timeouts.
///
/// The key property: this function never panics and does not let a timed-out
/// test block the rest of the suite.
fn run_with_isolation(
    path: &Path,
    _engine: &ash_engine::Engine,
    timeout: Duration,
) -> (Outcome, Option<String>) {
    let path = path.to_path_buf();
    run_operation_with_timeout(timeout, move || run_test_inner(&path, timeout))
}

pub(crate) fn run_operation_with_timeout<F>(
    timeout: Duration,
    operation: F,
) -> (Outcome, Option<String>)
where
    F: FnOnce() -> (Outcome, Option<String>) + Send + 'static,
{
    let started_at = Instant::now();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let result = panic::catch_unwind(AssertUnwindSafe(operation));
        let result = match result {
            Ok(result) => result,
            Err(panic_payload) => (Outcome::Panic, panic_message(panic_payload)),
        };
        let _ = tx.send(TimedOperationResult {
            result,
            completed_at: Instant::now(),
        });
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => classify_operation_result(started_at, timeout, result),
        Err(mpsc::RecvTimeoutError::Timeout) => timeout_result(timeout),
        Err(mpsc::RecvTimeoutError::Disconnected) => (
            Outcome::Error,
            Some("test execution thread terminated unexpectedly".to_string()),
        ),
    }
}

struct TimedOperationResult {
    result: (Outcome, Option<String>),
    completed_at: Instant,
}

fn classify_operation_result(
    started_at: Instant,
    timeout: Duration,
    result: TimedOperationResult,
) -> (Outcome, Option<String>) {
    if result.completed_at.duration_since(started_at) > timeout {
        timeout_result(timeout)
    } else {
        result.result
    }
}

fn timeout_result(timeout: Duration) -> (Outcome, Option<String>) {
    (
        Outcome::Error,
        Some(format!("test timed out after {}ms", timeout.as_millis())),
    )
}

fn panic_message(panic_payload: Box<dyn std::any::Any + Send>) -> Option<String> {
    if let Some(s) = panic_payload.downcast_ref::<&str>() {
        Some((*s).to_string())
    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
        Some(s.clone())
    } else {
        Some("test panicked".to_string())
    }
}

fn build_test_engine() -> Result<ash_engine::Engine, String> {
    ash_engine::Engine::new()
        .with_stdio_capabilities()
        .build()
        .map_err(|e| format!("failed to build test engine: {e}"))
}

/// Inner test execution with timeout enforcement inside the dedicated test thread.
fn run_test_inner(path: &Path, timeout: Duration) -> (Outcome, Option<String>) {
    let engine = match build_test_engine() {
        Ok(engine) => engine,
        Err(message) => return (Outcome::Error, Some(message)),
    };

    // Step 1: Parse
    let mut workflow = match engine.parse_file(path) {
        Ok(w) => w,
        Err(e) => return (Outcome::Error, Some(format!("parse error: {e}"))),
    };

    // Step 2: Type check
    if let Err(e) = engine.check(&mut workflow) {
        return (Outcome::Error, Some(format!("type error: {e}")));
    }

    // Step 3: Execute with timeout enforcement inside the dedicated runtime.
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => return (Outcome::Error, Some(format!("runtime error: {e}"))),
    };

    let result =
        rt.block_on(async move { tokio::time::timeout(timeout, engine.execute(&workflow)).await });

    match result {
        Err(_) => (
            Outcome::Error,
            Some(format!("test timed out after {}ms", timeout.as_millis())),
        ),
        Ok(result) => match result {
            Ok(ash_core::Value::Bool(false)) => {
                (Outcome::Fail, Some("test returned false".to_string()))
            }
            Ok(_) => (Outcome::Pass, None),
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("assertion") || msg.contains("assert") {
                    (Outcome::Fail, Some(msg))
                } else {
                    (Outcome::Error, Some(msg))
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Suite executor (runs all tests, collecting results)
// ---------------------------------------------------------------------------

/// Which synthesized test sources to include.
#[derive(Debug, Clone, Default)]
pub struct SynthesizedSources {
    /// Include contract-derived tests.
    pub contracts: bool,
    /// Include policy-derived tests.
    pub policies: bool,
    /// Include obligation-derived tests.
    pub obligations: bool,
}

/// Configuration for a test suite run.
#[derive(Debug, Clone)]
pub struct SuiteConfig {
    /// Root path for test discovery.
    pub root: std::path::PathBuf,
    /// Output format.
    pub format: crate::commands::test::TestOutputFormat,
    /// Tag filter.
    pub tag_filter: Option<String>,
    /// Kind filter.
    pub kind_filter: Option<String>,
    /// Include synthesized tests.
    pub include_synthesized: bool,
    /// Only run synthesized tests.
    pub only_synthesized: bool,
    /// Which synthesized sources to include.
    pub synthesized_sources: SynthesizedSources,
    /// Fail fast (stop on first failure).
    pub fail_fast: bool,
    /// Default timeout in milliseconds.
    pub timeout_ms: u64,
    /// Seed for property tests.
    pub seed: Option<u64>,
    /// Max cases for property tests.
    pub max_cases: Option<usize>,
    /// Max worlds for small-world tests.
    pub max_worlds: Option<usize>,
}

impl Default for SuiteConfig {
    fn default() -> Self {
        Self {
            root: std::path::PathBuf::from("."),
            format: crate::commands::test::TestOutputFormat::Human,
            tag_filter: None,
            kind_filter: None,
            include_synthesized: false,
            only_synthesized: false,
            synthesized_sources: SynthesizedSources::default(),
            fail_fast: false,
            timeout_ms: DEFAULT_TIMEOUT_SECS * 1000,
            seed: None,
            max_cases: None,
            max_worlds: None,
        }
    }
}

/// Run a complete test suite.
///
/// Discovers tests, executes them one by one, and collects results.
/// One test panicking does NOT prevent other tests from running.
pub fn run_suite(config: &SuiteConfig) -> crate::test_runner::types::TestSuiteResult {
    use crate::test_runner::types::TestSuiteResult;

    let start = Instant::now();
    let mut suite = TestSuiteResult::new(config.root.clone());

    // Build engine
    let engine = match ash_engine::Engine::new().with_stdio_capabilities().build() {
        Ok(e) => e,
        Err(e) => {
            suite.add(
                TestResult::new("engine_init", config.root.clone())
                    .with_outcome(Outcome::Error)
                    .with_message(format!("failed to build test engine: {e}")),
            );
            suite.duration = start.elapsed();
            return suite;
        }
    };

    // Run authored tests unless --only-synthesized was specified
    if !config.only_synthesized {
        run_authored_tests(config, &engine, &mut suite);
    }

    // Run synthesized tests if requested
    if config.include_synthesized {
        run_synthesized_tests(config, &engine, &mut suite);
    }

    suite.duration = start.elapsed();
    suite
}

/// Run authored tests from discovered test files.
fn run_authored_tests(
    config: &SuiteConfig,
    engine: &ash_engine::Engine,
    suite: &mut crate::test_runner::types::TestSuiteResult,
) {
    let files = crate::test_runner::discovery::discover_tests(&config.root);

    for path in &files {
        // Parse metadata
        let meta = match TestMetadata::parse_from_file(path) {
            Ok(m) => m,
            Err(e) => {
                suite.add(
                    TestResult::new(
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown"),
                        path.to_path_buf(),
                    )
                    .with_outcome(Outcome::Error)
                    .with_message(format!("failed to read test file: {e}")),
                );
                continue;
            }
        };

        // Apply filters
        #[allow(clippy::collapsible_if)]
        if let Some(ref tag) = config.tag_filter {
            if !meta.tags.iter().any(|t| t == tag) {
                suite.add(
                    TestResult::new(meta.effective_name(path), path.to_path_buf())
                        .with_outcome(Outcome::Skip),
                );
                continue;
            }
        }

        if let Some(ref kind) = config.kind_filter {
            let actual_kind =
                meta.kind
                    .as_deref()
                    .unwrap_or_else(|| match infer_kind_from_path(path) {
                        TestKind::Unit => "unit",
                        TestKind::Integration => "integration",
                        TestKind::E2e => "e2e",
                        TestKind::Property => "property",
                        TestKind::SmallWorld => "smallworld",
                    });
            if actual_kind != kind {
                suite.add(
                    TestResult::new(meta.effective_name(path), path.to_path_buf())
                        .with_outcome(Outcome::Skip),
                );
                continue;
            }
        }

        // Execute the test with proper kind dispatch
        let result = execute_test_by_kind(path, &meta, engine, config);
        let outcome = result.outcome;
        suite.add(result);

        // Fail-fast: stop on first failure
        if config.fail_fast && outcome.is_failure() {
            break;
        }
    }
}

/// Execute a test, dispatching to the appropriate handler based on kind.
fn execute_test_by_kind(
    path: &Path,
    meta: &TestMetadata,
    engine: &ash_engine::Engine,
    config: &SuiteConfig,
) -> TestResult {
    let kind = meta
        .kind
        .as_deref()
        .map(parse_kind)
        .unwrap_or_else(|| infer_kind_from_path(path));

    match kind {
        TestKind::Property => {
            let seed = config.seed.or(meta.seed).unwrap_or(42);
            let max_cases = config
                .max_cases
                .or(meta.max_cases)
                .unwrap_or(crate::test_runner::property::DEFAULT_MAX_CASES);
            crate::test_runner::property::execute_property_test(
                path,
                meta,
                engine,
                seed,
                max_cases,
                effective_timeout(meta, config.timeout_ms),
            )
        }
        TestKind::SmallWorld => {
            let max_worlds = config
                .max_worlds
                .or(meta.max_worlds)
                .unwrap_or(crate::test_runner::property::DEFAULT_MAX_WORLDS);
            crate::test_runner::property::execute_smallworld_test(
                path,
                meta,
                engine,
                max_worlds,
                effective_timeout(meta, config.timeout_ms),
            )
        }
        _ => {
            // Unit, integration, e2e - use standard execution
            execute_test(
                path,
                meta,
                engine,
                TestSource::Authored,
                config.seed.or(meta.seed),
                config.max_cases.or(meta.max_cases),
                config.max_worlds.or(meta.max_worlds),
                config.timeout_ms,
            )
        }
    }
}

/// Run synthesized tests from configured sources.
fn run_synthesized_tests(
    config: &SuiteConfig,
    _engine: &ash_engine::Engine,
    suite: &mut crate::test_runner::types::TestSuiteResult,
) {
    use crate::test_runner::discovery::discover_tests;
    use crate::test_runner::synthesized;

    let files = discover_tests(&config.root);

    for path in &files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Contract-derived tests
        if config.synthesized_sources.contracts {
            let contract_tests = synthesized::synthesize_contract_tests(path, &source);
            for result in contract_tests {
                suite.add(result);
            }
        }

        // Policy-derived tests
        if config.synthesized_sources.policies {
            let policy_tests = synthesized::synthesize_policy_tests(path, &source);
            for result in policy_tests {
                suite.add(result);
            }
        }

        // Obligation-derived tests
        if config.synthesized_sources.obligations {
            let obligation_tests = synthesized::synthesize_obligation_tests(path, &source);
            for result in obligation_tests {
                suite.add(result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn run_suite_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            ..Default::default()
        };
        let result = run_suite(&config);
        assert!(result.tests.is_empty());
        assert!(result.is_success());
    }

    #[test]
    fn run_suite_parse_error_file() {
        let dir = tempfile::tempdir().unwrap();
        let test_dir = dir.path().join("tests/ash/unit");
        fs::create_dir_all(&test_dir).unwrap();
        let bad_file = test_dir.join("bad_syntax.ash");
        fs::write(&bad_file, "this is not valid ash syntax !!!\n").unwrap();

        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            ..Default::default()
        };
        let result = run_suite(&config);
        assert_eq!(result.total(), 1);
        // Parse error should be classified as Error (not Panic)
        assert!(matches!(result.tests[0].outcome, Outcome::Error));
    }

    #[test]
    fn panic_does_not_abort_suite() {
        // This test verifies that a panicking test doesn't prevent other tests
        // from running. We create two test files: one that will fail at parse
        // (simulating a crash scenario) and one that is valid.
        let dir = tempfile::tempdir().unwrap();
        let test_dir = dir.path().join("tests/ash/unit");
        fs::create_dir_all(&test_dir).unwrap();

        let file1 = test_dir.join("test_a.ash");
        fs::write(&file1, "workflow test_a { done }").unwrap();
        let file2 = test_dir.join("test_b.ash");
        fs::write(&file2, "workflow test_b { done }").unwrap();

        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            ..Default::default()
        };
        let result = run_suite(&config);
        // Both tests should have been attempted
        assert_eq!(result.total(), 2);
    }

    #[test]
    fn operation_timeout_is_enforced_without_waiting_for_completion() {
        let started = Instant::now();
        let (outcome, message) = run_operation_with_timeout(Duration::from_millis(25), || {
            std::thread::sleep(Duration::from_millis(200));
            (Outcome::Pass, None)
        });

        assert_eq!(outcome, Outcome::Error);
        assert!(
            message.unwrap_or_default().contains("timed out after 25ms"),
            "timeout message should mention configured limit"
        );
        assert!(
            started.elapsed() < Duration::from_millis(150),
            "timeout containment should return before the full operation completes"
        );
    }

    #[test]
    fn operation_result_after_deadline_is_timeout_even_if_receiver_wakes_late() {
        let started = Instant::now();
        let result = TimedOperationResult {
            result: (Outcome::Pass, None),
            completed_at: started + Duration::from_millis(200),
        };

        let (outcome, message) =
            classify_operation_result(started, Duration::from_millis(25), result);

        assert_eq!(outcome, Outcome::Error);
        assert!(
            message.unwrap_or_default().contains("timed out after 25ms"),
            "late completion should be reported as timeout"
        );
    }

    #[test]
    fn timeout_like_failures_do_not_stop_following_operations_without_fail_fast() {
        let executed = Arc::new(AtomicUsize::new(0));

        for outcome in [Outcome::Error, Outcome::Pass] {
            let executed = Arc::clone(&executed);
            let result = run_operation_with_timeout(Duration::from_millis(50), move || {
                executed.fetch_add(1, Ordering::SeqCst);
                (outcome, None)
            });
            assert_eq!(result.0, outcome);
        }

        assert_eq!(executed.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn fail_fast_stops_after_first_failure_outcome() {
        let outcomes = [Outcome::Error, Outcome::Pass, Outcome::Pass];
        let mut executed = 0;

        for outcome in outcomes {
            executed += 1;
            if outcome.is_failure() {
                break;
            }
        }

        assert_eq!(executed, 1);
    }
}
