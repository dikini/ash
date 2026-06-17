//! Version-moderated empirical law and QuickCheck evidence cache schema.
//!
//! Phase 151 keeps pass evidence as accumulated empirical history while
//! preserving counterexamples and errors as active findings until the relevant
//! compatibility identity changes.

use serde::Serialize;

use crate::test_runner::quickcheck::QUICKCHECK_RNG_ALGORITHM_V1;

/// Stable key material for cached law/property evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceCacheKey {
    /// Cache schema version.
    pub schema_version: String,
    /// Compiler or runner version string.
    pub runner_version: String,
    /// Source artifact hash or stable identity.
    pub source_artifact_id: String,
    /// Law/property identity.
    pub proposition_id: String,
    /// Proof body or test metadata hash.
    pub evidence_body_hash: String,
    /// Strategy/Arbitrary identities used for generation and shrinking.
    pub strategy_ids: Vec<String>,
    /// Seed schedule identity.
    pub seed_policy: String,
    /// Case/world bound used to collect evidence.
    pub bound_policy: String,
}

/// Cached empirical evidence result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LawEvidenceCacheEntry {
    /// Cache key.
    pub key: EvidenceCacheKey,
    /// Evidence family, for example `test`.
    pub evidence_family: String,
    /// Evidence mode, for example `quickcheck`, `property`, or `small_world`.
    pub test_mode: String,
    /// Evidence status: `satisfied`, `broken`, `invalid_evidence`, `deferred`, or `stale`.
    pub evidence_status: String,
    /// Human-readable invalidation reason when stale/deferred.
    pub invalidation_reason: Option<String>,
}

impl LawEvidenceCacheEntry {
    /// Return true when this cache entry can satisfy a law under a strict policy.
    #[must_use]
    pub fn is_strictly_accepted(&self) -> bool {
        self.evidence_status == "satisfied" && self.invalidation_reason.is_none()
    }
}

/// Individual QuickCheck execution record used for empirical evidence history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickCheckRunRecord {
    /// Schema version for individual QuickCheck run records.
    pub schema_version: String,
    /// Broad compatibility identity for source/strategy/backend/RNG material.
    pub compatible_identity: String,
    /// Run outcome: `pass`, `fail`, or `error`.
    pub outcome: String,
    /// Execution failure class, when outcome is fail/error.
    pub failure_class: Option<String>,
    /// Effective seed consumed by this run.
    pub seed: u64,
    /// Seed source label: random, cli, source, replay, or project.
    pub seed_source: String,
    /// Exact case budget for this run.
    pub cases: usize,
    /// RNG/split algorithm version.
    pub rng_algorithm: String,
    /// QuickCheck backend semantics version.
    pub quickcheck_backend: String,
}

impl QuickCheckRunRecord {
    /// Construct a passing run record.
    #[must_use]
    pub fn pass(
        compatible_identity: impl Into<String>,
        seed: u64,
        seed_source: &str,
        cases: usize,
    ) -> Self {
        Self {
            schema_version: "ash-quickcheck-run-v1".to_string(),
            compatible_identity: compatible_identity.into(),
            outcome: "pass".to_string(),
            failure_class: None,
            seed,
            seed_source: seed_source.to_string(),
            cases,
            rng_algorithm: QUICKCHECK_RNG_ALGORITHM_V1.to_string(),
            quickcheck_backend: "ash-quickcheck-v1".to_string(),
        }
    }

    /// Construct an errored run record.
    #[must_use]
    pub fn error(
        compatible_identity: impl Into<String>,
        seed: u64,
        seed_source: &str,
        cases: usize,
        failure_class: impl Into<String>,
    ) -> Self {
        let mut record = Self::pass(compatible_identity, seed, seed_source, cases);
        record.outcome = "error".to_string();
        record.failure_class = Some(failure_class.into());
        record
    }

    /// Construct a counterexample run record.
    #[must_use]
    pub fn fail(
        compatible_identity: impl Into<String>,
        seed: u64,
        seed_source: &str,
        cases: usize,
        failure_class: impl Into<String>,
    ) -> Self {
        let mut record = Self::pass(compatible_identity, seed, seed_source, cases);
        record.outcome = "fail".to_string();
        record.failure_class = Some(failure_class.into());
        record
    }
}

/// Aggregate empirical evidence over compatible QuickCheck runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickCheckAggregateEvidence {
    /// Schema version for aggregate records.
    pub schema_version: String,
    /// Broad compatibility identity being summarized.
    pub compatible_identity: String,
    /// Passing compatible runs.
    pub passed_runs: usize,
    /// Total passed cases across compatible passing runs.
    pub passed_cases: usize,
    /// Compatible counterexample runs.
    pub counterexamples: usize,
    /// Compatible errored runs. Sticky until identity changes.
    pub errored_runs: usize,
    /// Whether same-seed divergent outcomes were observed.
    pub nondeterminism_detected: bool,
    /// Exact per-run case-count buckets.
    pub case_buckets: std::collections::BTreeMap<usize, usize>,
    /// Summary status: `empirical_pass_history` or `findings_observed`.
    pub aggregate_summary: String,
}

impl QuickCheckAggregateEvidence {
    /// Build aggregate evidence from records with the same compatibility identity.
    #[must_use]
    pub fn from_runs(compatible_identity: impl Into<String>, runs: &[QuickCheckRunRecord]) -> Self {
        let compatible_identity = compatible_identity.into();
        let mut passed_runs = 0usize;
        let mut passed_cases = 0usize;
        let mut counterexamples = 0usize;
        let mut errored_runs = 0usize;
        let mut case_buckets = std::collections::BTreeMap::new();
        let mut outcomes_by_seed = std::collections::BTreeMap::<u64, String>::new();
        let mut nondeterminism_detected = false;

        for run in runs
            .iter()
            .filter(|run| run.compatible_identity == compatible_identity)
        {
            *case_buckets.entry(run.cases).or_insert(0) += 1;
            if let Some(previous) = outcomes_by_seed.insert(run.seed, run.outcome.clone())
                && previous != run.outcome
            {
                nondeterminism_detected = true;
            }
            match run.outcome.as_str() {
                "pass" => {
                    passed_runs += 1;
                    passed_cases += run.cases;
                }
                "fail" => counterexamples += 1,
                "error" => errored_runs += 1,
                _ => errored_runs += 1,
            }
        }

        let aggregate_summary =
            if counterexamples == 0 && errored_runs == 0 && !nondeterminism_detected {
                "empirical_pass_history"
            } else {
                "findings_observed"
            }
            .to_string();

        Self {
            schema_version: "ash-quickcheck-aggregate-v1".to_string(),
            compatible_identity,
            passed_runs,
            passed_cases,
            counterexamples,
            errored_runs,
            nondeterminism_detected,
            case_buckets,
            aggregate_summary,
        }
    }

    /// True only when every compatible run has passed so far.
    #[must_use]
    pub fn is_empirical_pass_history(&self) -> bool {
        self.aggregate_summary == "empirical_pass_history"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_and_refuted_law_cache_states_are_distinct() {
        let key = EvidenceCacheKey {
            schema_version: "ash-law-evidence-cache-v1".to_string(),
            runner_version: "test-runner".to_string(),
            source_artifact_id: "source".to_string(),
            proposition_id: "law:identity".to_string(),
            evidence_body_hash: "body".to_string(),
            strategy_ids: vec!["test::quickcheck::arbitrary<Int>".to_string()],
            seed_policy: "seed:0".to_string(),
            bound_policy: "max_cases:32".to_string(),
        };
        let stale = LawEvidenceCacheEntry {
            key: key.clone(),
            evidence_family: "test".to_string(),
            test_mode: "quickcheck".to_string(),
            evidence_status: "stale".to_string(),
            invalidation_reason: Some("strategy hash changed".to_string()),
        };
        let broken = LawEvidenceCacheEntry {
            key,
            evidence_family: "test".to_string(),
            test_mode: "quickcheck".to_string(),
            evidence_status: "broken".to_string(),
            invalidation_reason: None,
        };
        assert!(!stale.is_strictly_accepted());
        assert!(!broken.is_strictly_accepted());
        assert_ne!(stale.evidence_status, broken.evidence_status);
    }

    #[test]
    fn quickcheck_aggregate_rolls_up_case_budgets() {
        let runs = vec![
            QuickCheckRunRecord::pass("id", 1, "random", 100),
            QuickCheckRunRecord::pass("id", 2, "random", 500),
            QuickCheckRunRecord::pass("other", 3, "random", 1000),
        ];
        let aggregate = QuickCheckAggregateEvidence::from_runs("id", &runs);
        assert!(aggregate.is_empirical_pass_history());
        assert_eq!(aggregate.passed_runs, 2);
        assert_eq!(aggregate.passed_cases, 600);
        assert_eq!(aggregate.case_buckets.get(&100), Some(&1));
        assert_eq!(aggregate.case_buckets.get(&500), Some(&1));
    }

    #[test]
    fn quickcheck_errors_and_counterexamples_are_sticky_findings() {
        let runs = vec![
            QuickCheckRunRecord::pass("id", 1, "random", 100),
            QuickCheckRunRecord::fail("id", 2, "random", 100, "property_false"),
            QuickCheckRunRecord::error("id", 3, "random", 100, "generator_error"),
            QuickCheckRunRecord::pass("id", 4, "random", 100),
        ];
        let aggregate = QuickCheckAggregateEvidence::from_runs("id", &runs);
        assert!(!aggregate.is_empirical_pass_history());
        assert_eq!(aggregate.aggregate_summary, "findings_observed");
        assert_eq!(aggregate.counterexamples, 1);
        assert_eq!(aggregate.errored_runs, 1);
        assert_eq!(aggregate.passed_runs, 2);
    }

    #[test]
    fn quickcheck_same_seed_divergence_is_nondeterminism() {
        let runs = vec![
            QuickCheckRunRecord::pass("id", 42, "cli", 100),
            QuickCheckRunRecord::error("id", 42, "cli", 100, "generator_error"),
        ];
        let aggregate = QuickCheckAggregateEvidence::from_runs("id", &runs);
        assert!(aggregate.nondeterminism_detected);
        assert_eq!(aggregate.aggregate_summary, "findings_observed");
    }
}
