//! Runtime identity and failure carrier substrate.
//!
//! This module contains identity newtypes and inert carrier types used by the
//! process/workflow runtime semantics. It intentionally does not wire runtime
//! admission, scheduling, or `Proc` operations.

use crate::{Value, WorkflowId};
use serde::{Deserialize, Serialize};
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
    /// Evidence used to satisfy admission-time `requires` checks.
    pub requires_evidence: Vec<String>,
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
