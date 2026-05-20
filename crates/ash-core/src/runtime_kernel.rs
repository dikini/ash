//! Runtime kernel identity carriers for the alpha OS-facing runtime regime.
//!
//! These carriers define the stable identity substrate required by SPEC-070
//! without routing `ash run`, starting a daemon, or changing execution
//! semantics. The existing `ash-engine::Engine` remains embedded under the
//! kernel as an execution/checking component; future routing tasks can attach
//! the real engine value outside `ash-core`, which deliberately cannot depend
//! on `ash-engine`.
//!
//! Inventory and migration notes:
//! - `ash-core::runtime::{RunId, ProcessId, ResourceId, CapabilityBindingId,
//!   WorkflowAdmissionContext}` remain the lower runtime/admission identities.
//! - `ash-engine::WorkflowAdmissionRequest` and `WorkflowAdmissionOutcome`
//!   remain the current admission boundary until TASK-928 routes starts
//!   through the kernel.
//! - `ash-interp::RuntimeState` and `ash-interp::Context` remain provider,
//!   resource, capability-binding, and execution-context state owners.
//! - This module adds the missing host/root/definition/artifact/instance/cache
//!   identities above those existing carriers. Provider registry identity is
//!   intentionally separate from admission authority grants.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::runtime::{CapabilityBindingId, ProcessId, ResourceId};

/// Host lifetime/control mode for one runtime kernel container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeHostMode {
    /// Embedded entry/evaluation host used by library callers and tests.
    Entry,
    /// One-shot `ash run` host process.
    OneShot,
    /// Trace host mode using the same semantic lifecycle with trace sinks.
    Trace,
    /// Long-lived local daemon host mode.
    Daemon,
}

/// Relationship between SPEC-070 `RuntimeKernel` and the existing engine API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeEngineRelationship {
    /// The existing `ash-engine::Engine` is embedded under the kernel.
    ExistingAshEngineEmbedded,
}

/// Explicit identity for a runtime root set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeRootSetId(String);

impl RuntimeRootSetId {
    /// Create a root-set identity from an already canonicalized root label.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the root-set identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime roots participating in definition identity and cache invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeRootSet {
    /// Stable identity for this root set.
    pub id: RuntimeRootSetId,
    /// Source roots used for module loading.
    pub source_roots: Vec<String>,
    /// Library roots used for dependency lookup.
    pub library_roots: Vec<String>,
    /// Configuration roots used for profile/config selection.
    pub config_roots: Vec<String>,
    /// Runtime state directory.
    pub state_dir: String,
    /// Artifact cache directory.
    pub cache_dir: String,
    /// Runtime log/report directory.
    pub log_dir: String,
}

impl RuntimeRootSet {
    /// Create an explicit runtime root-set carrier.
    #[must_use]
    pub fn new(
        id: RuntimeRootSetId,
        source_roots: Vec<String>,
        library_roots: Vec<String>,
        config_roots: Vec<String>,
        state_dir: impl Into<String>,
        cache_dir: impl Into<String>,
        log_dir: impl Into<String>,
    ) -> Self {
        Self {
            id,
            source_roots,
            library_roots,
            config_roots,
            state_dir: state_dir.into(),
            cache_dir: cache_dir.into(),
            log_dir: log_dir.into(),
        }
    }
}

/// Selected runtime profile identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeProfileId(String);

impl RuntimeProfileId {
    /// Create a runtime profile identity.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the profile identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Selected runtime config identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeConfigId(String);

impl RuntimeConfigId {
    /// Create a runtime config identity.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the config identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn push_length_prefixed(out: &mut String, value: &str) {
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push('|');
}

fn structured_identity(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        push_length_prefixed(&mut out, part);
    }
    out
}

/// Profile/config selection used for definition identity and admission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeProfileIdentity {
    /// Selected profile identity.
    pub profile_id: RuntimeProfileId,
    /// Selected config identity.
    pub config_id: RuntimeConfigId,
    /// Audit notes describing profile/config selection inputs.
    pub selection_facts: Vec<String>,
}

impl RuntimeProfileIdentity {
    /// Create a runtime profile/config identity carrier.
    #[must_use]
    pub fn new(
        profile_id: RuntimeProfileId,
        config_id: RuntimeConfigId,
        selection_facts: Vec<String>,
    ) -> Self {
        Self {
            profile_id,
            config_id,
            selection_facts,
        }
    }
}

/// Artifact version or content-addressed execution artifact identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactVersion(String);

impl ArtifactVersion {
    /// Create an artifact-version identity.
    #[must_use]
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Borrow the artifact-version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Cache key for source/check-summary/artifact selection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeArtifactCacheKey {
    /// Runtime roots participating in module identity and invalidation.
    pub root_id: RuntimeRootSetId,
    /// Selected runtime profile.
    pub profile_id: RuntimeProfileId,
    /// Selected runtime config.
    pub config_id: RuntimeConfigId,
    /// Source content hash or interim source digest.
    pub source_hash: String,
    /// Check-summary digest or interim semantic summary digest.
    pub check_summary_hash: String,
    /// Selected artifact version.
    pub artifact_version: ArtifactVersion,
}

impl RuntimeArtifactCacheKey {
    /// Create a cache key for runtime artifact selection.
    #[must_use]
    pub fn new(
        root_id: RuntimeRootSetId,
        profile_id: RuntimeProfileId,
        config_id: RuntimeConfigId,
        source_hash: impl Into<String>,
        check_summary_hash: impl Into<String>,
        artifact_version: ArtifactVersion,
    ) -> Self {
        Self {
            root_id,
            profile_id,
            config_id,
            source_hash: source_hash.into(),
            check_summary_hash: check_summary_hash.into(),
            artifact_version,
        }
    }
}

/// Stable workflow-definition identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowDefinitionId(String);

impl WorkflowDefinitionId {
    /// Borrow the workflow-definition identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Compiled/named workflow exported from a source root/module/artifact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowDefinitionIdentity {
    /// Stable definition identity.
    pub id: WorkflowDefinitionId,
    /// Runtime root set containing the definition.
    pub root_id: RuntimeRootSetId,
    /// Relative module path under the selected source root.
    pub relative_module_path: String,
    /// Exported workflow name.
    pub workflow_name: String,
    /// Selected profile participating in definition identity.
    pub profile_id: RuntimeProfileId,
    /// Selected config participating in definition identity.
    pub config_id: RuntimeConfigId,
    /// Source or artifact version/hash for this definition index entry.
    pub source_identity: String,
}

impl WorkflowDefinitionIdentity {
    /// Create a workflow-definition identity.
    #[must_use]
    pub fn new(
        root_id: RuntimeRootSetId,
        relative_module_path: impl Into<String>,
        workflow_name: impl Into<String>,
        profile_id: RuntimeProfileId,
        config_id: RuntimeConfigId,
        source_identity: impl Into<String>,
    ) -> Self {
        let relative_module_path = relative_module_path.into();
        let workflow_name = workflow_name.into();
        let source_identity = source_identity.into();
        let id = WorkflowDefinitionId(structured_identity(&[
            root_id.as_str(),
            &relative_module_path,
            &workflow_name,
            profile_id.as_str(),
            config_id.as_str(),
            &source_identity,
        ]));
        Self {
            id,
            root_id,
            relative_module_path,
            workflow_name,
            profile_id,
            config_id,
            source_identity,
        }
    }
}

/// Stable workflow-artifact identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowArtifactId(String);

impl WorkflowArtifactId {
    /// Borrow the workflow-artifact identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime execution artifact selected for a workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowArtifactIdentity {
    /// Stable artifact identity.
    pub id: WorkflowArtifactId,
    /// Workflow definition this artifact implements.
    pub definition_id: WorkflowDefinitionId,
    /// Cache key used to select or invalidate this artifact.
    pub cache_key: RuntimeArtifactCacheKey,
    /// Artifact version pinned for admitted starts.
    pub version: ArtifactVersion,
}

impl WorkflowArtifactIdentity {
    /// Create a workflow-artifact identity.
    #[must_use]
    pub fn new(
        definition_id: WorkflowDefinitionId,
        cache_key: RuntimeArtifactCacheKey,
        version: ArtifactVersion,
    ) -> Self {
        let id = WorkflowArtifactId(structured_identity(&[
            definition_id.as_str(),
            &cache_key.source_hash,
            &cache_key.check_summary_hash,
            version.as_str(),
        ]));
        Self {
            id,
            definition_id,
            cache_key,
            version,
        }
    }
}

/// Identity for a host provider registry snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderRegistryIdentity {
    id: Uuid,
    /// Registered provider names in this host registry snapshot.
    pub provider_names: Vec<String>,
}

impl ProviderRegistryIdentity {
    /// Create a provider registry identity from registered provider names.
    #[must_use]
    pub fn new(provider_names: Vec<String>) -> Self {
        let mut provider_names = provider_names;
        provider_names.sort();
        provider_names.dedup();
        Self {
            id: Uuid::new_v4(),
            provider_names,
        }
    }

    /// Return this provider registry identity.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Provider existence is host inventory, not admission authority.
    #[must_use]
    pub const fn grants_admission_authority(&self) -> bool {
        false
    }
}

/// Explicit admission authority grants for one workflow instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdmissionIdentity {
    id: Uuid,
    /// Explicit capability binding grants.
    pub capability_grants: Vec<CapabilityBindingId>,
    /// Explicit resource grants.
    pub resource_grants: Vec<ResourceId>,
}

impl AdmissionIdentity {
    /// Create an admission identity with no grants.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            id: Uuid::new_v4(),
            capability_grants: Vec::new(),
            resource_grants: Vec::new(),
        }
    }

    /// Return the admission identity.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Add an explicit capability binding grant.
    #[must_use]
    pub fn with_capability_grant(mut self, binding_id: CapabilityBindingId) -> Self {
        self.capability_grants.push(binding_id);
        self
    }

    /// Add an explicit resource grant.
    #[must_use]
    pub fn with_resource_grant(mut self, resource_id: ResourceId) -> Self {
        self.resource_grants.push(resource_id);
        self
    }

    /// Return true when this admission identity carries explicit authority.
    #[must_use]
    pub fn has_authority_grants(&self) -> bool {
        !self.capability_grants.is_empty() || !self.resource_grants.is_empty()
    }
}

/// Unique workflow instance identity for one admitted start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowInstanceId(pub Uuid);

impl WorkflowInstanceId {
    /// Create a fresh workflow instance identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// One admitted workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowInstanceIdentity {
    /// Stable identity for this admitted execution.
    pub id: WorkflowInstanceId,
    /// Host mode that admitted this instance.
    pub host_mode: RuntimeHostMode,
    /// Definition started by this instance.
    pub definition_id: WorkflowDefinitionId,
    /// Artifact pinned for this instance.
    pub artifact_id: WorkflowArtifactId,
    /// Profile/config identity used for this start.
    pub profile: RuntimeProfileIdentity,
    /// Provider registry inventory available to the host.
    pub provider_registry: ProviderRegistryIdentity,
    /// Explicit admission authority grants.
    pub admission: AdmissionIdentity,
}

impl WorkflowInstanceIdentity {
    /// Admit one workflow instance identity without executing it.
    #[must_use]
    pub fn admit(
        host_mode: RuntimeHostMode,
        definition_id: WorkflowDefinitionId,
        artifact_id: WorkflowArtifactId,
        profile: RuntimeProfileIdentity,
        provider_registry: ProviderRegistryIdentity,
        admission: AdmissionIdentity,
    ) -> Self {
        Self {
            id: WorkflowInstanceId::new(),
            host_mode,
            definition_id,
            artifact_id,
            profile,
            provider_registry,
            admission,
        }
    }

    /// Create a process-tree identity rooted in this workflow instance.
    #[must_use]
    pub fn process_tree(&self, root_process_id: ProcessId) -> ProcessTreeIdentity {
        ProcessTreeIdentity::new(self.id, root_process_id)
    }
}

/// Process-tree identity rooted by a workflow instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessTreeIdentity {
    /// Workflow instance that owns this process tree.
    pub workflow_instance_id: WorkflowInstanceId,
    root_process_id: ProcessId,
}

impl ProcessTreeIdentity {
    /// Create a process tree rooted in a workflow instance.
    #[must_use]
    pub const fn new(workflow_instance_id: WorkflowInstanceId, root_process_id: ProcessId) -> Self {
        Self {
            workflow_instance_id,
            root_process_id,
        }
    }

    /// Return the workflow instance that roots this process tree.
    #[must_use]
    pub const fn rooted_in(&self) -> WorkflowInstanceId {
        self.workflow_instance_id
    }

    /// Return the root process identity.
    #[must_use]
    pub const fn root_process_id(&self) -> ProcessId {
        self.root_process_id
    }
}

/// Identity for one runtime kernel container.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeKernelIdentity {
    /// Stable identity for this host kernel container.
    pub id: Uuid,
    /// Host mode for this kernel container.
    pub host_mode: RuntimeHostMode,
    /// Runtime roots owned by this kernel.
    pub roots: RuntimeRootSet,
    /// Current artifact cache key selection.
    pub cache_key: RuntimeArtifactCacheKey,
    /// Relationship to the existing engine API.
    pub engine_relationship: RuntimeEngineRelationship,
}

impl RuntimeKernelIdentity {
    /// Create a runtime kernel identity carrier.
    #[must_use]
    pub fn new(
        host_mode: RuntimeHostMode,
        roots: RuntimeRootSet,
        cache_key: RuntimeArtifactCacheKey,
        engine_relationship: RuntimeEngineRelationship,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            host_mode,
            roots,
            cache_key,
            engine_relationship,
        }
    }
}

/// Audit carrier documenting which existing runtime seams remain authoritative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeKernelCarrierInventory {
    /// Existing `ash-core::runtime` carriers reused by this layer.
    pub core_runtime_reused: Vec<String>,
    /// Existing `ash-engine` admission carriers reused by future routing.
    pub engine_admission_reused: Vec<String>,
    /// Existing `ash-interp` state/context carriers reused by future routing.
    pub interp_runtime_reused: Vec<String>,
    /// Carrier surfaces superseded by this module.
    pub superseded_by_runtime_kernel: Vec<String>,
}

impl RuntimeKernelCarrierInventory {
    /// Return the TASK-927 inventory for SPEC-070 runtime kernel identity work.
    #[must_use]
    pub fn task_927() -> Self {
        Self {
            core_runtime_reused: vec![
                "RunId".to_string(),
                "ProcessId".to_string(),
                "ResourceId".to_string(),
                "CapabilityBindingId".to_string(),
                "WorkflowAdmissionContext".to_string(),
            ],
            engine_admission_reused: vec![
                "WorkflowAdmissionRequest".to_string(),
                "WorkflowAdmissionOutcome".to_string(),
                "AdmittedWorkflowBoundary".to_string(),
            ],
            interp_runtime_reused: vec![
                "RuntimeState".to_string(),
                "Context".to_string(),
                "CapabilityContext".to_string(),
            ],
            superseded_by_runtime_kernel: vec![
                "name-only host mode labels".to_string(),
                "provider registry as authority".to_string(),
                "file presence as workflow execution identity".to_string(),
            ],
        }
    }
}
