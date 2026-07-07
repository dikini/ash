-- std::test - minimal authored-test helpers (TASK-511)
--
-- Keep this surface intentionally small and execution-stable.
--
-- V1 contract: helpers return Bool so authored tests can fail by returning
-- `false` without depending on richer panic/string-formatting support.

pub fn assert_true(value: Bool) -> Bool {
    value
}

pub fn assert_false(value: Bool) -> Bool {
    !value
}

pub fn assert_eq_int(expected: Int, actual: Int) -> Bool {
    expected == actual
}

pub fn assert_ne_int(expected: Int, actual: Int) -> Bool {
    expected != actual
}

pub fn assert_eq_string(expected: String, actual: String) -> Bool {
    expected == actual
}

pub fn assert_eq_bool(expected: Bool, actual: Bool) -> Bool {
    expected == actual
}

pub fn fail_test() -> Bool {
    false
}

-- QuickCheck property-testing substrate (Phase 151)
pub mod quickcheck;

-- Phase 199 productive testing helpers.
pub mod artifact;
pub mod fixtures;

pub use test::artifact::{
    AssertionEvidence,
    PropertyEvidence,
    LawEvidence,
    Counterexample,
    TestCoverageArtifact,
    TestMutationArtifact,
    FlakeQuarantine,
    ProviderEvidenceSummary,
    assertion_evidence,
    assert_named,
    property_evidence,
    law_evidence,
    counterexample,
    coverage_evidence,
    mutation_evidence,
    flake_quarantine,
    provider_evidence_summary,
};

pub use test::fixtures::{
    DeterministicProviderProfileFixture,
    CommonTestCase,
    deterministic_profile_fixture,
    test_clock_fixture,
    common_test_case,
};
