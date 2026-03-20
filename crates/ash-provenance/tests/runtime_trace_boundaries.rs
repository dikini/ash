use ash_core::WorkflowId;
use ash_provenance::{
    TraceEvent, WorkflowTraceSession, create_shared_trace_recorder, trace::InMemoryTraceStore,
};
use std::sync::Arc;

fn event_kinds(events: &[TraceEvent]) -> Vec<&'static str> {
    events
        .iter()
        .map(|event| match event {
            TraceEvent::WorkflowStarted { .. } => "started",
            TraceEvent::WorkflowCompleted { .. } => "completed",
            TraceEvent::Observation { .. } => "observation",
            TraceEvent::Orientation { .. } => "orientation",
            TraceEvent::Proposal { .. } => "proposal",
            TraceEvent::Decision { .. } => "decision",
            TraceEvent::Action { .. } => "action",
            TraceEvent::ObligationCheck { .. } => "obligation_check",
            TraceEvent::Error { .. } => "error",
        })
        .collect()
}

#[test]
fn workflow_trace_session_frames_success_with_terminal_completion() {
    let store = Arc::new(InMemoryTraceStore::new());
    let recorder = create_shared_trace_recorder(WorkflowId::new(), Arc::clone(&store));

    let mut session = WorkflowTraceSession::start(recorder, "main").expect("session should start");
    session
        .recorder_mut()
        .record_action("deploy", "approved")
        .expect("action should record");

    let recorder = session
        .finish_success()
        .expect("successful session should complete");
    let events = recorder.events();

    assert_eq!(event_kinds(&events), vec!["started", "action", "completed"]);
    assert!(matches!(
        events.last(),
        Some(TraceEvent::WorkflowCompleted { success: true, .. })
    ));
}

#[test]
fn workflow_trace_session_records_error_before_failed_completion() {
    let store = Arc::new(InMemoryTraceStore::new());
    let recorder = create_shared_trace_recorder(WorkflowId::new(), Arc::clone(&store));

    let session = WorkflowTraceSession::start(recorder, "main").expect("session should start");
    let recorder = session
        .finish_error("provider unavailable", Some("observe sensor"))
        .expect("failed session should still complete trace framing");
    let events = recorder.events();

    assert_eq!(event_kinds(&events), vec!["started", "error", "completed"]);
    assert!(matches!(
        &events[1],
        TraceEvent::Error { error, context, .. }
            if error == "provider unavailable" && context.as_deref() == Some("observe sensor")
    ));
    assert!(matches!(
        events.last(),
        Some(TraceEvent::WorkflowCompleted { success: false, .. })
    ));
}
