//! Version-moderated empirical law evidence cache schema.
//!
//! Phase 150 only freezes the schema used by the runner and repro artifacts; a
//! later policy slice may add read/write cache lookup. Missing/stale evidence is
//! distinct from refuted law evidence.

use serde::Serialize;

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
}
