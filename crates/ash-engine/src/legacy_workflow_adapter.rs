//! Compatibility adapter from legacy surface workflow headers into shared workflow carriers.
//!
//! This is intentionally conservative: role, authority/resource, and contract
//! headers are translated into the shared `WorkflowForm` contract/projection path
//! in legacy source order. Supported legacy body shapes are summarized as a
//! `FromProc` lower summary with explicit coverage-obligation nodes; opaque body
//! constructs are rejected with diagnostics rather than silently treated as covered.

use ash_core::workflow_carrier::{
    OpenPostcondition, ProcContractSummary, ProcFailureSummary, ProcLowerSummary,
    ProcProvenanceSummary, ProcResourceAuthoritySummary, SourceOrigin, WorkflowAuthorityEvent,
    WorkflowBinder, WorkflowConstraintValue, WorkflowForm, WorkflowNodeId,
    WorkflowOwnedResourceSummary, WorkflowRequiredCapability, WorkflowScope,
    WorkflowUsedBindingSummary,
};
use ash_parser::surface::{
    CapabilityDecl, ConstraintValue, ConstructorPayload, Expr, Type, Workflow, WorkflowDef,
    WorkflowHeaderEvent, WorkflowOwnedResource, WorkflowUsedBinding,
};
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
/// Authority/resource and contract header events enter `WorkflowForm` in source
/// order. Supported bodies are represented by a conservative
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
        let sources = if let WorkflowHeaderEvent::Capabilities(capabilities) = event {
            capabilities
                .iter()
                .rev()
                .map(|capability| {
                    let node = WorkflowNodeId(next_node);
                    next_node += 1;
                    WorkflowForm::Authority {
                        node,
                        authority: WorkflowAuthorityEvent::RequiredCapability(
                            required_capability_summary(capability),
                        ),
                    }
                })
                .collect::<Vec<_>>()
        } else {
            let node = WorkflowNodeId(next_node);
            next_node += 1;
            vec![match event {
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
                WorkflowHeaderEvent::PlaysRole(role) => WorkflowForm::Requires {
                    node,
                    requirement: ash_core::workflow_contract::Requirement::HasRole(
                        role.name.to_string(),
                    ),
                },
                WorkflowHeaderEvent::Owns(resource) => WorkflowForm::Authority {
                    node,
                    authority: WorkflowAuthorityEvent::OwnedResource(owned_resource_summary(
                        resource,
                    )),
                },
                WorkflowHeaderEvent::Uses(binding) => WorkflowForm::Authority {
                    node,
                    authority: WorkflowAuthorityEvent::UsedBinding(used_binding_summary(binding)),
                },
                WorkflowHeaderEvent::Capabilities(_) => {
                    unreachable!("capabilities handled before single-event source construction")
                }
            }]
        };
        for source in sources {
            form = WorkflowForm::Bind {
                node: WorkflowNodeId(next_node),
                source: Box::new(source),
                binder: WorkflowBinder::Ignored,
                next: Box::new(form),
            };
            next_node += 1;
        }
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

fn required_capability_summary(capability: &CapabilityDecl) -> WorkflowRequiredCapability {
    WorkflowRequiredCapability {
        capability: capability.capability.to_string(),
        constraints: capability
            .constraints
            .as_ref()
            .map(|constraints| {
                constraints
                    .fields
                    .iter()
                    .map(|field| {
                        (
                            field.name.to_string(),
                            workflow_constraint_value(&field.value),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn owned_resource_summary(resource: &WorkflowOwnedResource) -> WorkflowOwnedResourceSummary {
    WorkflowOwnedResourceSummary {
        name: resource.name.to_string(),
        ty: type_summary(&resource.ty),
    }
}

fn used_binding_summary(binding: &WorkflowUsedBinding) -> WorkflowUsedBindingSummary {
    WorkflowUsedBindingSummary {
        name: binding.name.to_string(),
        interface: type_summary(&binding.interface),
        implementation: expr_summary(&binding.implementation),
    }
}

fn type_summary(ty: &Type) -> String {
    match ty {
        Type::Name(name) | Type::Capability(name) => name.to_string(),
        Type::List(inner) => format!("[{}]", type_summary(inner)),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(type_summary)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, type_summary(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter().map(type_summary).collect::<Vec<_>>().join(", ")
        ),
        Type::Associated { base, name } => format!("{}::{}", type_summary(base), name),
        Type::Fn(args, ret) => format!(
            "Fn({}) -> {}",
            args.iter().map(type_summary).collect::<Vec<_>>().join(", "),
            type_summary(ret)
        ),
    }
}

fn expr_summary(expr: &Expr) -> String {
    match expr {
        Expr::Variable { name, .. } => name.to_string(),
        Expr::Call {
            func, module, args, ..
        } => {
            let callee = module
                .as_ref()
                .map_or_else(|| func.to_string(), |module| format!("{module}::{func}"));
            format!(
                "{}({})",
                callee,
                args.iter().map(expr_summary).collect::<Vec<_>>().join(", ")
            )
        }
        Expr::Constructor { name, payload, .. } => match payload {
            ConstructorPayload::Unit => name.to_string(),
            ConstructorPayload::Tuple(items) => format!(
                "{}({})",
                name,
                items
                    .iter()
                    .map(expr_summary)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ConstructorPayload::Record(fields) => format!(
                "{} {{ {} }}",
                name,
                fields
                    .iter()
                    .map(|(name, value)| format!("{}: {}", name, expr_summary(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        other => format!("{other:?}"),
    }
}

fn workflow_constraint_value(value: &ConstraintValue) -> WorkflowConstraintValue {
    match value {
        ConstraintValue::Bool(value) => WorkflowConstraintValue::Bool(*value),
        ConstraintValue::Int(value) => WorkflowConstraintValue::Int(*value),
        ConstraintValue::String(value) => WorkflowConstraintValue::String(value.clone()),
        ConstraintValue::Array(values) => {
            WorkflowConstraintValue::Array(values.iter().map(workflow_constraint_value).collect())
        }
        ConstraintValue::Object(fields) => WorkflowConstraintValue::Object(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), workflow_constraint_value(value)))
                .collect(),
        ),
    }
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
        failure_summary: Some(ProcFailureSummary {
            routes: Vec::new(),
            conservative: true,
        }),
        resource_authority_summary: Some(ProcResourceAuthoritySummary {
            resources: Vec::new(),
            conservative: true,
        }),
        provenance_summary: Some(ProcProvenanceSummary {
            event_kinds: Vec::new(),
            conservative: true,
        }),
        source_origin: Some(legacy_workflow_source_origin(workflow)),
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
