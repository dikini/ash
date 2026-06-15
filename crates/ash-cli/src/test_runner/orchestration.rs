//! Flake, quarantine, shard, and merge support for `ash test`.

use crate::test_runner::types::{Outcome, TestResult};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Schema version for retry/flake reporting.
pub const FLAKE_SCHEMA_VERSION: &str = "ash-flake-v1.0";
/// Schema version for shard reporting.
pub const SHARD_SCHEMA_VERSION: &str = "ash-shard-v1.0";
/// Schema version for merge reporting.
pub const MERGE_SCHEMA_VERSION: &str = "ash-merge-v1.0";

/// One execution attempt for a retried test row.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TestAttempt {
    /// One-based attempt number.
    pub attempt: usize,
    /// Attempt outcome.
    pub outcome: String,
    /// Attempt duration in milliseconds.
    pub duration_ms: f64,
    /// Attempt message, when any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl TestAttempt {
    /// Build an attempt row from a concrete test result.
    #[must_use]
    pub fn from_result(attempt: usize, result: &TestResult) -> Self {
        Self {
            attempt,
            outcome: outcome_json(result.outcome),
            duration_ms: result.duration.as_secs_f64() * 1000.0,
            message: result.message.clone(),
        }
    }
}

/// Flake classification for a test row.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FlakeReport {
    /// Stable schema version.
    pub schema_version: String,
    /// Flake status (`flaky`, `stable_failure`, or `stable_pass`).
    pub status: String,
    /// Number of attempts used.
    pub attempts: usize,
    /// Maximum retry count configured for the run.
    pub retries: usize,
}

/// Suite-level flake summary.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FlakeSummary {
    /// Stable schema version.
    pub schema_version: String,
    /// Configured retry count.
    pub retries: usize,
    /// Rows that used more than one attempt.
    pub retried: usize,
    /// Rows that failed before eventually passing.
    pub flaky: usize,
    /// Rows that still failed after retry budget.
    pub stable_failures: usize,
}

/// Quarantine status attached to a visible test row.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuarantineReport {
    /// Quarantine status.
    pub status: String,
    /// Human-authored quarantine reason.
    pub reason: String,
    /// Original outcome before quarantine remapping.
    pub original_outcome: String,
}

/// Parsed one-based shard selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShardSpec {
    /// One-based selected shard index.
    pub index: usize,
    /// Total shard count.
    pub total: usize,
}

impl std::str::FromStr for ShardSpec {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((index, total)) = value.split_once('/') else {
            return Err("shard must use INDEX/TOTAL form".to_string());
        };
        let index: usize = index
            .parse()
            .map_err(|_| "shard index must be a positive integer".to_string())?;
        let total: usize = total
            .parse()
            .map_err(|_| "shard total must be a positive integer".to_string())?;
        if index == 0 || total == 0 || index > total {
            return Err("shard must satisfy 1 <= INDEX <= TOTAL".to_string());
        }
        Ok(Self { index, total })
    }
}

impl std::fmt::Display for ShardSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.index, self.total)
    }
}

/// Per-row shard assignment.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShardAssignment {
    /// One-based selected shard index.
    pub index: usize,
    /// Total shard count.
    pub total: usize,
    /// Zero-based global sorted test index.
    pub ordinal: usize,
}

/// Suite-level shard report.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShardReport {
    /// Stable schema version.
    pub schema_version: String,
    /// One-based selected shard index.
    pub index: usize,
    /// Total shard count.
    pub total: usize,
    /// Number of discovered candidate tests before shard selection.
    pub candidate_count: usize,
    /// Number selected for this shard.
    pub selected_count: usize,
    /// Number skipped because they belong to another shard.
    pub skipped_count: usize,
}

impl ShardReport {
    /// Build a shard report from candidate and selected counts.
    #[must_use]
    pub fn new(spec: ShardSpec, candidate_count: usize, selected_count: usize) -> Self {
        Self {
            schema_version: SHARD_SCHEMA_VERSION.to_string(),
            index: spec.index,
            total: spec.total,
            candidate_count,
            selected_count,
            skipped_count: candidate_count.saturating_sub(selected_count),
        }
    }
}

/// Suite-level merge report.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MergeReport {
    /// Stable schema version.
    pub schema_version: String,
    /// Number of merged shard files.
    pub shards: usize,
    /// Expected total shard count.
    pub expected_shards: usize,
}

/// Return true when a zero-based ordinal belongs to a one-based shard spec.
#[must_use]
pub fn shard_contains(spec: ShardSpec, ordinal: usize) -> bool {
    ordinal % spec.total == spec.index - 1
}

/// JSON spelling for an outcome.
#[must_use]
pub fn outcome_json(outcome: Outcome) -> String {
    outcome.to_string().to_lowercase()
}

/// Attach quarantine metadata, remapping quarantined rows to `skip` so they do
/// not count as ordinary passes or failures while preserving original outcome.
pub fn apply_quarantine(result: &mut TestResult, reason: String) {
    let original = result.outcome;
    result.quarantine = Some(QuarantineReport {
        status: "quarantined".to_string(),
        reason: reason.clone(),
        original_outcome: outcome_json(original),
    });
    result.outcome = Outcome::Skip;
    result.message = Some(format!(
        "quarantined: {reason} (original outcome: {})",
        outcome_json(original)
    ));
}

/// Build a suite-level flake summary from current test rows.
#[must_use]
pub fn flake_summary(tests: &[TestResult], retries: usize) -> FlakeSummary {
    FlakeSummary {
        schema_version: FLAKE_SCHEMA_VERSION.to_string(),
        retries,
        retried: tests.iter().filter(|test| test.attempts.len() > 1).count(),
        flaky: tests
            .iter()
            .filter(|test| {
                test.flake
                    .as_ref()
                    .is_some_and(|flake| flake.status == "flaky")
            })
            .count(),
        stable_failures: tests
            .iter()
            .filter(|test| {
                test.quarantine.is_none()
                    && test
                        .flake
                        .as_ref()
                        .is_some_and(|flake| flake.status == "stable_failure")
            })
            .count(),
    }
}

/// Deterministic JSON merge for shard outputs.
pub fn merge_result_files(files: &[PathBuf]) -> Result<Value, String> {
    if files.is_empty() {
        return Err("--merge-results requires at least one shard JSON file".to_string());
    }

    let mut shard_indices = BTreeSet::new();
    let mut expected_total = None;
    let mut tests = Vec::new();
    let mut duration_ms = 0.0;

    for file in files {
        let value = read_json_file(file)?;
        let shard = value
            .get("shard")
            .ok_or_else(|| format!("missing shard metadata in {}", file.display()))?;
        if shard.get("schema_version").and_then(Value::as_str) != Some(SHARD_SCHEMA_VERSION) {
            return Err(format!("unsupported shard schema in {}", file.display()));
        }
        let index = json_usize(shard, "index")?;
        let total = json_usize(shard, "total")?;
        if index == 0 || total == 0 || index > total {
            return Err(format!(
                "invalid shard range in {}: expected 1 <= index <= total",
                file.display()
            ));
        }
        if value.get("success").and_then(Value::as_bool) != Some(true) {
            return Err(format!("shard input failed: {}", file.display()));
        }
        if let Some(expected) = expected_total {
            if expected != total {
                return Err("inconsistent shard totals in merge inputs".to_string());
            }
        } else {
            expected_total = Some(total);
        }
        if !shard_indices.insert(index) {
            return Err(format!("duplicate shard result for shard {index}/{total}"));
        }
        duration_ms += value
            .get("duration_ms")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let rows = value
            .get("tests")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("missing tests array in {}", file.display()))?;
        tests.extend(rows.iter().cloned());
    }

    let expected_total = expected_total.unwrap_or(0);
    for index in 1..=expected_total {
        if !shard_indices.contains(&index) {
            return Err(format!(
                "missing shard result for shard {index}/{expected_total}"
            ));
        }
    }

    tests.sort_by(|left, right| {
        let left_key = (
            left.get("path").and_then(Value::as_str).unwrap_or_default(),
            left.get("name").and_then(Value::as_str).unwrap_or_default(),
        );
        let right_key = (
            right
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            right
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });
    reject_duplicate_test_rows(&tests)?;

    let passed = tests
        .iter()
        .filter(|test| {
            test.get("outcome")
                .and_then(Value::as_str)
                .is_some_and(|outcome| matches!(outcome, "pass" | "xfail"))
        })
        .count();
    let failed = tests
        .iter()
        .filter(|test| {
            test.get("outcome")
                .and_then(Value::as_str)
                .is_some_and(|outcome| matches!(outcome, "fail" | "panic" | "error"))
        })
        .count();
    let skipped = tests
        .iter()
        .filter(|test| test.get("outcome").and_then(Value::as_str) == Some("skip"))
        .count();

    Ok(json!({
        "schema_version": "ash-test-v1.0",
        "root": "merged",
        "success": failed == 0,
        "total": tests.len(),
        "passed": passed,
        "failed": failed,
        "skipped": skipped,
        "duration_ms": duration_ms,
        "tests": tests,
        "merge": {
            "schema_version": MERGE_SCHEMA_VERSION,
            "shards": files.len(),
            "expected_shards": expected_total,
        }
    }))
}

fn reject_duplicate_test_rows(tests: &[Value]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for test in tests {
        let path = test
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "merged test row missing string path".to_string())?;
        let name = test
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "merged test row missing string name".to_string())?;
        if !seen.insert((path.to_string(), name.to_string())) {
            return Err(format!("duplicate test row in shard merge: {path}::{name}"));
        }
    }
    Ok(())
}

/// JSON error output for failed merge attempts.
#[must_use]
pub fn merge_error_json(message: &str) -> Value {
    json!({
        "schema_version": "ash-test-v1.0",
        "root": "merged",
        "success": false,
        "total": 1,
        "passed": 0,
        "failed": 1,
        "skipped": 0,
        "duration_ms": 0.0,
        "tests": [{
            "name": "merge_results",
            "path": "merged",
            "outcome": "error",
            "source": "authored",
            "kind": "unit",
            "duration_ms": 0.0,
            "message": message,
            "tags": []
        }],
        "merge": {
            "schema_version": MERGE_SCHEMA_VERSION,
            "shards": 0,
            "expected_shards": 0
        }
    })
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn json_usize(value: &Value, field: &str) -> Result<usize, String> {
    let raw = value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing numeric shard field `{field}`"))?;
    usize::try_from(raw).map_err(|_| format!("shard field `{field}` is too large"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_spec_parses_one_based_index_total() {
        let spec: ShardSpec = "2/3".parse().unwrap();
        assert_eq!(spec.index, 2);
        assert_eq!(spec.total, 3);
        assert!(shard_contains(spec, 1));
        assert!(!shard_contains(spec, 0));
    }

    #[test]
    fn shard_spec_rejects_invalid_shapes() {
        assert!("0/3".parse::<ShardSpec>().is_err());
        assert!("4/3".parse::<ShardSpec>().is_err());
        assert!("1:3".parse::<ShardSpec>().is_err());
    }
}
