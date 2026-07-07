//! Runtime identity and failure carrier substrate.
//!
//! This module contains identity newtypes and inert carrier types used by the
//! process/workflow runtime semantics. It intentionally does not wire runtime
//! admission, scheduling, or `Proc` operations.

use crate::{Value, WorkflowId, core_ash_contract::TraceFactKind};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// A unique identifier for one concrete runtime resource instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    /// Create a fresh resource instance identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable runtime identifier for an Ash resource type declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceTypeId(String);

impl ResourceTypeId {
    /// Create a resource type identifier from a static/type-checker resource type name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow the resource type name carried by this identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ResourceTypeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ResourceTypeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Runtime owner scope for a resource instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceOwner {
    /// Resource admitted for the whole run.
    Run(RunId),
    /// Resource owned by one workflow execution.
    Workflow(WorkflowId),
    /// Resource owned by one process.
    Process(ProcessId),
    /// Resource owned by one effectful/Act scope.
    EffectScope(EffectScopeId),
    /// Resource owned by one test harness execution.
    Test(TestId),
}

/// A unique identifier for one runtime test harness scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TestId(pub Uuid);

impl TestId {
    /// Create a fresh test scope identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TestId {
    fn default() -> Self {
        Self::new()
    }
}

/// Current lifecycle state of a runtime resource instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceLifecycle {
    /// Instance identity exists before admission to an execution scope.
    Allocated,
    /// Instance has been admitted to an owner scope.
    Admitted,
    /// Instance is active and available for later resource-backed operations.
    Active,
    /// Instance is being projected across a process split.
    Splitting,
    /// Split instance state has been joined.
    Joined,
    /// Instance has been released by or from its owner scope.
    Released,
    /// Instance reached a failed terminal resource state.
    Failed,
}

impl ResourceLifecycle {
    /// Return true for terminal resource lifecycle states.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Released | Self::Failed)
    }
}

/// MVP access policy categories for a resource instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessPolicy {
    /// Resource may be observed but not mutated.
    ReadOnly,
    /// Resource may be mutated by an admitted resource-backed operation.
    ReadWrite,
    /// Resource requires exclusive access by one owner/user at a time.
    Exclusive,
}

/// MVP process split/join/share/move policy categories for a resource instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceSplitJoinPolicy {
    /// Branches may share immutable/read-only access.
    ReadOnlyShare,
    /// Each branch receives isolated cloned state.
    BranchLocalClone,
    /// One branch receives ownership; others do not.
    LinearMove,
    /// Branch states can be joined by a later merge operation.
    Mergeable,
    /// Resource cannot cross a process split.
    NonShareable,
    /// Resource is accessed only through message/handle protocols.
    CommunicationOnly,
}

/// Minimal opaque runtime-owned resource state descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResourceRuntimeState {
    /// No runtime state payload is attached yet.
    #[default]
    Empty,
    /// Opaque host/runtime descriptor for state stored outside first-class Ash values.
    Opaque(String),
}

impl ResourceRuntimeState {
    /// Create an opaque state descriptor.
    #[must_use]
    pub fn opaque(descriptor: impl Into<String>) -> Self {
        Self::Opaque(descriptor.into())
    }
}

/// Runtime provenance category and notes for a resource instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceProvenance {
    /// Authority over an external/host resource admitted by the host runtime.
    HostAuthority { notes: Vec<String> },
    /// Authority over an Ash-created internal resource.
    InternalAuthority { notes: Vec<String> },
    /// Authority derived from declared dependencies.
    DerivedAuthority {
        sources: Vec<ResourceId>,
        notes: Vec<String>,
    },
}

impl ResourceProvenance {
    /// Construct internal authority provenance with one note.
    #[must_use]
    pub fn internal(note: impl Into<String>) -> Self {
        Self::InternalAuthority {
            notes: vec![note.into()],
        }
    }
}

impl Default for ResourceProvenance {
    fn default() -> Self {
        Self::InternalAuthority { notes: Vec::new() }
    }
}

/// Concrete identity-bearing runtime resource instance carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceInstance {
    /// Stable identity for the lifetime of the resource instance.
    pub id: ResourceId,
    /// Static resource type identifier.
    pub type_id: ResourceTypeId,
    /// Runtime owner scope metadata.
    pub owner: ResourceOwner,
    /// Runtime-owned state descriptor; not a first-class Ash [`Value`].
    pub state: ResourceRuntimeState,
    /// Current lifecycle state.
    pub lifecycle: ResourceLifecycle,
    /// Access discipline metadata.
    pub access_policy: AccessPolicy,
    /// Process split/join/share/move policy metadata.
    pub split_join_policy: ResourceSplitJoinPolicy,
    /// Authority provenance metadata.
    pub provenance: ResourceProvenance,
}

impl ResourceInstance {
    /// Create a resource instance with conservative default metadata.
    #[must_use]
    pub fn new(id: ResourceId, type_id: ResourceTypeId, owner: ResourceOwner) -> Self {
        Self {
            id,
            type_id,
            owner,
            state: ResourceRuntimeState::default(),
            lifecycle: ResourceLifecycle::Allocated,
            access_policy: AccessPolicy::Exclusive,
            split_join_policy: ResourceSplitJoinPolicy::NonShareable,
            provenance: ResourceProvenance::default(),
        }
    }

    /// Attach runtime-owned state metadata.
    #[must_use]
    pub fn with_state(mut self, state: ResourceRuntimeState) -> Self {
        self.state = state;
        self
    }

    /// Attach lifecycle metadata.
    #[must_use]
    pub fn with_lifecycle(mut self, lifecycle: ResourceLifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Attach access policy metadata.
    #[must_use]
    pub fn with_access_policy(mut self, access_policy: AccessPolicy) -> Self {
        self.access_policy = access_policy;
        self
    }

    /// Attach split/join policy metadata.
    #[must_use]
    pub fn with_split_join_policy(mut self, split_join_policy: ResourceSplitJoinPolicy) -> Self {
        self.split_join_policy = split_join_policy;
        self
    }

    /// Attach authority provenance metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: ResourceProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}

/// A unique identifier for one admitted runtime capability binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityBindingId(pub Uuid);

impl CapabilityBindingId {
    /// Create a fresh capability binding identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CapabilityBindingId {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable runtime identifier for an Ash capability interface declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityInterfaceId(String);

impl CapabilityInterfaceId {
    /// Create a capability interface identifier from a static interface name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow the interface name carried by this identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CapabilityInterfaceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CapabilityInterfaceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Stable runtime identifier for an Ash-defined capability implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityImplementationId(String);

impl CapabilityImplementationId {
    /// Create a capability implementation identifier from a static implementation name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow the implementation name carried by this identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CapabilityImplementationId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CapabilityImplementationId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Runtime provenance for an admitted capability binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityAuthorityProvenance {
    /// Authority over an external/host capability admitted by the host runtime.
    HostAuthority { notes: Vec<String> },
    /// Authority derived from explicitly admitted dependencies.
    DerivedAuthority {
        dependency_names: Vec<String>,
        notes: Vec<String>,
    },
}

/// Dependency metadata for an implementation-backed capability binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapabilityBindingDependency {
    /// Dependency on one environment-owned runtime resource instance.
    Resource {
        name: String,
        resource_id: ResourceId,
        type_id: ResourceTypeId,
    },
    /// Dependency on another explicitly admitted capability binding.
    Capability {
        name: String,
        binding_id: CapabilityBindingId,
        interface: CapabilityInterfaceId,
    },
    /// Inert configuration value dependency.
    Config { name: String, value: Value },
}

impl CapabilityBindingDependency {
    /// Borrow the dependency name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Resource { name, .. }
            | Self::Capability { name, .. }
            | Self::Config { name, .. } => name,
        }
    }

    /// Return true when this dependency can carry runtime authority.
    #[must_use]
    pub fn carries_authority(&self) -> bool {
        matches!(self, Self::Resource { .. } | Self::Capability { .. })
    }

    /// Render one audit/provenance note preserving stable source identity.
    #[must_use]
    pub fn provenance_note(&self) -> String {
        match self {
            Self::Resource {
                name,
                resource_id,
                type_id,
            } => format!(
                "resource {name}: id={} type={}",
                resource_id.0,
                type_id.as_str()
            ),
            Self::Capability {
                name,
                binding_id,
                interface,
            } => format!(
                "capability {name}: binding={} interface={}",
                binding_id.0,
                interface.as_str()
            ),
            Self::Config { name, .. } => format!("config {name}: inert dependency"),
        }
    }
}

/// Runtime shape of an admitted capability binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityBindingKind {
    /// Binding backed by an existing host provider registry entry.
    HostProvider {
        provider_name: String,
        admitted_capabilities: Vec<String>,
    },
    /// Binding backed by Ash-defined implementation metadata only.
    Implementation {
        implementation: CapabilityImplementationId,
    },
}

/// Identity-bearing runtime carrier for an admitted capability binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityBinding {
    /// Stable binding identity for the lifetime of the admission record.
    pub id: CapabilityBindingId,
    /// Runtime binding name selected by workflow/process/run headers.
    pub name: String,
    /// Static interface identifier this binding is meant to satisfy.
    pub interface: CapabilityInterfaceId,
    /// Backing kind metadata.
    pub kind: CapabilityBindingKind,
    /// Explicit dependency records for implementation-backed bindings.
    pub dependencies: Vec<CapabilityBindingDependency>,
    /// Authority provenance metadata.
    pub authority: CapabilityAuthorityProvenance,
}

impl CapabilityBinding {
    /// Create a host-provider binding carrier.
    #[must_use]
    pub fn host_provider(
        id: CapabilityBindingId,
        name: impl Into<String>,
        interface: CapabilityInterfaceId,
        provider_name: impl Into<String>,
        admitted_capabilities: Vec<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            interface,
            kind: CapabilityBindingKind::HostProvider {
                provider_name: provider_name.into(),
                admitted_capabilities,
            },
            dependencies: Vec::new(),
            authority: CapabilityAuthorityProvenance::HostAuthority {
                notes: vec!["host provider admitted by runtime".to_string()],
            },
        }
    }

    /// Create an implementation-backed binding carrier.
    #[must_use]
    pub fn implementation(
        id: CapabilityBindingId,
        name: impl Into<String>,
        interface: CapabilityInterfaceId,
        implementation: CapabilityImplementationId,
        dependencies: Vec<CapabilityBindingDependency>,
    ) -> Self {
        let dependency_names = dependencies
            .iter()
            .map(|dependency| dependency.name().to_string())
            .collect();
        let mut notes =
            vec!["implementation binding derives only from admitted dependencies".to_string()];
        notes.extend(
            dependencies
                .iter()
                .map(CapabilityBindingDependency::provenance_note),
        );
        Self {
            id,
            name: name.into(),
            interface,
            kind: CapabilityBindingKind::Implementation { implementation },
            dependencies,
            authority: CapabilityAuthorityProvenance::DerivedAuthority {
                dependency_names,
                notes,
            },
        }
    }

    /// Add one authority-provenance audit note.
    #[must_use]
    pub fn with_authority_note(mut self, note: impl Into<String>) -> Self {
        match &mut self.authority {
            CapabilityAuthorityProvenance::HostAuthority { notes }
            | CapabilityAuthorityProvenance::DerivedAuthority { notes, .. } => {
                notes.push(note.into())
            }
        }
        self
    }
}

/// A unique identifier for one runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    /// Create a fresh run identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unique identifier for one managed service runtime instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(pub Uuid);

impl ServiceId {
    /// Create a fresh service identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ServiceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Explicit lifecycle state for a managed long-running service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceLifecycleState {
    /// Service has been admitted and is starting.
    Starting,
    /// Service is running and health-checkable.
    Running,
    /// Service is applying a bounded reload.
    Reloading,
    /// Service is shutting down.
    Stopping,
    /// Service has reached terminal state and its report is retained.
    Terminated,
}

/// Health status for a managed service instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceHealthStatus {
    /// Service is available.
    Healthy,
    /// Service is degraded but still inspectable.
    Degraded,
    /// Service is no longer available.
    Unavailable,
}

/// Shutdown mode selected for a managed service instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceShutdownMode {
    /// Cooperative shutdown.
    Graceful,
    /// Forced terminal shutdown.
    Forced,
}

/// Structured diagnostic for service lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceLifecycleDiagnostic {
    /// Service name was missing at the runtime boundary.
    MissingServiceName,
    /// Service name was malformed.
    MalformedServiceName {
        /// Name supplied by the runtime boundary.
        service_name: String,
    },
    /// Requested service identity was not retained.
    UnknownService {
        /// Missing service identity.
        service_id: ServiceId,
    },
    /// A terminal service may be inspected but not restarted/reloaded in place.
    TerminalServiceRetained {
        /// Retained terminal service identity.
        service_id: ServiceId,
    },
}

/// Retained service health report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceHealthReport {
    /// Service identity being reported.
    pub service_id: ServiceId,
    /// Current lifecycle state.
    pub lifecycle: ServiceLifecycleState,
    /// Current health status.
    pub status: ServiceHealthStatus,
}

/// Explicit retained lifecycle state for one managed service.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceRuntimeRecord {
    /// Stable service identity.
    pub id: ServiceId,
    /// Runtime service name.
    pub name: String,
    /// Underlying process identity owned by this service record.
    pub process_id: ProcessId,
    /// Current lifecycle state.
    pub lifecycle: ServiceLifecycleState,
    /// Current health status.
    pub health: ServiceHealthStatus,
    /// Bounded reload generation.
    pub reload_generation: u32,
    /// Last reload identity, if any.
    pub last_reload: Option<String>,
    /// Shutdown mode, if terminal shutdown has happened.
    pub shutdown_mode: Option<ServiceShutdownMode>,
    /// Terminal reason or note.
    pub terminal_reason: Option<String>,
    /// Whether this service reached terminal state.
    pub terminal: bool,
    /// Whether terminal report/state is retained.
    pub retained: bool,
    /// Stable retained report identity.
    pub report_identity: Option<String>,
}

/// A unique identifier for one trusted runtime adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustedRuntimeAdapterId(pub Uuid);

impl TrustedRuntimeAdapterId {
    /// Create a fresh trusted runtime adapter identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrustedRuntimeAdapterId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata surface that a trusted runtime adapter is allowed to target.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustedRuntimeAdapterTarget {
    /// Adapter targets an explicitly authored provider operation.
    ProviderOperation {
        /// Provider metadata identity.
        provider_name: String,
        /// Provider operation metadata identity.
        operation_name: String,
        /// Required operation row this adapter expects to satisfy.
        required_row: String,
    },
    /// Adapter targets an explicitly declared builtin host hook.
    BuiltinHostHook {
        /// Builtin dispatch identity.
        builtin_name: String,
        /// Capability identity declared by the builtin host hook.
        capability: String,
        /// Operation identity declared by the builtin host hook.
        operation: String,
    },
}

/// Structured diagnostic emitted at trusted runtime adapter boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
pub enum TrustedRuntimeAdapterDiagnostic {
    /// Adapter name was missing.
    #[error("trusted runtime adapter is missing name")]
    MissingAdapterName,
    /// Adapter name was malformed.
    #[error("trusted runtime adapter name '{adapter_name}' is malformed")]
    MalformedAdapterName {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Adapter version was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing version")]
    MissingVersion {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Trust source metadata was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing trust source")]
    MissingTrustSource {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Admission source metadata was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing admission source")]
    MissingAdmissionSource {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Sandbox policy metadata was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing sandbox policy")]
    MissingSandboxPolicy {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Provenance policy metadata was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing provenance policy")]
    MissingProvenancePolicy {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Report identity metadata was missing.
    #[error("trusted runtime adapter '{adapter_name}' is missing report identity")]
    MissingReportIdentity {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Provider adapter target did not reference provider operation metadata.
    #[error("trusted runtime adapter '{adapter_name}' is missing provider metadata reference")]
    MissingProviderMetadataReference {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Builtin adapter target did not reference builtin host hook metadata.
    #[error("trusted runtime adapter '{adapter_name}' is missing builtin hook metadata reference")]
    MissingBuiltinHookMetadataReference {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Adapter attempted to grant authority directly.
    #[error("trusted runtime adapter '{adapter_name}' must not grant authority")]
    AuthorityWideningAdapter {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Requested adapter is not registered.
    #[error("trusted runtime adapter '{adapter_name}' is not registered")]
    UnknownAdapter {
        /// Missing adapter name.
        adapter_name: String,
    },
    /// Requested adapter version does not match the registered version.
    #[error(
        "trusted runtime adapter '{adapter_name}' requested version '{requested_version}' but registered version is '{registered_version}'"
    )]
    StaleAdapter {
        /// Adapter name.
        adapter_name: String,
        /// Requested version.
        requested_version: String,
        /// Registered version.
        registered_version: String,
    },
    /// Adapter metadata is incompatible with the referenced hook/provider metadata.
    #[error("trusted runtime adapter '{adapter_name}' is incompatible: {reason}")]
    IncompatibleAdapter {
        /// Adapter name.
        adapter_name: String,
        /// Bounded incompatibility reason.
        reason: String,
    },
}

/// Explicit metadata for a trusted runtime adapter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustedRuntimeAdapter {
    /// Stable adapter identity.
    pub id: TrustedRuntimeAdapterId,
    /// Runtime adapter name.
    pub name: String,
    /// Adapter version expected by runtime call sites.
    pub version: String,
    /// Trusted runtime or build source that supplied this adapter.
    pub trust_source: String,
    /// Admission boundary that authorized the adapter to exist.
    pub admission_source: String,
    /// Sandbox policy checked before host execution.
    pub sandbox_policy: String,
    /// Provenance policy used for host-boundary evidence.
    pub provenance_policy: String,
    /// Stable redacted report identity.
    pub report_identity: String,
    /// Provider or builtin hook metadata this adapter targets.
    pub target: TrustedRuntimeAdapterTarget,
    /// Whether this adapter directly grants authority. Valid adapters keep this false.
    pub grants_authority: bool,
}

impl TrustedRuntimeAdapter {
    /// Build trusted adapter metadata for an explicitly authored provider operation.
    ///
    /// # Errors
    ///
    /// Returns [`TrustedRuntimeAdapterDiagnostic`] when required identity, policy, target, or
    /// authority metadata is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new_provider_operation(
        name: impl Into<String>,
        version: impl Into<String>,
        trust_source: impl Into<String>,
        admission_source: impl Into<String>,
        sandbox_policy: impl Into<String>,
        provenance_policy: impl Into<String>,
        report_identity: impl Into<String>,
        provider_name: impl Into<String>,
        operation_name: impl Into<String>,
        required_row: impl Into<String>,
        grants_authority: bool,
    ) -> Result<Self, TrustedRuntimeAdapterDiagnostic> {
        Self::new(
            name,
            version,
            trust_source,
            admission_source,
            sandbox_policy,
            provenance_policy,
            report_identity,
            TrustedRuntimeAdapterTarget::ProviderOperation {
                provider_name: provider_name.into(),
                operation_name: operation_name.into(),
                required_row: required_row.into(),
            },
            grants_authority,
        )
    }

    /// Build trusted adapter metadata for an explicitly declared builtin host hook.
    ///
    /// # Errors
    ///
    /// Returns [`TrustedRuntimeAdapterDiagnostic`] when required identity, policy, target, or
    /// authority metadata is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new_builtin_host_hook(
        name: impl Into<String>,
        version: impl Into<String>,
        trust_source: impl Into<String>,
        admission_source: impl Into<String>,
        sandbox_policy: impl Into<String>,
        provenance_policy: impl Into<String>,
        report_identity: impl Into<String>,
        builtin_name: impl Into<String>,
        capability: impl Into<String>,
        operation: impl Into<String>,
        grants_authority: bool,
    ) -> Result<Self, TrustedRuntimeAdapterDiagnostic> {
        Self::new(
            name,
            version,
            trust_source,
            admission_source,
            sandbox_policy,
            provenance_policy,
            report_identity,
            TrustedRuntimeAdapterTarget::BuiltinHostHook {
                builtin_name: builtin_name.into(),
                capability: capability.into(),
                operation: operation.into(),
            },
            grants_authority,
        )
    }

    /// Validate existing trusted adapter metadata.
    ///
    /// # Errors
    ///
    /// Returns [`TrustedRuntimeAdapterDiagnostic`] when metadata is malformed or
    /// authority-widening.
    pub fn validate(&self) -> Result<(), TrustedRuntimeAdapterDiagnostic> {
        validate_trusted_runtime_adapter(self)
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        trust_source: impl Into<String>,
        admission_source: impl Into<String>,
        sandbox_policy: impl Into<String>,
        provenance_policy: impl Into<String>,
        report_identity: impl Into<String>,
        target: TrustedRuntimeAdapterTarget,
        grants_authority: bool,
    ) -> Result<Self, TrustedRuntimeAdapterDiagnostic> {
        let adapter = Self {
            id: TrustedRuntimeAdapterId::new(),
            name: name.into(),
            version: version.into(),
            trust_source: trust_source.into(),
            admission_source: admission_source.into(),
            sandbox_policy: sandbox_policy.into(),
            provenance_policy: provenance_policy.into(),
            report_identity: report_identity.into(),
            target,
            grants_authority,
        };
        adapter.validate()?;
        Ok(adapter)
    }
}

/// Validate trusted runtime adapter metadata.
///
/// # Errors
///
/// Returns [`TrustedRuntimeAdapterDiagnostic`] when required metadata is missing, malformed, or
/// authority-widening.
pub fn validate_trusted_runtime_adapter(
    adapter: &TrustedRuntimeAdapter,
) -> Result<(), TrustedRuntimeAdapterDiagnostic> {
    if adapter.name.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingAdapterName);
    }
    if !is_valid_runtime_boundary_name(&adapter.name) {
        return Err(TrustedRuntimeAdapterDiagnostic::MalformedAdapterName {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.version.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingVersion {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.trust_source.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingTrustSource {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.admission_source.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingAdmissionSource {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.sandbox_policy.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingSandboxPolicy {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.provenance_policy.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingProvenancePolicy {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.report_identity.is_empty() {
        return Err(TrustedRuntimeAdapterDiagnostic::MissingReportIdentity {
            adapter_name: adapter.name.clone(),
        });
    }
    if adapter.grants_authority {
        return Err(TrustedRuntimeAdapterDiagnostic::AuthorityWideningAdapter {
            adapter_name: adapter.name.clone(),
        });
    }
    match &adapter.target {
        TrustedRuntimeAdapterTarget::ProviderOperation {
            provider_name,
            operation_name,
            required_row,
        } if provider_name.is_empty() || operation_name.is_empty() || required_row.is_empty() => {
            Err(
                TrustedRuntimeAdapterDiagnostic::MissingProviderMetadataReference {
                    adapter_name: adapter.name.clone(),
                },
            )
        }
        TrustedRuntimeAdapterTarget::BuiltinHostHook {
            builtin_name,
            capability,
            operation,
        } if builtin_name.is_empty() || capability.is_empty() || operation.is_empty() => Err(
            TrustedRuntimeAdapterDiagnostic::MissingBuiltinHookMetadataReference {
                adapter_name: adapter.name.clone(),
            },
        ),
        _ => Ok(()),
    }
}

/// Host sandbox policy decision for one attempted host boundary crossing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HostSandboxDecision {
    /// Operation is allowed to continue to the host provider/hook.
    Allow,
    /// Operation is denied before host execution.
    Deny {
        /// Bounded redacted reason.
        reason: String,
    },
}

/// Host sandbox policy metadata retained by the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostSandboxPolicy {
    /// Stable policy identity referenced by provider/builtin/adapter metadata.
    pub identity: String,
    /// Whether the policy denies all attempts before more specific allow-list checks.
    pub deny_all: bool,
    /// Bounded denied reason.
    pub denied_reason: Option<String>,
    /// Allowed process command names.
    pub allowed_commands: Vec<String>,
    /// Allowed HTTP host names for network provider operations.
    pub allowed_hosts: Vec<String>,
    /// Allowed filesystem path prefixes for filesystem provider operations.
    pub allowed_paths: Vec<String>,
}

impl HostSandboxPolicy {
    /// Create a policy that allows host execution, with optional allow-list refinements.
    #[must_use]
    pub fn allow_all(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            deny_all: false,
            denied_reason: None,
            allowed_commands: Vec::new(),
            allowed_hosts: Vec::new(),
            allowed_paths: Vec::new(),
        }
    }

    /// Create a policy that denies all attempts before host execution.
    #[must_use]
    pub fn deny_all(identity: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            deny_all: true,
            denied_reason: Some(reason.into()),
            allowed_commands: Vec::new(),
            allowed_hosts: Vec::new(),
            allowed_paths: Vec::new(),
        }
    }

    /// Add one allowed command name.
    #[must_use]
    pub fn with_allowed_command(mut self, command: impl Into<String>) -> Self {
        self.allowed_commands.push(command.into());
        self
    }

    /// Add one allowed HTTP host name.
    #[must_use]
    pub fn with_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }

    /// Add one allowed filesystem path prefix.
    #[must_use]
    pub fn with_allowed_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Decide whether an operation may cross the host boundary.
    #[must_use]
    pub fn decide(&self, operation_name: &str, args: &[Value]) -> HostSandboxDecision {
        if self.deny_all {
            return HostSandboxDecision::Deny {
                reason: self
                    .denied_reason
                    .clone()
                    .unwrap_or_else(|| "sandbox policy denied host execution".to_string()),
            };
        }

        if !self.allowed_commands.is_empty()
            && matches!(operation_name, "run" | "spawn" | "which")
            && let Some(command) = first_string_arg(args)
            && !self
                .allowed_commands
                .iter()
                .any(|allowed| allowed == command)
        {
            return HostSandboxDecision::Deny {
                reason: "command not allowed by sandbox policy".to_string(),
            };
        }

        if !self.allowed_hosts.is_empty()
            && matches!(operation_name, "get" | "head" | "post" | "put" | "delete")
        {
            let Some(host) = first_string_arg(args).and_then(http_host_from_url) else {
                return HostSandboxDecision::Deny {
                    reason: "HTTP URL host missing or invalid".to_string(),
                };
            };
            if !self.allowed_hosts.iter().any(|allowed| allowed == host) {
                return HostSandboxDecision::Deny {
                    reason: "HTTP host not allowed by sandbox policy".to_string(),
                };
            }
        }

        if !self.allowed_paths.is_empty() && is_filesystem_operation(operation_name) {
            let Some(path) = first_string_arg(args) else {
                return HostSandboxDecision::Deny {
                    reason: "filesystem path missing or invalid".to_string(),
                };
            };
            if !self
                .allowed_paths
                .iter()
                .any(|allowed| Path::new(path).starts_with(Path::new(allowed)))
            {
                return HostSandboxDecision::Deny {
                    reason: "filesystem path not allowed by sandbox policy".to_string(),
                };
            }
        }

        HostSandboxDecision::Allow
    }
}

fn is_filesystem_operation(operation_name: &str) -> bool {
    matches!(
        operation_name,
        "exists"
            | "read_file"
            | "read_to_string"
            | "metadata"
            | "read_dir"
            | "write_file"
            | "write"
            | "write_string"
            | "append"
            | "copy"
            | "rename"
            | "remove_file"
            | "create_dir"
            | "create_dir_all"
            | "remove_dir"
            | "remove_dir_all"
    )
}

/// Redacted retained evidence for a denied host sandbox attempt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostSandboxDenialRecord {
    /// Sandbox policy identity that denied the attempt.
    pub policy_identity: String,
    /// Provider or hook family being protected.
    pub provider_name: String,
    /// Operation name that was denied.
    pub operation_name: String,
    /// Redacted report subject. Must not include raw argument values.
    pub redacted_subject: String,
    /// Bounded denial reason.
    pub reason: String,
}

/// A unique identifier for one host boundary evidence record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostBoundaryEvidenceId(pub Uuid);

impl HostBoundaryEvidenceId {
    /// Create a fresh host boundary evidence identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for HostBoundaryEvidenceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Outcome for a host boundary crossing attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostBoundaryOutcome {
    /// Host operation completed successfully.
    Succeeded,
    /// Host provider/hook returned a failure.
    Failed,
    /// Sandbox or admission policy denied the attempt before host execution.
    Denied,
    /// Host operation timed out.
    TimedOut,
    /// Host operation was cancelled.
    Cancelled,
    /// Boundary metadata was malformed or stale.
    MalformedMetadata,
}

/// Redacted authority-neutral evidence for one host boundary crossing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HostBoundaryEvidence {
    /// Stable evidence identity.
    pub id: HostBoundaryEvidenceId,
    /// Provider or host-hook family.
    pub provider_name: String,
    /// Operation attempted at the host boundary.
    pub operation_name: String,
    /// Optional trusted runtime adapter identity.
    pub adapter_name: Option<String>,
    /// Sandbox policy identity applied before execution.
    pub sandbox_policy: String,
    /// Provenance/redaction policy identity applied to evidence.
    pub provenance_policy: String,
    /// Crossing outcome.
    pub outcome: HostBoundaryOutcome,
    /// Redacted subject suitable for reports/traces.
    pub redacted_subject: String,
    /// Optional bounded diagnostic without raw payload contents.
    pub diagnostic: Option<String>,
    /// Evidence records must not grant or mutate authority.
    pub authority_neutral: bool,
}

impl HostBoundaryEvidence {
    /// Create one redacted host boundary evidence record.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_name: impl Into<String>,
        operation_name: impl Into<String>,
        adapter_name: Option<String>,
        sandbox_policy: impl Into<String>,
        provenance_policy: impl Into<String>,
        outcome: HostBoundaryOutcome,
        diagnostic: Option<String>,
    ) -> Self {
        let provider_name = provider_name.into();
        let operation_name = operation_name.into();
        let sandbox_policy = sandbox_policy.into();
        let provenance_policy = provenance_policy.into();
        Self {
            id: HostBoundaryEvidenceId::new(),
            redacted_subject: format!(
                "host:{provider_name}.{operation_name}:{sandbox_policy}:{provenance_policy}:redacted"
            ),
            provider_name,
            operation_name,
            adapter_name,
            sandbox_policy,
            provenance_policy,
            outcome,
            diagnostic,
            authority_neutral: true,
        }
    }
}

fn first_string_arg(args: &[Value]) -> Option<&str> {
    args.iter().find_map(|arg| match arg {
        Value::String(value) => Some(value.as_str()),
        Value::Record(fields) => fields
            .get("cmd")
            .or_else(|| fields.get("command"))
            .or_else(|| fields.get("program"))
            .and_then(|value| match value {
                Value::String(command) => Some(command.as_str()),
                _ => None,
            }),
        _ => None,
    })
}

fn http_host_from_url(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host_port_path = rest.split(['/', '?', '#']).next()?;
    let host = host_port_path.split(':').next()?;
    (!host.is_empty()).then_some(host)
}

/// A unique identifier for one registered external actor adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalActorAdapterId(pub Uuid);

impl ExternalActorAdapterId {
    /// Create a fresh external actor adapter identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ExternalActorAdapterId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unique identifier for one external actor call boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorCallId(pub Uuid);

impl ActorCallId {
    /// Create a fresh actor call identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ActorCallId {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported external actor adapter protocols.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorProtocol {
    /// HTTP request/response using JSON payloads.
    HttpJson,
    /// Message queue request/response using structured payloads.
    MessageQueue,
    /// Webhook delivery with structured callback payloads.
    Webhook,
    /// Unsupported protocol description used to fail closed at adapter construction.
    Unsupported {
        /// Human-readable reason.
        reason: String,
    },
}

/// Bounded external actor call policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorCallPolicy {
    /// Maximum retry attempts after an actor failure.
    pub max_retries: u32,
    /// Timeout budget in milliseconds.
    pub timeout_millis: u64,
}

impl ActorCallPolicy {
    /// Create a bounded call policy.
    #[must_use]
    pub fn bounded(max_retries: u32, timeout_millis: u64) -> Self {
        Self {
            max_retries,
            timeout_millis,
        }
    }
}

/// Structured diagnostic emitted at external actor boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExternalActorDiagnostic {
    /// Adapter name was missing.
    MissingAdapterName,
    /// Adapter name was malformed.
    MalformedAdapterName {
        /// Name supplied by the runtime boundary.
        adapter_name: String,
    },
    /// Actor type name was missing.
    MissingActorType,
    /// Capability boundary metadata was missing.
    MissingCapabilityBoundary {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Adapter attempted to grant authority directly.
    AuthorityWideningAdapter {
        /// Adapter name being constructed.
        adapter_name: String,
    },
    /// Adapter protocol is unsupported.
    UnsupportedProtocol {
        /// Adapter name being constructed.
        adapter_name: String,
        /// Unsupported protocol reason.
        reason: String,
    },
    /// Requested adapter is not registered.
    UnknownAdapter {
        /// Missing adapter name.
        adapter_name: String,
    },
    /// Requested call is not retained.
    UnknownCall {
        /// Missing call identity.
        call_id: ActorCallId,
    },
    /// Inbound payload did not match the adapter schema.
    InboundTypeMismatch {
        /// Adapter name.
        adapter_name: String,
        /// Expected inbound schema.
        expected: String,
        /// Actual value type.
        actual: String,
    },
    /// Outbound response did not match the adapter schema.
    OutboundTypeMismatch {
        /// Adapter name.
        adapter_name: String,
        /// Expected outbound schema.
        expected: String,
        /// Actual value type.
        actual: String,
    },
    /// Payload cannot cross an external actor boundary.
    NonSendablePayload {
        /// Adapter name.
        adapter_name: String,
        /// Structured sendability reason rendered without payload contents.
        reason: String,
    },
    /// Retry budget was exhausted.
    RetryBudgetExhausted {
        /// Call identity.
        call_id: ActorCallId,
        /// Configured retry budget.
        max_retries: u32,
    },
    /// Retained call is already terminal.
    TerminalCallRetained {
        /// Retained call identity.
        call_id: ActorCallId,
    },
}

/// Explicit typed adapter metadata for crossing an external actor boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalActorAdapter {
    /// Stable adapter identity.
    pub id: ExternalActorAdapterId,
    /// Runtime adapter name.
    pub name: String,
    /// External protocol.
    pub protocol: ActorProtocol,
    /// Stable actor type label used in reports.
    pub actor_type: String,
    /// Rendered inbound payload schema.
    pub inbound_schema: String,
    /// Rendered outbound response schema.
    pub outbound_schema: String,
    /// Capability or policy boundary that authorizes this adapter.
    pub capability_boundary: String,
    /// Bounded call policy.
    pub policy: ActorCallPolicy,
    /// Whether this adapter directly grants authority. Valid adapters keep this false.
    pub grants_authority: bool,
    /// Ownership discipline for values crossing the adapter.
    pub ownership: String,
}

impl ExternalActorAdapter {
    /// Build external actor adapter metadata selected at a runtime boundary.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalActorDiagnostic`] when the adapter name, actor type, protocol, capability
    /// boundary, or authority metadata is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        protocol: ActorProtocol,
        actor_type: impl Into<String>,
        inbound_schema: impl std::fmt::Display,
        outbound_schema: impl std::fmt::Display,
        capability_boundary: impl Into<String>,
        policy: ActorCallPolicy,
        grants_authority: bool,
    ) -> Result<Self, ExternalActorDiagnostic> {
        let name = name.into();
        if name.is_empty() {
            return Err(ExternalActorDiagnostic::MissingAdapterName);
        }
        if !is_valid_runtime_boundary_name(&name) {
            return Err(ExternalActorDiagnostic::MalformedAdapterName { adapter_name: name });
        }
        let actor_type = actor_type.into();
        if actor_type.is_empty() {
            return Err(ExternalActorDiagnostic::MissingActorType);
        }
        let capability_boundary = capability_boundary.into();
        if capability_boundary.is_empty() {
            return Err(ExternalActorDiagnostic::MissingCapabilityBoundary { adapter_name: name });
        }
        if grants_authority {
            return Err(ExternalActorDiagnostic::AuthorityWideningAdapter { adapter_name: name });
        }
        if let ActorProtocol::Unsupported { reason } = protocol {
            return Err(ExternalActorDiagnostic::UnsupportedProtocol {
                adapter_name: name,
                reason,
            });
        }
        Ok(Self {
            id: ExternalActorAdapterId::new(),
            name,
            protocol,
            actor_type,
            inbound_schema: inbound_schema.to_string(),
            outbound_schema: outbound_schema.to_string(),
            capability_boundary,
            policy,
            grants_authority,
            ownership: "owned-sendable".to_string(),
        })
    }
}

/// Outcome category for one external actor call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorCallOutcome {
    /// Actor call completed with a valid response.
    Succeeded,
    /// Actor call failed with a structured diagnostic.
    Failed,
    /// Actor call exceeded its timeout budget.
    TimedOut,
    /// Actor call was cancelled by the runtime.
    Cancelled,
    /// Actor call is scheduled for a bounded retry.
    RetryScheduled,
}

/// Retained external actor call report and trace carrier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalActorCallRecord {
    /// Stable call identity.
    pub call_id: ActorCallId,
    /// Adapter identity used by this call.
    pub adapter_id: ExternalActorAdapterId,
    /// Adapter name.
    pub adapter_name: String,
    /// Actor type label.
    pub actor_type: String,
    /// Capability or policy boundary that authorized this adapter.
    pub capability_boundary: String,
    /// External actor protocol.
    pub protocol: ActorProtocol,
    /// Call outcome.
    pub outcome: ActorCallOutcome,
    /// Retry attempt represented by this retained record.
    pub retry_attempt: u32,
    /// Whether this call reached a terminal retained state.
    pub terminal: bool,
    /// Human-readable inbound payload type label.
    pub payload_type: String,
    /// Human-readable response type label.
    pub response_type: Option<String>,
    /// Payload redaction policy marker.
    pub payload_redaction: String,
    /// Redacted trace subject, safe for reports.
    pub trace_subject: String,
    /// Optional bounded diagnostic detail.
    pub diagnostic: Option<String>,
}

/// A unique identifier for one process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub Uuid);

impl ProcessId {
    /// Create a fresh process identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal branch/scheduler identity subordinate to a [`ProcessId`].
///
/// `BranchId` is an internal runtime/scheduler identity and must not be exposed
/// as the public identity of a process handle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct BranchId {
    id: Uuid,
    process_id: ProcessId,
}

#[allow(dead_code)]
impl BranchId {
    /// Create a fresh branch identifier subordinate to `process_id`.
    #[must_use]
    pub(crate) fn new(process_id: ProcessId) -> Self {
        Self {
            id: Uuid::new_v4(),
            process_id,
        }
    }

    /// Return the parent process identity this branch is subordinate to.
    #[must_use]
    pub(crate) fn process_id(self) -> ProcessId {
        self.process_id
    }
}

/// A lightweight identity for pure lexical frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LexicalFrameId(pub Uuid);

impl LexicalFrameId {
    /// Create a fresh lexical-frame identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LexicalFrameId {
    fn default() -> Self {
        Self::new()
    }
}

/// A lightweight identity for effectful/Act execution scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectScopeId(pub Uuid);

impl EffectScopeId {
    /// Create a fresh effect-scope identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EffectScopeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Current lifecycle state of a process computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessLifecycleState {
    /// Identity/admission is being established before the process runs.
    Admitting,
    /// The process is active or ready to be scheduled.
    Running,
    /// The process cooperatively yielded to the scheduler.
    Yielded,
    /// The process completed normally with a value.
    Succeeded { value: Value },
    /// The process failed with an operational failure.
    Failed {
        /// Process that reached the failed terminal state.
        process_id: ProcessId,
        /// Structured failure evidence.
        failure: Box<OperationalFailure>,
    },
    /// The process was cancelled with operational failure evidence.
    Cancelled {
        /// Process that reached the cancelled terminal state.
        process_id: ProcessId,
        /// Structured cancellation failure evidence.
        failure: Box<OperationalFailure>,
    },
}

impl ProcessLifecycleState {
    /// Return true for terminal process lifecycle states.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded { .. } | Self::Failed { .. } | Self::Cancelled { .. }
        )
    }
}

/// Terminal process outcome carrier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessTerminalState {
    /// The process completed normally with a value.
    Succeeded { value: Value },
    /// The process failed with an operational failure.
    Failed {
        /// Process that reached the failed terminal state.
        process_id: ProcessId,
        /// Structured failure evidence.
        failure: Box<OperationalFailure>,
    },
    /// The process was cancelled with operational failure evidence.
    Cancelled {
        /// Process that reached the cancelled terminal state.
        process_id: ProcessId,
        /// Structured cancellation failure evidence.
        failure: Box<OperationalFailure>,
    },
}

/// Semantic tower that attributed an operational failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TowerLevel {
    /// Pure-expression failure attribution.
    Pure,
    /// Effectful/Act failure attribution.
    Effectful,
    /// Process failure attribution.
    Proc,
    /// Workflow-governance failure attribution.
    Workflow,
}

/// Entity identity associated with an operational failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureEntity {
    /// Pure lexical-frame identity.
    LexicalFrame(LexicalFrameId),
    /// Effectful/Act scope identity.
    EffectScope(EffectScopeId),
    /// Runtime execution identity.
    Run(RunId),
    /// Process identity.
    Process(ProcessId),
    /// Workflow identity.
    Workflow(WorkflowId),
}

/// Placeholder evidence attached to an operational failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FailureEvidence {
    /// Human- or runtime-readable notes for provenance/reporting.
    pub notes: Vec<String>,
    /// Lower-level evidence/provenance references, intentionally untyped here.
    pub provenance: Vec<String>,
}

/// Structured operational failure carrier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalFailure {
    /// Semantic tower that attributed this failure.
    pub tower: TowerLevel,
    /// Tower-specific entity identity.
    pub entity: FailureEntity,
    /// Failure payload value.
    pub payload: Value,
    /// Core-safe representation of the payload type.
    pub payload_type: String,
    /// Lower cause preserved when a higher tower wraps/reinterprets a failure.
    pub cause: Option<Box<OperationalFailure>>,
    /// Evidence/provenance placeholders for matching and reporting.
    pub evidence: FailureEvidence,
}

impl OperationalFailure {
    /// Create a structured operational failure without a lower cause.
    #[must_use]
    pub fn new(
        tower: TowerLevel,
        entity: FailureEntity,
        payload: Value,
        payload_type: impl Into<String>,
    ) -> Self {
        Self {
            tower,
            entity,
            payload,
            payload_type: payload_type.into(),
            cause: None,
            evidence: FailureEvidence::default(),
        }
    }

    /// Replace the entity identity while preserving all other fields.
    #[must_use]
    pub fn with_entity(mut self, entity: FailureEntity) -> Self {
        self.entity = entity;
        self
    }

    /// Attach a lower operational failure cause.
    #[must_use]
    pub fn with_cause(mut self, cause: OperationalFailure) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Attach evidence/provenance placeholders.
    #[must_use]
    pub fn with_evidence(mut self, evidence: FailureEvidence) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Failure observed for one process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessFailure {
    /// Process that produced or is associated with the failure.
    pub process_id: ProcessId,
    /// Structured failure details.
    pub failure: OperationalFailure,
}

impl ProcessFailure {
    /// Create a process failure carrier.
    #[must_use]
    pub fn new(process_id: ProcessId, failure: OperationalFailure) -> Self {
        Self {
            process_id,
            failure,
        }
    }
}

/// Alias for process failures observed by await/join/gather-like boundaries.
pub type ObservedProcessFailure = ProcessFailure;

/// Aggregate carrier for observed process failures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessFailureAggregate {
    /// Per-process failures with identity preserved.
    pub failures: Vec<ProcessFailure>,
}

impl ProcessFailureAggregate {
    /// Create an aggregate from per-process failures.
    #[must_use]
    pub fn new(failures: Vec<ProcessFailure>) -> Self {
        Self { failures }
    }

    /// Whether no failures were aggregated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Runtime boundary that observed a process terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessPropagationBoundary {
    /// Single-handle await observation.
    Await,
    /// Two-handle join observation.
    Join,
    /// Ordered handle-list gather observation.
    Gather,
}

/// Supervisor policy selected at an application/runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisorPolicy {
    /// Restart failed children up to a bounded attempt count, then escalate.
    BoundedRestart {
        /// Maximum restart attempts before escalation.
        max_restarts: u32,
    },
    /// Cancel a supervised child through process cancellation semantics.
    Cancel,
    /// Escalate child failure/cancellation to the supervising runtime boundary.
    Escalate,
    /// Unsupported policy description used to fail closed at profile construction.
    Unsupported {
        /// Human-readable unsupported policy reason.
        reason: String,
    },
}

/// Structured diagnostic emitted while resolving supervisor runtime profiles.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisorDiagnostic {
    /// Supervisor profile name was missing.
    MissingProfileName,
    /// Supervisor profile name was malformed.
    MalformedProfileName {
        /// Profile name supplied by the runtime boundary.
        profile_name: String,
    },
    /// Supervisor profile attempted to grant authority directly.
    AuthorityWideningProfile {
        /// Profile name being resolved.
        profile_name: String,
    },
    /// Supervisor policy is unsupported and must fail closed.
    UnsupportedPolicy {
        /// Profile name being resolved.
        profile_name: String,
        /// Unsupported policy reason.
        reason: String,
    },
    /// Supervisor tried to observe a child without retained terminal state.
    ProcessNotTerminal {
        /// Profile name being evaluated.
        profile_name: String,
        /// Process whose terminal state was required.
        process_id: ProcessId,
    },
    /// Process registry integration rejected the supervisor operation.
    RuntimeRegistryFailure {
        /// Failure detail from the process registry.
        message: String,
    },
}

/// Runtime supervisor profile over Phase 195 process handles and trace facts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SupervisorRuntimeProfile {
    /// Stable profile name selected by the application runtime boundary.
    pub profile_name: String,
    /// Supervising process identity.
    pub supervisor_process_id: ProcessId,
    /// Runtime policy for child terminal observation.
    pub policy: SupervisorPolicy,
    /// Whether this profile directly grants authority. Valid profiles keep this false.
    pub grants_authority: bool,
}

impl SupervisorRuntimeProfile {
    /// Build a bounded restart supervisor profile.
    ///
    /// # Errors
    ///
    /// Returns [`SupervisorDiagnostic`] when the profile name is malformed or the policy is
    /// unsupported.
    pub fn bounded_restart(
        profile_name: impl Into<String>,
        supervisor_process_id: ProcessId,
        max_restarts: u32,
    ) -> Result<Self, SupervisorDiagnostic> {
        Self::runtime_boundary(
            profile_name,
            supervisor_process_id,
            SupervisorPolicy::BoundedRestart { max_restarts },
            false,
        )
    }

    /// Build a cancellation supervisor profile.
    ///
    /// # Errors
    ///
    /// Returns [`SupervisorDiagnostic`] when the profile name is malformed.
    pub fn cancel_policy(
        profile_name: impl Into<String>,
        supervisor_process_id: ProcessId,
    ) -> Result<Self, SupervisorDiagnostic> {
        Self::runtime_boundary(
            profile_name,
            supervisor_process_id,
            SupervisorPolicy::Cancel,
            false,
        )
    }

    /// Build supervisor profile metadata selected at a runtime boundary.
    ///
    /// # Errors
    ///
    /// Returns [`SupervisorDiagnostic`] when the profile name is missing/malformed, the profile
    /// attempts to grant authority, or the selected policy is unsupported.
    pub fn runtime_boundary(
        profile_name: impl Into<String>,
        supervisor_process_id: ProcessId,
        policy: SupervisorPolicy,
        grants_authority: bool,
    ) -> Result<Self, SupervisorDiagnostic> {
        let profile_name = profile_name.into();
        if profile_name.is_empty() {
            return Err(SupervisorDiagnostic::MissingProfileName);
        }
        if !is_valid_supervisor_profile_name(&profile_name) {
            return Err(SupervisorDiagnostic::MalformedProfileName { profile_name });
        }
        if grants_authority {
            return Err(SupervisorDiagnostic::AuthorityWideningProfile { profile_name });
        }
        if let SupervisorPolicy::Unsupported { reason } = policy {
            return Err(SupervisorDiagnostic::UnsupportedPolicy {
                profile_name,
                reason,
            });
        }
        Ok(Self {
            profile_name,
            supervisor_process_id,
            policy,
            grants_authority,
        })
    }
}

fn is_valid_supervisor_profile_name(name: &str) -> bool {
    is_valid_runtime_boundary_name(name)
}

fn is_valid_runtime_boundary_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
}

/// Supervisor decision emitted after observing or controlling a process handle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisorDecisionRecord {
    /// Supervisor profile name that made the decision.
    pub profile_name: String,
    /// Supervising process identity.
    pub supervisor_process_id: ProcessId,
    /// Process observed or controlled by this decision.
    pub observed_process_id: ProcessId,
    /// Observed terminal outcome category, if a terminal child was observed.
    pub observed_outcome: Option<ProcessPropagationOutcome>,
    /// Decision kind selected by the supervisor policy.
    pub decision: SupervisorDecisionKind,
    /// Restart attempts consumed by this profile after the decision.
    pub restart_attempt: u32,
    /// Replacement child process identity when a restart is requested.
    pub replacement_process_id: Option<ProcessId>,
    /// Whether this decision is terminal for supervisor reporting.
    pub terminal: bool,
    /// Optional bounded reason for diagnostics and reports.
    pub reason: Option<String>,
}

/// Bounded supervisor decision kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisorDecisionKind {
    /// Child completed successfully.
    Complete,
    /// Child should be restarted.
    Restart,
    /// Child should be cancelled.
    Cancel,
    /// Failure/cancellation should be escalated to the supervising boundary.
    Escalate,
}

/// Terminal outcome category propagated to a supervisor-facing diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessPropagationOutcome {
    /// Observed process completed normally.
    Succeeded,
    /// Observed process failed with operational failure evidence.
    Failed,
    /// Observed process was cancelled with cancellation evidence.
    Cancelled,
}

/// Bounded diagnostic for process failure/cancellation propagation decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessPropagationDiagnostic {
    /// Boundary that observed the terminal process state.
    pub boundary: ProcessPropagationBoundary,
    /// Supervising/observing process, when the observer has process identity.
    pub observer_process_id: Option<ProcessId>,
    /// Process whose terminal state was observed.
    pub observed_process_id: ProcessId,
    /// Observed terminal outcome category.
    pub outcome: ProcessPropagationOutcome,
    /// Payload preserved for failure or cancellation evidence.
    pub payload: Option<Value>,
    /// Payload type when failure/cancellation evidence was present.
    pub payload_type: Option<String>,
    /// Bounded propagation decision label.
    pub decision: String,
}

impl ProcessPropagationDiagnostic {
    /// Create a diagnostic from a terminal process state observation.
    #[must_use]
    pub fn from_terminal_state(
        boundary: ProcessPropagationBoundary,
        observer_process_id: Option<ProcessId>,
        observed_process_id: ProcessId,
        terminal_state: &ProcessTerminalState,
    ) -> Self {
        match terminal_state {
            ProcessTerminalState::Succeeded { .. } => Self {
                boundary,
                observer_process_id,
                observed_process_id,
                outcome: ProcessPropagationOutcome::Succeeded,
                payload: None,
                payload_type: None,
                decision: "return-success".to_string(),
            },
            ProcessTerminalState::Failed { failure, .. } => Self {
                boundary,
                observer_process_id,
                observed_process_id,
                outcome: ProcessPropagationOutcome::Failed,
                payload: Some(failure.payload.clone()),
                payload_type: Some(failure.payload_type.clone()),
                decision: "propagate-failure".to_string(),
            },
            ProcessTerminalState::Cancelled { failure, .. } => Self {
                boundary,
                observer_process_id,
                observed_process_id,
                outcome: ProcessPropagationOutcome::Cancelled,
                payload: Some(failure.payload.clone()),
                payload_type: Some(failure.payload_type.clone()),
                decision: "propagate-cancellation".to_string(),
            },
        }
    }
}

/// Stable runtime event label for process/channel trace facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeTraceEvent {
    Register,
    Spawn,
    Start,
    Complete,
    Fail,
    Cancel,
    Restart,
    Escalate,
    Health,
    Reload,
    Shutdown,
    Join,
    Send,
    Receive,
    Close,
}

/// Runtime trace fact suitable for later temporal monitor evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeTraceFact {
    /// Coarse trace fact family.
    pub kind: TraceFactKind,
    /// Stable event label.
    pub event: RuntimeTraceEvent,
    /// Stable subject identifier rendered by the runtime.
    pub subject: String,
}

impl RuntimeTraceFact {
    /// Create a runtime trace fact.
    #[must_use]
    pub fn new(kind: TraceFactKind, event: RuntimeTraceEvent, subject: impl Into<String>) -> Self {
        Self {
            kind,
            event,
            subject: subject.into(),
        }
    }
}

/// Workflow-boundary terminal failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowFailureKind {
    /// Workflow could not be admitted before body execution.
    AdmissionFailure,
    /// A `requires` predicate failed at admission/call boundary.
    RequiresViolation,
    /// Required role context could not be admitted.
    RoleAdmissionFailure,
    /// Required capability surface could not be admitted.
    CapabilityAdmissionFailure,
    /// Lower body/process/effect failure escaped the governed body.
    BodyFailureEscaped,
    /// An `ensures` predicate failed after normal body completion.
    EnsuresViolation,
    /// Workflow-local obligations were not discharged at completion.
    LocalObligationsUndischarged,
    /// Active-role obligations were not discharged at completion.
    RoleObligationsUndischarged,
    /// Report/audit sink commit failed after constructing a boundary outcome.
    ReportCommitFailure,
    /// Runtime invariant or host-boundary failure.
    RuntimeFailure,
}

/// Placeholder evidence attached to a workflow failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowFailureEvidence {
    /// Human- or runtime-readable notes for reporting.
    pub notes: Vec<String>,
    /// Provenance placeholders, intentionally untyped in the substrate.
    pub provenance: Vec<String>,
}

/// Workflow-boundary failure carrier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowFailure {
    /// Workflow execution identity at the boundary.
    pub workflow_id: WorkflowId,
    /// Host/runtime run identity containing the workflow.
    pub run_id: RunId,
    /// Workflow-boundary failure classification.
    pub kind: WorkflowFailureKind,
    /// Lower operational failure preserved across boundary reinterpretation.
    pub cause: Option<Box<OperationalFailure>>,
    /// Governance/reporting evidence placeholders.
    pub evidence: WorkflowFailureEvidence,
}

impl WorkflowFailure {
    /// Create a workflow failure, preserving any lower operational cause.
    #[must_use]
    pub fn new(
        workflow_id: WorkflowId,
        run_id: RunId,
        kind: WorkflowFailureKind,
        cause: Option<OperationalFailure>,
    ) -> Self {
        Self {
            workflow_id,
            run_id,
            kind,
            cause: cause.map(Box::new),
            evidence: WorkflowFailureEvidence::default(),
        }
    }

    /// Attach workflow failure evidence/provenance placeholders.
    #[must_use]
    pub fn with_evidence(mut self, evidence: WorkflowFailureEvidence) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Workflow report status skeleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowReportStatus {
    /// Workflow boundary succeeded after governance checks.
    Succeeded,
    /// Workflow boundary failed by admission, body escape, or completion governance.
    Failed,
}

/// Admitted workflow boundary context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowAdmissionContext {
    /// Active admitted role name, if any.
    pub active_role: Option<String>,
    /// Capability surface admitted to the workflow boundary.
    pub admitted_capabilities: Vec<String>,
    /// Explicit capability binding identities admitted to the workflow boundary.
    pub admitted_capability_bindings: Vec<CapabilityBindingId>,
    /// Evidence used to satisfy admission-time `requires` checks.
    pub requires_evidence: Vec<String>,
}

impl WorkflowAdmissionContext {
    /// Return a new admission context with one explicit admitted capability binding identity.
    #[must_use]
    pub fn with_admitted_capability_binding(mut self, binding_id: CapabilityBindingId) -> Self {
        self.admitted_capability_bindings.push(binding_id);
        self
    }
}

/// Structured workflow contract evidence status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowEvidenceStatus {
    /// The contract clause has not yet been evaluated.
    Pending,
    /// The contract clause evaluated successfully.
    Passed,
    /// The contract clause evaluated unsuccessfully.
    Failed,
}

/// Structured workflow contract evidence for `requires` / `ensures` reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowContractCheckEvidence {
    /// Clause or label being checked.
    pub clause: String,
    /// Current evidence status.
    pub status: WorkflowEvidenceStatus,
    /// Human- or runtime-readable evidence notes.
    pub notes: Vec<String>,
}

impl WorkflowContractCheckEvidence {
    /// Construct pending evidence for deferred completion-time checks.
    #[must_use]
    pub fn pending(clause: impl Into<String>, notes: Vec<String>) -> Self {
        Self {
            clause: clause.into(),
            status: WorkflowEvidenceStatus::Pending,
            notes,
        }
    }

    /// Construct passed evidence for admission-time checks.
    #[must_use]
    pub fn passed(clause: impl Into<String>, notes: Vec<String>) -> Self {
        Self {
            clause: clause.into(),
            status: WorkflowEvidenceStatus::Passed,
            notes,
        }
    }

    /// Construct failed evidence for admission/completion checks.
    #[must_use]
    pub fn failed(clause: impl Into<String>, notes: Vec<String>) -> Self {
        Self {
            clause: clause.into(),
            status: WorkflowEvidenceStatus::Failed,
            notes,
        }
    }
}

/// Workflow boundary report skeleton.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowReport {
    /// Workflow execution identity at the boundary.
    pub workflow_id: WorkflowId,
    /// Host/runtime run identity containing the workflow.
    pub run_id: RunId,
    /// Boundary report status.
    pub status: WorkflowReportStatus,
    /// Failure details for failed reports.
    pub failure: Option<WorkflowFailure>,
    /// Admitted workflow context captured at the boundary.
    pub admission: WorkflowAdmissionContext,
    /// Admission-time `requires` evidence recorded for the boundary.
    pub requires_evidence: Vec<WorkflowContractCheckEvidence>,
    /// Completion-time `ensures` evidence placeholders.
    pub ensures_evidence: Vec<WorkflowContractCheckEvidence>,
    /// Completion obligation evidence placeholders.
    pub obligation_evidence: Vec<String>,
    /// Lower process failures observed at or preserved for the boundary.
    pub lower_process_failures: Vec<ProcessFailure>,
    /// Generic evidence placeholders for initial substrate stability.
    pub evidence: Vec<String>,
    /// Lower operational causes preserved for report consumers.
    pub lower_causes: Vec<OperationalFailure>,
    /// Provenance/audit placeholders for initial substrate stability.
    pub provenance: Vec<String>,
    /// Successful workflow result, when available.
    pub result: Option<Value>,
    /// Placeholder external report sink identity/reference.
    pub external_report_sink: Option<String>,
}

impl WorkflowReport {
    /// Create a successful workflow report skeleton.
    #[must_use]
    pub fn succeeded(workflow_id: WorkflowId, run_id: RunId) -> Self {
        Self {
            workflow_id,
            run_id,
            status: WorkflowReportStatus::Succeeded,
            failure: None,
            admission: WorkflowAdmissionContext::default(),
            requires_evidence: Vec::new(),
            ensures_evidence: Vec::new(),
            obligation_evidence: Vec::new(),
            lower_process_failures: Vec::new(),
            evidence: Vec::new(),
            lower_causes: Vec::new(),
            provenance: Vec::new(),
            result: None,
            external_report_sink: None,
        }
    }

    fn completion_failure_kind(&self) -> Option<WorkflowFailureKind> {
        self.failure
            .as_ref()
            .map(|failure| failure.kind)
            .filter(|kind| {
                matches!(
                    kind,
                    WorkflowFailureKind::EnsuresViolation
                        | WorkflowFailureKind::LocalObligationsUndischarged
                        | WorkflowFailureKind::RoleObligationsUndischarged
                )
            })
    }

    fn default_obligation_evidence_for(kind: WorkflowFailureKind) -> Vec<String> {
        match kind {
            WorkflowFailureKind::LocalObligationsUndischarged => {
                vec!["workflow-boundary local obligations left undischarged".to_string()]
            }
            WorkflowFailureKind::RoleObligationsUndischarged => {
                vec!["workflow-boundary role obligations left undischarged".to_string()]
            }
            _ => Vec::new(),
        }
    }

    fn normalize_completion_failure_evidence(&mut self) {
        match self.completion_failure_kind() {
            Some(WorkflowFailureKind::EnsuresViolation) => {
                self.ensures_evidence = self
                    .ensures_evidence
                    .drain(..)
                    .map(|entry| match entry.status {
                        WorkflowEvidenceStatus::Pending => {
                            WorkflowContractCheckEvidence::failed(entry.clause, entry.notes)
                        }
                        WorkflowEvidenceStatus::Passed | WorkflowEvidenceStatus::Failed => entry,
                    })
                    .collect();
            }
            Some(
                kind @ (WorkflowFailureKind::LocalObligationsUndischarged
                | WorkflowFailureKind::RoleObligationsUndischarged),
            ) if self.obligation_evidence.is_empty() => {
                self.obligation_evidence = Self::default_obligation_evidence_for(kind);
            }
            _ => {}
        }
    }

    /// Create a failed workflow report skeleton.
    #[must_use]
    pub fn failed(workflow_id: WorkflowId, run_id: RunId, failure: WorkflowFailure) -> Self {
        let lower_cause = failure.cause.as_deref().cloned();
        let lower_causes = lower_cause.iter().cloned().collect();
        let lower_process_failures = lower_cause
            .and_then(|cause| match cause.entity {
                FailureEntity::Process(process_id) => {
                    Some(vec![ProcessFailure::new(process_id, cause)])
                }
                _ => None,
            })
            .unwrap_or_default();
        let mut report = Self {
            workflow_id,
            run_id,
            status: WorkflowReportStatus::Failed,
            failure: Some(failure),
            admission: WorkflowAdmissionContext::default(),
            requires_evidence: Vec::new(),
            ensures_evidence: Vec::new(),
            obligation_evidence: Vec::new(),
            lower_process_failures,
            evidence: Vec::new(),
            lower_causes,
            provenance: Vec::new(),
            result: None,
            external_report_sink: None,
        };
        report.normalize_completion_failure_evidence();
        report
    }

    /// Attach admitted workflow context and project admission evidence into the report.
    #[must_use]
    pub fn with_admission_context(mut self, admission: WorkflowAdmissionContext) -> Self {
        self.requires_evidence = admission
            .requires_evidence
            .iter()
            .cloned()
            .map(|note| WorkflowContractCheckEvidence::passed(note.clone(), vec![note]))
            .collect();
        self.admission = admission;
        self
    }

    /// Attach structured admission-time `requires` evidence.
    #[must_use]
    pub fn with_requires_evidence(
        mut self,
        requires_evidence: Vec<WorkflowContractCheckEvidence>,
    ) -> Self {
        self.requires_evidence = requires_evidence;
        self
    }

    /// Attach structured completion-time `ensures` evidence/plumbing.
    #[must_use]
    pub fn with_ensures_evidence(
        mut self,
        ensures_evidence: Vec<WorkflowContractCheckEvidence>,
    ) -> Self {
        self.ensures_evidence = ensures_evidence;
        self.normalize_completion_failure_evidence();
        self
    }

    /// Attach completion-boundary obligation evidence.
    #[must_use]
    pub fn with_obligation_evidence(mut self, obligation_evidence: Vec<String>) -> Self {
        self.obligation_evidence = obligation_evidence;
        self.normalize_completion_failure_evidence();
        self
    }

    /// Attach local workflow evidence notes.
    #[must_use]
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    /// Attach workflow provenance/audit notes.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Vec<String>) -> Self {
        self.provenance = provenance;
        self
    }

    /// Attach observed lower process failures.
    #[must_use]
    pub fn with_lower_process_failures(
        mut self,
        lower_process_failures: Vec<ProcessFailure>,
    ) -> Self {
        self.lower_process_failures = lower_process_failures;
        self
    }

    /// Attach preserved lower operational causes.
    #[must_use]
    pub fn with_lower_causes(mut self, lower_causes: Vec<OperationalFailure>) -> Self {
        self.lower_causes = lower_causes;
        self
    }

    /// Attach the normal workflow result to a success report.
    #[must_use]
    pub fn with_result(mut self, value: Value) -> Self {
        self.result = Some(value);
        self
    }
}

/// Outer workflow-boundary outcome carrier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowBoundaryOutcome {
    /// Workflow body and boundary governance completed successfully.
    WorkflowSucceeded {
        value: Value,
        report: WorkflowReport,
    },
    /// Workflow failed at admission, by escaped body failure, or completion governance.
    WorkflowFailed {
        failure: WorkflowFailure,
        report: WorkflowReport,
    },
}

impl WorkflowBoundaryOutcome {
    /// Construct a successful workflow boundary outcome.
    #[must_use]
    pub fn succeeded(value: Value, report: WorkflowReport) -> Self {
        Self::WorkflowSucceeded { value, report }
    }

    /// Construct a failed workflow boundary outcome.
    #[must_use]
    pub fn failed(failure: WorkflowFailure, report: WorkflowReport) -> Self {
        Self::WorkflowFailed { failure, report }
    }

    /// Return the workflow identity associated with this boundary outcome.
    #[must_use]
    pub fn workflow_id(&self) -> WorkflowId {
        self.report().workflow_id
    }

    /// Return the host/runtime run identity associated with this boundary outcome.
    #[must_use]
    pub fn run_id(&self) -> RunId {
        self.report().run_id
    }

    /// Borrow the boundary report carried by this outcome.
    #[must_use]
    pub fn report(&self) -> &WorkflowReport {
        match self {
            Self::WorkflowSucceeded { report, .. } | Self::WorkflowFailed { report, .. } => report,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ControlLink, Value, WorkflowId};
    use proptest::prelude::*;

    #[test]
    fn run_and_process_ids_are_unique_and_serde_roundtrip() {
        let run_id = RunId::new();
        let other_run_id = RunId::new();
        assert_ne!(run_id, other_run_id);

        let process_id = ProcessId::new();
        let other_process_id = ProcessId::new();
        assert_ne!(process_id, other_process_id);

        let encoded_run = serde_json::to_string(&run_id).expect("RunId serializes");
        let decoded_run: RunId = serde_json::from_str(&encoded_run).expect("RunId deserializes");
        assert_eq!(run_id, decoded_run);

        let encoded_process = serde_json::to_string(&process_id).expect("ProcessId serializes");
        let decoded_process: ProcessId =
            serde_json::from_str(&encoded_process).expect("ProcessId deserializes");
        assert_eq!(process_id, decoded_process);
    }

    #[test]
    fn branch_id_is_subordinate_to_parent_process() {
        let parent = ProcessId::new();
        let branch = BranchId::new(parent);

        assert_eq!(branch.process_id(), parent);
        assert_ne!(branch, BranchId::new(parent));
    }

    #[test]
    fn operational_failure_entities_cover_each_semantic_tower_identity() {
        let lexical = LexicalFrameId::new();
        let effect = EffectScopeId::new();
        let process = ProcessId::new();
        let workflow = WorkflowId::new();
        let run = RunId::new();

        let cases = [
            (TowerLevel::Pure, FailureEntity::LexicalFrame(lexical)),
            (TowerLevel::Effectful, FailureEntity::EffectScope(effect)),
            (TowerLevel::Proc, FailureEntity::Process(process)),
            (TowerLevel::Workflow, FailureEntity::Workflow(workflow)),
            (TowerLevel::Workflow, FailureEntity::Run(run)),
        ];

        for (tower, entity) in cases {
            let failure = OperationalFailure::new(tower, entity, Value::Null, "Unit");
            assert_eq!(failure.tower, tower);
            assert_eq!(failure.entity, entity);
        }
    }

    #[test]
    fn lifecycle_terminal_classification_matches_process_semantics() {
        let failed_process = ProcessId::new();
        let cancelled_process = ProcessId::new();
        let failed = operational_failure(failed_process, "failed");
        let cancelled = operational_failure(cancelled_process, "cancelled");

        let non_terminal = [
            ProcessLifecycleState::Admitting,
            ProcessLifecycleState::Running,
            ProcessLifecycleState::Yielded,
        ];
        for state in non_terminal {
            assert!(!state.is_terminal(), "{state:?} must not be terminal");
        }

        let terminal = [
            ProcessLifecycleState::Succeeded { value: Value::Null },
            ProcessLifecycleState::Failed {
                process_id: failed_process,
                failure: Box::new(failed),
            },
            ProcessLifecycleState::Cancelled {
                process_id: cancelled_process,
                failure: Box::new(cancelled),
            },
        ];
        for state in terminal {
            assert!(state.is_terminal(), "{state:?} must be terminal");
        }
    }

    #[test]
    fn process_terminal_state_preserves_failed_process_identity() {
        let failed_process = ProcessId::new();
        let cancelled_process = ProcessId::new();
        let failed = ProcessTerminalState::Failed {
            process_id: failed_process,
            failure: Box::new(operational_failure(failed_process, "failed")),
        };
        let cancelled = ProcessTerminalState::Cancelled {
            process_id: cancelled_process,
            failure: Box::new(operational_failure(cancelled_process, "cancelled")),
        };

        match failed {
            ProcessTerminalState::Failed {
                process_id,
                failure,
            } => {
                assert_eq!(process_id, failed_process);
                assert_eq!(failure.entity, FailureEntity::Process(failed_process));
            }
            other => panic!("expected failed terminal state, got {other:?}"),
        }

        match cancelled {
            ProcessTerminalState::Cancelled {
                process_id,
                failure,
            } => {
                assert_eq!(process_id, cancelled_process);
                assert_eq!(failure.entity, FailureEntity::Process(cancelled_process));
            }
            other => panic!("expected cancelled terminal state, got {other:?}"),
        }
    }

    #[test]
    fn operational_failure_preserves_tower_entity_and_lower_cause_identity() {
        let lower_process = ProcessId::new();
        let upper_process = ProcessId::new();
        let lower = operational_failure(lower_process, "provider unavailable");
        let upper = OperationalFailure::new(
            TowerLevel::Proc,
            FailureEntity::Process(lower_process),
            Value::String("observed process failed".to_string()),
            "String",
        )
        .with_entity(FailureEntity::Process(upper_process))
        .with_cause(lower.clone());

        assert_eq!(upper.tower, TowerLevel::Proc);
        assert_eq!(upper.entity, FailureEntity::Process(upper_process));
        let cause = upper.cause.as_deref().expect("lower cause preserved");
        assert_eq!(cause.entity, FailureEntity::Process(lower_process));
        assert_eq!(
            cause.payload,
            Value::String("provider unavailable".to_string())
        );
        assert_eq!(cause.payload_type, "String");
    }

    #[test]
    fn process_failure_and_aggregate_preserve_observed_process_identity() {
        let first_process = ProcessId::new();
        let second_process = ProcessId::new();
        let first = ProcessFailure::new(first_process, operational_failure(first_process, "first"));
        let second = ProcessFailure::new(
            second_process,
            operational_failure(second_process, "second"),
        );
        let aggregate = ProcessFailureAggregate::new(vec![first.clone(), second.clone()]);

        assert_eq!(first.process_id, first_process);
        assert_eq!(first.failure.entity, FailureEntity::Process(first_process));
        assert_eq!(aggregate.failures[0].process_id, first_process);
        assert_eq!(aggregate.failures[1].process_id, second_process);
    }

    #[test]
    fn workflow_failure_preserves_boundary_identity_run_id_and_cause() {
        let workflow_id = WorkflowId::new();
        let run_id = RunId::new();
        let process_id = ProcessId::new();
        let cause = operational_failure(process_id, "body failure escaped");

        let failure = WorkflowFailure::new(
            workflow_id,
            run_id,
            WorkflowFailureKind::BodyFailureEscaped,
            Some(cause.clone()),
        );

        assert_eq!(failure.workflow_id, workflow_id);
        assert_eq!(failure.run_id, run_id);
        assert_eq!(failure.kind, WorkflowFailureKind::BodyFailureEscaped);
        assert_eq!(
            failure.cause.as_deref().map(|f| f.entity),
            Some(FailureEntity::Process(process_id))
        );

        let report = WorkflowReport::failed(workflow_id, run_id, failure.clone());
        assert_eq!(report.workflow_id, workflow_id);
        assert_eq!(report.run_id, run_id);
        assert_eq!(report.status, WorkflowReportStatus::Failed);
        assert_eq!(
            report.failure.as_ref().map(|f| f.kind),
            Some(WorkflowFailureKind::BodyFailureEscaped)
        );
    }

    proptest! {
        #[test]
        fn process_failure_aggregate_preserves_input_order_and_identity(messages in proptest::collection::vec(".*", 0..16)) {
            let failures: Vec<_> = messages
                .iter()
                .map(|message| {
                    let process_id = ProcessId::new();
                    ProcessFailure::new(process_id, operational_failure(process_id, message))
                })
                .collect();
            let expected_process_ids: Vec<_> = failures.iter().map(|failure| failure.process_id).collect();

            let aggregate = ProcessFailureAggregate::new(failures);

            prop_assert_eq!(aggregate.failures.len(), expected_process_ids.len());
            for (failure, expected_process_id) in aggregate.failures.iter().zip(expected_process_ids) {
                prop_assert_eq!(failure.process_id, expected_process_id);
                prop_assert_eq!(failure.failure.entity, FailureEntity::Process(expected_process_id));
            }
        }
    }

    #[test]
    fn control_link_is_not_process_handle_substrate() {
        let workflow_id = WorkflowId::new();
        let control_link = ControlLink {
            instance_id: workflow_id,
        };
        let process_id = ProcessId::new();

        assert_eq!(control_link.instance_id, workflow_id);
        assert_ne!(format!("{control_link:?}"), format!("{process_id:?}"));
    }

    fn operational_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
        OperationalFailure::new(
            TowerLevel::Proc,
            FailureEntity::Process(process_id),
            Value::String(message.to_string()),
            "String",
        )
    }
}
