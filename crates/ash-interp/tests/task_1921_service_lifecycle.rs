use ash_core::core_ash_contract::{MonitorAuthorityEnv, TraceFactKind};
use ash_core::runtime::{
    ProcessId, RuntimeTraceEvent, ServiceHealthStatus, ServiceLifecycleDiagnostic,
    ServiceLifecycleState, ServiceShutdownMode,
};
use ash_interp::RuntimeState;

#[tokio::test]
async fn service_start_health_reload_and_graceful_shutdown_are_retained() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();

    let started = runtime_state
        .start_service("service:api", process_id)
        .await
        .expect("service starts");
    assert_eq!(started.name, "service:api");
    assert_eq!(started.process_id, process_id);
    assert_eq!(started.lifecycle, ServiceLifecycleState::Running);
    assert_eq!(started.health, ServiceHealthStatus::Healthy);
    assert_eq!(started.reload_generation, 0);
    assert!(!started.terminal);
    assert!(started.report_identity.is_some());

    let health = runtime_state
        .service_health(started.id)
        .await
        .expect("service health is inspectable");
    assert_eq!(health.status, ServiceHealthStatus::Healthy);
    assert_eq!(health.lifecycle, ServiceLifecycleState::Running);

    let reloaded = runtime_state
        .reload_service(started.id, "config:v2")
        .await
        .expect("service reload succeeds");
    assert_eq!(reloaded.lifecycle, ServiceLifecycleState::Running);
    assert_eq!(reloaded.reload_generation, 1);
    assert_eq!(reloaded.last_reload.as_deref(), Some("config:v2"));
    assert_eq!(reloaded.report_identity, started.report_identity);

    let stopped = runtime_state
        .shutdown_service(started.id, ServiceShutdownMode::Graceful, "deploy complete")
        .await
        .expect("graceful shutdown succeeds");
    assert_eq!(stopped.lifecycle, ServiceLifecycleState::Terminated);
    assert_eq!(stopped.shutdown_mode, Some(ServiceShutdownMode::Graceful));
    assert!(stopped.terminal);
    assert!(stopped.retained);

    let retained = runtime_state
        .service_record(started.id)
        .await
        .expect("terminal service record is retained");
    assert_eq!(retained, stopped);
    assert_eq!(retained.report_identity, started.report_identity);

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Service && fact.event == RuntimeTraceEvent::Start
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Service && fact.event == RuntimeTraceEvent::Health
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Service && fact.event == RuntimeTraceEvent::Reload
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Service && fact.event == RuntimeTraceEvent::Shutdown
    }));
    let env =
        MonitorAuthorityEnv::recorded_facts_only(facts.iter().map(|fact| fact.kind).collect());
    assert!(env.can_consume(&TraceFactKind::Service));
    assert!(!env.has_provider_authority());
}

#[tokio::test]
async fn service_forced_shutdown_and_terminal_reload_fail_closed() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    let service = runtime_state
        .start_service("service:worker", process_id)
        .await
        .expect("service starts");

    let stopped = runtime_state
        .shutdown_service(service.id, ServiceShutdownMode::Forced, "operator kill")
        .await
        .expect("forced shutdown succeeds");
    assert_eq!(stopped.lifecycle, ServiceLifecycleState::Terminated);
    assert_eq!(stopped.shutdown_mode, Some(ServiceShutdownMode::Forced));
    assert_eq!(stopped.health, ServiceHealthStatus::Unavailable);

    assert_eq!(
        runtime_state
            .reload_service(service.id, "config:after-stop")
            .await
            .expect_err("reload after terminal retention fails closed"),
        ServiceLifecycleDiagnostic::TerminalServiceRetained {
            service_id: service.id,
        }
    );
    assert_eq!(
        runtime_state
            .start_service("", ProcessId::new())
            .await
            .expect_err("missing service name fails closed"),
        ServiceLifecycleDiagnostic::MissingServiceName
    );
}
