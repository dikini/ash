-- Phase 199 testing evidence helpers.
--
-- These helpers package already-produced test, property, law, coverage, and
-- provider evidence metadata. They are pure record constructors and do not run
-- tests or create a parallel evidence mechanism.

use evidence::{has_evidence, is_redacted, is_authority_neutral};

pub type AssertionEvidence = AssertionEvidence {
    name: String,
    passed: Bool,
};

pub type PropertyEvidence = PropertyEvidence {
    name: String,
    passed: Bool,
    cases: Int,
    seed: Int,
};

pub type LawEvidence = LawEvidence {
    law_name: String,
    carrier: String,
    passed: Bool,
};

pub type Counterexample = Counterexample {
    input: String,
    shrink_steps: Int,
};

pub type TestCoverageArtifact = TestCoverageArtifact {
    subject: String,
    covered_cases: Int,
    total: Int,
};

pub type TestMutationArtifact = TestMutationArtifact {
    subject: String,
    killed_cases: Int,
    total: Int,
};

pub type FlakeQuarantine = FlakeQuarantine {
    test_name: String,
    reason: String,
};

pub type ProviderEvidenceSummary = ProviderEvidenceSummary {
    provider: String,
    present: Bool,
    redacted: Bool,
    authority_neutral: Bool,
};

pub fn assertion_evidence(name: String, passed: Bool) -> AssertionEvidence {
    AssertionEvidence { name: name, passed: passed }
}

pub fn assert_named(name: String, value: Bool) -> AssertionEvidence {
    assertion_evidence(name, value)
}

pub fn property_evidence(name: String, passed: Bool, cases: Int, seed: Int) -> PropertyEvidence {
    PropertyEvidence { name: name, passed: passed, cases: cases, seed: seed }
}

pub fn law_evidence(law_name: String, carrier: String, passed: Bool) -> LawEvidence {
    LawEvidence { law_name: law_name, carrier: carrier, passed: passed }
}

pub fn counterexample(input: String, shrink_steps: Int) -> Counterexample {
    Counterexample { input: input, shrink_steps: shrink_steps }
}

pub fn coverage_evidence(subject: String, observed: Int, total: Int) -> TestCoverageArtifact {
    TestCoverageArtifact { subject: subject, covered_cases: observed, total: total }
}

pub fn mutation_evidence(subject: String, detected: Int, total: Int) -> TestMutationArtifact {
    TestMutationArtifact { subject: subject, killed_cases: detected, total: total }
}

pub fn flake_quarantine(test_name: String, reason: String) -> FlakeQuarantine {
    FlakeQuarantine { test_name: test_name, reason: reason }
}

pub fn provider_evidence_summary(provider: String, count: Int, redacted: Bool, authority_neutral: Bool) -> ProviderEvidenceSummary {
    ProviderEvidenceSummary {
        provider: provider,
        present: has_evidence(count),
        redacted: is_redacted(redacted),
        authority_neutral: is_authority_neutral(authority_neutral),
    }
}
