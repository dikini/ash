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
    pub failure_summary: Option<ProcFailureSummary>,
    pub resource_authority_summary: Option<ProcResourceAuthoritySummary>,
    pub provenance_summary: Option<ProcProvenanceSummary>,
    pub source_origin: Option<SourceOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcFailureSummary {
    pub routes: Vec<String>,
    pub conservative: bool,
}

impl Default for ProcFailureSummary {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            conservative: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcResourceAuthoritySummary {
    pub resources: Vec<String>,
    pub conservative: bool,
}

impl Default for ProcResourceAuthoritySummary {
    fn default() -> Self {
        Self {
            resources: Vec::new(),
            conservative: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcProvenanceSummary {
    pub event_kinds: Vec<String>,
    pub conservative: bool,
}

impl Default for ProcProvenanceSummary {
    fn default() -> Self {
        Self {
            event_kinds: Vec::new(),
            conservative: true,
        }
    }
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

/// Public Proc-shaped runtime projection derived from a [`WorkflowForm`].
///
/// This is intentionally owned by `ash-core` so runtime crates can consume a
/// shared carrier without depending on parser ASTs or typechecker-private typed
/// artifacts. Governance-only workflow nodes project to [`Self::Neutral`] in the
/// Proc view while remaining present in contract/evidence metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowProcProjection<A> {
    Unit {
        node: WorkflowNodeId,
        value: A,
    },
    Bind {
        node: WorkflowNodeId,
        source: Box<WorkflowProcProjection<A>>,
        binder: WorkflowBinder,
        next: Box<WorkflowProcProjection<A>>,
    },
    FromProc {
        node: WorkflowNodeId,
        summary: ProcLowerSummary,
    },
    FromAct {
        node: WorkflowNodeId,
        summary: ActLowerSummary,
    },
    Scope {
        node: WorkflowNodeId,
        scope: WorkflowScope,
        body: Box<WorkflowProcProjection<A>>,
    },
    Neutral {
        node: WorkflowNodeId,
    },
}

/// Shared lowering result for first-class Workflow values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoweredWorkflowProjection<A> {
    pub proc_projection: WorkflowProcProjection<A>,
    pub contract: WorkflowContract<A>,
    pub coverage: CoverageEvidence,
    pub projection_events: Vec<ProjectionEvent>,
}

/// Lower a shared [`WorkflowForm`] carrier into public runtime/projection
/// metadata.
#[must_use]
pub fn lower_workflow_form<A: Clone>(
    form: &WorkflowForm<A>,
    origin: SourceOrigin,
) -> LoweredWorkflowProjection<A> {
    let mut lowering = WorkflowFormLowering::new(origin);
    let proc_projection = lowering.proc_projection(form);
    let plan = lowering.contract_plan(form);
    LoweredWorkflowProjection {
        proc_projection,
        contract: WorkflowContract {
            admission: AdmissionEnvelope {
                requirements: lowering.requirements,
            },
            plan,
            legacy_contract: Contract::default(),
        },
        coverage: lowering.coverage,
        projection_events: lowering.projection_events,
    }
}

struct WorkflowFormLowering {
    origin: SourceOrigin,
    projection_events: Vec<ProjectionEvent>,
    coverage: CoverageEvidence,
    requirements: Vec<Requirement>,
}

impl WorkflowFormLowering {
    fn new(origin: SourceOrigin) -> Self {
        Self {
            origin,
            projection_events: Vec::new(),
            coverage: CoverageEvidence::default(),
            requirements: Vec::new(),
        }
    }

    fn event(&mut self, node: WorkflowNodeId, kind: ProjectionEventKind) {
        self.projection_events.push(ProjectionEvent {
            node,
            projection: ProjectionKind::Proc,
            origin: self.origin.clone(),
            kind,
        });
    }

    fn proc_projection<A: Clone>(&mut self, form: &WorkflowForm<A>) -> WorkflowProcProjection<A> {
        match form {
            WorkflowForm::Unit { node, value } => {
                self.event(
                    *node,
                    ProjectionEventKind::Unit {
                        value_erased: false,
                    },
                );
                WorkflowProcProjection::Unit {
                    node: *node,
                    value: value.clone(),
                }
            }
            WorkflowForm::Bind {
                node,
                source,
                binder,
                next,
            } => {
                let source = Box::new(self.proc_projection(source));
                let kind = if *binder == WorkflowBinder::Ignored {
                    ProjectionEventKind::Then
                } else {
                    ProjectionEventKind::Bind {
                        binder: binder.clone(),
                    }
                };
                self.event(*node, kind);
                let next = Box::new(self.proc_projection(next));
                WorkflowProcProjection::Bind {
                    node: *node,
                    source,
                    binder: binder.clone(),
                    next,
                }
            }
            WorkflowForm::FromProc { node, summary } => {
                self.event(
                    *node,
                    ProjectionEventKind::FromProc {
                        summary: summary.clone(),
                    },
                );
                self.coverage
                    .obligations
                    .push(WorkflowObligation::LowerProcCovered {
                        node: *node,
                        summary: summary.contract_summary.clone().unwrap_or_default(),
                    });
                WorkflowProcProjection::FromProc {
                    node: *node,
                    summary: summary.clone(),
                }
            }
            WorkflowForm::FromAct { node, summary } => {
                self.event(
                    *node,
                    ProjectionEventKind::FromAct {
                        summary: summary.clone(),
                    },
                );
                self.coverage
                    .obligations
                    .push(WorkflowObligation::LowerActCovered {
                        node: *node,
                        summary: summary.contract_summary.clone().unwrap_or_default(),
                    });
                WorkflowProcProjection::FromAct {
                    node: *node,
                    summary: summary.clone(),
                }
            }
            WorkflowForm::Requires { node, requirement } => {
                self.event(
                    *node,
                    ProjectionEventKind::Requires {
                        requirement: requirement.clone(),
                    },
                );
                self.requirements.push(requirement.clone());
                self.coverage
                    .obligations
                    .push(WorkflowObligation::RequirementMustHold {
                        node: *node,
                        requirement: requirement.clone(),
                    });
                WorkflowProcProjection::Neutral { node: *node }
            }
            WorkflowForm::Ensures {
                node,
                postcondition,
            } => {
                self.event(
                    *node,
                    ProjectionEventKind::Ensures {
                        postcondition: postcondition.clone(),
                    },
                );
                self.coverage
                    .obligations
                    .push(WorkflowObligation::OpenPostconditionTarget {
                        node: *node,
                        postcondition: postcondition.clone(),
                        target_type: "WorkflowResult".to_string(),
                    });
                WorkflowProcProjection::Neutral { node: *node }
            }
            WorkflowForm::Scope { node, scope, body } => {
                self.event(
                    *node,
                    ProjectionEventKind::Scope {
                        scope: scope.clone(),
                    },
                );
                WorkflowProcProjection::Scope {
                    node: *node,
                    scope: scope.clone(),
                    body: Box::new(self.proc_projection(body)),
                }
            }
        }
    }

    fn contract_plan<A: Clone>(&self, form: &WorkflowForm<A>) -> ContractPlan<A> {
        match form {
            WorkflowForm::Unit { value, .. } => ContractPlan::EmptyContract {
                result_marker: Some(value.clone()),
            },
            WorkflowForm::Bind {
                node,
                source,
                binder,
                next,
            } => ContractPlan::BindContract {
                node: *node,
                first: Box::new(self.contract_plan(source)),
                binder: binder.clone(),
                second: Box::new(self.contract_plan(next)),
            },
            WorkflowForm::FromProc { node, summary } => ContractPlan::LowerProcContract {
                node: *node,
                summary: summary.contract_summary.clone().unwrap_or_default(),
            },
            WorkflowForm::FromAct { node, summary } => ContractPlan::LowerActContract {
                node: *node,
                summary: summary.contract_summary.clone().unwrap_or_default(),
            },
            WorkflowForm::Requires { node, requirement } => ContractPlan::RequirementContract {
                node: *node,
                requirement: requirement.clone(),
            },
            WorkflowForm::Ensures {
                node,
                postcondition,
            } => ContractPlan::EnsuresContract {
                node: *node,
                postcondition: postcondition.clone(),
                target: PostconditionTarget::WorkflowResult,
            },
            WorkflowForm::Scope { scope, body, .. } => ContractPlan::ScopeContract {
                scope: scope.clone(),
                plan: Box::new(self.contract_plan(body)),
            },
        }
    }
}
