//! TASK-1927 provider metadata admission tests.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Constraint, Effect, Value,
};
use ash_runtime::RuntimeState;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
struct ExplicitProvider {
    metadata: ProviderAuthoringMetadata,
}

impl ExplicitProvider {
    fn new(metadata: ProviderAuthoringMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait]
impl CapabilityProvider for ExplicitProvider {
    fn name(&self) -> &str {
        &self.metadata.provider_name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        self.metadata.clone()
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Ok(Value::Null)
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        Ok(Value::Null)
    }
}

fn binding(provider: &str, admitted_capabilities: Vec<&str>) -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        provider,
        CapabilityInterfaceId::new(format!("{provider}.iface")),
        provider,
        admitted_capabilities
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    )
}

#[tokio::test]
async fn host_binding_admission_accepts_rows_declared_by_provider_metadata() {
    let metadata = ProviderAuthoringMetadata::new("sensor").with_operation(
        ProviderOperationMetadata::new("read", Effect::Epistemic)
            .with_required_row("sensor.read")
            .with_resource("sensor")
            .with_sandbox_policy("host.sensor.read")
            .with_provenance_policy("host.sensor.read.redacted"),
    );
    let runtime_state =
        RuntimeState::new().with_provider("sensor", Arc::new(ExplicitProvider::new(metadata)));

    let admitted = runtime_state
        .admit_capability_binding(binding("sensor", vec!["sensor.read"]))
        .await
        .expect("declared provider row should be admissible");

    assert!(runtime_state.has_capability_binding(admitted).await);
}

#[tokio::test]
async fn host_binding_admission_rejects_rows_not_declared_by_provider_metadata() {
    let metadata = ProviderAuthoringMetadata::new("sensor").with_operation(
        ProviderOperationMetadata::new("read", Effect::Epistemic)
            .with_required_row("sensor.read")
            .with_resource("sensor")
            .with_sandbox_policy("host.sensor.read")
            .with_provenance_policy("host.sensor.read.redacted"),
    );
    let runtime_state =
        RuntimeState::new().with_provider("sensor", Arc::new(ExplicitProvider::new(metadata)));

    let err = runtime_state
        .admit_capability_binding(binding("sensor", vec!["sensor.write"]))
        .await
        .expect_err("undeclared provider rows must fail closed");

    assert!(
        err.to_string().contains(
            "provider 'sensor' metadata does not declare admitted capability 'sensor.write'"
        ),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn host_binding_admission_rejects_invalid_provider_metadata() {
    let runtime_state = RuntimeState::new().with_provider(
        "empty",
        Arc::new(ExplicitProvider::new(ProviderAuthoringMetadata::new(
            "empty",
        ))),
    );

    let err = runtime_state
        .admit_capability_binding(binding("empty", vec!["empty.read"]))
        .await
        .expect_err("invalid provider metadata must fail closed before admission");

    assert!(
        err.to_string()
            .contains("missing operation surface metadata"),
        "unexpected error: {err}"
    );
}
