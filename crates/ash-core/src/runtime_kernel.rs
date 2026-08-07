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
//! - `ash-core::runtime::{RunId, ProcessId, ResourceId, CapabilityBindingId}`
//!   remain the lower runtime/admission identities.
//! - The engine admission request/outcome carriers remain the current admission
//!   boundary until TASK-928 routes starts through the kernel.
//! - `ash-runtime::RuntimeState` and `ash-runtime::Context` remain provider,
//!   resource, capability-binding, and execution-context state owners.
//! - This module adds the missing host/root/definition/artifact/instance/cache
//!   identities above those existing carriers. Provider registry identity is
//!   intentionally separate from admission authority grants.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::Expr;
use crate::amir::{
    AmirModule, AmirSectionKind, AmirVerifier, BytecodeModule, BytecodeOpcode, BytecodeSectionKind,
    BytecodeVerifier,
};
use crate::core_ash::{CoreRow, CoreType};
use crate::core_ash_contract::RuntimeMonitorEvidence;
use crate::runtime::{CapabilityBindingId, ProcessId, ResourceId, RuntimeTraceFact};
use crate::semantic_summary::SourceAnchor;
use crate::type_ir::{TcirComputationExpression, TcirFunctionArtifactProvenance, TcirStatementId};

/// Minimal alpha admission profile for one-shot RuntimeKernel starts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaAdmissionProfile {
    /// No requested grants or policy constraints; preserves existing alpha behavior.
    #[default]
    Empty,
    /// Explicitly admit the one-shot application instance.
    Allow,
    /// Reject the one-shot application instance before body execution.
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
    /// Admission rejected before entry body execution.
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

    /// Returns true when the application instance is admitted.
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

/// Stable application-definition identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationDefinitionId(String);

impl ApplicationDefinitionId {
    /// Borrow the application-definition identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Compiled/named application entry exported from a source root/module/artifact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationDefinitionIdentity {
    /// Stable definition identity.
    pub id: ApplicationDefinitionId,
    /// Runtime root set containing the definition.
    pub root_id: RuntimeRootSetId,
    /// Relative module path under the selected source root.
    pub relative_module_path: String,
    /// Exported application entry name.
    pub entry_name: String,
    /// Selected profile participating in definition identity.
    pub profile_id: RuntimeProfileId,
    /// Selected config participating in definition identity.
    pub config_id: RuntimeConfigId,
    /// Source or artifact version/hash for this definition index entry.
    pub source_identity: String,
}

impl ApplicationDefinitionIdentity {
    /// Create an application-definition identity.
    #[must_use]
    pub fn new(
        root_id: RuntimeRootSetId,
        relative_module_path: impl Into<String>,
        entry_name: impl Into<String>,
        profile_id: RuntimeProfileId,
        config_id: RuntimeConfigId,
        source_identity: impl Into<String>,
    ) -> Self {
        let relative_module_path = relative_module_path.into();
        let entry_name = entry_name.into();
        let source_identity = source_identity.into();
        let id = ApplicationDefinitionId(structured_identity(&[
            root_id.as_str(),
            &relative_module_path,
            &entry_name,
            profile_id.as_str(),
            config_id.as_str(),
            &source_identity,
        ]));
        Self {
            id,
            root_id,
            relative_module_path,
            entry_name,
            profile_id,
            config_id,
            source_identity,
        }
    }
}

/// Stable application-artifact identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationArtifactId(String);

impl ApplicationArtifactId {
    /// Borrow the application-artifact identity string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime execution artifact selected for an application definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationArtifactIdentity {
    /// Stable artifact identity.
    pub id: ApplicationArtifactId,
    /// Application definition this artifact implements.
    pub definition_id: ApplicationDefinitionId,
    /// Cache key used to select or invalidate this artifact.
    pub cache_key: RuntimeArtifactCacheKey,
    /// Artifact version pinned for admitted starts.
    pub version: ArtifactVersion,
}

impl ApplicationArtifactIdentity {
    /// Create an application-artifact identity.
    #[must_use]
    pub fn new(
        definition_id: ApplicationDefinitionId,
        cache_key: RuntimeArtifactCacheKey,
        version: ArtifactVersion,
    ) -> Self {
        let id = ApplicationArtifactId(structured_identity(&[
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

/// Kind of application entrypoint selected by a runtime invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationEntrypointKind {
    /// Target application entrypoint backed by an ordinary checked callable.
    CheckedCallable,
}

/// Structured diagnostic emitted while resolving application entrypoint metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplicationEntrypointDiagnostic {
    /// No entrypoint name was provided at the application boundary.
    #[error("missing application entrypoint name")]
    MissingEntrypointName,
    /// A checked-callable entrypoint was selected without a callable identity.
    #[error("missing callable identity for application entrypoint `{entrypoint_name}`")]
    MissingCallableIdentity {
        /// Entrypoint name being resolved.
        entrypoint_name: String,
    },
    /// Multiple checked computations match one entrypoint selection.
    #[error("ambiguous application entrypoint `{entrypoint_name}`")]
    AmbiguousEntrypoint {
        /// Entrypoint name being resolved.
        entrypoint_name: String,
        /// Candidate callable identities.
        candidates: Vec<String>,
    },
    /// Entrypoint metadata was derived from stale source/check identity.
    #[error("stale application entrypoint `{entrypoint_name}`")]
    StaleEntrypoint {
        /// Entrypoint name being resolved.
        entrypoint_name: String,
        /// Source/check identity used by the metadata.
        expected_identity: String,
        /// Current source/check identity observed at invocation.
        actual_identity: String,
    },
    /// Entrypoint metadata is incompatible with the runtime target.
    #[error("incompatible application entrypoint `{entrypoint_name}`")]
    IncompatibleEntrypoint {
        /// Entrypoint name being resolved.
        entrypoint_name: String,
        /// Expected entrypoint shape or boundary condition.
        expected: String,
        /// Actual entrypoint shape or boundary condition.
        actual: String,
    },
}

impl ApplicationEntrypointDiagnostic {
    /// Build an ambiguous-entrypoint diagnostic.
    #[must_use]
    pub fn ambiguous<I, S>(entrypoint_name: impl Into<String>, candidates: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::AmbiguousEntrypoint {
            entrypoint_name: entrypoint_name.into(),
            candidates: candidates.into_iter().map(Into::into).collect(),
        }
    }

    /// Build a stale-entrypoint diagnostic.
    #[must_use]
    pub fn stale(
        entrypoint_name: impl Into<String>,
        expected_identity: impl Into<String>,
        actual_identity: impl Into<String>,
    ) -> Self {
        Self::StaleEntrypoint {
            entrypoint_name: entrypoint_name.into(),
            expected_identity: expected_identity.into(),
            actual_identity: actual_identity.into(),
        }
    }

    /// Build an incompatible-entrypoint diagnostic.
    #[must_use]
    pub fn incompatible(
        entrypoint_name: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::IncompatibleEntrypoint {
            entrypoint_name: entrypoint_name.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Application/runtime entrypoint metadata over a checked computation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationEntrypointMetadata {
    /// Runtime entrypoint name selected by the host.
    pub name: String,
    /// Entrypoint kind.
    pub kind: ApplicationEntrypointKind,
    /// Checked callable identity for target application entrypoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callable_identity: Option<String>,
    /// Relative module path containing the selected entrypoint.
    pub relative_module_path: String,
    /// Runtime target identity selected by the host before artifact identity is built.
    pub runtime_target_identity: String,
}

impl ApplicationEntrypointMetadata {
    /// Build checked-callable application entrypoint metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationEntrypointDiagnostic`] when required metadata is missing.
    pub fn checked_callable(
        name: impl Into<String>,
        callable_identity: impl Into<String>,
        relative_module_path: impl Into<String>,
        runtime_target_identity: impl Into<String>,
    ) -> Result<Self, ApplicationEntrypointDiagnostic> {
        let name = name.into();
        if name.is_empty() {
            return Err(ApplicationEntrypointDiagnostic::MissingEntrypointName);
        }
        let callable_identity = callable_identity.into();
        if callable_identity.is_empty() {
            return Err(ApplicationEntrypointDiagnostic::MissingCallableIdentity {
                entrypoint_name: name,
            });
        }
        Ok(Self {
            name,
            kind: ApplicationEntrypointKind::CheckedCallable,
            callable_identity: Some(callable_identity),
            relative_module_path: relative_module_path.into(),
            runtime_target_identity: runtime_target_identity.into(),
        })
    }
}

/// Structured diagnostic emitted while resolving admission profile boundary metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplicationAdmissionProfileDiagnostic {
    /// No admission profile name was provided at the application boundary.
    #[error("missing admission profile name")]
    MissingProfileName,
    /// Admission profile name is not a stable profile identifier.
    #[error("malformed admission profile name `{profile_name}`")]
    MalformedProfileName {
        /// Profile name supplied by the boundary.
        profile_name: String,
    },
    /// Admission profile metadata was derived from stale source/check/profile identity.
    #[error("stale admission profile `{profile_name}`")]
    StaleProfile {
        /// Profile name being resolved.
        profile_name: String,
        /// Expected profile identity.
        expected_identity: String,
        /// Actual profile identity observed at invocation.
        actual_identity: String,
    },
    /// Admission profile metadata is incompatible with the runtime target.
    #[error("incompatible admission profile `{profile_name}`")]
    IncompatibleProfile {
        /// Profile name being resolved.
        profile_name: String,
        /// Expected profile shape or boundary condition.
        expected: String,
        /// Actual profile shape or boundary condition.
        actual: String,
    },
    /// Admission profile metadata attempted to grant authority directly.
    #[error("admission profile `{profile_name}` attempted to widen authority")]
    AuthorityWideningProfile {
        /// Profile name being resolved.
        profile_name: String,
    },
}

impl ApplicationAdmissionProfileDiagnostic {
    /// Build a stale-profile diagnostic.
    #[must_use]
    pub fn stale(
        profile_name: impl Into<String>,
        expected_identity: impl Into<String>,
        actual_identity: impl Into<String>,
    ) -> Self {
        Self::StaleProfile {
            profile_name: profile_name.into(),
            expected_identity: expected_identity.into(),
            actual_identity: actual_identity.into(),
        }
    }

    /// Build an incompatible-profile diagnostic.
    #[must_use]
    pub fn incompatible(
        profile_name: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::IncompatibleProfile {
            profile_name: profile_name.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Admission profile metadata selected at an application/runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationAdmissionProfile {
    /// Stable profile name selected by the boundary.
    pub name: String,
    /// Stable profile identity used in artifacts and reports.
    pub profile_identity: String,
    /// Boundary source that supplied this profile.
    pub boundary_source: String,
    /// Whether this metadata directly grants authority. Valid profile metadata must keep this false.
    pub grants_authority: bool,
}

impl ApplicationAdmissionProfile {
    /// Build non-authority metadata for one of the alpha admission profiles.
    #[must_use]
    pub fn alpha(profile: AlphaAdmissionProfile) -> Self {
        let name = profile.as_str().to_string();
        Self {
            profile_identity: format!("admission-profile:{name}"),
            name,
            boundary_source: "alpha-admission-profile".to_string(),
            grants_authority: false,
        }
    }

    /// Build admission profile metadata from a runtime boundary input.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationAdmissionProfileDiagnostic`] when the profile name is missing,
    /// malformed, or attempts to grant authority directly.
    pub fn runtime_boundary(
        name: impl Into<String>,
        boundary_source: impl Into<String>,
        grants_authority: bool,
    ) -> Result<Self, ApplicationAdmissionProfileDiagnostic> {
        let name = name.into();
        if name.is_empty() {
            return Err(ApplicationAdmissionProfileDiagnostic::MissingProfileName);
        }
        if !is_valid_admission_profile_name(&name) {
            return Err(
                ApplicationAdmissionProfileDiagnostic::MalformedProfileName { profile_name: name },
            );
        }
        if grants_authority {
            return Err(
                ApplicationAdmissionProfileDiagnostic::AuthorityWideningProfile {
                    profile_name: name,
                },
            );
        }
        Ok(Self {
            profile_identity: format!("admission-profile:{name}"),
            name,
            boundary_source: boundary_source.into(),
            grants_authority,
        })
    }
}

fn is_valid_admission_profile_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
}

/// Structured diagnostic emitted while resolving application boundary binding metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplicationBoundaryBindingDiagnostic {
    /// A binding identity was required but not provided.
    #[error("missing {family} boundary binding identity")]
    MissingBindingIdentity {
        /// Binding family being resolved.
        family: String,
    },
    /// A binding identity is not a stable boundary identifier.
    #[error("malformed {family} boundary binding identity `{binding_identity}`")]
    MalformedBindingIdentity {
        /// Binding family being resolved.
        family: String,
        /// Identity supplied by the boundary.
        binding_identity: String,
    },
    /// Boundary binding metadata was derived from stale evidence.
    #[error("stale {family} boundary binding `{binding_identity}`")]
    StaleBinding {
        /// Binding family being resolved.
        family: String,
        /// Binding identity being resolved.
        binding_identity: String,
        /// Expected evidence identity.
        expected_identity: String,
        /// Actual evidence identity observed at invocation.
        actual_identity: String,
    },
    /// Boundary binding metadata is incompatible with the runtime target.
    #[error("incompatible {family} boundary binding `{binding_identity}`")]
    IncompatibleBinding {
        /// Binding family being resolved.
        family: String,
        /// Binding identity being resolved.
        binding_identity: String,
        /// Expected boundary condition.
        expected: String,
        /// Actual boundary condition.
        actual: String,
    },
    /// Boundary binding metadata attempted to grant authority directly.
    #[error("{family} boundary binding `{binding_identity}` attempted to widen authority")]
    AuthorityWideningBinding {
        /// Binding family being resolved.
        family: String,
        /// Binding identity being resolved.
        binding_identity: String,
    },
}

impl ApplicationBoundaryBindingDiagnostic {
    /// Build a stale-binding diagnostic.
    #[must_use]
    pub fn stale(
        family: impl Into<String>,
        binding_identity: impl Into<String>,
        expected_identity: impl Into<String>,
        actual_identity: impl Into<String>,
    ) -> Self {
        Self::StaleBinding {
            family: family.into(),
            binding_identity: binding_identity.into(),
            expected_identity: expected_identity.into(),
            actual_identity: actual_identity.into(),
        }
    }

    /// Build an incompatible-binding diagnostic.
    #[must_use]
    pub fn incompatible(
        family: impl Into<String>,
        binding_identity: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::IncompatibleBinding {
            family: family.into(),
            binding_identity: binding_identity.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Application/runtime boundary binding request before validation and redaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationBoundaryBindingManifest {
    /// Role identities selected at the boundary.
    pub roles: Vec<String>,
    /// Policy identities selected at the boundary.
    pub policies: Vec<String>,
    /// Resource identities selected at the boundary.
    pub resources: Vec<String>,
    /// Provider identities selected at the boundary.
    pub providers: Vec<String>,
    /// Contract or evidence identities selected at the boundary.
    pub contracts: Vec<String>,
    /// Whether this metadata directly grants authority. Valid bindings must keep this false.
    pub grants_authority: bool,
}

/// Auditable, non-authority bindings selected at an application/runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationBoundaryBindings {
    /// Boundary source that supplied the bindings.
    pub boundary_source: String,
    /// Redacted stable identity for the boundary evidence set.
    pub redacted_evidence_identity: String,
    /// Role identities selected at the boundary.
    pub roles: Vec<String>,
    /// Policy identities selected at the boundary.
    pub policies: Vec<String>,
    /// Resource identities selected at the boundary.
    pub resources: Vec<String>,
    /// Provider identities selected at the boundary.
    pub providers: Vec<String>,
    /// Contract or evidence identities selected at the boundary.
    pub contracts: Vec<String>,
    /// Whether this metadata directly grants authority. Valid bindings must keep this false.
    pub grants_authority: bool,
}

impl ApplicationBoundaryBindings {
    /// Build an empty non-authority boundary binding record.
    #[must_use]
    pub fn empty(boundary_source: impl Into<String>) -> Self {
        Self::from_manifest(
            boundary_source,
            ApplicationBoundaryBindingManifest::default(),
        )
        .expect("empty boundary binding manifest is valid")
    }

    /// Build non-authority boundary bindings from a runtime boundary manifest.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationBoundaryBindingDiagnostic`] when a binding identity is missing,
    /// malformed, or attempts to grant authority directly.
    pub fn from_manifest(
        boundary_source: impl Into<String>,
        manifest: ApplicationBoundaryBindingManifest,
    ) -> Result<Self, ApplicationBoundaryBindingDiagnostic> {
        let boundary_source = boundary_source.into();
        let roles = normalize_boundary_binding_list("role", manifest.roles)?;
        let policies = normalize_boundary_binding_list("policy", manifest.policies)?;
        let resources = normalize_boundary_binding_list("resource", manifest.resources)?;
        let providers = normalize_boundary_binding_list("provider", manifest.providers)?;
        let contracts = normalize_boundary_binding_list("contract", manifest.contracts)?;
        if manifest.grants_authority {
            let (family, binding_identity) = first_boundary_binding_identity(
                &roles,
                &policies,
                &resources,
                &providers,
                &contracts,
                &boundary_source,
            );
            return Err(
                ApplicationBoundaryBindingDiagnostic::AuthorityWideningBinding {
                    family,
                    binding_identity,
                },
            );
        }
        let redacted_evidence_identity = boundary_binding_evidence_identity(
            &boundary_source,
            &roles,
            &policies,
            &resources,
            &providers,
            &contracts,
        );
        Ok(Self {
            boundary_source,
            redacted_evidence_identity,
            roles,
            policies,
            resources,
            providers,
            contracts,
            grants_authority: manifest.grants_authority,
        })
    }
}

fn normalize_boundary_binding_list(
    family: &'static str,
    identities: Vec<String>,
) -> Result<Vec<String>, ApplicationBoundaryBindingDiagnostic> {
    let mut normalized = Vec::with_capacity(identities.len());
    for identity in identities {
        let identity = identity.trim().to_string();
        if identity.is_empty() {
            return Err(
                ApplicationBoundaryBindingDiagnostic::MissingBindingIdentity {
                    family: family.to_string(),
                },
            );
        }
        if !is_valid_boundary_binding_identity(&identity) {
            return Err(
                ApplicationBoundaryBindingDiagnostic::MalformedBindingIdentity {
                    family: family.to_string(),
                    binding_identity: identity,
                },
            );
        }
        normalized.push(identity);
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn is_valid_boundary_binding_identity(identity: &str) -> bool {
    identity
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
}

fn first_boundary_binding_identity(
    roles: &[String],
    policies: &[String],
    resources: &[String],
    providers: &[String],
    contracts: &[String],
    boundary_source: &str,
) -> (String, String) {
    [
        ("role", roles),
        ("policy", policies),
        ("resource", resources),
        ("provider", providers),
        ("contract", contracts),
    ]
    .into_iter()
    .find_map(|(family, identities)| {
        identities
            .first()
            .map(|identity| (family.to_string(), identity.clone()))
    })
    .unwrap_or_else(|| ("boundary".to_string(), boundary_source.to_string()))
}

fn boundary_binding_evidence_identity(
    boundary_source: &str,
    roles: &[String],
    policies: &[String],
    resources: &[String],
    providers: &[String],
    contracts: &[String],
) -> String {
    stable_sha256(&[
        "application-boundary-bindings",
        boundary_source,
        &structured_identity(&roles.iter().map(String::as_str).collect::<Vec<_>>()),
        &structured_identity(&policies.iter().map(String::as_str).collect::<Vec<_>>()),
        &structured_identity(&resources.iter().map(String::as_str).collect::<Vec<_>>()),
        &structured_identity(&providers.iter().map(String::as_str).collect::<Vec<_>>()),
        &structured_identity(&contracts.iter().map(String::as_str).collect::<Vec<_>>()),
    ])
}

/// Invocation packet tying application entrypoint metadata to source/check/runtime identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationInvocationPacket {
    /// Selected entrypoint metadata.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Admission profile metadata selected at the runtime boundary.
    pub admission_profile: ApplicationAdmissionProfile,
    /// Non-authority role/policy/resource/provider/contract bindings selected at the boundary.
    pub boundary_bindings: ApplicationBoundaryBindings,
    /// Source identity used to derive the invocation artifact.
    pub source_identity: String,
    /// Check/type-summary identity used to derive the invocation artifact.
    pub check_identity: String,
    /// Runtime artifact identity selected for execution.
    pub runtime_target_identity: String,
}

impl ApplicationInvocationPacket {
    /// Create an invocation packet from entrypoint and artifact identities.
    #[must_use]
    pub fn new(
        entrypoint: ApplicationEntrypointMetadata,
        admission_profile: ApplicationAdmissionProfile,
        boundary_bindings: ApplicationBoundaryBindings,
        source_identity: impl Into<String>,
        check_identity: impl Into<String>,
        runtime_target_identity: impl Into<String>,
    ) -> Self {
        Self {
            entrypoint,
            admission_profile,
            boundary_bindings,
            source_identity: source_identity.into(),
            check_identity: check_identity.into(),
            runtime_target_identity: runtime_target_identity.into(),
        }
    }
}

/// Terminal status projected into an application runtime report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationTerminalStatus {
    /// Runtime admission succeeded but body execution has not reached a terminal outcome.
    Admitted,
    /// Runtime invocation completed successfully.
    Succeeded,
    /// Runtime invocation failed after admission.
    Failed,
    /// Runtime invocation was cancelled.
    Cancelled,
    /// Runtime invocation was rejected before body execution.
    Rejected,
}

impl ApplicationTerminalStatus {
    /// Stable terminal status label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Rejected => "rejected",
        }
    }
}

/// Terminal outcome evidence for an application runtime invocation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationTerminalOutcome {
    /// Stable terminal status.
    pub status: ApplicationTerminalStatus,
    /// Whether this outcome is terminal for the invocation.
    pub is_terminal: bool,
    /// Optional human-readable failure, cancellation, or rejection reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ApplicationTerminalOutcome {
    /// Build an admitted-but-not-terminal outcome.
    #[must_use]
    pub const fn admitted() -> Self {
        Self {
            status: ApplicationTerminalStatus::Admitted,
            is_terminal: false,
            reason: None,
        }
    }

    /// Build a successful terminal outcome.
    #[must_use]
    pub const fn succeeded() -> Self {
        Self {
            status: ApplicationTerminalStatus::Succeeded,
            is_terminal: true,
            reason: None,
        }
    }

    /// Build a failed terminal outcome.
    #[must_use]
    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            status: ApplicationTerminalStatus::Failed,
            is_terminal: true,
            reason: Some(reason.into()),
        }
    }

    /// Build a cancelled terminal outcome.
    #[must_use]
    pub fn cancelled(reason: impl Into<String>) -> Self {
        Self {
            status: ApplicationTerminalStatus::Cancelled,
            is_terminal: true,
            reason: Some(reason.into()),
        }
    }

    /// Build a rejected terminal outcome.
    #[must_use]
    pub fn rejected(reason: impl Into<String>) -> Self {
        Self {
            status: ApplicationTerminalStatus::Rejected,
            is_terminal: true,
            reason: Some(reason.into()),
        }
    }
}

/// Authority-neutral trace bundle attached to an application runtime report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationTraceBundle {
    /// Stable redacted identity for this trace bundle.
    pub trace_identity: String,
    /// Source identity from the invocation packet.
    pub source_identity: String,
    /// Check identity from the invocation packet.
    pub check_identity: String,
    /// Entrypoint/runtime target identity from the invocation packet.
    pub entrypoint_identity: String,
    /// Admission profile identity from the invocation packet.
    pub admission_profile_identity: String,
    /// Redacted evidence identity for boundary bindings.
    pub boundary_evidence_identity: String,
    /// Admission facts projected as evidence, not grants.
    pub admission_facts: Vec<String>,
    /// Boundary facts projected as evidence, not grants.
    pub boundary_facts: Vec<String>,
    /// Retained process/channel runtime facts.
    pub process_facts: Vec<RuntimeTraceFact>,
    /// Contract/evidence identities observed at the boundary.
    pub contract_evidence: Vec<String>,
    /// Runtime monitor evidence rows.
    pub monitor_evidence: Vec<RuntimeMonitorEvidence>,
    /// Whether this trace bundle grants authority. Application trace bundles must keep this false.
    pub grants_authority: bool,
    /// Whether this trace bundle mutates authority state. Application trace bundles must keep this false.
    pub mutates_authority: bool,
}

impl ApplicationTraceBundle {
    /// Project an authority-neutral trace bundle from an invocation packet.
    #[must_use]
    pub fn from_invocation_packet(
        invocation_packet: &ApplicationInvocationPacket,
        process_facts: Vec<RuntimeTraceFact>,
        monitor_evidence: Vec<RuntimeMonitorEvidence>,
    ) -> Self {
        let source_identity = invocation_packet.source_identity.clone();
        let check_identity = invocation_packet.check_identity.clone();
        let entrypoint_identity = invocation_packet.entrypoint.runtime_target_identity.clone();
        let admission_profile_identity =
            invocation_packet.admission_profile.profile_identity.clone();
        let boundary_evidence_identity = invocation_packet
            .boundary_bindings
            .redacted_evidence_identity
            .clone();
        let admission_facts = vec![format!(
            "admission_profile:{}",
            invocation_packet.admission_profile.profile_identity
        )];
        let boundary_facts = application_boundary_facts(&invocation_packet.boundary_bindings);
        let contract_evidence = invocation_packet.boundary_bindings.contracts.clone();
        let trace_identity = application_trace_identity(ApplicationTraceIdentityInput {
            source_identity: &source_identity,
            check_identity: &check_identity,
            entrypoint_identity: &entrypoint_identity,
            admission_profile_identity: &admission_profile_identity,
            boundary_evidence_identity: &boundary_evidence_identity,
            admission_facts: &admission_facts,
            boundary_facts: &boundary_facts,
            process_facts: &process_facts,
            contract_evidence: &contract_evidence,
            monitor_evidence_count: monitor_evidence.len(),
        });
        Self {
            trace_identity,
            source_identity,
            check_identity,
            entrypoint_identity,
            admission_profile_identity,
            boundary_evidence_identity,
            admission_facts,
            boundary_facts,
            process_facts,
            contract_evidence,
            monitor_evidence,
            grants_authority: false,
            mutates_authority: false,
        }
    }
}

/// Authority-neutral application runtime report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationRuntimeReport {
    /// Stable redacted identity for this report.
    pub report_identity: String,
    /// Source identity from the invocation packet.
    pub source_identity: String,
    /// Check identity from the invocation packet.
    pub check_identity: String,
    /// Entrypoint/runtime target identity from the invocation packet.
    pub entrypoint_identity: String,
    /// Admission profile metadata selected at the runtime boundary.
    pub admission_profile: ApplicationAdmissionProfile,
    /// Boundary binding metadata selected at the runtime boundary.
    pub boundary_bindings: ApplicationBoundaryBindings,
    /// Terminal outcome projected for this invocation.
    pub terminal_outcome: ApplicationTerminalOutcome,
    /// Authority-neutral trace bundle for this invocation.
    pub trace_bundle: ApplicationTraceBundle,
    /// Whether this report grants authority. Application reports must keep this false.
    pub grants_authority: bool,
    /// Whether this report mutates authority state. Application reports must keep this false.
    pub mutates_authority: bool,
}

impl ApplicationRuntimeReport {
    /// Build an authority-neutral application runtime report.
    #[must_use]
    pub fn new(
        invocation_packet: &ApplicationInvocationPacket,
        terminal_outcome: ApplicationTerminalOutcome,
        trace_bundle: ApplicationTraceBundle,
    ) -> Self {
        let report_identity = stable_sha256(&[
            "application-runtime-report",
            &trace_bundle.trace_identity,
            terminal_outcome.status.as_str(),
            terminal_outcome.reason.as_deref().unwrap_or(""),
        ]);
        Self {
            report_identity,
            source_identity: invocation_packet.source_identity.clone(),
            check_identity: invocation_packet.check_identity.clone(),
            entrypoint_identity: invocation_packet.entrypoint.runtime_target_identity.clone(),
            admission_profile: invocation_packet.admission_profile.clone(),
            boundary_bindings: invocation_packet.boundary_bindings.clone(),
            terminal_outcome,
            trace_bundle,
            grants_authority: false,
            mutates_authority: false,
        }
    }
}

fn application_boundary_facts(boundary_bindings: &ApplicationBoundaryBindings) -> Vec<String> {
    let mut facts = vec![
        format!("boundary_source:{}", boundary_bindings.boundary_source),
        format!(
            "boundary_evidence:{}",
            boundary_bindings.redacted_evidence_identity
        ),
    ];
    facts.extend(
        boundary_bindings
            .roles
            .iter()
            .map(|identity| format!("role:{identity}")),
    );
    facts.extend(
        boundary_bindings
            .policies
            .iter()
            .map(|identity| format!("policy:{identity}")),
    );
    facts.extend(
        boundary_bindings
            .resources
            .iter()
            .map(|identity| format!("resource:{identity}")),
    );
    facts.extend(
        boundary_bindings
            .providers
            .iter()
            .map(|identity| format!("provider:{identity}")),
    );
    facts.extend(
        boundary_bindings
            .contracts
            .iter()
            .map(|identity| format!("contract:{identity}")),
    );
    facts
}

struct ApplicationTraceIdentityInput<'a> {
    source_identity: &'a str,
    check_identity: &'a str,
    entrypoint_identity: &'a str,
    admission_profile_identity: &'a str,
    boundary_evidence_identity: &'a str,
    admission_facts: &'a [String],
    boundary_facts: &'a [String],
    process_facts: &'a [RuntimeTraceFact],
    contract_evidence: &'a [String],
    monitor_evidence_count: usize,
}

fn application_trace_identity(input: ApplicationTraceIdentityInput<'_>) -> String {
    let process_fact_text = input
        .process_facts
        .iter()
        .map(|fact| format!("{:?}:{:?}:{}", fact.kind, fact.event, fact.subject))
        .collect::<Vec<_>>();
    stable_sha256(&[
        "application-trace-bundle",
        input.source_identity,
        input.check_identity,
        input.entrypoint_identity,
        input.admission_profile_identity,
        input.boundary_evidence_identity,
        &structured_identity(
            &input
                .admission_facts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        &structured_identity(
            &input
                .boundary_facts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        &structured_identity(
            &process_fact_text
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        &structured_identity(
            &input
                .contract_evidence
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ),
        &input.monitor_evidence_count.to_string(),
    ])
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

/// Explicit admission authority grants for one application instance.
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

/// Unique application instance identity for one admitted start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationInstanceId(pub Uuid);

impl ApplicationInstanceId {
    /// Create a fresh application instance identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ApplicationInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// One admitted application execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationInstanceIdentity {
    /// Stable identity for this admitted execution.
    pub id: ApplicationInstanceId,
    /// Host mode that admitted this instance.
    pub host_mode: RuntimeHostMode,
    /// Definition started by this instance.
    pub definition_id: ApplicationDefinitionId,
    /// Artifact pinned for this instance.
    pub artifact_id: ApplicationArtifactId,
    /// Profile/config identity used for this start.
    pub profile: RuntimeProfileIdentity,
    /// Provider registry inventory available to the host.
    pub provider_registry: ProviderRegistryIdentity,
    /// Explicit admission authority grants.
    pub admission: AdmissionIdentity,
}

impl ApplicationInstanceIdentity {
    /// Admit one application instance identity without executing it.
    #[must_use]
    pub fn admit(
        host_mode: RuntimeHostMode,
        definition_id: ApplicationDefinitionId,
        artifact_id: ApplicationArtifactId,
        profile: RuntimeProfileIdentity,
        provider_registry: ProviderRegistryIdentity,
        admission: AdmissionIdentity,
    ) -> Self {
        Self {
            id: ApplicationInstanceId::new(),
            host_mode,
            definition_id,
            artifact_id,
            profile,
            provider_registry,
            admission,
        }
    }

    /// Create a process-tree identity rooted in this application instance.
    #[must_use]
    pub fn process_tree(&self, root_process_id: ProcessId) -> ProcessTreeIdentity {
        ProcessTreeIdentity::new(self.id, root_process_id)
    }
}

/// Process-tree identity rooted by an application instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessTreeIdentity {
    /// Application instance that owns this process tree.
    pub application_instance_id: ApplicationInstanceId,
    root_process_id: ProcessId,
}

impl ProcessTreeIdentity {
    /// Create a process tree rooted in an application instance.
    #[must_use]
    pub const fn new(
        application_instance_id: ApplicationInstanceId,
        root_process_id: ProcessId,
    ) -> Self {
        Self {
            application_instance_id,
            root_process_id,
        }
    }

    /// Return the application instance that roots this process tree.
    #[must_use]
    pub const fn rooted_in(&self) -> ApplicationInstanceId {
        self.application_instance_id
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

/// Checked ordinary-function facts required to build a runtime artifact.
///
/// Runtime artifact construction consumes this checked/lowered boundary directly;
/// it never reparses source or invents an entry-specific computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckedFunctionArtifact {
    /// Stable identity of the checked function selected for invocation.
    pub function_identity: String,
    /// Canonical Core effect row checked for the function.
    pub effect_row: CoreRow,
    /// Lowered, checked function body.
    pub body: Expr,
    /// Source anchor retained from the checked entry boundary.
    pub source_anchor: SourceAnchor,
    /// Checked Core result type of the function body.
    pub result_type: CoreType,
}

/// Inputs needed to build a verifier-normalized RuntimeKernel artifact summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeArtifactBuildInput {
    /// Runtime root set identity used for definition/cache identity.
    pub root_id: RuntimeRootSetId,
    /// Selected profile/config identity used for definition/cache identity.
    pub profile: RuntimeProfileIdentity,
    /// Relative module path under the selected root.
    pub relative_module_path: String,
    /// Exported application entry name.
    pub entry_name: String,
    /// Application/runtime entrypoint metadata selected for this artifact.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Admission profile metadata selected at the runtime boundary.
    pub admission_profile: ApplicationAdmissionProfile,
    /// Non-authority boundary bindings selected at the runtime boundary.
    pub boundary_bindings: ApplicationBoundaryBindings,
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
            entry_name: identity.entry_name,
            entrypoint: identity.entrypoint,
            admission_profile: identity.admission_profile,
            boundary_bindings: identity.boundary_bindings,
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
    /// Exported application entry name.
    pub entry_name: String,
    /// Application/runtime entrypoint metadata selected for this artifact.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Admission profile metadata selected at the runtime boundary.
    pub admission_profile: ApplicationAdmissionProfile,
    /// Non-authority boundary bindings selected at the runtime boundary.
    pub boundary_bindings: ApplicationBoundaryBindings,
}

impl RuntimeArtifactBuildIdentity {
    /// Create artifact-builder identity inputs.
    #[must_use]
    pub fn new(
        root_id: RuntimeRootSetId,
        profile: RuntimeProfileIdentity,
        relative_module_path: impl Into<String>,
        entry_name: impl Into<String>,
    ) -> Self {
        let relative_module_path = relative_module_path.into();
        let entry_name = entry_name.into();
        let entrypoint = ApplicationEntrypointMetadata {
            name: entry_name.clone(),
            kind: ApplicationEntrypointKind::CheckedCallable,
            callable_identity: Some(format!("callable:{relative_module_path}::{entry_name}")),
            relative_module_path: relative_module_path.clone(),
            runtime_target_identity: format!("runtime-target:application-entry:{entry_name}"),
        };
        let admission_profile = ApplicationAdmissionProfile::alpha(AlphaAdmissionProfile::Empty);
        let boundary_bindings = ApplicationBoundaryBindings::empty("alpha-boundary-bindings");
        Self {
            root_id,
            profile,
            relative_module_path,
            entry_name,
            entrypoint,
            admission_profile,
            boundary_bindings,
        }
    }

    /// Replace the default application entrypoint metadata.
    #[must_use]
    pub fn with_entrypoint(mut self, entrypoint: ApplicationEntrypointMetadata) -> Self {
        self.entrypoint = entrypoint;
        self
    }

    /// Replace the default empty admission profile metadata.
    #[must_use]
    pub fn with_admission_profile(
        mut self,
        admission_profile: ApplicationAdmissionProfile,
    ) -> Self {
        self.admission_profile = admission_profile;
        self
    }

    /// Replace the default empty boundary binding metadata.
    #[must_use]
    pub fn with_boundary_bindings(
        mut self,
        boundary_bindings: ApplicationBoundaryBindings,
    ) -> Self {
        self.boundary_bindings = boundary_bindings;
        self
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
        // The source/check-summary boundary may be shared by distinct checked
        // artifacts (for example, when a host supplies a normalized summary).
        // Bind the cache and artifact identities to the complete typed TCIR as
        // well, so they cannot select an artifact compiled from a different
        // body, result type, or checked-function provenance.
        let checked_tcir_fingerprint = checked_tcir_artifact_fingerprint(&input.tcir)?;
        let check_summary_hash = stable_sha256(&[
            "check-summary",
            input.profile.profile_id.as_str(),
            input.profile.config_id.as_str(),
            &source_hash,
            &input.check_summary,
            "checked-tcir-fingerprint",
            &checked_tcir_fingerprint,
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
        let definition = ApplicationDefinitionIdentity::new(
            input.root_id,
            input.relative_module_path,
            input.entry_name,
            input.profile.profile_id,
            input.profile.config_id,
            source_hash.clone(),
        );
        let artifact = ApplicationArtifactIdentity::new(
            definition.id.clone(),
            cache_key.clone(),
            artifact_version.clone(),
        );
        let invocation_packet = ApplicationInvocationPacket::new(
            input.entrypoint.clone(),
            input.admission_profile.clone(),
            input.boundary_bindings.clone(),
            source_hash.clone(),
            check_summary_hash.clone(),
            artifact.id.as_str(),
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
            entrypoint: input.entrypoint,
            invocation_packet,
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
    /// Application definition identity derived from the builder input.
    pub definition: ApplicationDefinitionIdentity,
    /// Application artifact identity derived from the builder input.
    pub artifact: ApplicationArtifactIdentity,
    /// Application/runtime entrypoint metadata derived from checked computation selection.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Invocation packet carrying source, check, and runtime target identity.
    pub invocation_packet: ApplicationInvocationPacket,
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
    /// Application/runtime entrypoint metadata, excluding host-specific identity.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Invocation packet carrying source, check, and runtime target identity.
    pub invocation_packet: ApplicationInvocationPacket,
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
            entrypoint: artifact.entrypoint.clone(),
            invocation_packet: artifact.invocation_packet.clone(),
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
    /// Checked ordinary-function artifact supplied by the engine pipeline.
    CheckedFunctionArtifact,
    /// Non-authorizing daemon metadata projected from a canonical module closure.
    ///
    /// This scope describes reporting/index metadata only. It is never an
    /// execution artifact or an admission credential; canonical execution
    /// remains owned by the Engine-linked Core/CPS route.
    CanonicalModuleClosureMetadata,
}

/// Runtime artifact build errors.
#[derive(Debug, Error)]
pub enum RuntimeArtifactBuildError {
    /// Checked TCIR could not be serialized for deterministic artifact identity.
    #[error("could not serialize checked TCIR for artifact identity: {0}")]
    TcirIdentitySerialization(#[from] serde_json::Error),
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
    /// Semantic boundary attributed to the computation.
    pub boundary_level: crate::runtime::FailureBoundary,
    /// Ordered TCIR statement identities.
    pub statement_ids: Vec<TcirStatementId>,
    /// Checked function provenance propagated through TCIR and AMIR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_artifact: Option<TcirFunctionArtifactProvenance>,
}

impl RuntimeTcirArtifactSummary {
    fn from_tcir(tcir: &TcirComputationExpression, carrier_scope: RuntimeTcirCarrierScope) -> Self {
        Self {
            carrier_scope,
            target_display: tcir.target.display.clone(),
            evidence_key: tcir.evidence.evidence_key.clone(),
            boundary_level: tcir.boundary_level,
            statement_ids: tcir
                .statements
                .iter()
                .map(|statement| statement.id)
                .collect(),
            function_artifact: tcir.function_artifact.clone(),
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
    /// Whole-computation provenance copied into AMIR.
    pub provenance: crate::amir::TcirComputationProvenance,
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
            provenance: module
                .provenance
                .computation
                .clone()
                .expect("verified AMIR module always carries computation provenance"),
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

fn checked_tcir_artifact_fingerprint(
    tcir: &TcirComputationExpression,
) -> Result<String, serde_json::Error> {
    let tcir_json = serde_json::to_value(tcir)?;
    let mut encoded = Vec::new();
    append_canonical_semantic_json(&tcir_json, &mut encoded, false)?;
    Ok(format!("sha256:{}", hex_lower(&Sha256::digest(encoded))))
}

/// Append a deterministic semantic JSON representation where every object key is sorted.
///
/// TCIR carries runtime [`crate::Value`] records backed by `HashMap`; serializing
/// those maps directly would make a logical artifact's cache identity depend on
/// insertion and iteration order. Converting to JSON first lets this one
/// canonicalizer cover those records and every other map-like typed carrier.
/// Diagnostic-only anchors, AST spans, target displays, and failure notes are
/// excluded, except when one is an actual runtime-record field name.
fn append_canonical_semantic_json(
    value: &serde_json::Value,
    encoded: &mut Vec<u8>,
    inside_runtime_record: bool,
) -> Result<(), serde_json::Error> {
    match value {
        serde_json::Value::Null => encoded.extend_from_slice(b"null"),
        serde_json::Value::Bool(boolean) => {
            if *boolean {
                encoded.extend_from_slice(b"true");
            } else {
                encoded.extend_from_slice(b"false");
            }
        }
        serde_json::Value::Number(number) => {
            encoded.extend_from_slice(number.to_string().as_bytes())
        }
        serde_json::Value::String(string) => {
            encoded.extend_from_slice(serde_json::to_string(string)?.as_bytes());
        }
        serde_json::Value::Array(values) => {
            encoded.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index > 0 {
                    encoded.push(b',');
                }
                append_canonical_semantic_json(item, encoded, inside_runtime_record)?;
            }
            encoded.push(b']');
        }
        serde_json::Value::Object(fields) => {
            let mut fields = fields
                .iter()
                .filter(|(key, _)| inside_runtime_record || !is_diagnostic_tcir_field(key.as_str()))
                .collect::<Vec<_>>();
            fields.sort_unstable_by_key(|(key, _)| *key);
            encoded.push(b'{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index > 0 {
                    encoded.push(b',');
                }
                encoded.extend_from_slice(serde_json::to_string(key)?.as_bytes());
                encoded.push(b':');
                append_canonical_semantic_json(
                    value,
                    encoded,
                    inside_runtime_record
                        || (key.as_str() == "Record" && is_runtime_value_record(value)),
                )?;
            }
            encoded.push(b'}');
        }
    }
    Ok(())
}

fn is_diagnostic_tcir_field(field: &str) -> bool {
    matches!(field, "source_anchor" | "span" | "display" | "notes")
}

fn is_runtime_value_record(value: &serde_json::Value) -> bool {
    let serde_json::Value::Object(fields) = value else {
        return false;
    };
    !matches!(fields.get("fields"), Some(serde_json::Value::Array(_)))
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
    /// Existing `ash-runtime` state/context carriers reused by future routing.
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
                "runtime admission context".to_string(),
            ],
            engine_admission_reused: vec![
                "engine admission request".to_string(),
                "engine admission outcome".to_string(),
                "admitted application boundary".to_string(),
            ],
            interp_runtime_reused: vec![
                "RuntimeState".to_string(),
                "Context".to_string(),
                "CapabilityContext".to_string(),
            ],
            superseded_by_runtime_kernel: vec![
                "name-only host mode labels".to_string(),
                "provider registry as authority".to_string(),
                "file presence as application execution identity".to_string(),
            ],
        }
    }
}
