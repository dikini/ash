//! Test execution: run individual tests with isolation and panic capture.
//!
//! TASK-510: Per-test isolation, panic capture, timeout handling.
//! TASK-512: Authored test execution model.

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::test_runner::discovery::infer_kind_from_path;
use crate::test_runner::metadata::TestMetadata;
use crate::test_runner::orchestration::{
    self, FlakeReport, ShardAssignment, ShardReport, ShardSpec, TestAttempt,
};
use crate::test_runner::quickcheck::{QuickCheckSeedPolicy, source_seed_warning};
use crate::test_runner::synthesized::RunnerIntrospectionSnapshot;
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

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => return (Outcome::Error, Some(format!("source error: {error}"))),
    };
    match crate::test_runner::engine_execution::execute_admitted_source(
        &engine, path, &source, timeout,
    ) {
        Ok(terminal) => crate::test_runner::engine_execution::classify_authored_terminal(&terminal),
        Err(message) => (Outcome::Error, Some(message)),
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
    /// Include obligation-derived tests.
    pub obligations: bool,
    /// Include law-derived tests.
    pub laws: bool,
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
    /// Structured checked/lowered snapshots available to the runner for synthesized execution.
    pub synthesized_snapshots: Vec<(std::path::PathBuf, RunnerIntrospectionSnapshot)>,
    /// Skip all law-derived synthesized tests.
    pub skip_law_tests: bool,
    /// Declared law names or generated law test names to skip.
    pub skip_law_test_names: Vec<String>,
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
    /// Include law/test coverage report.
    pub coverage: bool,
    /// Include bounded mutation report.
    pub mutation: bool,
    /// Maximum mutants to generate.
    pub mutation_limit: usize,
    /// Optional exact mutant id to report/replay.
    pub mutation_id: Option<String>,
    /// Retry failing tests up to this many times.
    pub retries: usize,
    /// Optional deterministic local shard selector.
    pub shard: Option<ShardSpec>,
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
            synthesized_snapshots: Vec::new(),
            skip_law_tests: false,
            skip_law_test_names: Vec::new(),
            fail_fast: false,
            timeout_ms: DEFAULT_TIMEOUT_SECS * 1000,
            seed: None,
            max_cases: None,
            max_worlds: None,
            coverage: false,
            mutation: false,
            mutation_limit: 20,
            mutation_id: None,
            retries: 0,
            shard: None,
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

    // Run authored tests unless --only-synthesized was specified. Law `by test`
    // evidence still needs the authored-test registry even in only-synthesized
    // runs, so build it without appending authored rows when laws are selected.
    let authored_tests = if !config.only_synthesized {
        run_authored_tests(config, &engine, &mut suite)
    } else if config.include_synthesized && config.synthesized_sources.laws {
        collect_authored_test_registry(config, &engine)
    } else {
        BTreeMap::new()
    };

    // Run synthesized tests if requested
    if config.include_synthesized {
        run_synthesized_tests(config, &engine, &mut suite, &authored_tests);
    }

    if config.retries > 0 {
        suite.flake_summary = Some(orchestration::flake_summary(&suite.tests, config.retries));
    }

    if config.coverage || config.mutation {
        if !config.root.exists() {
            suite.add(
                TestResult::new("coverage_root", config.root.clone())
                    .with_outcome(Outcome::Error)
                    .with_message(format!(
                        "coverage/mutation root does not exist: {}",
                        config.root.display()
                    )),
            );
        }
        let snapshots = collect_runner_snapshots(config, &engine, &mut suite);
        let coverage =
            crate::test_runner::coverage_mutation::coverage_report(&snapshots, &authored_tests);
        if config.mutation {
            suite.mutation = Some(crate::test_runner::coverage_mutation::mutation_report(
                &config.root,
                &coverage,
                config.mutation_limit,
                config.mutation_id.as_deref(),
            ));
        }
        if config.coverage {
            suite.coverage = Some(coverage);
        }
    }

    suite.duration = start.elapsed();
    suite
}

fn collect_runner_snapshots(
    config: &SuiteConfig,
    engine: &ash_engine::Engine,
    suite: &mut crate::test_runner::types::TestSuiteResult,
) -> Vec<(std::path::PathBuf, RunnerIntrospectionSnapshot)> {
    use crate::test_runner::synthesized;

    let mut snapshots = Vec::new();
    for path in discover_coverage_sources(&config.root) {
        match synthesized::build_runner_introspection_snapshot(&path, engine) {
            Ok(snapshot) => snapshots.push((path, snapshot)),
            Err(error) => suite.add(
                TestResult::new("coverage_snapshot", path)
                    .with_outcome(Outcome::Error)
                    .with_message(format!("failed to collect law coverage snapshot: {error}")),
            ),
        }
    }
    snapshots
}

fn discover_coverage_sources(root: &Path) -> Vec<std::path::PathBuf> {
    if root.is_file() && root.extension().is_some_and(|ext| ext == "ash") {
        return vec![root.to_path_buf()];
    }

    let mut files = Vec::new();
    if root.is_dir() {
        for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "ash") {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Run authored tests from discovered test files.
fn run_authored_tests(
    config: &SuiteConfig,
    engine: &ash_engine::Engine,
    suite: &mut crate::test_runner::types::TestSuiteResult,
) -> BTreeMap<String, TestResult> {
    let mut registry = BTreeMap::new();
    let files = crate::test_runner::discovery::discover_tests(&config.root);
    let mut selected_count = 0usize;

    for (ordinal, path) in files.iter().enumerate() {
        if let Some(shard) = config.shard
            && !orchestration::shard_contains(shard, ordinal)
        {
            continue;
        }
        selected_count += 1;
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

        if meta.quarantine_malformed {
            suite.add(
                TestResult::new(meta.effective_name(path), path.to_path_buf())
                    .with_outcome(Outcome::Error)
                    .with_message("malformed quarantine metadata: reason is required"),
            );
            if config.fail_fast {
                break;
            }
            continue;
        }

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
        let mut result = execute_test_with_retries(path, &meta, engine, config);
        if let Some(shard) = config.shard {
            result.shard = Some(ShardAssignment {
                index: shard.index,
                total: shard.total,
                ordinal,
            });
        }
        if let Some(reason) = meta.quarantine.clone() {
            orchestration::apply_quarantine(&mut result, reason);
        }
        let outcome = result.outcome;
        insert_authored_registry_result(&mut registry, result.clone(), suite);
        suite.add(result);

        // Fail-fast: stop on first failure
        if config.fail_fast && outcome.is_failure() {
            break;
        }
    }

    if let Some(shard) = config.shard {
        suite.shard = Some(ShardReport::new(shard, files.len(), selected_count));
    }

    registry
}

fn collect_authored_test_registry(
    config: &SuiteConfig,
    engine: &ash_engine::Engine,
) -> BTreeMap<String, TestResult> {
    let mut registry = BTreeMap::new();
    let mut hidden_suite = crate::test_runner::types::TestSuiteResult::new(config.root.clone());
    for path in crate::test_runner::discovery::discover_tests(&config.root) {
        if let Ok(meta) = TestMetadata::parse_from_file(&path) {
            let result = execute_test_by_kind(&path, &meta, engine, config);
            insert_authored_registry_result(&mut registry, result, &mut hidden_suite);
        }
    }
    registry
}

fn insert_authored_registry_result(
    registry: &mut BTreeMap<String, TestResult>,
    result: TestResult,
    suite: &mut crate::test_runner::types::TestSuiteResult,
) {
    if registry.contains_key(&result.name) {
        let duplicate = TestResult::new(result.name.clone(), result.path.clone())
            .with_outcome(Outcome::Error)
            .with_source(TestSource::Authored)
            .with_kind(result.kind)
            .with_message(format!(
                "duplicate authored Ash test name '{}' in test registry",
                result.name
            ));
        registry.insert(result.name.clone(), duplicate.clone());
        suite.add(duplicate);
    } else {
        registry.insert(result.name.clone(), result);
    }
}

/// Execute a test, dispatching to the appropriate handler based on kind.
fn execute_test_with_retries(
    path: &Path,
    meta: &TestMetadata,
    engine: &ash_engine::Engine,
    config: &SuiteConfig,
) -> TestResult {
    let max_attempts = config.retries.saturating_add(1);
    let mut attempts = Vec::new();
    let mut final_result = None;

    for attempt in 1..=max_attempts {
        let result = if meta
            .flaky_until_attempt
            .is_some_and(|passing_attempt| attempt < passing_attempt)
        {
            TestResult::new(meta.effective_name(path), path.to_path_buf())
                .with_outcome(Outcome::Fail)
                .with_message(format!("simulated flaky failure on attempt {attempt}"))
        } else {
            execute_test_by_kind(path, meta, engine, config)
        };
        attempts.push(TestAttempt::from_result(attempt, &result));
        let should_stop = !result.outcome.is_failure() || attempt == max_attempts;
        final_result = Some(result);
        if should_stop {
            break;
        }
    }

    let mut result =
        final_result.unwrap_or_else(|| execute_test_by_kind(path, meta, engine, config));
    if attempts.len() > 1 {
        let had_prior_failure = attempts[..attempts.len() - 1]
            .iter()
            .any(|attempt| matches!(attempt.outcome.as_str(), "fail" | "panic" | "error"));
        let status = if result.outcome == Outcome::Pass && had_prior_failure {
            "flaky"
        } else if result.outcome.is_failure() {
            "stable_failure"
        } else {
            "stable_pass"
        };
        result.flake = Some(FlakeReport {
            schema_version: orchestration::FLAKE_SCHEMA_VERSION.to_string(),
            status: status.to_string(),
            attempts: attempts.len(),
            retries: config.retries,
        });
        result.attempts = attempts;
    }
    result
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
            let seed_policy = QuickCheckSeedPolicy::resolve(config.seed, meta.seed);
            if let Some(warning) = source_seed_warning(meta.seed, seed_policy) {
                eprintln!("warning: {warning}");
            }
            let max_cases = meta
                .max_cases
                .or(config.max_cases)
                .unwrap_or(crate::test_runner::property::DEFAULT_MAX_CASES);
            crate::test_runner::property::execute_property_test(
                path,
                meta,
                engine,
                seed_policy.seed,
                seed_policy.seed_source.as_str(),
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
    engine: &ash_engine::Engine,
    suite: &mut crate::test_runner::types::TestSuiteResult,
    authored_tests: &BTreeMap<String, TestResult>,
) {
    use crate::test_runner::discovery::discover_tests;
    use crate::test_runner::synthesized;

    if !config.synthesized_snapshots.is_empty() {
        for (path, snapshot) in &config.synthesized_snapshots {
            if config.synthesized_sources.laws {
                for result in synthesized::authored_law_test_results(path, snapshot, authored_tests)
                {
                    if !add_synthesized_result(config, suite, result) {
                        return;
                    }
                }
            }
            for result in synthesized::synthesize_from_snapshot_with_engine_limits(
                path,
                snapshot,
                engine,
                config.seed,
                config.max_cases,
                config.max_worlds,
            ) {
                if !add_synthesized_result(config, suite, result) {
                    return;
                }
            }
        }
        return;
    }

    for path in discover_tests(&config.root) {
        match synthesized::build_runner_introspection_snapshot(&path, engine) {
            Ok(snapshot) => {
                if config.synthesized_sources.laws {
                    for result in
                        synthesized::authored_law_test_results(&path, &snapshot, authored_tests)
                    {
                        if !add_synthesized_result(config, suite, result) {
                            return;
                        }
                    }
                }
                for result in synthesized::synthesize_from_snapshot_with_engine_limits(
                    &path,
                    &snapshot,
                    engine,
                    config.seed,
                    config.max_cases,
                    config.max_worlds,
                ) {
                    if !add_synthesized_result(config, suite, result) {
                        return;
                    }
                }
            }
            Err(_) => {
                let source = match std::fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Contract-derived tests
                if config.synthesized_sources.contracts {
                    let contract_tests = synthesized::synthesize_contract_tests(&path, &source);
                    for result in contract_tests {
                        if !add_synthesized_result(config, suite, result) {
                            return;
                        }
                    }
                }

                // Obligation-derived tests
                if config.synthesized_sources.obligations {
                    let obligation_tests = synthesized::synthesize_obligation_tests(&path, &source);
                    for result in obligation_tests {
                        if !add_synthesized_result(config, suite, result) {
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn add_synthesized_result(
    config: &SuiteConfig,
    suite: &mut crate::test_runner::types::TestSuiteResult,
    mut result: TestResult,
) -> bool {
    if !synthesized_result_selected(config, &result) {
        return true;
    }

    apply_synthesized_timeout(config, &mut result);
    let stop = config.fail_fast && result.outcome.is_failure();
    suite.add(result);
    !stop
}

fn apply_synthesized_timeout(config: &SuiteConfig, result: &mut TestResult) {
    let timeout = if config.timeout_ms > 0 {
        Duration::from_millis(config.timeout_ms)
    } else {
        Duration::from_secs(DEFAULT_TIMEOUT_SECS)
    };
    if result.duration > timeout {
        let (outcome, message) = timeout_result(timeout);
        result.outcome = outcome;
        result.message = message;
    }
}

fn synthesized_result_selected(config: &SuiteConfig, result: &TestResult) -> bool {
    if !synthesized_source_enabled(config, result.source) {
        return false;
    }

    if law_result_skipped(config, result) {
        return false;
    }

    if let Some(ref tag) = config.tag_filter
        && !result.tags.iter().any(|candidate| candidate == tag)
    {
        return false;
    }

    if let Some(ref kind) = config.kind_filter
        && parse_kind(kind) != result.kind
    {
        return false;
    }

    true
}

fn law_result_skipped(config: &SuiteConfig, result: &TestResult) -> bool {
    if result.source != TestSource::Law {
        return false;
    }
    if config.skip_law_tests {
        return true;
    }
    if config.skip_law_test_names.is_empty() {
        return false;
    }

    let declared_law_name = result
        .repro_artifact
        .as_ref()
        .and_then(|artifact| artifact.oracle_snapshot.get("law"))
        .and_then(serde_json::Value::as_str);

    config.skip_law_test_names.iter().any(|name| {
        name == &result.name || declared_law_name.is_some_and(|law_name| name == law_name)
    })
}

fn synthesized_source_enabled(config: &SuiteConfig, source: TestSource) -> bool {
    match source {
        TestSource::Contract => config.synthesized_sources.contracts,
        TestSource::Obligation => config.synthesized_sources.obligations,
        TestSource::Law => config.synthesized_sources.laws,
        TestSource::Authored => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::synthesized::{
        ContractExecutableTarget, ContractExecutableTargetKind, ContractExecutionSetup,
        ContractPostconditionOracle, ContractTargetBody, LawScope, RUNNER_SYNTHESIS_SCHEMA_VERSION,
        RunnerContractMetadata, RunnerIntrospectionSnapshot, RunnerLawMetadata,
        SynthesizedOracleKind, TypeGeneratorDescriptor, TypeGeneratorSource,
    };
    use ash_core::{Expr as CoreExpr, Span as CoreSpan};
    use serde_json::json;
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
        fs::write(&file1, "fn test_a() { {} }").unwrap();
        let file2 = test_dir.join("test_b.ash");
        fs::write(&file2, "fn test_b() { {} }").unwrap();

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

    #[test]
    fn run_suite_executes_structured_snapshot_contract_cases_without_raw_source_scan() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            synthesized_snapshots: vec![
                (snapshot_path.clone(), contract_snapshot()),
                (snapshot_path.clone(), law_snapshot()),
            ],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(result.total(), 1, "runner should use the snapshot seam");
        assert!(
            result
                .tests
                .iter()
                .all(|test| test.source == TestSource::Contract && test.outcome == Outcome::Skip),
            "metadata-only contract rows should defer without a source wrapper: {result:#?}"
        );
    }

    #[test]
    fn run_suite_executes_structured_snapshot_law_cases_when_laws_selected() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: false,
                obligations: false,
                laws: true,
            },
            synthesized_snapshots: vec![(snapshot_path, law_snapshot())],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(
            result.total(),
            1,
            "metadata-only laws should produce one deferred row"
        );
        assert!(
            result
                .tests
                .iter()
                .all(|test| test.source == TestSource::Law && test.outcome == Outcome::Skip),
            "law metadata must defer without a TASK-2035 source identity: {result:#?}"
        );
    }

    #[test]
    fn run_suite_skip_law_tests_omits_all_law_rows() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: true,
            },
            synthesized_snapshots: vec![
                (snapshot_path.clone(), contract_snapshot()),
                (snapshot_path, law_snapshot()),
            ],
            skip_law_tests: true,
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(result.total(), 1, "contract rows should remain selected");
        assert!(
            result
                .tests
                .iter()
                .all(|test| test.source != TestSource::Law),
            "--skip-law-tests should omit all law-derived rows: {result:#?}"
        );
    }

    #[test]
    fn run_suite_skip_law_test_name_omits_only_matching_law_rows() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: false,
                obligations: false,
                laws: true,
            },
            synthesized_snapshots: vec![(snapshot_path, two_law_snapshot())],
            skip_law_tests: false,
            skip_law_test_names: vec!["reflexive".to_string()],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(
            result.total(),
            1,
            "only the unskipped second law should remain"
        );
        assert!(
            result
                .tests
                .iter()
                .all(|test| test.name.starts_with("synthesized/identity/")),
            "--skip-law-test=reflexive should omit reflexive law rows only: {result:#?}"
        );
    }

    fn contract_snapshot() -> RunnerIntrospectionSnapshot {
        RunnerIntrospectionSnapshot {
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            module_identity: "test-module".to_string(),
            source_artifact_id: "source:checked-summary.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:positive".to_string(),
                callable_name: "positive".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Int".to_string()],
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![
                    TypeGeneratorDescriptor {
                        id: "x-valid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractValid,
                        exact_values: vec![json!(1)],
                        ..TypeGeneratorDescriptor::default()
                    },
                    TypeGeneratorDescriptor {
                        id: "x-invalid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractInvalidNearby,
                        exact_values: vec![json!(0)],
                        ..TypeGeneratorDescriptor::default()
                    },
                ],
                executable_case_kinds: vec![SynthesizedOracleKind::PreconditionBoundary],
                ..RunnerContractMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        }
    }

    fn law_snapshot() -> RunnerIntrospectionSnapshot {
        RunnerIntrospectionSnapshot {
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            module_identity: "test-module".to_string(),
            source_artifact_id: "source:checked-summary.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            laws: vec![RunnerLawMetadata {
                id: "law:module:reflexive".to_string(),
                name: "reflexive".to_string(),
                scope: LawScope::Module,
                owner: None,
                params: vec!["x: Int".to_string()],
                proposition: "x == x".to_string(),
                delegated_test: None,
                test_evidence: None,
            }],
            ..RunnerIntrospectionSnapshot::default()
        }
    }

    fn two_law_snapshot() -> RunnerIntrospectionSnapshot {
        let mut snapshot = law_snapshot();
        snapshot.laws.push(RunnerLawMetadata {
            id: "law:module:identity".to_string(),
            name: "identity".to_string(),
            scope: LawScope::Module,
            owner: None,
            params: vec!["x: Int".to_string()],
            proposition: "x == x".to_string(),
            delegated_test: None,
            test_evidence: None,
        });
        snapshot
    }

    #[test]
    fn run_suite_executes_structured_snapshot_contract_postconditions_against_target_output() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            synthesized_snapshots: vec![(
                snapshot_path.clone(),
                RunnerIntrospectionSnapshot {
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    module_identity: "test-module".to_string(),
                    source_artifact_id: "source:checked-summary.ash".to_string(),
                    check_summary_id: "check:summary".to_string(),
                    contracts: vec![RunnerContractMetadata {
                        id: "contract:identity".to_string(),
                        callable_name: "identity".to_string(),
                        callable_kind: "pure_function".to_string(),
                        param_names: vec!["x".to_string()],
                        param_types: vec!["Int".to_string()],
                        return_type: Some("Int".to_string()),
                        lowered_requires: vec!["x >= 0".to_string()],
                        lowered_ensures: vec!["result == x".to_string()],
                        executable_postconditions: vec![ContractPostconditionOracle {
                            display: "result == x".to_string(),
                            expression: CoreExpr::Binary {
                                op: ash_core::BinaryOp::Eq,
                                left: Box::new(CoreExpr::Variable {
                                    name: "result".to_string(),
                                    span: CoreSpan::default(),
                                }),
                                right: Box::new(CoreExpr::Variable {
                                    name: "x".to_string(),
                                    span: CoreSpan::default(),
                                }),
                            },
                        }],
                        executable_target: Some(ContractExecutableTarget {
                            kind: ContractExecutableTargetKind::PureFunction,
                            target_ref: "identity".to_string(),
                            setup: ContractExecutionSetup::PureNoSetup,
                            body: ContractTargetBody::ReturnExpression {
                                expression: CoreExpr::Variable {
                                    name: "x".to_string(),
                                    span: CoreSpan::default(),
                                },
                            },
                        }),
                        generation_hints: vec![TypeGeneratorDescriptor {
                            id: "x-valid".to_string(),
                            target_type: "Int".to_string(),
                            source: TypeGeneratorSource::ContractValid,
                            exact_values: vec![json!(7)],
                            ..TypeGeneratorDescriptor::default()
                        }],
                        executable_case_kinds: vec![SynthesizedOracleKind::PostconditionHolds],
                        ..RunnerContractMetadata::default()
                    }],
                    ..RunnerIntrospectionSnapshot::default()
                },
            )],
            ..Default::default()
        };

        let result = run_suite(&config);

        let test = result
            .tests
            .iter()
            .find(|test| test.source == TestSource::Contract)
            .unwrap_or_else(|| panic!("runner should emit a contract deferral: {result:#?}"));
        assert_eq!(test.source, TestSource::Contract);
        assert_eq!(test.outcome, Outcome::Skip);
        assert_eq!(
            test.message.as_deref(),
            Some("deferred: source identity is not in the TASK-2035 catalogue")
        );
        let repro = test
            .repro_artifact
            .as_ref()
            .expect("deferred contract row should include repro artifact");
        assert_eq!(
            repro.oracle_snapshot["execution_route"],
            "catalogue_rejection"
        );
    }

    #[test]
    fn synthesized_snapshot_results_honor_kind_filter() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            kind_filter: Some("property".to_string()),
            synthesized_snapshots: vec![(snapshot_path, mixed_contract_and_property_snapshot())],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(
            result.tests.len(),
            1,
            "the property filter must retain the deferred metadata row: {result:#?}"
        );
        assert!(
            result.tests.iter().all(|test| {
                test.source == TestSource::Contract
                    && test.kind == TestKind::Property
                    && test.outcome == Outcome::Skip
            }),
            "generated property metadata must remain deferred: {result:#?}"
        );
    }

    #[test]
    fn synthesized_snapshot_results_honor_tag_filter() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            tag_filter: Some("property".to_string()),
            synthesized_snapshots: vec![(snapshot_path, mixed_contract_and_property_snapshot())],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(
            result.tests.len(),
            1,
            "the property tag must retain the deferred metadata row: {result:#?}"
        );
        assert!(
            result.tests.iter().all(|test| {
                test.source == TestSource::Contract
                    && test.kind == TestKind::Property
                    && test.outcome == Outcome::Skip
            }),
            "generated property metadata must remain deferred: {result:#?}"
        );
    }

    #[test]
    fn synthesized_result_exceeding_timeout_is_reported_as_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let mut suite = crate::test_runner::types::TestSuiteResult::new(dir.path().to_path_buf());
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            timeout_ms: 1,
            ..Default::default()
        };
        let result = TestResult::new("synthesized/slow", dir.path().join("slow.ash"))
            .with_source(TestSource::Contract)
            .with_kind(TestKind::Unit)
            .with_duration(Duration::from_millis(2))
            .with_outcome(Outcome::Pass);

        assert!(add_synthesized_result(&config, &mut suite, result));
        assert_eq!(suite.tests.len(), 1);
        assert_eq!(suite.tests[0].outcome, Outcome::Error);
        assert!(
            suite.tests[0]
                .message
                .as_deref()
                .is_some_and(|message| message.contains("timed out after 1ms")),
            "synthesized rows that exceed the configured timeout must be classified as timeouts: {suite:#?}"
        );
    }

    #[test]
    fn synthesized_snapshot_results_honor_fail_fast() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot_path = dir.path().join("checked-summary.ash");
        let config = SuiteConfig {
            root: dir.path().to_path_buf(),
            include_synthesized: true,
            only_synthesized: true,
            synthesized_sources: SynthesizedSources {
                contracts: true,
                obligations: false,
                laws: false,
            },
            fail_fast: true,
            synthesized_snapshots: vec![(
                snapshot_path,
                RunnerIntrospectionSnapshot {
                    source_artifact_id: "source:checked-summary.ash".to_string(),
                    check_summary_id: "check:summary".to_string(),
                    generators: vec![TypeGeneratorDescriptor {
                        id: "failing-then-passing-property".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::FiniteDomain,
                        exact_values: vec![
                            json!({ "input": 0, "property_holds": false }),
                            json!({ "input": 1, "property_holds": true }),
                        ],
                        ..TypeGeneratorDescriptor::default()
                    }],
                    ..RunnerIntrospectionSnapshot::default()
                },
            )],
            ..Default::default()
        };

        let result = run_suite(&config);

        assert_eq!(
            result.tests.len(),
            1,
            "a deferred metadata row must not be mistaken for a local failure: {result:#?}"
        );
        assert_eq!(result.tests[0].outcome, Outcome::Skip);
        assert_eq!(result.tests[0].kind, TestKind::Property);
    }

    fn mixed_contract_and_property_snapshot() -> RunnerIntrospectionSnapshot {
        RunnerIntrospectionSnapshot {
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            module_identity: "test-module".to_string(),
            source_artifact_id: "source:checked-summary.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:positive".to_string(),
                callable_name: "positive".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Int".to_string()],
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![
                    TypeGeneratorDescriptor {
                        id: "x-valid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractValid,
                        exact_values: vec![json!(1)],
                        ..TypeGeneratorDescriptor::default()
                    },
                    TypeGeneratorDescriptor {
                        id: "x-invalid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractInvalidNearby,
                        exact_values: vec![json!(0)],
                        ..TypeGeneratorDescriptor::default()
                    },
                ],
                executable_case_kinds: vec![SynthesizedOracleKind::PreconditionBoundary],
                ..RunnerContractMetadata::default()
            }],
            generators: vec![TypeGeneratorDescriptor {
                id: "property-cases".to_string(),
                target_type: "Int".to_string(),
                source: TypeGeneratorSource::FiniteDomain,
                exact_values: vec![json!({ "input": 1, "property_holds": true })],
                ..TypeGeneratorDescriptor::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        }
    }
}
