//! Compatibility adapter from legacy surface workflow headers into shared workflow carriers.
//!
//! This is intentionally conservative: header `requires`/`ensures` clauses are
//! translated into the shared `WorkflowForm` contract/projection path in legacy
//! source order, while the legacy body is represented as an opaque `FromProc`
//! summary. A future TASK-775 slice should replace the placeholder body summary
//! with a full body-to-`FromProc` adapter.

use ash_core::workflow_carrier::{
    OpenPostcondition, ProcContractSummary, ProcLowerSummary, SourceOrigin, WorkflowBinder,
    WorkflowForm, WorkflowNodeId, WorkflowScope,
};
use ash_parser::surface::{WorkflowDef, WorkflowHeaderEvent};
use ash_parser::workflow_contract_classifier::{
    ContractClassificationError, classify_postcondition, classify_requirement,
};

/// Errors produced while conservatively classifying legacy workflow header
/// contract clauses for the shared carrier adapter.
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
}

/// Translate legacy `WorkflowDef.header_events` into the shared `WorkflowForm`
/// contract path, preserving the legacy header source order.
///
/// Non-contract header events remain in `WorkflowDef` legacy fields today and are
/// skipped by this slice. The body is represented by a conservative opaque
/// `FromProc` summary anchored to `legacy_body_as_proc_summary` so callers can
/// exercise the shared projection/lowering path without claiming a full body
/// adapter exists yet.
///
/// # Errors
///
/// Returns `LegacyWorkflowAdapterError` when a legacy `requires:` or `ensures:`
/// header expression cannot be classified into the shared workflow contract
/// carrier by the conservative Phase 108 classifier slice.
pub fn legacy_workflow_def_to_workflow_form(
    workflow: &WorkflowDef,
) -> Result<WorkflowForm<()>, LegacyWorkflowAdapterError> {
    let mut next_node = 1_u64;
    let body_node = WorkflowNodeId(next_node);
    next_node += 1;

    let mut form = WorkflowForm::FromProc {
        node: body_node,
        summary: ProcLowerSummary {
            coverage_obligation_nodes: Vec::new(),
            contract_summary: Some(ProcContractSummary {
                obligations: Vec::new(),
                public_anchor: Some("legacy_body_as_proc_summary".to_string()),
            }),
        },
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

/// Build the synthetic source origin used for compatibility lowering of a
/// legacy surface workflow definition.
#[must_use]
pub fn legacy_workflow_source_origin(workflow: &WorkflowDef) -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: Some(format!("{:?}", workflow.span)),
        reason: "legacy WorkflowDef.header_events compatibility adapter".to_string(),
    }
}
