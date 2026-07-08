use ash_core::Value;
use ash_core::core_ash_contract::{MonitorAuthorityEnv, TraceFactKind};
use ash_core::runtime::{
    ActorCallOutcome, ActorCallPolicy, ActorProtocol, ExternalActorAdapter,
    ExternalActorDiagnostic, RuntimeTraceEvent,
};
use ash_interp::RuntimeState;
use ash_typeck::Type;

#[tokio::test]
async fn actor_adapter_registration_retains_metadata_without_authority() {
    let runtime_state = RuntimeState::new();
    let adapter = ExternalActorAdapter::new(
        "actor:payments",
        ActorProtocol::HttpJson,
        "PaymentRequest",
        Type::Record(vec![
            ("id".into(), Type::String),
            ("amount".into(), Type::Int),
        ]),
        Type::String,
        "capability:payments.charge",
        ActorCallPolicy::bounded(2, 5_000),
        false,
    )
    .expect("adapter metadata is valid");

    let registered = runtime_state
        .register_external_actor_adapter(adapter.clone())
        .await
        .expect("adapter registers");

    assert_eq!(registered, adapter);
    assert_eq!(
        runtime_state
            .external_actor_adapter("actor:payments")
            .await
            .expect("adapter is retained"),
        adapter
    );

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::ExternalActor && fact.event == RuntimeTraceEvent::Register
    }));
    let env =
        MonitorAuthorityEnv::recorded_facts_only(facts.iter().map(|fact| fact.kind).collect());
    assert!(env.can_consume(&TraceFactKind::ExternalActor));
    assert!(!env.has_provider_authority());
}

#[tokio::test]
async fn actor_calls_validate_payload_type_sendability_and_redact_traces() {
    let runtime_state = RuntimeState::new();
    register_payment_adapter(&runtime_state).await;

    let accepted = runtime_state
        .record_external_actor_call(
            "actor:payments",
            payment_payload("order-1", 42),
            Value::String("accepted".to_string()),
        )
        .await
        .expect("sendable typed payload crosses adapter");
    assert_eq!(accepted.outcome, ActorCallOutcome::Succeeded);
    assert_eq!(accepted.payload_type, "PaymentRequest");
    assert_eq!(accepted.response_type, Some("String".to_string()));
    assert_eq!(accepted.payload_redaction, "redacted");
    assert!(!accepted.trace_subject.contains("order-1"));

    let wrong_type = runtime_state
        .record_external_actor_call(
            "actor:payments",
            Value::String("not a payment request".to_string()),
            Value::String("accepted".to_string()),
        )
        .await
        .expect_err("wrong inbound type is rejected");
    assert!(matches!(
        wrong_type,
        ExternalActorDiagnostic::InboundTypeMismatch { .. }
    ));

    let non_sendable = runtime_state
        .record_external_actor_call(
            "actor:payments",
            Value::Record(Box::new(
                [
                    ("id".to_string(), Value::String("order-2".to_string())),
                    ("amount".to_string(), Value::Int(17)),
                    (
                        "secret_ref".to_string(),
                        Value::Ref("vault://token".to_string()),
                    ),
                ]
                .into_iter()
                .collect(),
            )),
            Value::String("accepted".to_string()),
        )
        .await
        .expect_err("borrowed authority cannot cross actor boundary");
    assert!(matches!(
        non_sendable,
        ExternalActorDiagnostic::NonSendablePayload { .. }
    ));

    let facts = runtime_state.runtime_trace_facts().await;
    let actor_subjects: Vec<_> = facts
        .iter()
        .filter(|fact| fact.kind == TraceFactKind::ExternalActor)
        .map(|fact| fact.subject.as_str())
        .collect();
    assert!(
        actor_subjects
            .iter()
            .any(|subject| subject.contains("actor:payments"))
    );
    assert!(
        !actor_subjects
            .iter()
            .any(|subject| subject.contains("order-1"))
    );
    assert!(
        !actor_subjects
            .iter()
            .any(|subject| subject.contains("vault://token"))
    );
}

#[tokio::test]
async fn actor_failure_retry_timeout_cancellation_and_unsupported_protocol_are_structured() {
    let runtime_state = RuntimeState::new();
    register_payment_adapter(&runtime_state).await;

    let failed = runtime_state
        .record_external_actor_failure("actor:payments", payment_payload("order-3", 11), "502")
        .await
        .expect("actor failure is retained");
    assert_eq!(failed.outcome, ActorCallOutcome::Failed);
    assert_eq!(failed.retry_attempt, 0);
    assert_eq!(failed.diagnostic.as_deref(), Some("502"));

    let retried = runtime_state
        .retry_external_actor_call(failed.call_id)
        .await
        .expect("first retry is within policy");
    assert_eq!(retried.outcome, ActorCallOutcome::RetryScheduled);
    assert_eq!(retried.retry_attempt, 1);

    let cancelled = runtime_state
        .cancel_external_actor_call(retried.call_id, "operator stop")
        .await
        .expect("actor call cancellation is retained");
    assert_eq!(cancelled.outcome, ActorCallOutcome::Cancelled);
    assert!(cancelled.terminal);

    let timeout = runtime_state
        .record_external_actor_timeout("actor:payments", payment_payload("order-4", 21))
        .await
        .expect("timeout is retained");
    assert_eq!(timeout.outcome, ActorCallOutcome::TimedOut);
    assert!(timeout.terminal);

    let unsupported = ExternalActorAdapter::new(
        "actor:unsupported",
        ActorProtocol::Unsupported {
            reason: "raw socket actor protocol has no typed adapter".to_string(),
        },
        "LegacyRequest",
        Type::String,
        Type::String,
        "capability:unsupported.call",
        ActorCallPolicy::bounded(0, 100),
        false,
    )
    .expect_err("unsupported protocol fails closed");
    assert!(matches!(
        unsupported,
        ExternalActorDiagnostic::UnsupportedProtocol { .. }
    ));
}

async fn register_payment_adapter(runtime_state: &RuntimeState) {
    let adapter = ExternalActorAdapter::new(
        "actor:payments",
        ActorProtocol::HttpJson,
        "PaymentRequest",
        Type::Record(vec![
            ("id".into(), Type::String),
            ("amount".into(), Type::Int),
        ]),
        Type::String,
        "capability:payments.charge",
        ActorCallPolicy::bounded(1, 5_000),
        false,
    )
    .expect("adapter metadata is valid");
    runtime_state
        .register_external_actor_adapter(adapter)
        .await
        .expect("adapter registers");
}

fn payment_payload(id: &str, amount: i64) -> Value {
    Value::Record(Box::new(
        [
            ("id".to_string(), Value::String(id.to_string())),
            ("amount".to_string(), Value::Int(amount)),
        ]
        .into_iter()
        .collect(),
    ))
}
