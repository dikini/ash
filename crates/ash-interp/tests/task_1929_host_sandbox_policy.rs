//! TASK-1929 host sandbox policy enforcement tests.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Constraint, Effect,
    HostSandboxPolicy, Value,
};
use ash_interp::RuntimeState;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct CountingProcessProvider {
    calls: Arc<AtomicUsize>,
}

impl CountingProcessProvider {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self { calls }
    }
}

#[async_trait]
impl CapabilityProvider for CountingProcessProvider {
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
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Value::String("executed".to_string()))
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

#[tokio::test]
async fn sandbox_denial_happens_before_host_provider_execution_and_records_redacted_evidence() {
    let calls = Arc::new(AtomicUsize::new(0));
    let runtime_state = RuntimeState::new().with_provider(
        "process",
        Arc::new(CountingProcessProvider::new(calls.clone())),
    );
    let binding_id = runtime_state
        .admit_capability_binding(process_binding())
        .await
        .expect("provider binding should admit");
    runtime_state
        .register_host_sandbox_policy(HostSandboxPolicy::deny_all(
            "host.process.run",
            "no process execution in this admission profile",
        ))
        .await
        .expect("sandbox policy should register");

    let capability_context = runtime_state
        .create_capability_context_for_bindings(&[binding_id])
        .await
        .expect("projected context should build");
    let err = capability_context
        .execute(
            "process",
            "run",
            &[Value::String("secret-command".to_string())],
        )
        .await
        .expect_err("sandbox denial should fail before provider execution");

    assert!(
        err.to_string()
            .contains("sandbox policy 'host.process.run' denied process.run"),
        "unexpected error: {err}"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "provider execute must not run after sandbox denial"
    );

    let denials = runtime_state.host_sandbox_denials().await;
    assert_eq!(denials.len(), 1);
    assert_eq!(denials[0].policy_identity, "host.process.run");
    assert_eq!(denials[0].provider_name, "process");
    assert_eq!(denials[0].operation_name, "run");
    assert!(denials[0].redacted_subject.contains("process.run"));
    assert!(!denials[0].redacted_subject.contains("secret-command"));
}

#[tokio::test]
async fn sandbox_allow_policy_permits_host_provider_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let runtime_state = RuntimeState::new().with_provider(
        "process",
        Arc::new(CountingProcessProvider::new(calls.clone())),
    );
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
        .expect("allowed sandbox policy should permit execution");

    assert_eq!(result, Value::String("executed".to_string()));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(runtime_state.host_sandbox_denials().await.is_empty());
}
