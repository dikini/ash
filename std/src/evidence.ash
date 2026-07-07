-- Small evidence helper functions for provider/profile tests and contracts.
--
-- These helpers inspect already-produced report/evidence fields. They do not
-- acquire providers, discharge rows, or perform host effects.

pub fn has_evidence(count: Int) -> Bool {
    count != 0
}

pub fn is_redacted(redacted: Bool) -> Bool {
    redacted
}

pub fn is_authority_neutral(authority_neutral: Bool) -> Bool {
    authority_neutral
}

pub fn provider_outcome_is_success(outcome: String) -> Bool {
    outcome == "succeeded"
}

pub fn provider_outcome_is_denied(outcome: String) -> Bool {
    outcome == "denied"
}

pub fn provider_outcome_is_failure(outcome: String) -> Bool {
    outcome == "failed"
}
