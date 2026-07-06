//! TASK-1930 host boundary provenance and redaction tests.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::core_ash_contract::TraceFactKind;
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Constraint, Effect,
    HostBoundaryOutcome, HostSandboxPolicy, RuntimeTraceEvent, Value,
};
use ash_interp::RuntimeState;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
struct RecordingProvider {
    fail: bool,
}

#[async_trait]
impl CapabilityProvider for RecordingProvider {
    fn name(&self) -> &str {
        "process"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new("process").with_operation(
            ProviderOperationMetadata::new("run", Effect::Operational)
                .with_required_row("process.run")
                .with_resource("process")
                .with_sandbox_policy("host.process.run")
                .with_provenance_policy("host.process.run.redacted"),
        )
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Ok(Value::Null)
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        if self.fail {
            Err(CapabilityError::ExecutionFailed(
                "host failed while handling secret-token".to_string(),
            ))
        } else {
            Ok(Value::String("ok".to_string()))
        }
    }
}

async fn admitted_context(
    runtime_state: &RuntimeState,
    fail: bool,
) -> ash_interp::CapabilityContext {
    let binding_id = runtime_state
        .clone()
        .with_provider("process", Arc::new(RecordingProvider { fail }))
        .admit_capability_binding(CapabilityBinding::host_provider(
            CapabilityBindingId::new(),
            "process",
            CapabilityInterfaceId::new("process.iface"),
            "process",
            vec!["process.run".to_string()],
        ))
        .await
        .expect("binding should admit");
    runtime_state
        .register_host_sandbox_policy(
            HostSandboxPolicy::allow_all("host.process.run").with_allowed_command("echo"),
        )
        .await
        .expect("sandbox policy should register");
    runtime_state
        .create_capability_context_for_bindings(&[binding_id])
        .await
        .expect("projected context should build")
}

#[tokio::test]
async fn host_success_records_redacted_evidence_and_trace() {
    let runtime_state = RuntimeState::new();
    let capability_context = admitted_context(&runtime_state, false).await;

    capability_context
        .execute(
            "process",
            "run",
            &[Value::Record(Box::new(HashMap::from([
                ("cmd".to_string(), Value::String("echo".to_string())),
                (
                    "token".to_string(),
                    Value::String("secret-token".to_string()),
                ),
            ])))],
        )
        .await
        .expect("allowed provider call should succeed");

    let evidence = runtime_state.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Succeeded);
    assert_eq!(evidence[0].provider_name, "process");
    assert_eq!(evidence[0].operation_name, "run");
    assert_eq!(evidence[0].provenance_policy, "host.process.run.redacted");
    assert!(evidence[0].authority_neutral);
    assert!(evidence[0].redacted_subject.contains("process.run"));
    assert!(!evidence[0].redacted_subject.contains("secret-token"));

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Operation
            && fact.event == RuntimeTraceEvent::Complete
            && fact.subject.contains("host:process.run")
            && !fact.subject.contains("secret-token")
    }));
    assert!(!runtime_state.runtime_monitor_evidence().await.is_empty());
}

#[tokio::test]
async fn host_failure_records_redacted_failure_without_leaking_provider_error() {
    let runtime_state = RuntimeState::new();
    let capability_context = admitted_context(&runtime_state, true).await;

    let err = capability_context
        .execute("process", "run", &[Value::String("echo".to_string())])
        .await
        .expect_err("provider should fail");
    assert!(err.to_string().contains("secret-token"));

    let evidence = runtime_state.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Failed);
    assert_eq!(
        evidence[0].diagnostic.as_deref(),
        Some("provider execution failed")
    );
    assert!(!format!("{:?}", evidence[0]).contains("secret-token"));
}

#[tokio::test]
async fn sandbox_denial_records_host_boundary_denial_evidence() {
    let runtime_state =
        RuntimeState::new().with_provider("process", Arc::new(RecordingProvider { fail: false }));
    let binding_id = runtime_state
        .admit_capability_binding(CapabilityBinding::host_provider(
            CapabilityBindingId::new(),
            "process",
            CapabilityInterfaceId::new("process.iface"),
            "process",
            vec!["process.run".to_string()],
        ))
        .await
        .expect("binding should admit");
    runtime_state
        .register_host_sandbox_policy(HostSandboxPolicy::deny_all(
            "host.process.run",
            "blocked by admission profile",
        ))
        .await
        .expect("sandbox policy should register");

    let capability_context = runtime_state
        .create_capability_context_for_bindings(&[binding_id])
        .await
        .expect("projected context should build");
    capability_context
        .execute(
            "process",
            "run",
            &[Value::String("secret-token".to_string())],
        )
        .await
        .expect_err("sandbox should deny");

    let evidence = runtime_state.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Denied);
    assert_eq!(evidence[0].sandbox_policy, "host.process.run");
    assert_eq!(
        evidence[0].diagnostic.as_deref(),
        Some("blocked by admission profile")
    );
    assert!(!format!("{:?}", evidence[0]).contains("secret-token"));
}
