//! Shared runtime-owned state for interpreter executions.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;

use ash_core::capability::ProviderAuthoringMetadata;
use ash_core::capability::{CapabilityError, validate_provider_authoring_metadata};
use ash_core::core_ash_contract::{
    ContractDischargeRecord, MonitorEvaluationResult, PredicateBinderId, RuntimeMonitorEvidence,
    SnapshotRef, TraceFactKind,
};
use ash_core::runtime::{
    ActorCallId, ActorCallOutcome, CapabilityBinding, CapabilityBindingDependency,
    CapabilityBindingId, CapabilityBindingKind, CapabilityImplementationId, CapabilityInterfaceId,
    ExternalActorAdapter, ExternalActorCallRecord, ExternalActorDiagnostic, FailureBoundary,
    FailureEntity, HostBoundaryEvidence, HostBoundaryOutcome, HostSandboxDecision,
    HostSandboxDenialRecord, HostSandboxPolicy, OperationalFailure, ProcessId,
    ProcessPropagationDiagnostic, ProcessPropagationOutcome, ProcessTerminalState, ResourceId,
    ResourceInstance, ResourceLifecycle, ResourceOwner, ResourceProvenance,
    ResourceSplitJoinPolicy, ResourceTypeId, RuntimeTraceEvent, RuntimeTraceFact,
    ServiceHealthReport, ServiceHealthStatus, ServiceId, ServiceLifecycleDiagnostic,
    ServiceLifecycleState, ServiceRuntimeRecord, ServiceShutdownMode, SupervisorDecisionKind,
    SupervisorDecisionRecord, SupervisorDiagnostic, SupervisorPolicy, SupervisorRuntimeProfile,
    TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic, TrustedRuntimeAdapterTarget,
};
use ash_core::{ApplicationId, ControlLink, Effect, Expr, Value};

use crate::capability::CapabilityProvider;
use crate::channel::{ChannelError, ChannelId, ChannelRegistry};
use crate::control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkRegistry, LinkState,
    RetainedCompletionRecord, RetainedCompletionWaiter,
};
use crate::{ExecError, ExecResult};

use crate::execution_record::{ExecutionAdmissionFacts, ExecutionRecord};
use crate::process_registry::{ProcessRecord, ProcessRegistry, ProcessRegistryError};
use crate::runtime_outcome_state::RuntimeOutcomeState;
use std::time::Duration;

pub(crate) const SPAWNED_CHILD_CONTROL_BINDING: &str = "__ash_spawn_control_link";

mod implementation;
pub use implementation::{
    ImplementationBindingAdmission, ImplementationBindingDependencySource,
    ImplementationOperationBody,
};

/// Standard internal resource pilot that Phase 104 can admit without host authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardPilotResource {
    /// Application-local opaque key/value resource pilot.
    ApplicationKv,
    /// Deterministic frozen/test clock resource pilot.
    FrozenClock,
}

impl StandardPilotResource {
    /// Static runtime resource type identifier used by this pilot.
    #[must_use]
    pub fn resource_type_id(self) -> ResourceTypeId {
        match self {
            Self::ApplicationKv => ResourceTypeId::new("ApplicationKV"),
            Self::FrozenClock => ResourceTypeId::new("FrozenClock"),
        }
    }

    fn provenance_note(self) -> &'static str {
        match self {
            Self::ApplicationKv => "standard ApplicationKV pilot admitted by runtime",
            Self::FrozenClock => "standard FrozenClock pilot admitted by runtime",
        }
    }
}

/// Host-facing request to admit one standard internal capability/resource pilot.
#[derive(Debug, Clone, PartialEq)]
pub struct StandardInternalPilot {
    binding_name: String,
    resource_name: String,
    resource: StandardPilotResource,
    interface: CapabilityInterfaceId,
    implementation: CapabilityImplementationId,
    operation: String,
    fixture: Value,
}

impl StandardInternalPilot {
    /// Create the standard ApplicationKV pilot.
    #[must_use]
    pub fn application_kv(
        binding_name: impl Into<String>,
        resource_name: impl Into<String>,
        fixture: Value,
    ) -> Self {
        Self {
            binding_name: binding_name.into(),
            resource_name: resource_name.into(),
            resource: StandardPilotResource::ApplicationKv,
            interface: CapabilityInterfaceId::new("KeyValue"),
            implementation: CapabilityImplementationId::new("__ash_standard_pilot.ApplicationKV"),
            operation: "get".to_string(),
            fixture,
        }
    }

    /// Create the standard FrozenClock/TestClock pilot.
    #[must_use]
    pub fn frozen_clock(
        binding_name: impl Into<String>,
        resource_name: impl Into<String>,
        frozen_epoch_millis: i64,
    ) -> Self {
        Self {
            binding_name: binding_name.into(),
            resource_name: resource_name.into(),
            resource: StandardPilotResource::FrozenClock,
            interface: CapabilityInterfaceId::new("Clock"),
            implementation: CapabilityImplementationId::new("__ash_standard_pilot.FrozenClock"),
            operation: "epoch_millis".to_string(),
            fixture: Value::Int(frozen_epoch_millis),
        }
    }
}

/// Identity pair returned by standard internal pilot admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StandardPilotBinding {
    /// Admitted implementation-backed binding id.
    pub binding_id: CapabilityBindingId,
    /// Admitted internal resource id that backs the binding authority.
    pub resource_id: ResourceId,
}

/// Wrapper that adapts an `Arc<dyn CapabilityProvider>` to work as a `Box<dyn CapabilityProvider>`.
///
/// This is used internally by `RuntimeState` to create a `CapabilityContext` from
/// its stored providers. The wrapper delegates all trait methods to the inner
/// Arc-wrapped provider.
#[derive(Clone)]
struct ArcProviderWrapper {
    inner: Arc<dyn CapabilityProvider>,
}

impl ArcProviderWrapper {
    /// Create a new wrapper around the given Arc-wrapped provider.
    fn new(inner: Arc<dyn CapabilityProvider>) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for ArcProviderWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcProviderWrapper")
            .field("name", &self.inner.name())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProviderAdmissionSurface {
    Provider,
    Actions(HashSet<String>),
}

impl ProviderAdmissionSurface {
    fn from_capabilities(provider_name: &str, admitted_capabilities: &[String]) -> Option<Self> {
        let mut admitted_actions = HashSet::new();

        for capability in admitted_capabilities {
            if capability == provider_name || capability == &format!("{provider_name}.*") {
                return Some(Self::Provider);
            }

            for separator in ['.', ':'] {
                if let Some((candidate_provider, action_name)) = capability.split_once(separator)
                    && candidate_provider == provider_name
                    && !action_name.is_empty()
                {
                    admitted_actions.insert(action_name.to_string());
                }
            }
        }

        if admitted_actions.is_empty() {
            None
        } else {
            Some(Self::Actions(admitted_actions))
        }
    }

    fn allows_action(&self, action_name: &str) -> bool {
        match self {
            Self::Provider => true,
            Self::Actions(actions) => actions.contains(action_name),
        }
    }
}

fn normalized_action_grant(provider_name: &str, grant: &str) -> String {
    if grant == provider_name {
        return format!("{provider_name}.*");
    }
    if let Some((candidate_provider, action_name)) = grant.split_once('.')
        && candidate_provider == provider_name
        && !action_name.is_empty()
    {
        return format!("{candidate_provider}.{action_name}");
    }
    if let Some((candidate_provider, action_name)) = grant.split_once(':')
        && candidate_provider == provider_name
        && !action_name.is_empty()
    {
        return format!("{candidate_provider}.{action_name}");
    }
    grant.to_string()
}

#[derive(Debug, Clone)]
struct ProjectedProviderRuntimeStores {
    sandbox_policies: Arc<AsyncMutex<HashMap<String, HostSandboxPolicy>>>,
    sandbox_denials: Arc<AsyncMutex<Vec<HostSandboxDenialRecord>>>,
    host_boundary_evidence: Arc<AsyncMutex<Vec<HostBoundaryEvidence>>>,
    runtime_trace_facts: Arc<AsyncMutex<Vec<RuntimeTraceFact>>>,
    runtime_monitor_evidence: Arc<AsyncMutex<Vec<RuntimeMonitorEvidence>>>,
}

#[derive(Debug, Clone)]
struct ProjectedProviderWrapper {
    inner: Arc<dyn CapabilityProvider>,
    provider_name: String,
    projected_name: String,
    surface: ProviderAdmissionSurface,
    stores: ProjectedProviderRuntimeStores,
}

impl ProjectedProviderWrapper {
    fn new(
        inner: Arc<dyn CapabilityProvider>,
        provider_name: String,
        projected_name: String,
        surface: ProviderAdmissionSurface,
        stores: ProjectedProviderRuntimeStores,
    ) -> Self {
        Self {
            inner,
            provider_name,
            projected_name,
            surface,
            stores,
        }
    }

    fn with_projected_name(mut self, projected_name: String) -> Self {
        self.projected_name = projected_name;
        self
    }

    async fn enforce_sandbox(
        &self,
        action_name: &str,
        args: &[Value],
    ) -> Result<Option<(String, String)>, CapabilityError> {
        let metadata = self.inner.provider_metadata();
        let Some(operation) = metadata.operation(action_name) else {
            return Ok(None);
        };
        let Some(policy_identity) = operation.sandbox_policy.as_deref() else {
            return Ok(None);
        };
        let provenance_policy = operation
            .provenance_policy
            .clone()
            .unwrap_or_else(|| "host.boundary.redacted".to_string());
        let Some(policy) = self
            .stores
            .sandbox_policies
            .lock()
            .await
            .get(policy_identity)
            .cloned()
        else {
            return Ok(Some((policy_identity.to_string(), provenance_policy)));
        };

        match policy.decide(action_name, args) {
            HostSandboxDecision::Allow => Ok(Some((policy.identity.clone(), provenance_policy))),
            HostSandboxDecision::Deny { reason } => {
                let record = HostSandboxDenialRecord {
                    policy_identity: policy.identity.clone(),
                    provider_name: self.provider_name.clone(),
                    operation_name: action_name.to_string(),
                    redacted_subject: format!(
                        "sandbox:{}:{}.{action_name}:redacted",
                        policy.identity, self.provider_name
                    ),
                    reason: reason.clone(),
                };
                self.stores.sandbox_denials.lock().await.push(record);
                self.record_host_boundary_evidence(
                    action_name,
                    &policy.identity,
                    &provenance_policy,
                    HostBoundaryOutcome::Denied,
                    Some(reason.clone()),
                )
                .await;
                Err(CapabilityError::PermissionDenied(format!(
                    "sandbox policy '{}' denied {}.{action_name}: {reason}",
                    policy.identity, self.provider_name
                )))
            }
        }
    }

    fn admits_operation(&self, action_name: &str) -> bool {
        if self.surface.allows_action(action_name) {
            return true;
        }

        let ProviderAdmissionSurface::Actions(actions) = &self.surface else {
            return false;
        };

        self.inner
            .provider_metadata()
            .operations
            .iter()
            .find(|operation| operation.operation_name == action_name)
            .is_some_and(|operation| {
                operation.required_rows.iter().any(|row| {
                    let Some((provider, row_action)) = row.split_once('.') else {
                        return actions.contains(row);
                    };
                    (provider == self.provider_name || provider == self.projected_name)
                        && actions.contains(row_action)
                })
            })
    }

    async fn record_host_boundary_evidence(
        &self,
        action_name: &str,
        sandbox_policy: &str,
        provenance_policy: &str,
        outcome: HostBoundaryOutcome,
        diagnostic: Option<String>,
    ) {
        let evidence = HostBoundaryEvidence::new(
            self.provider_name.clone(),
            action_name.to_string(),
            None,
            sandbox_policy.to_string(),
            provenance_policy.to_string(),
            outcome,
            diagnostic,
        );
        let event = match outcome {
            HostBoundaryOutcome::Succeeded => RuntimeTraceEvent::Complete,
            _ => RuntimeTraceEvent::Fail,
        };
        self.stores
            .runtime_trace_facts
            .lock()
            .await
            .push(RuntimeTraceFact::new(
                TraceFactKind::Operation,
                event,
                evidence.redacted_subject.clone(),
            ));
        self.stores
            .runtime_monitor_evidence
            .lock()
            .await
            .push(RuntimeMonitorEvidence::new(
                "phase-197-host-boundary-monitor",
                "phase-197-host-provenance",
                format!("{:?}:{event:?}", TraceFactKind::Operation),
                MonitorEvaluationResult::Pending,
            ));
        self.stores
            .host_boundary_evidence
            .lock()
            .await
            .push(evidence);
    }
}

#[async_trait]
impl ash_core::capability::CapabilityProvider for ArcProviderWrapper {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn effect(&self) -> Effect {
        self.inner.effect()
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        self.inner.provider_metadata()
    }

    async fn observe(
        &self,
        constraints: &[ash_core::Constraint],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        self.inner.observe(constraints).await
    }

    async fn execute(
        &self,
        action_name: &str,
        args: &[Value],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        self.inner.execute(action_name, args).await
    }
}

#[async_trait]
impl ash_core::capability::CapabilityProvider for ProjectedProviderWrapper {
    fn name(&self) -> &str {
        &self.projected_name
    }

    fn effect(&self) -> Effect {
        self.inner.effect()
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        self.inner.provider_metadata()
    }

    async fn observe(
        &self,
        constraints: &[ash_core::Constraint],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        let Some(action_name) = constraints
            .first()
            .map(|constraint| constraint.predicate.name.as_str())
        else {
            if matches!(self.surface, ProviderAdmissionSurface::Provider) {
                return self.inner.observe(constraints).await;
            }
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        };
        if self.admits_operation(action_name) {
            let sandbox_args = constraints
                .first()
                .map(|constraint| {
                    constraint
                        .predicate
                        .arguments
                        .iter()
                        .map(|expr| match expr {
                            ash_core::Expr::Literal(value) => value.clone(),
                            _ => Value::Null,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let policy = self.enforce_sandbox(action_name, &sandbox_args).await?;
            let result = self.inner.observe(constraints).await;
            if let Some((sandbox_policy, provenance_policy)) = policy {
                match &result {
                    Ok(_) => {
                        self.record_host_boundary_evidence(
                            action_name,
                            &sandbox_policy,
                            &provenance_policy,
                            HostBoundaryOutcome::Succeeded,
                            None,
                        )
                        .await;
                    }
                    Err(_) => {
                        self.record_host_boundary_evidence(
                            action_name,
                            &sandbox_policy,
                            &provenance_policy,
                            HostBoundaryOutcome::Failed,
                            Some("provider observation failed".to_string()),
                        )
                        .await;
                    }
                }
            }
            result
        } else {
            Err(CapabilityError::NotAvailable(format!(
                "{}.{}",
                self.provider_name, action_name
            )))
        }
    }

    async fn execute(
        &self,
        action_name: &str,
        args: &[Value],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        if self.admits_operation(action_name) {
            let policy = self.enforce_sandbox(action_name, args).await?;
            let result = self.inner.execute(action_name, args).await;
            if let Some((sandbox_policy, provenance_policy)) = policy {
                match &result {
                    Ok(_) => {
                        self.record_host_boundary_evidence(
                            action_name,
                            &sandbox_policy,
                            &provenance_policy,
                            HostBoundaryOutcome::Succeeded,
                            None,
                        )
                        .await;
                    }
                    Err(_) => {
                        self.record_host_boundary_evidence(
                            action_name,
                            &sandbox_policy,
                            &provenance_policy,
                            HostBoundaryOutcome::Failed,
                            Some("provider execution failed".to_string()),
                        )
                        .await;
                    }
                }
            }
            result
        } else {
            Err(CapabilityError::NotAvailable(format!(
                "{}.{}",
                self.provider_name, action_name
            )))
        }
    }
}

/// Shared runtime state that must persist across related top-level executions.
///
/// This is the runtime-owned carrier for lifecycle state such as reusable control authority
/// and proxy registrations.
///
/// # Provider Registry
///
/// RuntimeState also maintains a registry of capability providers that can be
/// used during target expression execution. Providers can be registered using
/// [`RuntimeState::with_provider`] or [`RuntimeState::with_providers`].
/// Explicit metadata for admitting one resource owned by an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryOwnedResourceAdmission {
    /// Entry-local owned resource name.
    pub name: String,
    /// Static resource type identifier.
    pub type_id: ResourceTypeId,
}

impl EntryOwnedResourceAdmission {
    /// Create owned resource admission metadata from an entry-local name and resource type.
    #[must_use]
    pub fn new(name: impl Into<String>, type_id: ResourceTypeId) -> Self {
        Self {
            name: name.into(),
            type_id,
        }
    }
}

mod resource_admission;
pub use resource_admission::ResourceSplitJoinViolation;

#[derive(Clone, Default)]
pub struct RuntimeState {
    control_registry: Arc<AsyncMutex<ControlLinkRegistry>>,
    process_registry: Arc<AsyncMutex<ProcessRegistry>>,
    channel_registry: Arc<AsyncMutex<ChannelRegistry>>,
    process_propagation_diagnostics: Arc<AsyncMutex<Vec<ProcessPropagationDiagnostic>>>,
    supervisor_decisions: Arc<AsyncMutex<Vec<SupervisorDecisionRecord>>>,
    service_records: Arc<AsyncMutex<HashMap<ServiceId, ServiceRuntimeRecord>>>,
    trusted_runtime_adapters: Arc<AsyncMutex<HashMap<String, TrustedRuntimeAdapter>>>,
    host_sandbox_policies: Arc<AsyncMutex<HashMap<String, HostSandboxPolicy>>>,
    host_sandbox_denials: Arc<AsyncMutex<Vec<HostSandboxDenialRecord>>>,
    host_boundary_evidence: Arc<AsyncMutex<Vec<HostBoundaryEvidence>>>,
    external_actor_adapters: Arc<AsyncMutex<HashMap<String, ExternalActorAdapter>>>,
    external_actor_calls: Arc<AsyncMutex<HashMap<ActorCallId, ExternalActorCallRecord>>>,
    runtime_trace_facts: Arc<AsyncMutex<Vec<RuntimeTraceFact>>>,
    runtime_monitor_evidence: Arc<AsyncMutex<Vec<RuntimeMonitorEvidence>>>,
    resource_instances: Arc<AsyncMutex<HashMap<ResourceId, ResourceInstance>>>,
    capability_bindings: Arc<AsyncMutex<HashMap<CapabilityBindingId, CapabilityBinding>>>,
    capability_interface_operations:
        Arc<AsyncMutex<HashMap<CapabilityInterfaceId, HashSet<String>>>>,
    implementation_operation_bodies:
        Arc<AsyncMutex<HashMap<(CapabilityImplementationId, String), ImplementationOperationBody>>>,
    last_execution_record: Arc<AsyncMutex<Option<ExecutionRecord>>>,
    /// Capability provider registry for execution
    providers: Arc<StdMutex<HashMap<String, Arc<dyn CapabilityProvider>>>>,
    /// Contract-discharge sidecar records keyed by callable name.
    ///
    /// These are metadata-only records attached to callable rows. They do not
    /// grant authority or register providers; they are used for admission and
    /// runtime check planning.
    pub contract_discharge_records: Arc<StdMutex<HashMap<String, ContractDischargeRecord>>>,
    /// Captured predicate-environment values keyed by binder id.
    ///
    /// Used by the authority-free contract predicate evaluator to resolve
    /// parameter, result, and snapshot references at runtime.
    pub predicate_binder_values: Arc<StdMutex<HashMap<PredicateBinderId, Value>>>,
    /// Captured snapshot values keyed by `SnapshotRef`.
    ///
    /// Snapshots are boundary-local copies of values captured at the moment a
    /// contract becomes active. They remain immutable for the duration of the
    /// contract boundary.
    pub predicate_snapshot_values: Arc<StdMutex<HashMap<SnapshotRef, Value>>>,
}

impl RuntimeState {
    /// Returns the contract-discharge record for the given callable, if any.
    pub fn contract_discharge_record(
        &self,
        callable_name: &str,
    ) -> Option<ContractDischargeRecord> {
        self.contract_discharge_records
            .lock()
            .expect("contract discharge registry mutex poisoned")
            .get(callable_name)
            .cloned()
    }

    /// Capture a value for a predicate binder.
    ///
    /// # Panics
    ///
    /// Panics if the binder-value registry mutex is poisoned.
    pub fn capture_predicate_binder(
        &self,
        binder: PredicateBinderId,
        value: Value,
    ) -> Option<Value> {
        self.predicate_binder_values
            .lock()
            .expect("predicate binder registry mutex poisoned")
            .insert(binder, value)
    }

    /// Look up a captured predicate binder value.
    ///
    /// # Panics
    ///
    /// Panics if the binder-value registry mutex is poisoned.
    pub fn predicate_binder_value(&self, binder: &PredicateBinderId) -> Option<Value> {
        self.predicate_binder_values
            .lock()
            .expect("predicate binder registry mutex poisoned")
            .get(binder)
            .cloned()
    }

    /// Capture a snapshot value at a contract boundary.
    ///
    /// # Panics
    ///
    /// Panics if the snapshot registry mutex is poisoned.
    pub fn capture_predicate_snapshot(&self, snapshot: SnapshotRef, value: Value) -> Option<Value> {
        self.predicate_snapshot_values
            .lock()
            .expect("predicate snapshot registry mutex poisoned")
            .insert(snapshot, value)
    }

    /// Look up a captured snapshot value.
    ///
    /// # Panics
    ///
    /// Panics if the snapshot registry mutex is poisoned.
    pub fn predicate_snapshot_value(&self, snapshot: &SnapshotRef) -> Option<Value> {
        self.predicate_snapshot_values
            .lock()
            .expect("predicate snapshot registry mutex poisoned")
            .get(snapshot)
            .cloned()
    }

    /// Clear all captured predicate binder and snapshot values.
    ///
    /// Typically called after a contract boundary terminates to prevent stale
    /// snapshot/binder references from leaking across boundaries.
    ///
    /// # Panics
    ///
    /// Panics if either registry mutex is poisoned.
    pub fn clear_predicate_environment(&self) {
        self.predicate_binder_values
            .lock()
            .expect("predicate binder registry mutex poisoned")
            .clear();
        self.predicate_snapshot_values
            .lock()
            .expect("predicate snapshot registry mutex poisoned")
            .clear();
    }
}

impl std::fmt::Debug for RuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeState")
            .field("control_registry", &self.control_registry)
            .field("process_registry", &self.process_registry)
            .field("channel_registry", &self.channel_registry)
            .field(
                "process_propagation_diagnostics",
                &"<Vec<ProcessPropagationDiagnostic>>",
            )
            .field("supervisor_decisions", &"<Vec<SupervisorDecisionRecord>>")
            .field(
                "service_records",
                &"<HashMap<ServiceId, ServiceRuntimeRecord>>",
            )
            .field(
                "trusted_runtime_adapters",
                &"<HashMap<String, TrustedRuntimeAdapter>>",
            )
            .field(
                "host_sandbox_policies",
                &"<HashMap<String, HostSandboxPolicy>>",
            )
            .field("host_sandbox_denials", &"<Vec<HostSandboxDenialRecord>>")
            .field("host_boundary_evidence", &"<Vec<HostBoundaryEvidence>>")
            .field(
                "external_actor_adapters",
                &"<HashMap<String, ExternalActorAdapter>>",
            )
            .field(
                "external_actor_calls",
                &"<HashMap<ActorCallId, ExternalActorCallRecord>>",
            )
            .field("runtime_trace_facts", &"<Vec<RuntimeTraceFact>>")
            .field("runtime_monitor_evidence", &"<Vec<RuntimeMonitorEvidence>>")
            .field(
                "resource_instances",
                &"<HashMap<ResourceId, ResourceInstance>>",
            )
            .field(
                "capability_bindings",
                &"<HashMap<CapabilityBindingId, CapabilityBinding>>",
            )
            .field(
                "capability_interface_operations",
                &"<HashMap<CapabilityInterfaceId, HashSet<String>>>",
            )
            .field("last_execution_record", &self.last_execution_record)
            .field(
                "providers",
                &"<HashMap<String, Arc<dyn CapabilityProvider>>>",
            )
            .field(
                "contract_discharge_records",
                &"<HashMap<String, ContractDischargeRecord>>",
            )
            .finish()
    }
}

impl RuntimeState {
    fn projected_provider_runtime_stores(&self) -> ProjectedProviderRuntimeStores {
        ProjectedProviderRuntimeStores {
            sandbox_policies: self.host_sandbox_policies.clone(),
            sandbox_denials: self.host_sandbox_denials.clone(),
            host_boundary_evidence: self.host_boundary_evidence.clone(),
            runtime_trace_facts: self.runtime_trace_facts.clone(),
            runtime_monitor_evidence: self.runtime_monitor_evidence.clone(),
        }
    }

    pub(crate) async fn implementation_binding_dependency_context(
        &self,
        binding: &CapabilityBinding,
    ) -> ExecResult<(
        crate::capability::CapabilityContext,
        HashMap<String, Value>,
        Vec<CapabilityBindingId>,
    )> {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut values = HashMap::new();
        let mut registry = CapabilityRegistry::new();
        let mut admitted_capability_dependencies = Vec::new();

        for dependency in &binding.dependencies {
            match dependency {
                CapabilityBindingDependency::Capability {
                    name, binding_id, ..
                } => {
                    let source = self.capability_binding(*binding_id).await.ok_or_else(|| {
                        ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' references unknown binding {binding_id:?}"
                        ))
                    })?;
                    values.insert(name.clone(), Value::String(name.clone()));
                    admitted_capability_dependencies.push(*binding_id);

                    let CapabilityBindingKind::HostProvider {
                        provider_name,
                        admitted_capabilities,
                    } = &source.kind
                    else {
                        values.insert(
                            format!("__ash_capability_dependency_alias:{name}"),
                            Value::String(source.name.clone()),
                        );
                        continue;
                    };

                    let providers = self
                        .providers
                        .lock()
                        .expect("provider registry mutex poisoned");
                    let provider = providers
                        .get(provider_name)
                        .ok_or_else(|| ExecError::CapabilityNotAvailable(provider_name.clone()))?;
                    let mut dependency_capabilities = admitted_capabilities.clone();
                    if source.name != *provider_name {
                        dependency_capabilities.extend(admitted_capabilities.iter().filter_map(
                            |grant| {
                                grant
                                    .strip_prefix(&format!("{}.", source.name))
                                    .map(|action| format!("{provider_name}.{action}"))
                                    .or_else(|| {
                                        grant
                                            .strip_prefix(&format!("{}:", source.name))
                                            .map(|action| format!("{provider_name}:{action}"))
                                    })
                            },
                        ));
                    }
                    let surface = ProviderAdmissionSurface::from_capabilities(
                        provider_name,
                        &dependency_capabilities,
                    )
                    .ok_or_else(|| {
                        ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' exposes no admitted provider surface"
                        ))
                    })?;
                    let wrapper = ProjectedProviderWrapper::new(
                        provider.clone(),
                        provider_name.clone(),
                        source.name.clone(),
                        surface,
                        self.projected_provider_runtime_stores(),
                    );
                    registry.register(Box::new(wrapper.clone()));
                    if source.name != *provider_name {
                        registry.register(Box::new(
                            wrapper.clone().with_projected_name(provider_name.clone()),
                        ));
                    }
                    if name != &source.name && name != provider_name {
                        registry.register(Box::new(wrapper.with_projected_name(name.clone())));
                    }
                    values.insert(
                        format!("__ash_capability_dependency_alias:{name}"),
                        Value::String(name.clone()),
                    );
                    values.insert(
                        format!("__ash_capability_dependency_local:{name}"),
                        Value::Bool(true),
                    );
                }
                CapabilityBindingDependency::Resource { .. } => {}
                CapabilityBindingDependency::Config { name, value } => {
                    values.insert(name.clone(), value.clone());
                }
            }
        }

        Ok((
            CapabilityContext::with_registry(registry),
            values,
            admitted_capability_dependencies,
        ))
    }

    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            control_registry: Arc::new(AsyncMutex::new(ControlLinkRegistry::new())),
            process_registry: Arc::new(AsyncMutex::new(ProcessRegistry::new())),
            channel_registry: Arc::new(AsyncMutex::new(ChannelRegistry::new())),
            process_propagation_diagnostics: Arc::new(AsyncMutex::new(Vec::new())),
            supervisor_decisions: Arc::new(AsyncMutex::new(Vec::new())),
            service_records: Arc::new(AsyncMutex::new(HashMap::new())),
            trusted_runtime_adapters: Arc::new(AsyncMutex::new(HashMap::new())),
            host_sandbox_policies: Arc::new(AsyncMutex::new(HashMap::new())),
            host_sandbox_denials: Arc::new(AsyncMutex::new(Vec::new())),
            host_boundary_evidence: Arc::new(AsyncMutex::new(Vec::new())),
            external_actor_adapters: Arc::new(AsyncMutex::new(HashMap::new())),
            external_actor_calls: Arc::new(AsyncMutex::new(HashMap::new())),
            runtime_trace_facts: Arc::new(AsyncMutex::new(Vec::new())),
            runtime_monitor_evidence: Arc::new(AsyncMutex::new(Vec::new())),
            resource_instances: Arc::new(AsyncMutex::new(HashMap::new())),
            capability_bindings: Arc::new(AsyncMutex::new(HashMap::new())),
            capability_interface_operations: Arc::new(AsyncMutex::new(HashMap::new())),
            implementation_operation_bodies: Arc::new(AsyncMutex::new(HashMap::new())),
            last_execution_record: Arc::new(AsyncMutex::new(None)),
            providers: Arc::new(StdMutex::new(HashMap::new())),
            contract_discharge_records: Arc::new(StdMutex::new(HashMap::new())),
            predicate_binder_values: Arc::new(StdMutex::new(HashMap::new())),
            predicate_snapshot_values: Arc::new(StdMutex::new(HashMap::new())),
        }
    }

    /// Add a capability provider to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register the provider under
    /// * `provider` - The capability provider to register
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::MockProvider;
    /// use ash_core::Effect;
    ///
    /// let state = RuntimeState::new()
    ///     .with_provider("test", std::sync::Arc::new(MockProvider::new("test", Effect::Epistemic)));
    /// ```
    pub fn with_provider(
        self,
        name: impl Into<String>,
        provider: Arc<dyn CapabilityProvider>,
    ) -> Self {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .insert(name.into(), provider);
        self
    }

    /// Register or replace one capability provider on an existing runtime state.
    ///
    /// This mutates only the runtime provider registry. It does not admit rows, install profiles,
    /// or grant authority; callers must still admit explicit capability bindings before projected
    /// execution can use the provider.
    pub fn register_provider(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn CapabilityProvider>,
    ) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .insert(name.into(), provider)
    }

    /// Add multiple capability providers to the registry.
    ///
    /// # Arguments
    ///
    /// * `providers` - A HashMap of provider names to providers
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::{CapabilityProvider, MockProvider};
    /// use ash_core::Effect;
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// let mut providers: HashMap<String, Arc<dyn CapabilityProvider>> = HashMap::new();
    /// providers.insert("test".to_string(), Arc::new(MockProvider::new("test", Effect::Epistemic)));
    ///
    /// let state = RuntimeState::new().with_providers(providers);
    /// ```
    pub fn with_providers(self, providers: HashMap<String, Arc<dyn CapabilityProvider>>) -> Self {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .extend(providers);
        self
    }

    /// Get a provider by name.
    ///
    /// Returns `Some(Arc<dyn CapabilityProvider>)` if a provider with the given
    /// name is registered, or `None` if not found.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the provider to look up
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::MockProvider;
    /// use ash_core::Effect;
    ///
    /// let state = RuntimeState::new()
    ///     .with_provider("test", std::sync::Arc::new(MockProvider::new("test", Effect::Epistemic)));
    ///
    /// let provider = state.get_provider("test");
    /// assert!(provider.is_some());
    /// ```
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .get(name)
            .cloned()
    }

    /// Get all registered provider names.
    ///
    /// Returns a vector of all provider names currently registered.
    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Check if a provider is registered.
    ///
    /// Returns `true` if a provider with the given name is registered.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .contains_key(name)
    }

    /// Get the number of registered providers.
    ///
    /// Returns the count of providers currently registered.
    pub fn provider_count(&self) -> usize {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .len()
    }

    /// Create a CapabilityContext from the registered providers.
    ///
    /// This allows the interpreter to access capability providers
    /// during application execution.
    pub async fn create_capability_context(&self) -> crate::capability::CapabilityContext {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut registry = CapabilityRegistry::new();
        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for provider in providers.values() {
            let wrapper = Box::new(ArcProviderWrapper::new(provider.clone()));
            registry.register(wrapper);
        }

        CapabilityContext::with_registry(registry)
    }

    /// Create a BehaviourContext backed by runtime capability providers.
    pub async fn create_behaviour_context(&self) -> crate::behaviour::BehaviourContext {
        use crate::behaviour::{BehaviourContext, BehaviourProvider, BehaviourRegistry};
        use crate::typed_provider::TypedBehaviourProvider;
        use ash_typeck::{Type, TypeVar};

        struct RuntimeProviderBehaviour {
            provider: Arc<dyn CapabilityProvider>,
            capability: String,
            channel: String,
        }

        #[async_trait::async_trait]
        impl BehaviourProvider for RuntimeProviderBehaviour {
            fn capability_name(&self) -> &str {
                &self.capability
            }

            fn channel_name(&self) -> &str {
                &self.channel
            }

            async fn sample(&self, constraints: &[ash_core::Constraint]) -> ExecResult<Value> {
                self.provider
                    .observe(constraints)
                    .await
                    .map_err(|error| ExecError::ExecutionFailed(error.to_string()))
            }
        }

        let mut registry = BehaviourRegistry::new();
        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for (name, provider) in providers.iter() {
            if let Some((capability, channel)) = name.split_once(':')
                && !capability.is_empty()
                && !channel.is_empty()
            {
                registry.register(TypedBehaviourProvider::new(
                    RuntimeProviderBehaviour {
                        provider: provider.clone(),
                        capability: capability.to_string(),
                        channel: channel.to_string(),
                    },
                    Type::Var(TypeVar::fresh()),
                ));
            }
        }

        BehaviourContext::with_registry(registry)
    }

    /// Create a CapabilityContext projected to the admitted provider/action surface.
    pub async fn create_projected_capability_context(
        &self,
        admitted_capabilities: &[String],
    ) -> crate::capability::CapabilityContext {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut registry = CapabilityRegistry::new();

        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for (name, provider) in providers.iter() {
            let Some(surface) =
                ProviderAdmissionSurface::from_capabilities(name, admitted_capabilities)
            else {
                continue;
            };

            let wrapper = Box::new(ProjectedProviderWrapper::new(
                provider.clone(),
                name.clone(),
                name.clone(),
                surface,
                self.projected_provider_runtime_stores(),
            ));
            registry.register(wrapper);
        }

        CapabilityContext::with_registry(registry)
    }

    /// Admit entry-owned resources from explicit entry resource metadata.
    ///
    /// Returned resource ids are keyed only by the explicit entry-local resource names supplied
    /// by the caller; this API does not perform ambient resource lookup.
    pub async fn admit_entry_owned_resources(
        &self,
        application_id: ApplicationId,
        resources: Vec<EntryOwnedResourceAdmission>,
    ) -> ExecResult<HashMap<String, ResourceId>> {
        let mut seen_names = HashSet::new();
        for resource in &resources {
            if !seen_names.insert(resource.name.clone()) {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "duplicate owned resource name '{}'",
                    resource.name
                )));
            }
        }

        let mut admitted = HashMap::new();
        let mut instances = Vec::with_capacity(resources.len());
        for resource in resources {
            let id = ResourceId::new();
            let type_name = resource.type_id.as_str().to_string();
            let instance = ResourceInstance::new(
                id,
                resource.type_id,
                ResourceOwner::Application(application_id),
            )
            .with_lifecycle(ResourceLifecycle::Admitted)
            .with_provenance(ResourceProvenance::internal(format!(
                "entry resource {}: {type_name}",
                resource.name
            )));
            admitted.insert(resource.name, id);
            instances.push(instance);
        }

        let mut registry = self.resource_instances.lock().await;
        for instance in instances {
            registry.insert(instance.id, instance);
        }
        Ok(admitted)
    }

    /// Register the canonical runtime operation surface for one capability interface.
    ///
    /// This metadata is normally sourced from already typechecked capability-interface declarations.
    /// Runtime admission uses it as the authority for metadata-only non-widening checks instead of
    /// trusting caller-supplied operation lists on individual implementation binding admissions.
    pub async fn register_capability_interface_operations<I, S>(
        &self,
        interface: CapabilityInterfaceId,
        operations: I,
    ) -> ExecResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let operations = operations
            .into_iter()
            .map(Into::into)
            .collect::<HashSet<_>>();
        if operations.is_empty() {
            return Err(ExecError::InvalidRuntimeState(format!(
                "capability interface {} must declare at least one operation",
                interface.as_str()
            )));
        }

        let mut interfaces = self.capability_interface_operations.lock().await;
        if interfaces.contains_key(&interface) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "capability interface {} operation surface already registered",
                interface.as_str()
            )));
        }
        interfaces.insert(interface, operations);
        Ok(())
    }

    /// Return the registered operation surface for one capability interface.
    pub async fn capability_interface_operations(
        &self,
        interface: &CapabilityInterfaceId,
    ) -> Option<HashSet<String>> {
        self.capability_interface_operations
            .lock()
            .await
            .get(interface)
            .cloned()
    }

    /// Register one callable Ash-defined implementation operation body.
    pub async fn register_implementation_operation_body(
        &self,
        implementation: CapabilityImplementationId,
        operation: impl Into<String>,
        body: ImplementationOperationBody,
    ) -> ExecResult<()> {
        let key = (implementation, operation.into());
        let mut bodies = self.implementation_operation_bodies.lock().await;
        if bodies.contains_key(&key) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "implementation operation body for {}.{} already registered",
                key.0.as_str(),
                key.1
            )));
        }
        bodies.insert(key, body);
        Ok(())
    }

    /// Look up one registered Ash-defined implementation operation body.
    pub async fn implementation_operation_body(
        &self,
        implementation: &CapabilityImplementationId,
        operation: &str,
    ) -> Option<ImplementationOperationBody> {
        self.implementation_operation_bodies
            .lock()
            .await
            .get(&(implementation.clone(), operation.to_string()))
            .cloned()
    }

    /// Admit one standard internal pilot resource and its implementation-backed binding.
    ///
    /// The pilot creates an internal runtime-owned resource, registers a deterministic
    /// Ash-defined implementation operation body backed by an inert config value, and then
    /// admits an implementation binding whose authority derives only from the explicit resource
    /// dependency. It does not expose the resource as a first-class Ash value and does not perform
    /// ambient capability/resource lookup.
    ///
    /// # Errors
    ///
    /// Returns an error if interface operation validation, implementation body registration, or
    /// implementation binding admission fails.
    pub async fn admit_standard_internal_pilot(
        &self,
        pilot: StandardInternalPilot,
    ) -> ExecResult<StandardPilotBinding> {
        if self
            .capability_interface_operations(&pilot.interface)
            .await
            .is_none()
        {
            self.register_capability_interface_operations(
                pilot.interface.clone(),
                [pilot.operation.as_str()],
            )
            .await?;
        }

        if self
            .implementation_operation_body(&pilot.implementation, &pilot.operation)
            .await
            .is_some()
        {
            return Err(ExecError::InvalidRuntimeState(format!(
                "standard internal pilot body {}.{} is already registered",
                pilot.implementation.as_str(),
                pilot.operation
            )));
        }

        let params = match pilot.resource {
            StandardPilotResource::ApplicationKv => vec!["__ash_standard_pilot_arg"],
            StandardPilotResource::FrozenClock => Vec::new(),
        };
        self.register_implementation_operation_body(
            pilot.implementation.clone(),
            pilot.operation.clone(),
            ImplementationOperationBody::new(
                params,
                Expr::Variable {
                    name: "__ash_standard_pilot_fixture".to_string(),
                    span: Default::default(),
                },
            ),
        )
        .await?;

        let resource_id = ResourceId::new();
        let resource = ResourceInstance::new(
            resource_id,
            pilot.resource.resource_type_id(),
            ResourceOwner::Application(ApplicationId::new()),
        )
        .with_lifecycle(ResourceLifecycle::Admitted)
        .with_provenance(ResourceProvenance::internal(
            pilot.resource.provenance_note(),
        ));
        self.register_resource_instance(resource).await;

        let mut resources = HashMap::new();
        resources.insert(pilot.resource_name.clone(), resource_id);
        let binding_id = self
            .admit_implementation_binding(
                ImplementationBindingAdmission::new(
                    pilot.binding_name,
                    pilot.interface,
                    pilot.implementation,
                )
                .with_dependency(ImplementationBindingDependencySource::resource(
                    pilot.resource_name,
                    pilot.resource.resource_type_id(),
                ))
                .with_dependency(ImplementationBindingDependencySource::config(
                    "__ash_standard_pilot_fixture",
                    pilot.fixture,
                ))
                .with_requested_operations([pilot.operation]),
                &resources,
            )
            .await?;

        Ok(StandardPilotBinding {
            binding_id,
            resource_id,
        })
    }

    /// Admit one implementation-backed capability binding from explicit dependency source names.
    ///
    /// Resource dependencies are resolved only from `resource_sources`, and capability dependencies
    /// are resolved only by previously admitted binding names. The resulting binding is admitted via
    /// [`Self::admit_capability_binding`] so existing dependency validation remains authoritative.
    pub async fn admit_implementation_binding(
        &self,
        admission: ImplementationBindingAdmission,
        resource_sources: &HashMap<String, ResourceId>,
    ) -> ExecResult<CapabilityBindingId> {
        if !admission.requested_operations.is_empty() {
            let Some(registered_operations) = self
                .capability_interface_operations(&admission.interface)
                .await
            else {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "cannot validate requested operations for unregistered interface {}",
                    admission.interface.as_str()
                )));
            };
            let allowed_operations: HashSet<&str> =
                registered_operations.iter().map(String::as_str).collect();
            for operation in &admission.requested_operations {
                if !allowed_operations.contains(operation.as_str()) {
                    return Err(ExecError::InvalidRuntimeState(format!(
                        "requested operation '{operation}' is outside registered interface {}",
                        admission.interface.as_str()
                    )));
                }
            }
        }

        let mut dependencies = Vec::with_capacity(admission.dependencies.len());
        for dependency in admission.dependencies {
            match dependency {
                ImplementationBindingDependencySource::Resource { name, type_id } => {
                    let Some(resource_id) = resource_sources.get(&name).copied() else {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing resource dependency source '{name}'"
                        )));
                    };
                    dependencies.push(CapabilityBindingDependency::Resource {
                        name,
                        resource_id,
                        type_id,
                    });
                }
                ImplementationBindingDependencySource::Capability {
                    name,
                    source_binding_name,
                    interface,
                } => {
                    let Some(binding) = self.capability_binding_by_name(&source_binding_name).await
                    else {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing capability dependency source '{source_binding_name}'"
                        )));
                    };
                    dependencies.push(CapabilityBindingDependency::Capability {
                        name,
                        binding_id: binding.id,
                        interface,
                    });
                }
                ImplementationBindingDependencySource::Config { name, value } => {
                    dependencies.push(CapabilityBindingDependency::Config { name, value });
                }
            }
        }

        let binding = CapabilityBinding::implementation(
            CapabilityBindingId::new(),
            admission.name,
            admission.interface,
            admission.implementation,
            dependencies,
        );
        let binding = admission
            .requested_operations
            .iter()
            .fold(binding, |binding, operation| {
                let source_names = binding
                    .dependencies
                    .iter()
                    .filter(|dependency| dependency.carries_authority())
                    .map(|dependency| dependency.name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                binding.with_authority_note(format!(
                    "operation {operation} derives from {source_names}"
                ))
            });
        self.admit_capability_binding(binding).await
    }

    /// Admit one host- or implementation-backed capability binding into runtime state.
    ///
    /// Host bindings require an already-registered provider. Implementation bindings store
    /// dependency metadata only and require resource/capability dependencies to have been admitted
    /// explicitly beforehand.
    pub async fn admit_capability_binding(
        &self,
        binding: CapabilityBinding,
    ) -> ExecResult<CapabilityBindingId> {
        Self::validate_capability_binding_authority(&binding)?;

        match &binding.kind {
            CapabilityBindingKind::HostProvider { provider_name, .. } => {
                self.validate_host_provider_binding_metadata(provider_name, &binding)?;
            }
            CapabilityBindingKind::Implementation { .. } => {
                if !binding
                    .dependencies
                    .iter()
                    .any(CapabilityBindingDependency::carries_authority)
                {
                    return Err(ExecError::InvalidRuntimeState(
                        "implementation capability binding must derive authority from at least one resource or capability dependency"
                            .to_string(),
                    ));
                }
                self.validate_capability_binding_dependencies(&binding.dependencies)
                    .await?;
            }
        }

        let id = binding.id;
        let mut bindings = self.capability_bindings.lock().await;
        if bindings.contains_key(&id) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "duplicate capability binding id {id:?}"
            )));
        }
        if bindings
            .values()
            .any(|existing| existing.name == binding.name)
        {
            return Err(ExecError::InvalidRuntimeState(format!(
                "duplicate capability binding name '{}'",
                binding.name
            )));
        }
        bindings.insert(id, binding);
        Ok(id)
    }

    /// Validate that a binding kind and authority provenance agree.
    fn validate_capability_binding_authority(binding: &CapabilityBinding) -> ExecResult<()> {
        match (&binding.kind, &binding.authority) {
            (
                CapabilityBindingKind::HostProvider { .. },
                ash_core::CapabilityAuthorityProvenance::HostAuthority { .. },
            ) => Ok(()),
            (
                CapabilityBindingKind::Implementation { .. },
                ash_core::CapabilityAuthorityProvenance::DerivedAuthority { .. },
            ) => Ok(()),
            (CapabilityBindingKind::HostProvider { .. }, _) => Err(ExecError::InvalidRuntimeState(
                "host capability binding must carry host authority provenance".to_string(),
            )),
            (CapabilityBindingKind::Implementation { .. }, _) => {
                Err(ExecError::InvalidRuntimeState(
                    "implementation capability binding must carry derived authority provenance"
                        .to_string(),
                ))
            }
        }
    }

    fn validate_host_provider_binding_metadata(
        &self,
        provider_name: &str,
        binding: &CapabilityBinding,
    ) -> ExecResult<()> {
        let provider = self
            .get_provider(provider_name)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(provider_name.to_string()))?;
        let metadata = provider.provider_metadata();
        validate_provider_authoring_metadata(&metadata).map_err(|error| {
            ExecError::InvalidRuntimeState(format!(
                "provider '{provider_name}' metadata invalid: {error}"
            ))
        })?;

        let CapabilityBindingKind::HostProvider {
            admitted_capabilities,
            ..
        } = &binding.kind
        else {
            return Ok(());
        };

        let mut declared_rows = metadata
            .operations
            .iter()
            .flat_map(|operation| operation.required_rows.iter())
            .cloned()
            .collect::<HashSet<_>>();
        if metadata.provider_name != provider_name {
            for operation in &metadata.operations {
                for row in &operation.required_rows {
                    if let Some((_, operation_name)) = row.split_once('.') {
                        declared_rows.insert(format!("{provider_name}.{operation_name}"));
                    }
                }
            }
        }
        for capability in admitted_capabilities {
            if !declared_rows.contains(capability.as_str()) {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "provider '{provider_name}' metadata does not declare admitted capability '{capability}'"
                )));
            }
        }

        Ok(())
    }

    async fn validate_capability_binding_dependencies(
        &self,
        dependencies: &[CapabilityBindingDependency],
    ) -> ExecResult<()> {
        let bindings = self.capability_bindings.lock().await;
        let resources = self.resource_instances.lock().await;

        for dependency in dependencies {
            match dependency {
                CapabilityBindingDependency::Resource {
                    name,
                    resource_id,
                    type_id,
                } => match resources.get(resource_id) {
                    Some(resource) if &resource.type_id == type_id => {}
                    Some(_) => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "resource dependency '{name}' has mismatched type"
                        )));
                    }
                    None => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing resource dependency '{name}'"
                        )));
                    }
                },
                CapabilityBindingDependency::Capability {
                    name,
                    binding_id,
                    interface,
                } => match bindings.get(binding_id) {
                    Some(binding) if &binding.interface == interface => {}
                    Some(_) => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' has mismatched interface"
                        )));
                    }
                    None => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing capability dependency '{name}'"
                        )));
                    }
                },
                CapabilityBindingDependency::Config { .. } => {}
            }
        }

        Ok(())
    }

    /// Look up one admitted capability binding by identity.
    pub async fn capability_binding(
        &self,
        binding_id: CapabilityBindingId,
    ) -> Option<CapabilityBinding> {
        self.capability_bindings
            .lock()
            .await
            .get(&binding_id)
            .cloned()
    }

    /// Look up one admitted capability binding by runtime binding name.
    pub async fn capability_binding_by_name(&self, name: &str) -> Option<CapabilityBinding> {
        self.capability_bindings
            .lock()
            .await
            .values()
            .find(|binding| binding.name == name)
            .cloned()
    }

    /// Return true if a capability binding has been explicitly admitted.
    pub async fn has_capability_binding(&self, binding_id: CapabilityBindingId) -> bool {
        self.capability_bindings
            .lock()
            .await
            .contains_key(&binding_id)
    }

    /// Return the number of admitted capability bindings.
    pub async fn capability_binding_count(&self) -> usize {
        self.capability_bindings.lock().await.len()
    }

    /// Create a capability context from explicit admitted binding identities.
    ///
    /// Host-provider bindings project their admitted provider/action surface into the existing
    /// `CapabilityContext` compatibility layer. Implementation bindings remain metadata-only and do
    /// not register executable providers.
    pub async fn create_capability_context_for_bindings(
        &self,
        binding_ids: &[CapabilityBindingId],
    ) -> ExecResult<crate::capability::CapabilityContext> {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let bindings = self.capability_bindings.lock().await;
        let mut projected_surfaces = Vec::new();

        for binding_id in binding_ids {
            let Some(binding) = bindings.get(binding_id) else {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "unadmitted capability binding {binding_id:?}"
                )));
            };

            if let CapabilityBindingKind::HostProvider {
                provider_name,
                admitted_capabilities,
            } = &binding.kind
            {
                projected_surfaces.push((
                    provider_name.clone(),
                    binding.name.clone(),
                    admitted_capabilities.clone(),
                ));
            }
        }
        drop(bindings);

        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");
        let mut registry = CapabilityRegistry::new();

        for (provider_name, projected_name, mut admitted_capabilities) in projected_surfaces {
            let Some(provider) = providers.get(&provider_name) else {
                return Err(ExecError::CapabilityNotAvailable(provider_name));
            };
            admitted_capabilities.sort();
            admitted_capabilities.dedup();
            let projected_capabilities = admitted_capabilities
                .iter()
                .map(|grant| {
                    if grant == &format!("{provider_name}.*")
                        || grant == &format!("{projected_name}.*")
                    {
                        return projected_name.clone();
                    }
                    if grant == &provider_name || grant == &projected_name {
                        return projected_name.clone();
                    }
                    if let Some((candidate_provider, action_name)) = grant.split_once('.')
                        && (candidate_provider == provider_name
                            || candidate_provider == projected_name)
                    {
                        return format!("{projected_name}.{action_name}");
                    }
                    if let Some((candidate_provider, action_name)) = grant.split_once(':')
                        && (candidate_provider == provider_name
                            || candidate_provider == projected_name)
                    {
                        return format!("{projected_name}:{action_name}");
                    }
                    grant.clone()
                })
                .collect::<Vec<_>>();
            let Some(surface) = ProviderAdmissionSurface::from_capabilities(
                &projected_name,
                &projected_capabilities,
            ) else {
                continue;
            };
            let wrapper = ProjectedProviderWrapper::new(
                provider.clone(),
                provider_name.clone(),
                projected_name.clone(),
                surface,
                self.projected_provider_runtime_stores(),
            );
            registry.register(Box::new(wrapper));
        }

        Ok(CapabilityContext::with_registry(registry))
    }

    /// Return all currently admitted capability binding identities.
    pub async fn admitted_capability_binding_ids(&self) -> Vec<CapabilityBindingId> {
        let mut binding_ids = self
            .capability_bindings
            .lock()
            .await
            .keys()
            .copied()
            .collect::<Vec<_>>();
        binding_ids.sort_unstable_by_key(|binding_id| binding_id.0);
        binding_ids
    }

    /// Resolve requested provider/action grant strings to explicit admitted binding identities.
    pub async fn resolve_admitted_capability_bindings(
        &self,
        admitted_capabilities: &[String],
    ) -> Vec<CapabilityBindingId> {
        let bindings = self.capability_bindings.lock().await;
        let mut binding_ids = Vec::new();
        for binding in bindings.values() {
            let CapabilityBindingKind::HostProvider {
                provider_name,
                admitted_capabilities: binding_capabilities,
            } = &binding.kind
            else {
                continue;
            };
            let grants_requested_surface = admitted_capabilities.iter().any(|requested| {
                let normalized_request = normalized_action_grant(provider_name, requested);
                binding.name == *requested
                    || provider_name == requested
                    || binding_capabilities.iter().any(|grant| {
                        let normalized_grant = normalized_action_grant(provider_name, grant);
                        normalized_grant == normalized_request
                            || (normalized_grant == format!("{provider_name}.*")
                                && normalized_request.starts_with(&format!("{provider_name}.")))
                    })
            });
            if grants_requested_surface {
                binding_ids.push(binding.id);
            }
        }
        binding_ids.sort_unstable_by_key(|binding_id| binding_id.0);
        binding_ids.dedup();
        binding_ids
    }

    /// Build audit facts for the projected alpha admission grant set.
    pub async fn execution_admission_facts(
        &self,
        binding_ids: &[CapabilityBindingId],
    ) -> ExecutionAdmissionFacts {
        let bindings = self.capability_bindings.lock().await;
        let mut capability_binding_grants = Vec::new();
        let mut resource_grants = Vec::new();
        let mut action_grants = Vec::new();

        for binding_id in binding_ids {
            let Some(binding) = bindings.get(binding_id) else {
                continue;
            };
            capability_binding_grants.push(format!("{binding_id:?}"));
            match &binding.kind {
                CapabilityBindingKind::HostProvider {
                    provider_name,
                    admitted_capabilities,
                } => {
                    for grant in admitted_capabilities {
                        action_grants.push(normalized_action_grant(provider_name, grant));
                    }
                }
                CapabilityBindingKind::Implementation { .. } => {
                    for dependency in &binding.dependencies {
                        if let CapabilityBindingDependency::Resource { resource_id, .. } =
                            dependency
                        {
                            resource_grants.push(format!("{resource_id:?}"));
                        }
                    }
                }
            }
        }

        capability_binding_grants.sort();
        capability_binding_grants.dedup();
        resource_grants.sort();
        resource_grants.dedup();
        action_grants.sort();
        action_grants.dedup();

        ExecutionAdmissionFacts::new(capability_binding_grants, resource_grants, action_grants)
    }

    /// Register one root process identity in the runtime process registry.
    pub async fn register_root_process(
        &self,
        process_id: ProcessId,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .register_root(process_id)?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Process,
            RuntimeTraceEvent::Spawn,
            format!("{process_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Register one child process identity in the runtime process registry.
    pub async fn register_child_process(
        &self,
        parent_process_id: ProcessId,
        child_process_id: ProcessId,
        child_index: usize,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry.lock().await.register_child(
            parent_process_id,
            child_process_id,
            child_index,
        )?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Process,
            RuntimeTraceEvent::Spawn,
            format!("{parent_process_id:?}->{child_process_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Register multiple child process identities atomically in the runtime process registry.
    pub async fn register_child_processes_batch(
        &self,
        parent_process_id: ProcessId,
        children: Vec<(ProcessId, usize)>,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .register_children_batch(parent_process_id, children.clone())?;
        for (child_process_id, _) in children {
            self.record_runtime_trace_fact(RuntimeTraceFact::new(
                TraceFactKind::Process,
                RuntimeTraceEvent::Spawn,
                format!("{parent_process_id:?}->{child_process_id:?}"),
            ))
            .await;
        }
        Ok(())
    }

    /// Transition one registered process identity to running.
    pub async fn mark_process_running(
        &self,
        process_id: ProcessId,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .mark_running(process_id)?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Process,
            RuntimeTraceEvent::Start,
            format!("{process_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Record a write-once terminal process state.
    pub async fn record_process_terminal(
        &self,
        process_id: ProcessId,
        terminal_state: ProcessTerminalState,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .record_terminal(process_id, terminal_state.clone())?;
        let event = match terminal_state {
            ProcessTerminalState::Succeeded { .. } => RuntimeTraceEvent::Complete,
            ProcessTerminalState::Failed { .. } => RuntimeTraceEvent::Fail,
            ProcessTerminalState::Cancelled { .. } => RuntimeTraceEvent::Cancel,
        };
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Process,
            event,
            format!("{process_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Look up one process record by identity.
    pub async fn process_record(&self, process_id: ProcessId) -> Option<ProcessRecord> {
        self.process_registry
            .lock()
            .await
            .record(process_id)
            .cloned()
    }

    /// Return one retained terminal process state by identity, if present.
    pub async fn process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        self.process_registry
            .lock()
            .await
            .record(process_id)
            .and_then(|record| record.terminal_state.clone())
    }

    /// Wait until a registered process reaches a retained terminal state.
    pub async fn wait_for_process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        loop {
            let maybe_state = {
                let registry = self.process_registry.lock().await;
                match registry.record(process_id) {
                    Some(record) => record.terminal_state.clone(),
                    None => return None,
                }
            };
            if maybe_state.is_some() {
                return maybe_state;
            }
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    /// Blocking version of [`Self::process_terminal_state`] for synchronous Proc observation.
    pub fn blocking_process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        if let Ok(guard) = self.process_registry.try_lock() {
            return guard
                .record(process_id)
                .and_then(|record| record.terminal_state.clone());
        }

        let registry = self.process_registry.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(async move {
                registry
                    .lock()
                    .await
                    .record(process_id)
                    .and_then(|record| record.terminal_state.clone())
            })
        })
        .join()
        .expect("blocking process terminal-state lookup thread panicked")
    }

    /// Return child process identities for a parent in child-index order.
    pub async fn process_children(&self, parent_process_id: ProcessId) -> Vec<ProcessId> {
        self.process_registry
            .lock()
            .await
            .children_of(parent_process_id)
    }

    /// Record one bounded process propagation diagnostic.
    pub async fn record_process_propagation_diagnostic(
        &self,
        diagnostic: ProcessPropagationDiagnostic,
    ) {
        self.process_propagation_diagnostics
            .lock()
            .await
            .push(diagnostic);
    }

    /// Return retained process propagation diagnostics.
    pub async fn process_propagation_diagnostics(&self) -> Vec<ProcessPropagationDiagnostic> {
        self.process_propagation_diagnostics.lock().await.clone()
    }

    /// Apply one supervisor policy decision to a retained terminal child process state.
    pub async fn supervise_process_terminal(
        &self,
        profile: &SupervisorRuntimeProfile,
        observed_process_id: ProcessId,
    ) -> Result<SupervisorDecisionRecord, SupervisorDiagnostic> {
        let terminal_state = self
            .process_terminal_state(observed_process_id)
            .await
            .ok_or_else(|| SupervisorDiagnostic::ProcessNotTerminal {
                profile_name: profile.profile_name.clone(),
                process_id: observed_process_id,
            })?;
        let observed_outcome = process_outcome_from_terminal_state(&terminal_state);
        let restart_attempt = self
            .supervisor_decisions
            .lock()
            .await
            .iter()
            .filter(|decision| {
                decision.profile_name == profile.profile_name
                    && decision.decision == SupervisorDecisionKind::Restart
            })
            .count() as u32;

        let record = match (&profile.policy, observed_outcome) {
            (_, ProcessPropagationOutcome::Succeeded) => SupervisorDecisionRecord {
                profile_name: profile.profile_name.clone(),
                supervisor_process_id: profile.supervisor_process_id,
                observed_process_id,
                observed_outcome: Some(observed_outcome),
                decision: SupervisorDecisionKind::Complete,
                restart_attempt,
                replacement_process_id: None,
                terminal: true,
                reason: None,
            },
            (SupervisorPolicy::BoundedRestart { max_restarts }, _) => {
                if restart_attempt < *max_restarts {
                    let replacement_process_id = ProcessId::new();
                    self.register_child_process(
                        profile.supervisor_process_id,
                        replacement_process_id,
                        restart_attempt as usize + 1,
                    )
                    .await
                    .map_err(supervisor_registry_failure)?;
                    SupervisorDecisionRecord {
                        profile_name: profile.profile_name.clone(),
                        supervisor_process_id: profile.supervisor_process_id,
                        observed_process_id,
                        observed_outcome: Some(observed_outcome),
                        decision: SupervisorDecisionKind::Restart,
                        restart_attempt: restart_attempt + 1,
                        replacement_process_id: Some(replacement_process_id),
                        terminal: false,
                        reason: Some("child failure restart requested".to_string()),
                    }
                } else {
                    SupervisorDecisionRecord {
                        profile_name: profile.profile_name.clone(),
                        supervisor_process_id: profile.supervisor_process_id,
                        observed_process_id,
                        observed_outcome: Some(observed_outcome),
                        decision: SupervisorDecisionKind::Escalate,
                        restart_attempt,
                        replacement_process_id: None,
                        terminal: true,
                        reason: Some("restart budget exhausted".to_string()),
                    }
                }
            }
            (SupervisorPolicy::Cancel, _) => SupervisorDecisionRecord {
                profile_name: profile.profile_name.clone(),
                supervisor_process_id: profile.supervisor_process_id,
                observed_process_id,
                observed_outcome: Some(observed_outcome),
                decision: SupervisorDecisionKind::Cancel,
                restart_attempt,
                replacement_process_id: None,
                terminal: true,
                reason: Some("supervisor cancel policy selected".to_string()),
            },
            (SupervisorPolicy::Escalate, _) => SupervisorDecisionRecord {
                profile_name: profile.profile_name.clone(),
                supervisor_process_id: profile.supervisor_process_id,
                observed_process_id,
                observed_outcome: Some(observed_outcome),
                decision: SupervisorDecisionKind::Escalate,
                restart_attempt,
                replacement_process_id: None,
                terminal: true,
                reason: Some("supervisor escalation policy selected".to_string()),
            },
            (SupervisorPolicy::Unsupported { reason }, _) => {
                return Err(SupervisorDiagnostic::UnsupportedPolicy {
                    profile_name: profile.profile_name.clone(),
                    reason: reason.clone(),
                });
            }
        };
        self.record_supervisor_decision(record).await
    }

    /// Cancel one supervised process through retained process terminal-state semantics.
    pub async fn cancel_supervised_process(
        &self,
        profile: &SupervisorRuntimeProfile,
        process_id: ProcessId,
        reason: impl Into<String>,
    ) -> Result<SupervisorDecisionRecord, SupervisorDiagnostic> {
        let reason = reason.into();
        let failure = OperationalFailure::new(
            FailureBoundary::Process,
            FailureEntity::Process(process_id),
            Value::String(reason.clone()),
            "String",
        );
        self.record_process_terminal(
            process_id,
            ProcessTerminalState::Cancelled {
                process_id,
                failure: Box::new(failure),
            },
        )
        .await
        .map_err(supervisor_registry_failure)?;
        let restart_attempt = self
            .supervisor_decisions
            .lock()
            .await
            .iter()
            .filter(|decision| {
                decision.profile_name == profile.profile_name
                    && decision.decision == SupervisorDecisionKind::Restart
            })
            .count() as u32;
        let record = SupervisorDecisionRecord {
            profile_name: profile.profile_name.clone(),
            supervisor_process_id: profile.supervisor_process_id,
            observed_process_id: process_id,
            observed_outcome: Some(ProcessPropagationOutcome::Cancelled),
            decision: SupervisorDecisionKind::Cancel,
            restart_attempt,
            replacement_process_id: None,
            terminal: true,
            reason: Some(reason),
        };
        self.record_supervisor_decision(record).await
    }

    /// Return retained supervisor decisions.
    pub async fn supervisor_decisions(&self) -> Vec<SupervisorDecisionRecord> {
        self.supervisor_decisions.lock().await.clone()
    }

    async fn record_supervisor_decision(
        &self,
        record: SupervisorDecisionRecord,
    ) -> Result<SupervisorDecisionRecord, SupervisorDiagnostic> {
        let event = match record.decision {
            SupervisorDecisionKind::Complete => RuntimeTraceEvent::Complete,
            SupervisorDecisionKind::Restart => RuntimeTraceEvent::Restart,
            SupervisorDecisionKind::Cancel => RuntimeTraceEvent::Cancel,
            SupervisorDecisionKind::Escalate => RuntimeTraceEvent::Escalate,
        };
        self.supervisor_decisions.lock().await.push(record.clone());
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Process,
            event,
            format!(
                "supervisor:{}:{:?}",
                record.profile_name, record.observed_process_id
            ),
        ))
        .await;
        Ok(record)
    }

    /// Start and retain one managed service runtime record.
    pub async fn start_service(
        &self,
        service_name: impl Into<String>,
        process_id: ProcessId,
    ) -> Result<ServiceRuntimeRecord, ServiceLifecycleDiagnostic> {
        let service_name = service_name.into();
        validate_service_name(&service_name)?;
        let service_id = ServiceId::new();
        let record = ServiceRuntimeRecord {
            id: service_id,
            name: service_name,
            process_id,
            lifecycle: ServiceLifecycleState::Running,
            health: ServiceHealthStatus::Healthy,
            reload_generation: 0,
            last_reload: None,
            shutdown_mode: None,
            terminal_reason: None,
            terminal: false,
            retained: true,
            report_identity: Some(format!("service-report:{service_id:?}")),
        };
        self.service_records
            .lock()
            .await
            .insert(service_id, record.clone());
        self.record_service_trace(&record, RuntimeTraceEvent::Start)
            .await;
        Ok(record)
    }

    /// Return retained service record by identity.
    pub async fn service_record(&self, service_id: ServiceId) -> Option<ServiceRuntimeRecord> {
        self.service_records.lock().await.get(&service_id).cloned()
    }

    /// Return retained service health without mutating authority.
    pub async fn service_health(
        &self,
        service_id: ServiceId,
    ) -> Result<ServiceHealthReport, ServiceLifecycleDiagnostic> {
        let record = self
            .service_record(service_id)
            .await
            .ok_or(ServiceLifecycleDiagnostic::UnknownService { service_id })?;
        self.record_service_trace(&record, RuntimeTraceEvent::Health)
            .await;
        Ok(ServiceHealthReport {
            service_id,
            lifecycle: record.lifecycle,
            status: record.health,
        })
    }

    /// Apply one bounded service reload and retain the updated record.
    pub async fn reload_service(
        &self,
        service_id: ServiceId,
        reload_identity: impl Into<String>,
    ) -> Result<ServiceRuntimeRecord, ServiceLifecycleDiagnostic> {
        let mut records = self.service_records.lock().await;
        let record = records
            .get_mut(&service_id)
            .ok_or(ServiceLifecycleDiagnostic::UnknownService { service_id })?;
        if record.terminal {
            return Err(ServiceLifecycleDiagnostic::TerminalServiceRetained { service_id });
        }
        record.lifecycle = ServiceLifecycleState::Reloading;
        record.reload_generation = record.reload_generation.saturating_add(1);
        record.last_reload = Some(reload_identity.into());
        record.lifecycle = ServiceLifecycleState::Running;
        record.health = ServiceHealthStatus::Healthy;
        let updated = record.clone();
        drop(records);
        self.record_service_trace(&updated, RuntimeTraceEvent::Reload)
            .await;
        Ok(updated)
    }

    /// Shut down a service and retain its terminal report/state.
    pub async fn shutdown_service(
        &self,
        service_id: ServiceId,
        mode: ServiceShutdownMode,
        reason: impl Into<String>,
    ) -> Result<ServiceRuntimeRecord, ServiceLifecycleDiagnostic> {
        let mut records = self.service_records.lock().await;
        let record = records
            .get_mut(&service_id)
            .ok_or(ServiceLifecycleDiagnostic::UnknownService { service_id })?;
        if record.terminal {
            return Ok(record.clone());
        }
        record.lifecycle = ServiceLifecycleState::Stopping;
        record.shutdown_mode = Some(mode);
        record.terminal_reason = Some(reason.into());
        record.health = ServiceHealthStatus::Unavailable;
        record.lifecycle = ServiceLifecycleState::Terminated;
        record.terminal = true;
        record.retained = true;
        let updated = record.clone();
        drop(records);
        self.record_service_trace(&updated, RuntimeTraceEvent::Shutdown)
            .await;
        Ok(updated)
    }

    async fn record_service_trace(&self, record: &ServiceRuntimeRecord, event: RuntimeTraceEvent) {
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Service,
            event,
            format!("{}:{:?}", record.name, record.id),
        ))
        .await;
    }

    /// Register one trusted runtime adapter as runtime-owned host metadata.
    pub async fn register_trusted_runtime_adapter(
        &self,
        adapter: TrustedRuntimeAdapter,
    ) -> Result<TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic> {
        adapter.validate()?;
        let name = adapter.name.clone();
        let version = adapter.version.clone();
        let report_identity = adapter.report_identity.clone();
        self.trusted_runtime_adapters
            .lock()
            .await
            .insert(name.clone(), adapter.clone());
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Operation,
            RuntimeTraceEvent::Register,
            format!("adapter:{name}:{version}:{report_identity}"),
        ))
        .await;
        Ok(adapter)
    }

    /// Return retained trusted runtime adapter metadata by name.
    pub async fn trusted_runtime_adapter(&self, name: &str) -> Option<TrustedRuntimeAdapter> {
        self.trusted_runtime_adapters
            .lock()
            .await
            .get(name)
            .cloned()
    }

    /// Look up one trusted runtime adapter and require an exact version match.
    pub async fn require_trusted_runtime_adapter(
        &self,
        adapter_name: &str,
        version: &str,
    ) -> Result<TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic> {
        let adapter = self
            .trusted_runtime_adapters
            .lock()
            .await
            .get(adapter_name)
            .cloned()
            .ok_or_else(|| TrustedRuntimeAdapterDiagnostic::UnknownAdapter {
                adapter_name: adapter_name.to_string(),
            })?;
        if adapter.version != version {
            return Err(TrustedRuntimeAdapterDiagnostic::StaleAdapter {
                adapter_name: adapter_name.to_string(),
                requested_version: version.to_string(),
                registered_version: adapter.version,
            });
        }
        Ok(adapter)
    }

    /// Validate a trusted runtime adapter against provider operation metadata before execution.
    pub async fn validate_trusted_runtime_adapter_for_provider_operation(
        &self,
        adapter_name: &str,
        version: &str,
        provider_metadata: &ProviderAuthoringMetadata,
        operation_name: &str,
    ) -> Result<TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic> {
        validate_provider_authoring_metadata(provider_metadata).map_err(|error| {
            TrustedRuntimeAdapterDiagnostic::IncompatibleAdapter {
                adapter_name: adapter_name.to_string(),
                reason: format!("provider metadata invalid: {error}"),
            }
        })?;

        let adapter = self
            .require_trusted_runtime_adapter(adapter_name, version)
            .await?;
        let operation = provider_metadata.operation(operation_name).ok_or_else(|| {
            TrustedRuntimeAdapterDiagnostic::IncompatibleAdapter {
                adapter_name: adapter.name.clone(),
                reason: format!(
                    "provider metadata {}.{operation_name} is missing",
                    provider_metadata.provider_name
                ),
            }
        })?;

        let TrustedRuntimeAdapterTarget::ProviderOperation {
            provider_name,
            operation_name: adapter_operation,
            required_row,
        } = &adapter.target
        else {
            return Err(TrustedRuntimeAdapterDiagnostic::IncompatibleAdapter {
                adapter_name: adapter.name.clone(),
                reason: "adapter target is not a provider operation".to_string(),
            });
        };

        if provider_name != &provider_metadata.provider_name
            || adapter_operation != operation_name
            || !operation.required_rows.contains(required_row)
            || adapter.sandbox_policy != operation.sandbox_policy.clone().unwrap_or_default()
            || adapter.provenance_policy != operation.provenance_policy.clone().unwrap_or_default()
        {
            return Err(TrustedRuntimeAdapterDiagnostic::IncompatibleAdapter {
                adapter_name: adapter.name.clone(),
                reason: format!(
                    "adapter target {provider_name}.{adapter_operation}/{required_row} does not match provider metadata {}.{}",
                    provider_metadata.provider_name, operation.operation_name
                ),
            });
        }

        Ok(adapter)
    }

    /// Register one host sandbox policy by identity.
    pub async fn register_host_sandbox_policy(
        &self,
        policy: HostSandboxPolicy,
    ) -> ExecResult<HostSandboxPolicy> {
        if policy.identity.is_empty() {
            return Err(ExecError::InvalidRuntimeState(
                "host sandbox policy is missing identity".to_string(),
            ));
        }
        self.host_sandbox_policies
            .lock()
            .await
            .insert(policy.identity.clone(), policy.clone());
        Ok(policy)
    }

    /// Return retained redacted host sandbox denial evidence.
    pub async fn host_sandbox_denials(&self) -> Vec<HostSandboxDenialRecord> {
        self.host_sandbox_denials.lock().await.clone()
    }

    /// Return retained redacted host boundary evidence.
    pub async fn host_boundary_evidence(&self) -> Vec<HostBoundaryEvidence> {
        self.host_boundary_evidence.lock().await.clone()
    }

    /// Register one external actor adapter at an explicit capability boundary.
    pub async fn register_external_actor_adapter(
        &self,
        adapter: ExternalActorAdapter,
    ) -> Result<ExternalActorAdapter, ExternalActorDiagnostic> {
        let name = adapter.name.clone();
        self.external_actor_adapters
            .lock()
            .await
            .insert(name.clone(), adapter.clone());
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::ExternalActor,
            RuntimeTraceEvent::Register,
            format!("adapter:{name}:{:?}", adapter.id),
        ))
        .await;
        Ok(adapter)
    }

    /// Return retained external actor adapter metadata by name.
    pub async fn external_actor_adapter(&self, name: &str) -> Option<ExternalActorAdapter> {
        self.external_actor_adapters.lock().await.get(name).cloned()
    }

    /// Record one successful external actor call after inbound/outbound boundary validation.
    pub async fn record_external_actor_call(
        &self,
        adapter_name: &str,
        payload: Value,
        response: Value,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let adapter = self.lookup_external_actor_adapter(adapter_name).await?;
        validate_actor_payload(&adapter, &payload)?;
        validate_actor_response(&adapter, &response)?;
        self.record_actor_call(
            &adapter,
            ActorCallOutcome::Succeeded,
            0,
            true,
            Some(actor_value_type_name(&response).to_string()),
            None,
            RuntimeTraceEvent::Send,
        )
        .await
    }

    /// Record one external actor failure with bounded diagnostic evidence.
    pub async fn record_external_actor_failure(
        &self,
        adapter_name: &str,
        payload: Value,
        diagnostic: impl Into<String>,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let adapter = self.lookup_external_actor_adapter(adapter_name).await?;
        validate_actor_payload(&adapter, &payload)?;
        self.record_actor_call(
            &adapter,
            ActorCallOutcome::Failed,
            0,
            false,
            None,
            Some(diagnostic.into()),
            RuntimeTraceEvent::Fail,
        )
        .await
    }

    /// Record one external actor timeout as a terminal retained call state.
    pub async fn record_external_actor_timeout(
        &self,
        adapter_name: &str,
        payload: Value,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let adapter = self.lookup_external_actor_adapter(adapter_name).await?;
        validate_actor_payload(&adapter, &payload)?;
        self.record_actor_call(
            &adapter,
            ActorCallOutcome::TimedOut,
            0,
            true,
            None,
            Some(format!("timeout after {}ms", adapter.policy.timeout_millis)),
            RuntimeTraceEvent::Fail,
        )
        .await
    }

    /// Schedule one bounded retry for a retained actor call.
    pub async fn retry_external_actor_call(
        &self,
        call_id: ActorCallId,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let mut calls = self.external_actor_calls.lock().await;
        let record = calls
            .get_mut(&call_id)
            .ok_or(ExternalActorDiagnostic::UnknownCall { call_id })?;
        if record.terminal {
            return Err(ExternalActorDiagnostic::TerminalCallRetained { call_id });
        }
        let adapter = self
            .external_actor_adapters
            .lock()
            .await
            .get(&record.adapter_name)
            .cloned()
            .ok_or_else(|| ExternalActorDiagnostic::UnknownAdapter {
                adapter_name: record.adapter_name.clone(),
            })?;
        if record.retry_attempt >= adapter.policy.max_retries {
            return Err(ExternalActorDiagnostic::RetryBudgetExhausted {
                call_id,
                max_retries: adapter.policy.max_retries,
            });
        }
        record.retry_attempt = record.retry_attempt.saturating_add(1);
        record.outcome = ActorCallOutcome::RetryScheduled;
        record.terminal = false;
        record.diagnostic = Some("retry scheduled".to_string());
        let updated = record.clone();
        drop(calls);
        self.record_external_actor_trace(&updated, RuntimeTraceEvent::Restart)
            .await;
        Ok(updated)
    }

    /// Cancel one retained external actor call.
    pub async fn cancel_external_actor_call(
        &self,
        call_id: ActorCallId,
        reason: impl Into<String>,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let mut calls = self.external_actor_calls.lock().await;
        let record = calls
            .get_mut(&call_id)
            .ok_or(ExternalActorDiagnostic::UnknownCall { call_id })?;
        if record.terminal {
            return Err(ExternalActorDiagnostic::TerminalCallRetained { call_id });
        }
        record.outcome = ActorCallOutcome::Cancelled;
        record.terminal = true;
        record.diagnostic = Some(reason.into());
        let updated = record.clone();
        drop(calls);
        self.record_external_actor_trace(&updated, RuntimeTraceEvent::Cancel)
            .await;
        Ok(updated)
    }

    async fn lookup_external_actor_adapter(
        &self,
        adapter_name: &str,
    ) -> Result<ExternalActorAdapter, ExternalActorDiagnostic> {
        self.external_actor_adapters
            .lock()
            .await
            .get(adapter_name)
            .cloned()
            .ok_or_else(|| ExternalActorDiagnostic::UnknownAdapter {
                adapter_name: adapter_name.to_string(),
            })
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_actor_call(
        &self,
        adapter: &ExternalActorAdapter,
        outcome: ActorCallOutcome,
        retry_attempt: u32,
        terminal: bool,
        response_type: Option<String>,
        diagnostic: Option<String>,
        event: RuntimeTraceEvent,
    ) -> Result<ExternalActorCallRecord, ExternalActorDiagnostic> {
        let call_id = ActorCallId::new();
        let trace_subject = format!("actor:{}:{call_id:?}:{outcome:?}", adapter.name);
        let record = ExternalActorCallRecord {
            call_id,
            adapter_id: adapter.id,
            adapter_name: adapter.name.clone(),
            actor_type: adapter.actor_type.clone(),
            capability_boundary: adapter.capability_boundary.clone(),
            protocol: adapter.protocol.clone(),
            outcome,
            retry_attempt,
            terminal,
            payload_type: adapter.actor_type.clone(),
            response_type,
            payload_redaction: "redacted".to_string(),
            trace_subject,
            diagnostic,
        };
        self.external_actor_calls
            .lock()
            .await
            .insert(call_id, record.clone());
        self.record_external_actor_trace(&record, event).await;
        Ok(record)
    }

    async fn record_external_actor_trace(
        &self,
        record: &ExternalActorCallRecord,
        event: RuntimeTraceEvent,
    ) {
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::ExternalActor,
            event,
            record.trace_subject.clone(),
        ))
        .await;
    }

    /// Record one runtime trace fact and matching authority-free monitor evidence row.
    pub async fn record_runtime_trace_fact(&self, fact: RuntimeTraceFact) {
        self.runtime_trace_facts.lock().await.push(fact.clone());
        self.runtime_monitor_evidence
            .lock()
            .await
            .push(RuntimeMonitorEvidence::new(
                "phase-195-runtime-monitor",
                "phase-195-runtime-trace",
                format!("{:?}:{:?}", fact.kind, fact.event),
                MonitorEvaluationResult::Pending,
            ));
    }

    /// Return retained runtime trace facts.
    pub async fn runtime_trace_facts(&self) -> Vec<RuntimeTraceFact> {
        self.runtime_trace_facts.lock().await.clone()
    }

    /// Return retained runtime monitor evidence rows.
    pub async fn runtime_monitor_evidence(&self) -> Vec<RuntimeMonitorEvidence> {
        self.runtime_monitor_evidence.lock().await.clone()
    }

    /// Create one bounded typed runtime channel.
    pub async fn create_channel(
        &self,
        payload_type: ash_typeck::Type,
        capacity: usize,
    ) -> ChannelId {
        self.channel_registry
            .lock()
            .await
            .create(payload_type, capacity)
    }

    /// Send one value through a runtime channel after type and sendability validation.
    pub async fn send_channel(
        &self,
        channel_id: ChannelId,
        value: Value,
    ) -> Result<(), ChannelError> {
        self.channel_registry.lock().await.send(channel_id, value)?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Channel,
            RuntimeTraceEvent::Send,
            format!("{channel_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Try to receive one value from a runtime channel without blocking.
    pub async fn try_receive_channel(&self, channel_id: ChannelId) -> Result<Value, ChannelError> {
        let value = self.channel_registry.lock().await.try_receive(channel_id)?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Channel,
            RuntimeTraceEvent::Receive,
            format!("{channel_id:?}"),
        ))
        .await;
        Ok(value)
    }

    /// Close a runtime channel against future sends.
    pub async fn close_channel(&self, channel_id: ChannelId) -> Result<(), ChannelError> {
        self.channel_registry.lock().await.close(channel_id)?;
        self.record_runtime_trace_fact(RuntimeTraceFact::new(
            TraceFactKind::Channel,
            RuntimeTraceEvent::Close,
            format!("{channel_id:?}"),
        ))
        .await;
        Ok(())
    }

    /// Return a ready channel for supported select shapes.
    pub async fn select_ready_channel(
        &self,
        channel_ids: &[ChannelId],
    ) -> Result<Option<ChannelId>, ChannelError> {
        self.channel_registry.lock().await.select_ready(channel_ids)
    }

    /// Register or replace one runtime-owned resource instance by stable identity.
    ///
    /// This stores environment-owned resource metadata only; it does not expose resources as
    /// first-class [`Value`] handles or execute resource-backed capability operations.
    pub async fn register_resource_instance(&self, instance: ResourceInstance) {
        self.resource_instances
            .lock()
            .await
            .insert(instance.id, instance);
    }

    /// Look up one runtime-owned resource instance by identity.
    pub async fn resource_instance(&self, resource_id: ResourceId) -> Option<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .get(&resource_id)
            .cloned()
    }

    /// Return true if a resource instance with `resource_id` is registered.
    pub async fn has_resource_instance(&self, resource_id: ResourceId) -> bool {
        self.resource_instances
            .lock()
            .await
            .contains_key(&resource_id)
    }

    /// Return resource instances matching one owner scope and resource type identifier.
    ///
    /// Runtime resource lookup remains scoped by owner to avoid ambient type-only discovery across
    /// unrelated runs, applications, processes, effect scopes, or test scopes.
    pub async fn resource_instances_for_owner_by_type(
        &self,
        owner: ResourceOwner,
        type_id: ResourceTypeId,
    ) -> Vec<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .values()
            .filter(|instance| instance.owner == owner && instance.type_id == type_id)
            .cloned()
            .collect()
    }

    /// Return all resource instances owned by one runtime owner scope.
    pub async fn resource_instances_for_owner(
        &self,
        owner: ResourceOwner,
    ) -> Vec<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .values()
            .filter(|instance| instance.owner == owner)
            .cloned()
            .collect()
    }

    /// Check and record process resource policy before a Proc split admits children.
    pub async fn apply_process_resource_split(
        &self,
        parent_process_id: ProcessId,
        child_count: usize,
        operation: &'static str,
    ) -> Result<(), ResourceSplitJoinViolation> {
        if child_count == 0 {
            return Ok(());
        }

        let mut instances = self.resource_instances.lock().await;
        let mut parent_resource_ids = instances
            .iter()
            .filter_map(|(id, instance)| {
                (instance.owner == ResourceOwner::Process(parent_process_id)).then_some(*id)
            })
            .collect::<Vec<_>>();
        parent_resource_ids.sort_by_key(|id| id.0);

        for resource_id in &parent_resource_ids {
            let instance = instances
                .get(resource_id)
                .expect("resource id collected from map must still exist");
            match instance.split_join_policy {
                ResourceSplitJoinPolicy::ReadOnlyShare
                | ResourceSplitJoinPolicy::CommunicationOnly
                | ResourceSplitJoinPolicy::Mergeable => {}
                ResourceSplitJoinPolicy::NonShareable => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "non-shareable resource cannot cross a process split",
                    ));
                }
                ResourceSplitJoinPolicy::BranchLocalClone => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "branch-local clone resource has no runtime clone implementation in the MVP",
                    ));
                }
                ResourceSplitJoinPolicy::LinearMove => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "linear-move resource requires an explicit destination child in the MVP",
                    ));
                }
            }
        }

        for resource_id in parent_resource_ids {
            if let Some(instance) = instances.get_mut(&resource_id)
                && matches!(
                    instance.split_join_policy,
                    ResourceSplitJoinPolicy::ReadOnlyShare
                        | ResourceSplitJoinPolicy::CommunicationOnly
                        | ResourceSplitJoinPolicy::Mergeable
                )
            {
                instance.lifecycle = ResourceLifecycle::Splitting;
            }
        }

        Ok(())
    }

    /// Apply parent-owned resource merge policy after `join`/`gather` observes child success.
    pub async fn apply_process_resource_join(
        &self,
        parent_process_id: ProcessId,
        child_process_ids: &[ProcessId],
        operation: &'static str,
    ) -> Result<(), ResourceSplitJoinViolation> {
        if child_process_ids.is_empty() {
            return Ok(());
        }

        let mut instances = self.resource_instances.lock().await;
        let mut parent_resource_ids = instances
            .iter()
            .filter_map(|(id, instance)| {
                (instance.owner == ResourceOwner::Process(parent_process_id)
                    && instance.lifecycle == ResourceLifecycle::Splitting)
                    .then_some(*id)
            })
            .collect::<Vec<_>>();
        parent_resource_ids.sort_by_key(|id| id.0);

        for resource_id in &parent_resource_ids {
            let instance = instances
                .get(resource_id)
                .expect("resource id collected from map must still exist");
            match instance.split_join_policy {
                ResourceSplitJoinPolicy::Mergeable => {}
                ResourceSplitJoinPolicy::ReadOnlyShare
                | ResourceSplitJoinPolicy::CommunicationOnly => {}
                ResourceSplitJoinPolicy::NonShareable
                | ResourceSplitJoinPolicy::BranchLocalClone
                | ResourceSplitJoinPolicy::LinearMove => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "split resource cannot be joined by the MVP merge policy",
                    ));
                }
            }
        }

        for resource_id in parent_resource_ids {
            if let Some(instance) = instances.get_mut(&resource_id) {
                match instance.split_join_policy {
                    ResourceSplitJoinPolicy::Mergeable => {
                        instance.lifecycle = ResourceLifecycle::Joined;
                    }
                    ResourceSplitJoinPolicy::ReadOnlyShare
                    | ResourceSplitJoinPolicy::CommunicationOnly => {
                        instance.lifecycle = ResourceLifecycle::Active;
                    }
                    ResourceSplitJoinPolicy::NonShareable
                    | ResourceSplitJoinPolicy::BranchLocalClone
                    | ResourceSplitJoinPolicy::LinearMove => {}
                }
            }
        }

        Ok(())
    }

    /// Return the number of registered resource instances.
    pub async fn resource_instance_count(&self) -> usize {
        self.resource_instances.lock().await.len()
    }

    /// Return the live control-link state for a application identity, if registered.
    pub async fn control_link_state(
        &self,
        instance_id: ash_core::ApplicationId,
    ) -> Option<LinkState> {
        let link = ControlLink { instance_id };
        self.control_registry.lock().await.check_health(&link).ok()
    }

    /// Register one spawned control target in the shared runtime state.
    pub async fn register_spawned_control_link(&self, instance_id: ash_core::ApplicationId) {
        self.control_registry.lock().await.register(instance_id);
    }

    /// Register one spawned control target together with the conservative runtime-owned spawn
    /// provenance the runtime can snapshot today.
    pub async fn register_spawned_control_link_with_provenance(
        &self,
        provenance: ConservativeRetainedProvenanceSummary,
    ) {
        self.control_registry
            .lock()
            .await
            .register_with_spawn_provenance(provenance);
    }

    /// Pause one controlled runtime target.
    pub async fn pause_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.pause(link)
    }

    /// Resume one controlled runtime target.
    pub async fn resume_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.resume(link)
    }

    /// Kill one controlled runtime target.
    pub async fn kill_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.kill(link)
    }

    /// Build the initial input bindings for a runtime-owned child entry execution.
    ///
    /// In addition to the user-visible `init` binding, the runtime injects one internal control
    /// binding so spawned child execution can cooperatively observe pause/resume/kill authority.
    pub fn spawned_child_init_bindings(
        init_value: Value,
        control_link: ControlLink,
    ) -> HashMap<String, Value> {
        HashMap::from([
            ("init".to_string(), init_value),
            (
                SPAWNED_CHILD_CONTROL_BINDING.to_string(),
                Value::ControlLink(control_link),
            ),
        ])
    }

    /// Wait until a controlled spawned child is allowed to make progress.
    ///
    /// This is intentionally cooperative rather than preemptive: it checks the control state at
    /// application entry boundaries, blocks while paused, and stops further progress after kill.
    pub async fn wait_for_control_authority(&self, link: &ControlLink) -> crate::ExecResult<()> {
        loop {
            let state = self.control_registry.lock().await.check_health(link);
            match state {
                Ok(LinkState::Running) => return Ok(()),
                Ok(LinkState::Paused) => tokio::time::sleep(Duration::from_millis(1)).await,
                Ok(LinkState::Terminated) => unreachable!(
                    "terminated links are reported as errors by ControlLinkRegistry::check_health"
                ),
                Err(error) => {
                    return Err(ExecError::InvalidRuntimeState(format!(
                        "spawned child control wait failed for instance {:?}: {error}",
                        link.instance_id
                    )));
                }
            }
        }
    }

    /// Record one retained terminal completion-style observation for a control target.
    pub async fn record_control_completion(
        &self,
        link: &ControlLink,
        result: ExecResult<Value>,
        effects: ConservativeRetainedEffectSummary,
        obligations: ConservativeRetainedObligationsSummary,
        provenance: Option<ConservativeRetainedProvenanceSummary>,
    ) -> Result<RetainedCompletionRecord, crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.record_completion(
            link,
            result,
            effects,
            obligations,
            provenance,
        )
    }

    /// Read the retained terminal completion-style observation for a control target, if one
    /// exists.
    ///
    /// This surface answers a different question from
    /// [`Self::control_link_runtime_outcome_state`]: once a control link is no longer live, that
    /// coarse runtime state reports terminal control-liveness (`InvalidOrTerminated`), while this
    /// retained record preserves the sealed terminal completion subtype/payload captured for that
    /// target.
    pub async fn retained_completion(
        &self,
        link: &ControlLink,
    ) -> Option<RetainedCompletionRecord> {
        self.control_registry
            .lock()
            .await
            .retained_completion(&link.instance_id)
    }

    /// Wait for the first sealed retained completion-style observation for a control target.
    ///
    /// This reuses the same retained completion carrier returned by [`Self::retained_completion`].
    /// If the target is already sealed when waiting begins, this returns immediately. If the
    /// target is registered but not yet sealed, this waits until the first authoritative retained
    /// record is sealed. Invalid or unregistered targets remain distinguishable as
    /// [`crate::control_link::ControlLinkError`] values rather than being synthesized into a fake
    /// completion record.
    pub async fn wait_for_retained_completion(
        &self,
        link: &ControlLink,
    ) -> Result<RetainedCompletionRecord, crate::control_link::ControlLinkError> {
        let waiter = {
            self.control_registry
                .lock()
                .await
                .retained_completion_waiter(link)?
        };

        match waiter {
            RetainedCompletionWaiter::Ready(record) => Ok(*record),
            RetainedCompletionWaiter::Pending(mut receiver) => loop {
                if let Some(record) = receiver.borrow().clone() {
                    return Ok(record);
                }

                if receiver.changed().await.is_err() {
                    return Err(crate::control_link::ControlLinkError::NotFound(
                        link.instance_id,
                    ));
                }
            },
        }
    }

    /// Classify the current runtime-visible control-liveness state of a control link using the
    /// authoritative runtime outcome/state surface.
    ///
    /// This method reports whether the control authority is still live/usable. After a child has
    /// sealed a retained completion and the link is terminal, this surface intentionally reports
    /// `InvalidOrTerminated`; callers that need the sealed terminal completion subtype/payload must
    /// consult [`Self::retained_completion`].
    pub async fn control_link_runtime_outcome_state(
        &self,
        link: &ControlLink,
    ) -> RuntimeOutcomeState {
        let registry = self.control_registry.lock().await;
        match registry.check_health(link) {
            Ok(state) => state.runtime_outcome_state(),
            Err(error) => error.runtime_outcome_state(),
        }
    }

    /// Store the most recent authoritative top-level execution record observed for this runtime state.
    ///
    /// Spawned-child and other auxiliary execution instances must not overwrite this slot; they own
    /// distinct execution instances and should surface their terminal state through retained-completion
    /// or other instance-scoped carriers instead.
    pub async fn set_last_execution_record(&self, record: ExecutionRecord) {
        *self.last_execution_record.lock().await = Some(record);
    }

    /// Read the most recent authoritative top-level execution record observed for this runtime state.
    pub async fn last_execution_record(&self) -> Option<ExecutionRecord> {
        (*self.last_execution_record.lock().await).clone()
    }
}

fn process_outcome_from_terminal_state(
    terminal_state: &ProcessTerminalState,
) -> ProcessPropagationOutcome {
    match terminal_state {
        ProcessTerminalState::Succeeded { .. } => ProcessPropagationOutcome::Succeeded,
        ProcessTerminalState::Failed { .. } => ProcessPropagationOutcome::Failed,
        ProcessTerminalState::Cancelled { .. } => ProcessPropagationOutcome::Cancelled,
    }
}

fn supervisor_registry_failure(error: ProcessRegistryError) -> SupervisorDiagnostic {
    SupervisorDiagnostic::RuntimeRegistryFailure {
        message: error.to_string(),
    }
}

fn validate_service_name(service_name: &str) -> Result<(), ServiceLifecycleDiagnostic> {
    if service_name.is_empty() {
        return Err(ServiceLifecycleDiagnostic::MissingServiceName);
    }
    if !service_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
    {
        return Err(ServiceLifecycleDiagnostic::MalformedServiceName {
            service_name: service_name.to_string(),
        });
    }
    Ok(())
}

fn validate_actor_payload(
    adapter: &ExternalActorAdapter,
    payload: &Value,
) -> Result<(), ExternalActorDiagnostic> {
    if !actor_schema_matches(&adapter.inbound_schema, payload) {
        return Err(ExternalActorDiagnostic::InboundTypeMismatch {
            adapter_name: adapter.name.clone(),
            expected: adapter.inbound_schema.clone(),
            actual: actor_value_type_name(payload).to_string(),
        });
    }
    payload
        .validate_sendable_for_process_boundary()
        .map_err(|reason| ExternalActorDiagnostic::NonSendablePayload {
            adapter_name: adapter.name.clone(),
            reason: reason.to_string(),
        })
}

fn validate_actor_response(
    adapter: &ExternalActorAdapter,
    response: &Value,
) -> Result<(), ExternalActorDiagnostic> {
    if !actor_schema_matches(&adapter.outbound_schema, response) {
        return Err(ExternalActorDiagnostic::OutboundTypeMismatch {
            adapter_name: adapter.name.clone(),
            expected: adapter.outbound_schema.clone(),
            actual: actor_value_type_name(response).to_string(),
        });
    }
    response
        .validate_sendable_for_process_boundary()
        .map_err(|reason| ExternalActorDiagnostic::NonSendablePayload {
            adapter_name: adapter.name.clone(),
            reason: reason.to_string(),
        })
}

fn actor_schema_matches(schema: &str, value: &Value) -> bool {
    match schema {
        "Int" => matches!(value, Value::Int(_)),
        "Float" => matches!(value, Value::Float(_)),
        "String" => matches!(value, Value::String(_)),
        "Bool" => matches!(value, Value::Bool(_)),
        "Null" => matches!(value, Value::Null),
        "Time" => matches!(value, Value::Time(_)),
        "Ref" => matches!(value, Value::Ref(_)),
        "Record" => matches!(value, Value::Record(_)),
        _ if schema.starts_with('{') && schema.ends_with('}') => {
            actor_record_schema_matches(schema, value)
        }
        _ => true,
    }
}

fn actor_record_schema_matches(schema: &str, value: &Value) -> bool {
    let Value::Record(record) = value else {
        return false;
    };
    let body = schema.trim_start_matches('{').trim_end_matches('}').trim();
    if body.is_empty() {
        return record.is_empty();
    }
    body.split(',').all(|field| {
        let Some((name, field_schema)) = field.trim().split_once(':') else {
            return false;
        };
        record
            .get(name.trim())
            .is_some_and(|field_value| actor_schema_matches(field_schema.trim(), field_value))
    })
}

fn actor_value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::String(_) => "String",
        Value::Bool(_) => "Bool",
        Value::Null => "Null",
        Value::Time(_) => "Time",
        Value::Ref(_) => "Ref",
        Value::Record(_) => "Record",
        Value::Cap(_) => "Cap",
        Value::Variant { .. } => "Variant",
        Value::Instance(_) => "Instance",
        Value::InstanceAddr(_) => "InstanceAddr",
        Value::ControlLink(_) => "ControlLink",
        Value::Stream(_) => "Stream",
        Value::ProcessHandle(_) => "ProcessHandle",
        Value::ProcAwaitCapture(_) => "ProcAwaitCapture",
        Value::ProcYieldCapture => "ProcYieldCapture",
        Value::ProcParCapture { .. } => "ProcParCapture",
        Value::ProcScatterCapture { .. } => "ProcScatterCapture",
        Value::ProcJoinCapture { .. } => "ProcJoinCapture",
        Value::ProcGatherCapture { .. } => "ProcGatherCapture",
        Value::Closure { .. } => "Closure",
        Value::ActEnvToken => "ActEnvToken",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Effect;
    use tokio::time::{Duration, timeout};

    fn retained_effect_summary(
        terminal: Effect,
        reached: &[Effect],
    ) -> crate::control_link::ConservativeRetainedEffectSummary {
        crate::control_link::ConservativeRetainedEffectSummary::new(
            terminal,
            reached.iter().copied().collect(),
        )
    }

    fn retained_obligations_summary() -> crate::control_link::ConservativeRetainedObligationsSummary
    {
        crate::control_link::ConservativeRetainedObligationsSummary::new(
            std::collections::BTreeSet::new(),
            None,
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        )
    }

    #[test]
    fn provider_builder_registers_provider() {
        let runtime_state = RuntimeState::new().with_provider(
            "test",
            Arc::new(crate::capability::MockProvider::new(
                "test",
                Effect::Epistemic,
            )),
        );

        assert!(runtime_state.has_provider("test"));
        assert_eq!(runtime_state.provider_count(), 1);
        assert_eq!(runtime_state.provider_names(), vec!["test".to_string()]);
        assert!(runtime_state.get_provider("test").is_some());
    }

    #[test]
    fn providers_builder_registers_all_providers() {
        let mut providers: HashMap<String, Arc<dyn CapabilityProvider>> = HashMap::new();
        providers.insert(
            "one".to_string(),
            Arc::new(crate::capability::MockProvider::new(
                "one",
                Effect::Epistemic,
            )),
        );
        providers.insert(
            "two".to_string(),
            Arc::new(crate::capability::MockProvider::new(
                "two",
                Effect::Epistemic,
            )),
        );

        let runtime_state = RuntimeState::new().with_providers(providers);

        assert!(runtime_state.has_provider("one"));
        assert!(runtime_state.has_provider("two"));
        assert_eq!(runtime_state.provider_count(), 2);
    }

    #[tokio::test]
    async fn control_link_runtime_outcome_state_reports_active_then_invalid() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::ApplicationId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;

        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::Active
        );

        runtime_state.kill_control_link(&link).await.unwrap();

        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::InvalidOrTerminated
        );
    }

    #[tokio::test]
    async fn retained_completion_round_trips_through_runtime_state() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::ApplicationId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let effects = retained_effect_summary(Effect::Operational, &[Effect::Operational]);
        let obligations = retained_obligations_summary();
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(7)),
                effects.clone(),
                obligations.clone(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(runtime_state.retained_completion(&link).await, Some(record));
        assert_eq!(
            runtime_state
                .retained_completion(&link)
                .await
                .unwrap()
                .conservative_effect_summary(),
            Some(&effects)
        );
        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::InvalidOrTerminated
        );
    }

    #[tokio::test]
    async fn retained_completion_is_write_once_through_runtime_state() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::ApplicationId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let effects = retained_effect_summary(Effect::Operational, &[Effect::Operational]);
        let obligations = retained_obligations_summary();
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(1)),
                effects.clone(),
                obligations.clone(),
                None,
            )
            .await
            .unwrap();
        let error = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(2)),
                retained_effect_summary(Effect::Epistemic, &[Effect::Epistemic]),
                retained_obligations_summary(),
                None,
            )
            .await
            .expect_err("retained completion should be sealed after first record");

        assert_eq!(
            error,
            crate::control_link::ControlLinkError::CompletionAlreadySealed(
                instance_id,
                Box::new(record.clone())
            )
        );
        assert_eq!(runtime_state.retained_completion(&link).await, Some(record));
    }

    #[tokio::test]
    async fn wait_for_retained_completion_returns_immediately_for_already_sealed_record() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::ApplicationId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(7)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                None,
            )
            .await
            .expect("completion should already be sealed");

        let waited = timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("already-sealed record should return immediately")
        .expect("already-sealed record should still be readable");

        assert_eq!(waited, record);
    }

    #[tokio::test]
    async fn wait_for_retained_completion_rejects_unregistered_targets() {
        let runtime_state = RuntimeState::new();
        let link = ControlLink {
            instance_id: ash_core::ApplicationId::new(),
        };

        let error = timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("unregistered completion wait should not hang")
        .expect_err("unregistered completion wait should not synthesize a retained record");

        assert!(matches!(
            error,
            crate::control_link::ControlLinkError::NotFound(not_found_id)
                if not_found_id == link.instance_id
        ));
    }
}
