-- Small evidence helper functions for provider/profile tests and contracts.
--
-- These helpers inspect already-produced report/evidence fields. They do not
-- acquire providers, discharge rows, or perform host effects.

pub builtin fn has_evidence(count: Int) -> Bool;

pub builtin fn is_redacted(redacted: Bool) -> Bool;

pub builtin fn is_authority_neutral(authority_neutral: Bool) -> Bool;

pub builtin fn provider_outcome_is_success(outcome: String) -> Bool;

pub builtin fn provider_outcome_is_denied(outcome: String) -> Bool;

pub builtin fn provider_outcome_is_failure(outcome: String) -> Bool;
