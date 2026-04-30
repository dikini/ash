//! Shared first-class workflow carrier definitions.
//!
//! These are semantic/runtime carriers shared by parser, typechecker, and future
//! lowering/runtime layers. The public Ash type remains `Workflow<A>`; contract
//! and evidence parameters are intentionally not source-denotable type arguments.

use crate::workflow_contract::{Contract, PostPredicate, Requirement};
use serde::{Deserialize, Serialize};
use std::fmt;

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
    ImportedSummary {
        node: WorkflowNodeId,
        summary: PublicWorkflowSummary,
    },
    Requires {
        node: WorkflowNodeId,
        requirement: Requirement,
    },
    Ensures {
        node: WorkflowNodeId,
        postcondition: OpenPostcondition,
    },
    Authority {
        node: WorkflowNodeId,
        authority: WorkflowAuthorityEvent,
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
    Authority { authority: WorkflowAuthorityEvent },
    Scope { scope: WorkflowScope },
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowAuthorityEvent {
    RequiredCapability(WorkflowRequiredCapability),
    OwnedResource(WorkflowOwnedResourceSummary),
    UsedBinding(WorkflowUsedBindingSummary),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowRequiredCapability {
    pub capability: String,
    pub constraints: Vec<(String, WorkflowConstraintValue)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowConstraintValue {
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<WorkflowConstraintValue>),
    Object(Vec<(String, WorkflowConstraintValue)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowOwnedResourceSummary {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowUsedBindingSummary {
    pub name: String,
    pub interface: String,
    pub implementation: String,
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

impl CoverageError {
    #[must_use]
    pub fn evidence_component(&self) -> &'static str {
        match self {
            Self::MissingProjectionEvent { key } => key.projection.evidence_component(),
            Self::MissingLowerContract { .. }
            | Self::UncoveredRequirement { .. }
            | Self::UncoveredPostcondition { .. }
            | Self::OpaqueSummaryRejected { .. } => "obligations",
        }
    }
}

impl fmt::Display for CoverageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLowerContract { node } => write!(
                f,
                "missing lower contract coverage for obligations evidence at node {}",
                node.0
            ),
            Self::UncoveredRequirement { node } => write!(
                f,
                "uncovered workflow requirement in obligations evidence at node {}",
                node.0
            ),
            Self::UncoveredPostcondition { node } => write!(
                f,
                "uncovered workflow postcondition in obligations evidence at node {}",
                node.0
            ),
            Self::OpaqueSummaryRejected {
                node,
                imported_name,
            } => write!(
                f,
                "opaque imported workflow summary `{imported_name}` rejected for obligations evidence at node {}",
                node.0
            ),
            Self::MissingProjectionEvent { key } => write!(
                f,
                "missing {} projection event for {} evidence at node {}",
                key.projection.diagnostic_label(),
                key.projection.evidence_component(),
                key.node.0
            ),
        }
    }
}

impl std::error::Error for CoverageError {}

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
    CapabilityBindingAvailable {
        node: WorkflowNodeId,
        binding: String,
        interface: String,
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

impl WorkflowObligation {
    #[must_use]
    pub fn evidence_component(&self) -> &'static str {
        "obligations"
    }

    #[must_use]
    pub fn diagnostic_label(&self) -> &'static str {
        match self {
            Self::RequirementMustHold { .. } => "workflow requirement coverage",
            Self::RequirementRefinementCovered { .. } => "requirement refinement coverage",
            Self::OpenPostconditionTarget { .. } => "open postcondition target coverage",
            Self::LowerProcCovered { .. } => "lower Proc contract coverage",
            Self::LowerActCovered { .. } => "lower Act contract coverage",
            Self::RequiredCapabilityCovered { .. } => "required capability coverage",
            Self::ResourceAvailable { .. } => "resource availability coverage",
            Self::CapabilityBindingAvailable { .. } => "capability binding coverage",
            Self::FailureRouteDefined { .. } => "failure route coverage",
            Self::ProvenanceRecordable { .. } => "provenance record coverage",
            Self::OpaqueSummaryRejected { .. } => "opaque summary rejection",
        }
    }

    #[must_use]
    pub fn diagnostic_message(&self) -> String {
        match self {
            Self::RequirementMustHold { node, .. } => format!(
                "workflow requirement must be proven by final admission obligations evidence at node {}",
                node.0
            ),
            Self::RequirementRefinementCovered { node, .. } => format!(
                "requirement assumed by requires refines checking context but is not final proof; it must be covered by obligations evidence at node {}",
                node.0
            ),
            Self::OpenPostconditionTarget {
                node, target_type, ..
            } => format!(
                "open postcondition target `{target_type}` must be tied to the successful result boundary and covered by obligations evidence at node {}",
                node.0
            ),
            Self::LowerProcCovered { node, summary } => lower_contract_message(
                "Proc",
                *node,
                summary.public_anchor.as_deref(),
                summary.obligations.len(),
            ),
            Self::LowerActCovered { node, summary } => lower_contract_message(
                "Act",
                *node,
                summary.public_anchor.as_deref(),
                summary.obligations.len(),
            ),
            Self::RequiredCapabilityCovered {
                node,
                capability,
                mode,
            } => format!(
                "required capability `{capability}` ({mode}) must be covered by obligations evidence at node {}",
                node.0
            ),
            Self::ResourceAvailable {
                node,
                resource,
                access_mode,
            } => format!(
                "resource `{resource}` with {access_mode} access must be covered by obligations evidence at node {}",
                node.0
            ),
            Self::CapabilityBindingAvailable {
                node,
                binding,
                interface,
            } => format!(
                "capability binding `{binding}` for interface `{interface}` must be covered by obligations evidence at node {}",
                node.0
            ),
            Self::FailureRouteDefined {
                node,
                failure_event_kind,
            } => format!(
                "failure route `{failure_event_kind}` must be covered by obligations evidence at node {}",
                node.0
            ),
            Self::ProvenanceRecordable {
                node,
                provenance_event_kind,
            } => format!(
                "provenance event `{provenance_event_kind}` must be recordable in obligations evidence at node {}",
                node.0
            ),
            Self::OpaqueSummaryRejected {
                node,
                imported_name,
            } => format!(
                "opaque imported workflow summary `{imported_name}` cannot satisfy obligations evidence at node {}",
                node.0
            ),
        }
    }
}

impl ProjectionKind {
    #[must_use]
    pub fn evidence_component(self) -> &'static str {
        match self {
            Self::Proc => "proc",
            Self::Contract => "obligations",
            Self::Check => "checks",
            Self::AuthorityResource => "authority/resources",
            Self::Failure => "failure",
            Self::Reporting => "reporting",
            Self::Provenance => "provenance",
        }
    }

    #[must_use]
    pub fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Proc => "Proc",
            Self::Contract => "contract",
            Self::Check => "check",
            Self::AuthorityResource => "authority/resource",
            Self::Failure => "failure",
            Self::Reporting => "reporting",
            Self::Provenance => "provenance",
        }
    }
}

fn lower_contract_message(
    kind: &str,
    node: WorkflowNodeId,
    public_anchor: Option<&str>,
    obligation_count: usize,
) -> String {
    match public_anchor {
        Some(anchor) => format!(
            "lower {kind} contract coverage for obligations evidence at node {} using public anchor `{anchor}` with {obligation_count} obligation(s)",
            node.0
        ),
        None => format!(
            "lower {kind} contract coverage for obligations evidence at node {} with {obligation_count} obligation(s)",
            node.0
        ),
    }
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
            WorkflowForm::ImportedSummary { node, summary } => {
                self.projection_events
                    .extend(summary.projection_events.iter().cloned());
                self.coverage
                    .obligations
                    .extend(summary.coverage.obligations.iter().cloned());
                WorkflowProcProjection::Neutral { node: *node }
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
            WorkflowForm::Authority { node, authority } => {
                self.event(
                    *node,
                    ProjectionEventKind::Authority {
                        authority: authority.clone(),
                    },
                );
                self.coverage.authority.push(AlignmentKey {
                    node: *node,
                    projection: ProjectionKind::AuthorityResource,
                });
                match authority {
                    WorkflowAuthorityEvent::RequiredCapability(capability) => {
                        self.coverage.obligations.push(
                            WorkflowObligation::RequiredCapabilityCovered {
                                node: *node,
                                capability: capability.capability.clone(),
                                mode: "required capability".to_string(),
                            },
                        );
                    }
                    WorkflowAuthorityEvent::OwnedResource(resource) => {
                        self.coverage.resources.push(AlignmentKey {
                            node: *node,
                            projection: ProjectionKind::AuthorityResource,
                        });
                        self.coverage
                            .obligations
                            .push(WorkflowObligation::ResourceAvailable {
                                node: *node,
                                resource: resource.name.clone(),
                                access_mode: "owned resource".to_string(),
                            });
                    }
                    WorkflowAuthorityEvent::UsedBinding(binding) => {
                        self.coverage.obligations.push(
                            WorkflowObligation::CapabilityBindingAvailable {
                                node: *node,
                                binding: binding.name.clone(),
                                interface: binding.interface.clone(),
                            },
                        );
                    }
                }
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
            WorkflowForm::ImportedSummary { .. } => ContractPlan::EmptyContract {
                result_marker: None,
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
            WorkflowForm::Authority { .. } => ContractPlan::EmptyContract {
                result_marker: None,
            },
            WorkflowForm::Scope { scope, body, .. } => ContractPlan::ScopeContract {
                scope: scope.clone(),
                plan: Box::new(self.contract_plan(body)),
            },
        }
    }
}
