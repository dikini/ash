use ash_core::Value;
use ash_core::core_ash_contract::{MonitorAuthorityEnv, TraceFactKind};
use ash_core::runtime::{ProcessId, ProcessTerminalState, RuntimeTraceEvent};
use ash_interp::RuntimeState;
use ash_typeck::Type;

#[tokio::test]
async fn process_lifecycle_events_emit_trace_facts_and_monitor_evidence() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();

    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .mark_process_running(process_id)
        .await
        .expect("process starts");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(5),
            },
        )
        .await
        .expect("process completes");

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Spawn
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Start
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Complete
    }));

    let monitor_facts = facts.iter().map(|fact| fact.kind).collect();
    let env = MonitorAuthorityEnv::recorded_facts_only(monitor_facts);
    assert!(env.can_consume(&TraceFactKind::Process));
    assert!(!env.has_provider_authority());
    assert_eq!(
        runtime_state.runtime_monitor_evidence().await.len(),
        facts.len()
    );
}

#[tokio::test]
async fn channel_events_emit_trace_facts_without_granting_authority() {
    let runtime_state = RuntimeState::new();
    let channel = runtime_state.create_channel(Type::Int, 1).await;

    runtime_state
        .send_channel(channel, Value::Int(7))
        .await
        .expect("send succeeds");
    assert_eq!(
        runtime_state
            .try_receive_channel(channel)
            .await
            .expect("receive succeeds"),
        Value::Int(7)
    );
    runtime_state
        .close_channel(channel)
        .await
        .expect("close succeeds");

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Channel && fact.event == RuntimeTraceEvent::Send
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Channel && fact.event == RuntimeTraceEvent::Receive
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Channel && fact.event == RuntimeTraceEvent::Close
    }));

    let env =
        MonitorAuthorityEnv::recorded_facts_only(facts.iter().map(|fact| fact.kind).collect());
    assert!(env.can_consume(&TraceFactKind::Channel));
    assert!(!env.has_provider_authority());
}
