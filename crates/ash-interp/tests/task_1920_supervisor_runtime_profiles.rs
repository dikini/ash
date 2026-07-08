use ash_core::Value;
use ash_core::core_ash_contract::{MonitorAuthorityEnv, TraceFactKind};
use ash_core::runtime::{
    FailureBoundary, FailureEntity, OperationalFailure, ProcessId, ProcessTerminalState,
    RuntimeTraceEvent, SupervisorDecisionKind, SupervisorDiagnostic, SupervisorPolicy,
    SupervisorRuntimeProfile,
};
use ash_interp::RuntimeState;

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
}

#[tokio::test]
async fn supervisor_restarts_failed_child_until_budget_then_escalates() {
    let runtime_state = RuntimeState::new();
    let supervisor_process_id = ProcessId::new();
    let first_child = ProcessId::new();
    let profile =
        SupervisorRuntimeProfile::bounded_restart("supervisor:main", supervisor_process_id, 1)
            .expect("bounded restart profile is supported");

    runtime_state
        .register_root_process(supervisor_process_id)
        .await
        .expect("supervisor registers");
    runtime_state
        .register_child_process(supervisor_process_id, first_child, 0)
        .await
        .expect("child registers");
    runtime_state
        .record_process_terminal(
            first_child,
            ProcessTerminalState::Failed {
                process_id: first_child,
                failure: Box::new(proc_failure(first_child, "first failure")),
            },
        )
        .await
        .expect("first child failure records");

    let restart = runtime_state
        .supervise_process_terminal(&profile, first_child)
        .await
        .expect("supervisor observes first failure");
    assert_eq!(restart.decision, SupervisorDecisionKind::Restart);
    assert_eq!(restart.restart_attempt, 1);
    let replacement = restart
        .replacement_process_id
        .expect("restart decision allocates replacement process");
    assert!(!restart.terminal);

    runtime_state
        .record_process_terminal(
            replacement,
            ProcessTerminalState::Failed {
                process_id: replacement,
                failure: Box::new(proc_failure(replacement, "second failure")),
            },
        )
        .await
        .expect("replacement child failure records");

    let escalated = runtime_state
        .supervise_process_terminal(&profile, replacement)
        .await
        .expect("supervisor observes budget exhaustion");
    assert_eq!(escalated.decision, SupervisorDecisionKind::Escalate);
    assert_eq!(escalated.restart_attempt, 1);
    assert!(escalated.terminal);
    assert_eq!(
        escalated.reason.as_deref(),
        Some("restart budget exhausted")
    );

    let decisions = runtime_state.supervisor_decisions().await;
    assert_eq!(decisions, vec![restart, escalated]);
    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Restart
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Escalate
    }));
    let env =
        MonitorAuthorityEnv::recorded_facts_only(facts.iter().map(|fact| fact.kind).collect());
    assert!(env.can_consume(&TraceFactKind::Process));
    assert!(!env.has_provider_authority());
}

#[tokio::test]
async fn supervisor_cancel_records_terminal_process_state_and_trace_evidence() {
    let runtime_state = RuntimeState::new();
    let supervisor_process_id = ProcessId::new();
    let child_process_id = ProcessId::new();
    let profile =
        SupervisorRuntimeProfile::cancel_policy("supervisor:cancel", supervisor_process_id)
            .expect("cancel profile is supported");

    runtime_state
        .register_root_process(supervisor_process_id)
        .await
        .expect("supervisor registers");
    runtime_state
        .register_child_process(supervisor_process_id, child_process_id, 0)
        .await
        .expect("child registers");

    let decision = runtime_state
        .cancel_supervised_process(&profile, child_process_id, "shutdown requested")
        .await
        .expect("supervisor cancels child");

    assert_eq!(decision.decision, SupervisorDecisionKind::Cancel);
    assert!(decision.terminal);
    assert!(matches!(
        runtime_state.process_terminal_state(child_process_id).await,
        Some(ProcessTerminalState::Cancelled { .. })
    ));
    assert!(
        runtime_state
            .runtime_trace_facts()
            .await
            .iter()
            .any(|fact| {
                fact.kind == TraceFactKind::Process && fact.event == RuntimeTraceEvent::Cancel
            })
    );
}

#[test]
fn unsupported_supervisor_policies_fail_closed() {
    let supervisor_process_id = ProcessId::new();
    let err = SupervisorRuntimeProfile::runtime_boundary(
        "supervisor:bad",
        supervisor_process_id,
        SupervisorPolicy::Unsupported {
            reason: "unbounded restart".to_string(),
        },
        false,
    )
    .expect_err("unsupported supervisor policies fail closed");

    assert_eq!(
        err,
        SupervisorDiagnostic::UnsupportedPolicy {
            profile_name: "supervisor:bad".to_string(),
            reason: "unbounded restart".to_string(),
        }
    );

    assert!(matches!(
        SupervisorRuntimeProfile::runtime_boundary(
            "supervisor:authority",
            supervisor_process_id,
            SupervisorPolicy::Cancel,
            true,
        )
        .expect_err("supervisor profiles must not grant authority"),
        SupervisorDiagnostic::AuthorityWideningProfile { .. }
    ));
}
