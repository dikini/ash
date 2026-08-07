//! Ash test runner types and result model.
//!
//! TASK-509: Defines the canonical test result types used by `ash test`.
//! TASK-510: Sealed result classification (pass/fail/panic/error/skip/xfail).

use serde::Serialize;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crate::test_runner::coverage_mutation::{LawCoverageReport, MutationReport};
use crate::test_runner::orchestration::{
    FlakeReport, FlakeSummary, QuarantineReport, ShardAssignment, ShardReport,
};

// ---------------------------------------------------------------------------
// Outcome classification (TASK-510: sealed result classification)
// ---------------------------------------------------------------------------

/// The outcome of a single test case.
///
/// These are the only valid outcomes. No other classification is possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Test passed.
    Pass,
    /// Test failed (assertion violated or explicit failure).
    Fail,
    /// Test panicked (the runner caught the panic; the suite continues).
    Panic,
    /// Test encountered an infrastructure error (parse failure, type error, etc.).
    Error,
    /// Test was skipped (filtered out, missing capability, etc.).
    Skip,
    /// Test was expected to fail and did fail.
    Xfail,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Pass => write!(f, "PASS"),
            Outcome::Fail => write!(f, "FAIL"),
            Outcome::Panic => write!(f, "PANIC"),
            Outcome::Error => write!(f, "ERROR"),
            Outcome::Skip => write!(f, "SKIP"),
            Outcome::Xfail => write!(f, "XFAIL"),
        }
    }
}

impl Outcome {
    /// Returns true if this outcome is a passing result.
    pub fn is_pass(&self) -> bool {
        matches!(self, Outcome::Pass | Outcome::Xfail | Outcome::Skip)
    }

    /// Returns true if this outcome represents a problem.
    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::Fail | Outcome::Panic | Outcome::Error)
    }
}

// ---------------------------------------------------------------------------
// Test source classification
// ---------------------------------------------------------------------------

/// Whether a test was authored by a human or synthesized from metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestSource {
    /// Authored by a human (discovered from test files).
    Authored,
    /// Synthesized from contract metadata.
    Contract,
    /// Synthesized from obligation metadata.
    Obligation,
    /// Synthesized from law metadata.
    Law,
}

impl fmt::Display for TestSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestSource::Authored => write!(f, "authored"),
            TestSource::Contract => write!(f, "synthesized:contract"),
            TestSource::Obligation => write!(f, "synthesized:obligation"),
            TestSource::Law => write!(f, "synthesized:law"),
        }
    }
}

// ---------------------------------------------------------------------------
// Test kind classification
// ---------------------------------------------------------------------------

/// The kind of test (unit, integration, e2e, property, small-world).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    /// Unit test.
    Unit,
    /// Integration test.
    Integration,
    /// End-to-end test.
    E2e,
    /// Property-based test.
    Property,
    /// Small-world fuzzing test.
    SmallWorld,
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestKind::Unit => write!(f, "unit"),
            TestKind::Integration => write!(f, "integration"),
            TestKind::E2e => write!(f, "e2e"),
            TestKind::Property => write!(f, "property"),
            TestKind::SmallWorld => write!(f, "smallworld"),
        }
    }
}

// ---------------------------------------------------------------------------
// Single test result
// ---------------------------------------------------------------------------

/// Reproducible context for a generated or executed synthesized case.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReproArtifact {
    /// Runner schema version used for this case.
    pub runner_schema_version: String,
    /// Stable identity for the source artifact used to build the case.
    pub source_artifact_id: String,
    /// Stable identity for the checked/lowered summary consumed by the runner.
    pub check_summary_id: String,
    /// Stable synthesized case id.
    pub case_id: String,
    /// Seed used for deterministic generation.
    pub seed: u64,
    /// Generated case index, starting at 1.
    pub case_index: usize,
    /// Small-world index, starting at 1, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_index: Option<usize>,
    /// Canonical generated input snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_input_snapshot: Option<serde_json::Value>,
    /// Canonical world snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_snapshot: Option<serde_json::Value>,
    /// Canonical oracle snapshot.
    pub oracle_snapshot: serde_json::Value,
    /// Command hint for replaying or reselecting this synthesized source.
    pub replay_command: String,
}

/// Canonical result record for a single test.
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    /// Unique test name (file path + test function name).
    pub name: String,
    /// File path where the test was defined.
    pub path: PathBuf,
    /// The outcome of this test.
    pub outcome: Outcome,
    /// Whether this test was authored or synthesized.
    pub source: TestSource,
    /// The kind of test.
    pub kind: TestKind,
    /// Execution duration.
    #[serde(with = "duration_ms")]
    pub duration: Duration,
    /// Human-readable message (error message, assertion detail, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Tags associated with this test.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// For property tests: the seed used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// For property tests: the failing case index (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_case: Option<usize>,
    /// For small-world tests: the world index (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_index: Option<usize>,
    /// Reproducible context for generated or executed synthesized rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repro_artifact: Option<ReproArtifact>,
    /// Proof evidence family, when this row reports law proof evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_family: Option<String>,
    /// Law test evidence mode (`authored`, `property`, `small_world`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_mode: Option<String>,
    /// Law proof evidence status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_status: Option<String>,
    /// Execution attempts when retries were requested or used.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<crate::test_runner::orchestration::TestAttempt>,
    /// Flake classification when retry evidence exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<FlakeReport>,
    /// Quarantine metadata when the row is quarantined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarantine: Option<QuarantineReport>,
    /// Shard assignment when local sharding is requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<ShardAssignment>,
}

mod duration_ms {
    use serde::Serializer;
    use std::time::Duration;

    pub fn serialize<S: Serializer>(dur: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(dur.as_secs_f64() * 1000.0)
    }
}

impl TestResult {
    /// Create a new test result builder.
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            outcome: Outcome::Pass,
            source: TestSource::Authored,
            kind: TestKind::Unit,
            duration: Duration::ZERO,
            message: None,
            tags: Vec::new(),
            seed: None,
            failing_case: None,
            world_index: None,
            repro_artifact: None,
            evidence_family: None,
            test_mode: None,
            evidence_status: None,
            attempts: Vec::new(),
            flake: None,
            quarantine: None,
            shard: None,
        }
    }

    /// Set the outcome.
    pub fn with_outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set the source.
    pub fn with_source(mut self, source: TestSource) -> Self {
        self.source = source;
        self
    }

    /// Set the kind.
    pub fn with_kind(mut self, kind: TestKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the message.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the failing case index.
    pub fn with_failing_case(mut self, case: usize) -> Self {
        self.failing_case = Some(case);
        self
    }

    /// Set the reproducible artifact context.
    pub fn with_repro_artifact(mut self, artifact: ReproArtifact) -> Self {
        self.repro_artifact = Some(artifact);
        self
    }
}

// ---------------------------------------------------------------------------
// Suite result
// ---------------------------------------------------------------------------

/// Canonical result record for a complete test suite run.
#[derive(Debug, Clone, Serialize)]
pub struct TestSuiteResult {
    /// The root path that was tested.
    pub root: PathBuf,
    /// All individual test results.
    pub tests: Vec<TestResult>,
    /// Total suite duration.
    #[serde(with = "duration_ms")]
    pub duration: Duration,
    /// Optional law/test coverage report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<LawCoverageReport>,
    /// Optional bounded mutation report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<MutationReport>,
    /// Optional retry/flake summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake_summary: Option<FlakeSummary>,
    /// Optional shard execution report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<ShardReport>,
}

impl TestSuiteResult {
    /// Create a new suite result.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            tests: Vec::new(),
            duration: Duration::ZERO,
            coverage: None,
            mutation: None,
            flake_summary: None,
            shard: None,
        }
    }

    /// Add a test result.
    pub fn add(&mut self, result: TestResult) {
        self.tests.push(result);
    }

    /// Count tests with the given outcome.
    pub fn count(&self, outcome: Outcome) -> usize {
        self.tests.iter().filter(|t| t.outcome == outcome).count()
    }

    /// Total number of tests.
    pub fn total(&self) -> usize {
        self.tests.len()
    }

    /// Number of passing tests.
    pub fn passed(&self) -> usize {
        self.count(Outcome::Pass) + self.count(Outcome::Xfail)
    }

    /// Number of failing tests.
    pub fn failed(&self) -> usize {
        self.count(Outcome::Fail) + self.count(Outcome::Panic) + self.count(Outcome::Error)
    }

    /// Number of skipped tests.
    pub fn skipped(&self) -> usize {
        self.count(Outcome::Skip)
    }

    /// Returns true if the entire suite passed.
    pub fn is_success(&self) -> bool {
        self.tests.iter().all(|t| t.outcome.is_pass())
    }
}
