//! TASK-1932 host boundary cross-boundary fixtures.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::core_ash_contract::TraceFactKind;
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Constraint, Effect,
    HostBoundaryOutcome, HostSandboxPolicy, RuntimeTraceEvent, TrustedRuntimeAdapter, Value,
};
use ash_runtime::{RuntimeState, builtin_host_hook_metadata};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
struct FixtureProcessProvider;

#[async_trait]
impl CapabilityProvider for FixtureProcessProvider {
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
        Ok(Value::String("host-ok".to_string()))
    }
}

fn process_binding() -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "process",
        CapabilityInterfaceId::new("process.iface"),
        "process",
        vec!["process.run".to_string()],
    )
}

fn process_adapter() -> TrustedRuntimeAdapter {
    TrustedRuntimeAdapter::new_provider_operation(
        "host.process.run.adapter",
        "1.0.0",
        "ash-runtime",
        "admitted-provider:process",
        "host.process.run",
        "host.process.run.redacted",
        "report.host.process.run",
        "process",
        "run",
        "process.run",
        false,
    )
    .expect("fixture adapter should be valid")
}

#[tokio::test]
async fn host_boundary_fixture_covers_builtin_provider_adapter_sandbox_and_provenance() {
    let builtin_metadata =
        builtin_host_hook_metadata("process::run").expect("process::run has hook metadata");
    assert_eq!(builtin_metadata.operation_identity, "process.run");
    assert!(!builtin_metadata.grants_authority);

    let runtime_state =
        RuntimeState::new().with_provider("process", Arc::new(FixtureProcessProvider));
    runtime_state
        .register_trusted_runtime_adapter(process_adapter())
        .await
        .expect("trusted adapter should register");
    runtime_state
        .validate_trusted_runtime_adapter_for_provider_operation(
            "host.process.run.adapter",
            "1.0.0",
            &FixtureProcessProvider.provider_metadata(),
            "run",
        )
        .await
        .expect("adapter must match provider metadata before execution");
    let binding_id = runtime_state
        .admit_capability_binding(process_binding())
        .await
        .expect("provider binding should admit");
    runtime_state
        .register_host_sandbox_policy(
            HostSandboxPolicy::allow_all("host.process.run").with_allowed_command("echo"),
        )
        .await
        .expect("sandbox policy should register");

    let capability_context = runtime_state
        .create_capability_context_for_bindings(&[binding_id])
        .await
        .expect("projected context should build");
    let result = capability_context
        .execute("process", "run", &[Value::String("echo".to_string())])
        .await
        .expect("allowed host call should execute");
    assert_eq!(result, Value::String("host-ok".to_string()));

    let evidence = runtime_state.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Succeeded);
    assert!(evidence[0].authority_neutral);
    assert_eq!(evidence[0].sandbox_policy, "host.process.run");
    assert_eq!(evidence[0].provenance_policy, "host.process.run.redacted");

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Operation
            && fact.event == RuntimeTraceEvent::Register
            && fact.subject.contains("adapter:host.process.run.adapter")
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Operation
            && fact.event == RuntimeTraceEvent::Complete
            && fact.subject.contains("host:process.run")
    }));
}

#[tokio::test]
async fn host_boundary_fixture_denies_before_provider_when_sandbox_blocks() {
    let runtime_state =
        RuntimeState::new().with_provider("process", Arc::new(FixtureProcessProvider));
    let binding_id = runtime_state
        .admit_capability_binding(process_binding())
        .await
        .expect("provider binding should admit");
    runtime_state
        .register_host_sandbox_policy(HostSandboxPolicy::deny_all(
            "host.process.run",
            "fixture denied",
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
            &[Value::String("secret-command".to_string())],
        )
        .await
        .expect_err("sandbox denial should fail closed");

    let evidence = runtime_state.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Denied);
    assert!(!format!("{:?}", evidence[0]).contains("secret-command"));
}
