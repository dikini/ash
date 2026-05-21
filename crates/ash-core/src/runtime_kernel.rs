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
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::amir::{
    AmirModule, AmirSectionKind, AmirVerifier, BytecodeModule, BytecodeOpcode, BytecodeSectionKind,
    BytecodeVerifier,
};
use crate::runtime::{CapabilityBindingId, ProcessId, ResourceId};
use crate::type_ir::{TcirComputationExpression, TcirStatementId};

/// Minimal alpha admission profile for one-shot RuntimeKernel starts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaAdmissionProfile {
    /// No requested grants or policy constraints; preserves existing alpha behavior.
    #[default]
    Empty,
    /// Explicitly admit the one-shot workflow instance.
    Allow,
    /// Reject the one-shot workflow instance before body execution.
    Reject,
}

impl AlphaAdmissionProfile {
    /// Evaluate the profile into a one-shot admission decision.
    #[must_use]
    pub fn evaluate(self) -> AlphaAdmissionDecision {
        match self {
            Self::Empty | Self::Allow => AlphaAdmissionDecision::admitted(),
            Self::Reject => {
                AlphaAdmissionDecision::rejected("alpha admission profile requested rejection")
            }
        }
    }

    /// Stable profile label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Allow => "allow",
            Self::Reject => "reject",
        }
    }
}

/// Minimal alpha admission status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaAdmissionStatus {
    /// Admission succeeded.
    Admitted,
    /// Admission rejected before workflow body execution.
    Rejected,
}

impl AlphaAdmissionStatus {
    /// Stable status label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Rejected => "rejected",
        }
    }
}

/// Result of evaluating a minimal alpha admission profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlphaAdmissionDecision {
    /// Admission status.
    pub status: AlphaAdmissionStatus,
    /// Rejection reason or audit note.
    pub reason: Option<String>,
}

impl AlphaAdmissionDecision {
    /// Construct an admitted decision.
    #[must_use]
    pub fn admitted() -> Self {
        Self {
            status: AlphaAdmissionStatus::Admitted,
            reason: None,
        }
    }

    /// Construct a rejected decision.
    #[must_use]
    pub fn rejected(reason: impl Into<String>) -> Self {
        Self {
            status: AlphaAdmissionStatus::Rejected,
            reason: Some(reason.into()),
        }
    }

    /// Returns true when the workflow instance is admitted.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, AlphaAdmissionStatus::Admitted)
    }
}

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
    /// Explicit provider/action grants projected from admitted capability bindings.
    pub action_grants: Vec<AdmissionActionGrant>,
}

/// Minimal alpha provider/action grant projected during RuntimeKernel admission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdmissionActionGrant {
    /// Capability binding that authorizes this action surface.
    pub binding_id: CapabilityBindingId,
    /// Host provider or runtime binding name receiving the projected grant.
    pub provider_name: String,
    /// Action name admitted for execution.
    pub action_name: String,
}

impl AdmissionActionGrant {
    /// Create a provider/action grant associated with one capability binding.
    #[must_use]
    pub fn new(
        binding_id: CapabilityBindingId,
        provider_name: impl Into<String>,
        action_name: impl Into<String>,
    ) -> Self {
        Self {
            binding_id,
            provider_name: provider_name.into(),
            action_name: action_name.into(),
        }
    }

    /// Stable provider/action label used in reports.
    #[must_use]
    pub fn action_surface(&self) -> String {
        format!("{}.{}", self.provider_name, self.action_name)
    }
}

impl AdmissionIdentity {
    /// Create an admission identity with no grants.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            id: Uuid::new_v4(),
            capability_grants: Vec::new(),
            resource_grants: Vec::new(),
            action_grants: Vec::new(),
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

    /// Add an explicit provider/action grant.
    #[must_use]
    pub fn with_action_grant(mut self, grant: AdmissionActionGrant) -> Self {
        self.action_grants.push(grant);
        self
    }

    /// Return true when this admission identity carries explicit authority.
    #[must_use]
    pub fn has_authority_grants(&self) -> bool {
        !self.capability_grants.is_empty()
            || !self.resource_grants.is_empty()
            || !self.action_grants.is_empty()
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

/// Artifact version emitted by the shared RuntimeKernel verified-artifact builder.
pub const RUNTIME_KERNEL_ARTIFACT_VERSION: &str = "runtime-kernel-artifact-v1";

/// Inputs needed to build a verifier-normalized RuntimeKernel artifact summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeArtifactBuildInput {
    /// Runtime root set identity used for definition/cache identity.
    pub root_id: RuntimeRootSetId,
    /// Selected profile/config identity used for definition/cache identity.
    pub profile: RuntimeProfileIdentity,
    /// Relative module path under the selected root.
    pub relative_module_path: String,
    /// Exported workflow name.
    pub workflow_name: String,
    /// Source text used only for stable content hashing.
    pub source: String,
    /// Check/type-summary facts used only for stable check-summary hashing.
    pub check_summary: String,
    /// Typed computation expression already produced by the checking/lowering pipeline.
    pub tcir: TcirComputationExpression,
    /// Honesty boundary for the carried TCIR used by the verifier.
    pub tcir_carrier_scope: RuntimeTcirCarrierScope,
}

impl RuntimeArtifactBuildInput {
    /// Create builder inputs from source/check/profile facts and a typed TCIR carrier.
    #[must_use]
    pub fn new(
        identity: RuntimeArtifactBuildIdentity,
        source: impl Into<String>,
        check_summary: impl Into<String>,
        tcir: TcirComputationExpression,
        tcir_carrier_scope: RuntimeTcirCarrierScope,
    ) -> Self {
        Self {
            root_id: identity.root_id,
            profile: identity.profile,
            relative_module_path: identity.relative_module_path,
            workflow_name: identity.workflow_name,
            source: source.into(),
            check_summary: check_summary.into(),
            tcir,
            tcir_carrier_scope,
        }
    }
}

/// Identity fields for RuntimeKernel artifact construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactBuildIdentity {
    /// Runtime root set identity used for definition/cache identity.
    pub root_id: RuntimeRootSetId,
    /// Selected profile/config identity used for definition/cache identity.
    pub profile: RuntimeProfileIdentity,
    /// Relative module path under the selected root.
    pub relative_module_path: String,
    /// Exported workflow name.
    pub workflow_name: String,
}

impl RuntimeArtifactBuildIdentity {
    /// Create artifact-builder identity inputs.
    #[must_use]
    pub fn new(
        root_id: RuntimeRootSetId,
        profile: RuntimeProfileIdentity,
        relative_module_path: impl Into<String>,
        workflow_name: impl Into<String>,
    ) -> Self {
        Self {
            root_id,
            profile,
            relative_module_path: relative_module_path.into(),
            workflow_name: workflow_name.into(),
        }
    }
}

/// Shared RuntimeKernel verified-artifact builder.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeKernelArtifactBuilder;

impl RuntimeKernelArtifactBuilder {
    /// Create a RuntimeKernel artifact builder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Build a deterministic verifier-normalized artifact summary.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeArtifactBuildError`] when AMIR or bytecode verification
    /// rejects the supplied TCIR-derived artifact. Source text is never reparsed
    /// during bytecode verification; it only participates in stable hashing.
    pub fn build(
        &self,
        input: RuntimeArtifactBuildInput,
    ) -> Result<RuntimeKernelVerifiedArtifact, RuntimeArtifactBuildError> {
        let source_hash = stable_sha256(&["source", &input.source]);
        let check_summary_hash = stable_sha256(&[
            "check-summary",
            input.profile.profile_id.as_str(),
            input.profile.config_id.as_str(),
            &source_hash,
            &input.check_summary,
        ]);
        let artifact_version = ArtifactVersion::new(RUNTIME_KERNEL_ARTIFACT_VERSION);
        let cache_key = RuntimeArtifactCacheKey::new(
            input.root_id.clone(),
            input.profile.profile_id.clone(),
            input.profile.config_id.clone(),
            source_hash.clone(),
            check_summary_hash.clone(),
            artifact_version.clone(),
        );
        let definition = WorkflowDefinitionIdentity::new(
            input.root_id,
            input.relative_module_path,
            input.workflow_name,
            input.profile.profile_id,
            input.profile.config_id,
            source_hash.clone(),
        );
        let artifact = WorkflowArtifactIdentity::new(
            definition.id.clone(),
            cache_key.clone(),
            artifact_version.clone(),
        );

        let tcir = RuntimeTcirArtifactSummary::from_tcir(&input.tcir, input.tcir_carrier_scope);
        let amir_module = AmirModule::from_tcir(&input.tcir);
        AmirVerifier::verify(&amir_module, &input.tcir)
            .map_err(|source| RuntimeArtifactBuildError::AmirVerification { source })?;
        let bytecode_module = BytecodeModule::from_amir(&amir_module, &input.tcir)
            .map_err(|source| RuntimeArtifactBuildError::AmirVerification { source })?;
        BytecodeVerifier::verify(&bytecode_module, &input.tcir)
            .map_err(|source| RuntimeArtifactBuildError::BytecodeVerification { source })?;

        Ok(RuntimeKernelVerifiedArtifact {
            source_hash,
            check_summary_hash,
            artifact_version,
            cache_key,
            definition,
            artifact,
            tcir,
            amir: RuntimeAmirArtifactSummary::from_module(&amir_module),
            bytecode: RuntimeBytecodeArtifactSummary::from_module(&bytecode_module),
            verifier: RuntimeArtifactVerifierResult::Verified,
        })
    }
}

/// Deterministic RuntimeKernel artifact summary shared by one-shot and daemon hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeKernelVerifiedArtifact {
    /// Stable source content hash.
    pub source_hash: String,
    /// Stable hash of check/type-summary facts.
    pub check_summary_hash: String,
    /// RuntimeKernel artifact builder version.
    pub artifact_version: ArtifactVersion,
    /// Cache key derived from roots, profile/config, hashes, and version.
    pub cache_key: RuntimeArtifactCacheKey,
    /// Workflow definition identity derived from the builder input.
    pub definition: WorkflowDefinitionIdentity,
    /// Workflow artifact identity derived from the builder input.
    pub artifact: WorkflowArtifactIdentity,
    /// Verifier-normalized TCIR summary.
    pub tcir: RuntimeTcirArtifactSummary,
    /// Verifier-normalized AMIR summary.
    pub amir: RuntimeAmirArtifactSummary,
    /// Verifier-normalized bytecode summary.
    pub bytecode: RuntimeBytecodeArtifactSummary,
    /// Final verifier result.
    pub verifier: RuntimeArtifactVerifierResult,
}

/// Language-level verifier-normalized artifact summary, excluding host identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeKernelArtifactLanguageSummary {
    /// Stable source content hash.
    pub source_hash: String,
    /// Stable hash of check/type-summary facts.
    pub check_summary_hash: String,
    /// RuntimeKernel artifact builder version.
    pub artifact_version: ArtifactVersion,
    /// Verifier-normalized TCIR summary.
    pub tcir: RuntimeTcirArtifactSummary,
    /// Verifier-normalized AMIR summary.
    pub amir: RuntimeAmirArtifactSummary,
    /// Verifier-normalized bytecode summary.
    pub bytecode: RuntimeBytecodeArtifactSummary,
    /// Final verifier result.
    pub verifier: RuntimeArtifactVerifierResult,
}

impl RuntimeKernelArtifactLanguageSummary {
    /// Project the host-independent language artifact summary from a verified artifact.
    #[must_use]
    pub fn from_verified_artifact(artifact: &RuntimeKernelVerifiedArtifact) -> Self {
        Self {
            source_hash: artifact.source_hash.clone(),
            check_summary_hash: artifact.check_summary_hash.clone(),
            artifact_version: artifact.artifact_version.clone(),
            tcir: artifact.tcir.clone(),
            amir: artifact.amir.clone(),
            bytecode: artifact.bytecode.clone(),
            verifier: artifact.verifier,
        }
    }
}

/// Verifier result retained in the RuntimeKernel artifact summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeArtifactVerifierResult {
    /// AMIR and bytecode verified against carried TCIR provenance.
    Verified,
}

/// Scope of the TCIR carrier used by RuntimeKernel artifact verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTcirCarrierScope {
    /// Full checked TCIR supplied by a caller that owns actual typed lowering.
    CheckedTcir,
    /// Alpha host summary after parse/check when full workflow-body TCIR is not
    /// yet exposed by the production engine pipeline.
    AlphaCheckedWorkflowBoundary,
}

/// Runtime artifact build errors.
#[derive(Debug, Error)]
pub enum RuntimeArtifactBuildError {
    /// AMIR verification rejected the TCIR-derived artifact.
    #[error("AMIR verification failed: {source}")]
    AmirVerification {
        /// Underlying AMIR verification error.
        #[from]
        source: crate::amir::AmirVerificationError,
    },
    /// Bytecode verification rejected the TCIR-derived artifact.
    #[error("bytecode verification failed: {source}")]
    BytecodeVerification {
        /// Underlying bytecode verification error.
        #[from]
        source: crate::amir::BytecodeVerificationError,
    },
}

/// Verifier-normalized TCIR provenance summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTcirArtifactSummary {
    /// Honest scope of the TCIR carrier used for this alpha artifact summary.
    pub carrier_scope: RuntimeTcirCarrierScope,
    /// Source-facing target display retained by TCIR.
    pub target_display: String,
    /// Selected evidence key used for lowering.
    pub evidence_key: String,
    /// Semantic tower attributed to the computation.
    pub tower_level: crate::runtime::TowerLevel,
    /// Ordered TCIR statement identities.
    pub statement_ids: Vec<TcirStatementId>,
}

impl RuntimeTcirArtifactSummary {
    fn from_tcir(tcir: &TcirComputationExpression, carrier_scope: RuntimeTcirCarrierScope) -> Self {
        Self {
            carrier_scope,
            target_display: tcir.target.display.clone(),
            evidence_key: tcir.evidence.evidence_key.clone(),
            tower_level: tcir.tower_level,
            statement_ids: tcir
                .statements
                .iter()
                .map(|statement| statement.id)
                .collect(),
        }
    }
}

/// Verifier-normalized AMIR provenance summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAmirArtifactSummary {
    /// AMIR schema version.
    pub schema_version: u16,
    /// Stable AMIR section layout.
    pub sections: Vec<AmirSectionKind>,
    /// Number of AMIR instructions in stable lowering order.
    pub instruction_count: usize,
}

impl RuntimeAmirArtifactSummary {
    fn from_module(module: &AmirModule) -> Self {
        Self {
            schema_version: module.schema_version,
            sections: module.sections.iter().map(|section| section.kind).collect(),
            instruction_count: module
                .blocks
                .iter()
                .map(|block| block.instructions.len())
                .sum(),
        }
    }
}

/// Verifier-normalized bytecode provenance summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBytecodeArtifactSummary {
    /// Bytecode schema version.
    pub schema_version: u16,
    /// Stable bytecode section layout.
    pub sections: Vec<BytecodeSectionKind>,
    /// Stable bytecode opcodes in logical instruction order.
    pub opcodes: Vec<BytecodeOpcode>,
    /// Number of bytecode instructions in stable lowering order.
    pub instruction_count: usize,
    /// Whether verifier requires a source reparse.
    pub requires_source_reparse: bool,
}

impl RuntimeBytecodeArtifactSummary {
    fn from_module(module: &BytecodeModule) -> Self {
        Self {
            schema_version: module.schema_version,
            sections: module.sections.iter().map(|section| section.kind).collect(),
            opcodes: module
                .instructions
                .iter()
                .map(|instruction| instruction.opcode)
                .collect(),
            instruction_count: module.instructions.len(),
            requires_source_reparse: module.requires_source_reparse(),
        }
    }
}

fn stable_sha256(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("sha256:{}", hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
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
