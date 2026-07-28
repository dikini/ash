use ash_core::Value;
use ash_core::core_ash_contract::{MonitorAuthorityEnv, TraceFactKind};
use ash_core::runtime::{
    ActorCallOutcome, ActorCallPolicy, ActorProtocol, ExternalActorAdapter, FailureBoundary,
    FailureEntity, OperationalFailure, ProcessId, ProcessTerminalState, RuntimeTraceEvent,
    ServiceLifecycleState, ServiceShutdownMode, SupervisorDecisionKind, SupervisorRuntimeProfile,
};
use ash_runtime::RuntimeState;
use ash_typeck::Type;

#[tokio::test]
async fn application_runtime_boundaries_compose_without_authority_leakage() {
    let runtime_state = RuntimeState::new();
    let supervisor_process_id = ProcessId::new();
    let service_process_id = ProcessId::new();
    let child_process_id = ProcessId::new();

    runtime_state
        .register_root_process(supervisor_process_id)
        .await
        .expect("supervisor root registers");
    runtime_state
        .register_child_process(supervisor_process_id, child_process_id, 0)
        .await
        .expect("supervised child registers");
    runtime_state
        .record_process_terminal(
            child_process_id,
            ProcessTerminalState::Failed {
                process_id: child_process_id,
                failure: Box::new(proc_failure(child_process_id, "child failure")),
            },
        )
        .await
        .expect("child terminal failure records");

    let supervisor =
        SupervisorRuntimeProfile::bounded_restart("supervisor:phase-196", supervisor_process_id, 1)
            .expect("supervisor profile is supported");
    let decision = runtime_state
        .supervise_process_terminal(&supervisor, child_process_id)
        .await
        .expect("supervisor observes failed child");
    assert_eq!(decision.decision, SupervisorDecisionKind::Restart);
    assert!(decision.replacement_process_id.is_some());

    let service = runtime_state
        .start_service("service:phase-196", service_process_id)
        .await
        .expect("service starts");
    let stopped = runtime_state
        .shutdown_service(
            service.id,
            ServiceShutdownMode::Graceful,
            "closeout complete",
        )
        .await
        .expect("service stops with retained lifecycle");
    assert_eq!(stopped.lifecycle, ServiceLifecycleState::Terminated);
    assert!(stopped.retained);

    let adapter = ExternalActorAdapter::new(
        "actor:phase-196",
        ActorProtocol::HttpJson,
        "CloseoutRequest",
        Type::Record(vec![
            ("id".into(), Type::String),
            ("count".into(), Type::Int),
        ]),
        Type::String,
        "capability:phase-196.actor",
        ActorCallPolicy::bounded(1, 1_000),
        false,
    )
    .expect("external actor adapter is valid");
    runtime_state
        .register_external_actor_adapter(adapter)
        .await
        .expect("external actor adapter registers");
    let actor_call = runtime_state
        .record_external_actor_call(
            "actor:phase-196",
            Value::Record(Box::new(
                [
                    ("id".to_string(), Value::String("closeout".to_string())),
                    ("count".to_string(), Value::Int(1)),
                ]
                .into_iter()
                .collect(),
            )),
            Value::String("ok".to_string()),
        )
        .await
        .expect("external actor call crosses typed adapter");
    assert_eq!(actor_call.outcome, ActorCallOutcome::Succeeded);
    assert_eq!(actor_call.payload_redaction, "redacted");
    assert!(!actor_call.trace_subject.contains("closeout"));

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Restart
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Service && fact.event == RuntimeTraceEvent::Shutdown
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::ExternalActor && fact.event == RuntimeTraceEvent::Register
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::ExternalActor && fact.event == RuntimeTraceEvent::Send
    }));

    let env =
        MonitorAuthorityEnv::recorded_facts_only(facts.iter().map(|fact| fact.kind).collect());
    assert!(env.can_consume(&TraceFactKind::Process));
    assert!(env.can_consume(&TraceFactKind::Service));
    assert!(env.can_consume(&TraceFactKind::ExternalActor));
    assert!(!env.has_provider_authority());

    let monitor_evidence = runtime_state.runtime_monitor_evidence().await;
    assert!(monitor_evidence.len() >= facts.len());
    assert!(
        monitor_evidence
            .iter()
            .any(|evidence| evidence.boundary().as_str().contains("ExternalActor"))
    );
}

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
}
