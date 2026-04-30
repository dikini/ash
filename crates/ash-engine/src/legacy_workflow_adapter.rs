//! Compatibility adapter from legacy surface workflow headers into shared workflow carriers.
//!
//! This is intentionally conservative: header `requires`/`ensures` clauses are
//! translated into the shared `WorkflowForm` contract/projection path in legacy
//! source order. Supported legacy body shapes are summarized as a `FromProc`
//! lower summary with explicit coverage-obligation nodes; opaque body constructs
//! are rejected with diagnostics rather than silently treated as covered.

use ash_core::workflow_carrier::{
    OpenPostcondition, ProcContractSummary, ProcLowerSummary, SourceOrigin, WorkflowBinder,
    WorkflowForm, WorkflowNodeId, WorkflowScope,
};
use ash_parser::surface::{Workflow, WorkflowDef, WorkflowHeaderEvent};
use ash_parser::workflow_contract_classifier::{
    ContractClassificationError, classify_postcondition, classify_requirement,
};

/// Errors produced while conservatively classifying legacy workflow clauses and
/// body summaries for the shared carrier adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyWorkflowAdapterError {
    /// A legacy `requires:` header expression is not yet supported by the
    /// conservative classifier used for the shared carrier adapter.
    UnsupportedRequires {
        /// Zero-based index in `WorkflowDef.header_events`.
        header_index: usize,
        /// Classifier error that explains why the expression could not lower.
        source: ContractClassificationError,
    },
    /// A legacy `ensures:` header expression is not yet supported by the
    /// conservative classifier used for the shared carrier adapter.
    UnsupportedEnsures {
        /// Zero-based index in `WorkflowDef.header_events`.
        header_index: usize,
        /// Classifier error that explains why the expression could not lower.
        source: ContractClassificationError,
    },
    /// The legacy body contains a construct this conservative slice cannot
    /// summarize soundly yet.
    UnsupportedBody {
        /// Unsupported body construct.
        construct: UnsupportedLegacyBodyConstruct,
        /// Source span attached to the rejected construct.
        span: String,
    },
}

/// Legacy body constructs that still need fuller Proc/failure/provenance
/// summaries before they can enter `FromProc` honestly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedLegacyBodyConstruct {
    /// Stream/control receive bodies require runtime/failure summaries.
    Receive,
    /// Delegation/yield bodies require resumable provenance/failure summaries.
    Yield,
    /// Resume is only meaningful inside a yield/resumption protocol.
    Resume,
}

/// Translate legacy `WorkflowDef.header_events` into the shared `WorkflowForm`
/// contract path, preserving the legacy header source order.
///
/// Non-contract header events remain in `WorkflowDef` legacy fields today and are
/// skipped by this slice. Supported bodies are represented by a conservative
/// `FromProc` summary anchored to `legacy_body_as_proc_summary:<workflow-name>`;
/// unsupported opaque bodies reject rather than producing obligation-free
/// summaries.
///
/// # Errors
///
/// Returns `LegacyWorkflowAdapterError` when a legacy header expression cannot be
/// classified or when the body contains a construct this conservative body
/// summary slice cannot represent soundly yet.
pub fn legacy_workflow_def_to_workflow_form(
    workflow: &WorkflowDef,
) -> Result<WorkflowForm<()>, LegacyWorkflowAdapterError> {
    let mut next_node = 1_u64;
    let body_node = WorkflowNodeId(next_node);
    next_node += 1;

    let body_summary = legacy_body_as_proc_summary(workflow, body_node)?;
    let mut form = WorkflowForm::FromProc {
        node: body_node,
        summary: body_summary,
    };

    for (header_index, event) in workflow.header_events.iter().enumerate().rev() {
        let node = WorkflowNodeId(next_node);
        next_node += 1;
        let source = match event {
            WorkflowHeaderEvent::Requires { expr, .. } => WorkflowForm::Requires {
                node,
                requirement: classify_requirement(expr).map_err(|source| {
                    LegacyWorkflowAdapterError::UnsupportedRequires {
                        header_index,
                        source,
                    }
                })?,
            },
            WorkflowHeaderEvent::Ensures { expr, .. } => WorkflowForm::Ensures {
                node,
                postcondition: OpenPostcondition {
                    predicate: classify_postcondition(expr).map_err(|source| {
                        LegacyWorkflowAdapterError::UnsupportedEnsures {
                            header_index,
                            source,
                        }
                    })?,
                },
            },
            WorkflowHeaderEvent::PlaysRole(_)
            | WorkflowHeaderEvent::Capabilities(_)
            | WorkflowHeaderEvent::Owns(_)
            | WorkflowHeaderEvent::Uses(_) => continue,
        };
        form = WorkflowForm::Bind {
            node: WorkflowNodeId(next_node),
            source: Box::new(source),
            binder: WorkflowBinder::Ignored,
            next: Box::new(form),
        };
        next_node += 1;
    }

    Ok(WorkflowForm::Scope {
        node: WorkflowNodeId(next_node),
        scope: WorkflowScope {
            name: Some(workflow.name.to_string()),
            origin: legacy_workflow_source_origin(workflow),
        },
        body: Box::new(form),
    })
}

fn legacy_body_as_proc_summary(
    workflow: &WorkflowDef,
    body_node: WorkflowNodeId,
) -> Result<ProcLowerSummary, LegacyWorkflowAdapterError> {
    let mut next_node = body_node.0 + 1_000;
    let mut obligations = Vec::new();
    collect_supported_body_obligations(&workflow.body, &mut next_node, &mut obligations)?;
    Ok(ProcLowerSummary {
        coverage_obligation_nodes: obligations.clone(),
        contract_summary: Some(ProcContractSummary {
            obligations,
            public_anchor: Some(format!("legacy_body_as_proc_summary:{}", workflow.name)),
        }),
    })
}

fn collect_supported_body_obligations(
    body: &Workflow,
    next_node: &mut u64,
    obligations: &mut Vec<WorkflowNodeId>,
) -> Result<(), LegacyWorkflowAdapterError> {
    match body {
        Workflow::Done { .. } => Ok(()),
        Workflow::Observe { continuation, .. }
        | Workflow::Orient { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Check { continuation, .. }
        | Workflow::Act { continuation, .. }
        | Workflow::Let { continuation, .. }
        | Workflow::Set { continuation, .. }
        | Workflow::Send { continuation, .. } => {
            push_body_obligation(next_node, obligations);
            if let Some(continuation) = continuation {
                collect_supported_body_obligations(continuation, next_node, obligations)?;
            }
            Ok(())
        }
        Workflow::Oblige { .. } | Workflow::Ret { .. } => {
            push_body_obligation(next_node, obligations);
            Ok(())
        }
        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            push_body_obligation(next_node, obligations);
            collect_supported_body_obligations(then_branch, next_node, obligations)?;
            if let Some(else_branch) = else_branch {
                collect_supported_body_obligations(else_branch, next_node, obligations)?;
            }
            Ok(())
        }
        Workflow::For { body, .. } | Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            push_body_obligation(next_node, obligations);
            collect_supported_body_obligations(body, next_node, obligations)
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            push_body_obligation(next_node, obligations);
            collect_supported_body_obligations(primary, next_node, obligations)?;
            collect_supported_body_obligations(fallback, next_node, obligations)
        }
        Workflow::Seq { first, second, .. } => {
            collect_supported_body_obligations(first, next_node, obligations)?;
            collect_supported_body_obligations(second, next_node, obligations)
        }
        Workflow::Receive { span, .. } => Err(LegacyWorkflowAdapterError::UnsupportedBody {
            construct: UnsupportedLegacyBodyConstruct::Receive,
            span: format!("{span:?}"),
        }),
        Workflow::Yield { span, .. } => Err(LegacyWorkflowAdapterError::UnsupportedBody {
            construct: UnsupportedLegacyBodyConstruct::Yield,
            span: format!("{span:?}"),
        }),
        Workflow::Resume { span, .. } => Err(LegacyWorkflowAdapterError::UnsupportedBody {
            construct: UnsupportedLegacyBodyConstruct::Resume,
            span: format!("{span:?}"),
        }),
    }
}

fn push_body_obligation(next_node: &mut u64, obligations: &mut Vec<WorkflowNodeId>) {
    obligations.push(WorkflowNodeId(*next_node));
    *next_node += 1;
}

/// Build the synthetic source origin used for compatibility lowering of a
/// legacy surface workflow definition.
#[must_use]
pub fn legacy_workflow_source_origin(workflow: &WorkflowDef) -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: Some(format!("{:?}", workflow.span)),
        reason: "legacy WorkflowDef.header_events compatibility adapter".to_string(),
    }
}
