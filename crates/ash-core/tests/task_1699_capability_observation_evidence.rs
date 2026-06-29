use ash_core::core_ash::CoreType;
use ash_core::core_ash_contract::{
    ObservationEvidence, ObservationPolicy, ObservationValue, PredicateAuthorityEnv,
    PredicateFault, PredicateFunctionRef, PredicateObservationError, RedactionReason,
};

#[test]
fn operation_produced_value_evidence_is_visible_as_ordinary_observed_value() {
    let evidence = ObservationEvidence::operation_result(
        "obs:read-config",
        vec!["PosixFs".into()],
        "read",
        ObservationValue::Summary("/etc/app.conf: present".into()),
        ObservationPolicy::Summarize,
    );

    assert_eq!(evidence.operation(), "read");
    assert!(
        matches!(evidence.value(), ObservationValue::Summary(summary) if summary.contains("present"))
    );
    assert!(!evidence.grants_predicate_authority());
}

#[test]
fn predicate_authority_env_has_no_provider_handles_or_role_tokens() {
    let env = PredicateAuthorityEnv::contract_predicate_default();
    let provider = PredicateFunctionRef::new(
        vec!["PosixFs".into(), "read".into()],
        Vec::new(),
        CoreType::Base("String".into()),
    );

    let error = env
        .require_provider(&provider)
        .expect_err("predicate evaluation cannot acquire provider authority");

    assert_eq!(
        error,
        PredicateObservationError::ProviderAuthorityUnavailable
    );
    assert!(env.role_tokens().is_empty());
}

#[test]
fn redacted_observation_keeps_failure_visible_without_leaking_value() {
    let evidence = ObservationEvidence::operation_result(
        "obs:secret-token",
        vec!["Secrets".into()],
        "get",
        ObservationValue::Redacted(RedactionReason::Policy),
        ObservationPolicy::Redact,
    );

    assert!(evidence.failure_visible_for_diagnostics());
    assert!(matches!(
        evidence.value(),
        ObservationValue::Redacted(RedactionReason::Policy)
    ));
}

#[test]
fn diagnostic_classes_keep_admission_operation_predicate_false_and_fault_separate() {
    assert_ne!(
        PredicateObservationError::AdmissionDenied,
        PredicateObservationError::OperationFailed
    );
    assert_ne!(
        PredicateObservationError::PredicateFalse,
        PredicateObservationError::PredicateFault(Box::new(PredicateFault::MissingBinder(
            "x".into()
        )))
    );
}
