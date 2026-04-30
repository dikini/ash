//! Shared first-class workflow carrier definitions.
//!
//! These are semantic/runtime carriers shared by parser, typechecker, and future
//! lowering/runtime layers. The public Ash type remains `Workflow<A>`; contract
//! and evidence parameters are intentionally not source-denotable type arguments.

use crate::workflow_contract::{Contract, PostPredicate, Requirement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowNodeId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceOrigin {
    SourceSpan {
        span: String,
    },
    Synthetic {
        parent_span: Option<String>,
        reason: String,
    },
    ImportedSummary {
        module: String,
        public_anchor: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectionKind {
    Proc,
    Contract,
    Check,
    AuthorityResource,
    Failure,
    Reporting,
    Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlignmentKey {
    pub node: WorkflowNodeId,
    pub projection: ProjectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowBinder {
    Ignored,
    Named(String),
    Synthetic(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowScope {
    pub name: Option<String>,
    pub origin: SourceOrigin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowForm<A> {
    Unit {
        node: WorkflowNodeId,
        value: A,
    },
    Bind {
        node: WorkflowNodeId,
        source: Box<WorkflowForm<A>>,
        binder: WorkflowBinder,
        next: Box<WorkflowForm<A>>,
    },
    FromProc {
        node: WorkflowNodeId,
        summary: ProcLowerSummary,
    },
    FromAct {
        node: WorkflowNodeId,
        summary: ActLowerSummary,
    },
    Requires {
        node: WorkflowNodeId,
        requirement: Requirement,
    },
    Ensures {
        node: WorkflowNodeId,
        postcondition: OpenPostcondition,
    },
    Scope {
        node: WorkflowNodeId,
        scope: WorkflowScope,
        body: Box<WorkflowForm<A>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionEvent {
    pub node: WorkflowNodeId,
    pub projection: ProjectionKind,
    pub origin: SourceOrigin,
    pub kind: ProjectionEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectionEventKind {
    Unit { value_erased: bool },
    Bind { binder: WorkflowBinder },
    Then,
    FromProc { summary: ProcLowerSummary },
    FromAct { summary: ActLowerSummary },
    Requires { requirement: Requirement },
    Ensures { postcondition: OpenPostcondition },
    Scope { scope: WorkflowScope },
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AdmissionEnvelope {
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractPlan<A> {
    EmptyContract {
        result_marker: Option<A>,
    },
    BindContract {
        node: WorkflowNodeId,
        first: Box<ContractPlan<A>>,
        binder: WorkflowBinder,
        second: Box<ContractPlan<A>>,
    },
    RequirementContract {
        node: WorkflowNodeId,
        requirement: Requirement,
    },
    EnsuresContract {
        node: WorkflowNodeId,
        postcondition: OpenPostcondition,
        target: PostconditionTarget,
    },
    LowerProcContract {
        node: WorkflowNodeId,
        summary: ProcContractSummary,
    },
    LowerActContract {
        node: WorkflowNodeId,
        summary: ActContractSummary,
    },
    ScopeContract {
        scope: WorkflowScope,
        plan: Box<ContractPlan<A>>,
    },
}

impl<A> Default for ContractPlan<A> {
    fn default() -> Self {
        Self::EmptyContract {
            result_marker: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkflowContract<A> {
    pub admission: AdmissionEnvelope,
    pub plan: ContractPlan<A>,
    pub legacy_contract: Contract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageError {
    MissingLowerContract {
        node: WorkflowNodeId,
    },
    UncoveredRequirement {
        node: WorkflowNodeId,
    },
    UncoveredPostcondition {
        node: WorkflowNodeId,
    },
    OpaqueSummaryRejected {
        node: WorkflowNodeId,
        imported_name: String,
    },
    MissingProjectionEvent {
        key: AlignmentKey,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CoverageEvidence {
    pub authority: Vec<AlignmentKey>,
    pub resources: Vec<AlignmentKey>,
    pub roles: Vec<AlignmentKey>,
    pub checks: Vec<AlignmentKey>,
    pub obligations: Vec<WorkflowObligation>,
    pub failure: Vec<AlignmentKey>,
    pub reporting: Vec<AlignmentKey>,
    pub provenance: Vec<AlignmentKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenPostcondition {
    pub predicate: PostPredicate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostconditionTarget {
    WorkflowResult,
    Named(String),
    DelayedWorkflowResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowObligation {
    RequirementMustHold {
        node: WorkflowNodeId,
        requirement: Requirement,
    },
    RequirementRefinementCovered {
        node: WorkflowNodeId,
        requirement: Requirement,
    },
    OpenPostconditionTarget {
        node: WorkflowNodeId,
        postcondition: OpenPostcondition,
        target_type: String,
    },
    LowerProcCovered {
        node: WorkflowNodeId,
        summary: ProcContractSummary,
    },
    LowerActCovered {
        node: WorkflowNodeId,
        summary: ActContractSummary,
    },
    RequiredCapabilityCovered {
        node: WorkflowNodeId,
        capability: String,
        mode: String,
    },
    ResourceAvailable {
        node: WorkflowNodeId,
        resource: String,
        access_mode: String,
    },
    FailureRouteDefined {
        node: WorkflowNodeId,
        failure_event_kind: String,
    },
    ProvenanceRecordable {
        node: WorkflowNodeId,
        provenance_event_kind: String,
    },
    OpaqueSummaryRejected {
        node: WorkflowNodeId,
        imported_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProcLowerSummary {
    pub coverage_obligation_nodes: Vec<WorkflowNodeId>,
    pub contract_summary: Option<ProcContractSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ActLowerSummary {
    pub coverage_obligation_nodes: Vec<WorkflowNodeId>,
    pub contract_summary: Option<ActContractSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProcContractSummary {
    pub obligations: Vec<WorkflowNodeId>,
    pub public_anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActContractSummary {
    pub obligations: Vec<WorkflowNodeId>,
    pub public_anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkflowContractSummary {
    pub contract: WorkflowContract<()>,
    pub evidence: CoverageEvidence,
    pub projection_events: Vec<ProjectionEvent>,
    pub public_anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PublicWorkflowSummary {
    pub node_count: usize,
    pub projection_events: Vec<ProjectionEvent>,
    pub coverage: CoverageEvidence,
}
