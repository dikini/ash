//! Explicit row admission requirement derivation and checking.
//!
//! This module implements a derived, metadata-only admission view for explicit
//! callable row requirements. It does not register providers, resources, roles,
//! policies, handlers, or runtime authority. Its purpose is to make Phase 178
//! row metadata visible to existing admission checks so that missing authority
//! rejects with a precise diagnostic.
//!
//! # Design
//!
//! - [`RowAdmissionRequirement`] is a normalized admission-side view of one row
//!   item (operation, resource, role, policy, etc.).
//! - [`RowAdmissionRequirement::from_core_row`] derives requirements from a
//!   [`CoreRow`] without mutating any runtime state.
//! - [`Engine::admit_workflow_with_explicit_rows`] checks those requirements
//!   against already-registered authority, then delegates to the existing
//!   [`Engine::admit_workflow`] path.
//!
//! Rows alone do not grant authority. A satisfied operation row means the host
//! already registered a matching provider; a satisfied resource row means the
//! host already selected a matching resource initializer; a satisfied role row
//! means the admission request already carries the matching admitted role.

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_core::runtime::{
    WorkflowAdmissionContext, WorkflowFailure, WorkflowFailureEvidence, WorkflowFailureKind,
    WorkflowReport,
};

use crate::{
    Engine, WorkflowAdmissionOutcome, WorkflowAdmissionRequest, admitted_role_name,
    build_pending_ensures_evidence,
};

/// A row item interpreted as an admission-side requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowAdmissionRequirement {
    /// Operation requirement such as `posixfs.read` or `PosixFs::read`.
    Operation {
        /// Runtime authority identity required to discharge the operation.
        authority: String,
        /// Operation name (e.g. `read`).
        operation: String,
    },
    /// Resource requirement such as `resource vault write`.
    Resource {
        /// Resource type/name (e.g. `vault`).
        resource: String,
        /// Access mode (e.g. `read`, `write`, `use`).
        mode: String,
    },
    /// Role requirement such as `role tenant.admin`.
    Role {
        /// Role name path (e.g. `tenant.admin`).
        role: String,
    },
    /// Policy requirement such as `policy pii.redact`.
    Policy {
        /// Policy name path (e.g. `pii.redact`).
        policy: String,
    },
    /// Process requirement such as `process spawn`.
    Process {
        /// Process operation name (e.g. `spawn`).
        operation: String,
    },
    /// Failure row requirement.
    Failure {
        /// Optional failure type name.
        ty: Option<String>,
    },
    /// Evidence requirement.
    Evidence {
        /// Evidence name path.
        evidence: String,
    },
    /// Effect group reference.
    EffectGroup {
        /// Group name path.
        group: String,
    },
    /// Requirement family not yet supported by the admission substrate.
    Unsupported {
        /// Requirement family name for diagnostics.
        family: &'static str,
        /// Human-readable description.
        description: String,
    },
}

/// Admission-side discharge family for one explicit row requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowAdmissionDischarge {
    /// Operation rows discharge through existing registered operation authority.
    OperationAuthority {
        /// Human-readable operation identity, such as `PosixFs::read`.
        identity: String,
        /// Runtime authority identity required by the current admission substrate.
        authority: String,
        /// Operation method name.
        operation: String,
    },
    /// Resource rows discharge through selected resource authority.
    ResourceAuthority {
        /// Resource type/name.
        resource: String,
        /// Required access mode.
        mode: String,
    },
    /// Role rows discharge through admitted role authority.
    RoleAuthority {
        /// Required role path.
        role: String,
    },
    /// Policy rows discharge through policy evidence/admission.
    PolicyEvidence {
        /// Required policy path.
        policy: String,
    },
    /// Failure rows discharge through a failure handler.
    FailureHandler {
        /// Optional failure type name.
        ty: Option<String>,
    },
    /// Evidence rows discharge through named evidence.
    Evidence {
        /// Evidence path.
        evidence: String,
    },
    /// Effect group rows require group expansion before concrete discharge.
    EffectGroup {
        /// Group path.
        group: String,
    },
    /// Unsupported row families fail closed with an explicit family.
    Unsupported {
        /// Requirement family name.
        family: &'static str,
        /// Human-readable description.
        description: String,
    },
}

impl RowAdmissionRequirement {
    /// Derive admission requirements from a Core row.
    #[must_use]
    pub fn from_core_row(row: &CoreRow) -> Vec<Self> {
        row.items.iter().map(Self::from_core_row_item).collect()
    }

    fn from_core_row_item(item: &CoreRowItem) -> Self {
        match item {
            CoreRowItem::Capability { path, operation } => Self::Operation {
                authority: path.join("."),
                operation: operation.clone(),
            },
            CoreRowItem::Resource { path, mode } => Self::Resource {
                resource: path.join("."),
                mode: mode.clone(),
            },
            CoreRowItem::Role { path } => Self::Role {
                role: path.join("."),
            },
            CoreRowItem::Policy { path } => Self::Policy {
                policy: path.join("."),
            },
            CoreRowItem::Process { operation } => Self::Process {
                operation: operation.clone(),
            },
            CoreRowItem::Failure { ty } => Self::Failure {
                ty: ty.as_deref().map(format_core_type_name),
            },
            CoreRowItem::Evidence { path } => Self::Evidence {
                evidence: path.join("."),
            },
            CoreRowItem::EffectGroupRef { path } => Self::EffectGroup {
                group: path.join("."),
            },
            CoreRowItem::Contract { contract } => Self::Unsupported {
                family: "contract",
                description: format!(
                    "contract row item '{contract}' is not supported by admission"
                ),
            },
            CoreRowItem::Channel { path, mode, .. } => Self::Unsupported {
                family: "channel",
                description: format!(
                    "channel row item '{}.{mode}' is not supported by admission",
                    path.join(".")
                ),
            },
        }
    }

    /// Return a short diagnostic label describing the requirement.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Operation {
                authority,
                operation,
            } => format!(
                "operation {}",
                format_operation_identity(authority, operation)
            ),
            Self::Resource { resource, mode } => format!("resource {resource} {mode}"),
            Self::Role { role } => format!("role {role}"),
            Self::Policy { policy } => format!("policy {policy}"),
            Self::Process { operation } => format!("process {operation}"),
            Self::Failure { ty } => format!("fail {}", ty.as_deref().unwrap_or("<unknown>")),
            Self::Evidence { evidence } => format!("evidence {evidence}"),
            Self::EffectGroup { group } => format!("group {group}"),
            Self::Unsupported {
                family,
                description,
            } => format!("{family}: {description}"),
        }
    }

    /// Return the admission discharge family for this row requirement.
    #[must_use]
    pub fn discharge(&self) -> RowAdmissionDischarge {
        match self {
            Self::Operation {
                authority,
                operation,
            } => RowAdmissionDischarge::OperationAuthority {
                identity: format_operation_identity(authority, operation),
                authority: authority.clone(),
                operation: operation.clone(),
            },
            Self::Resource { resource, mode } => RowAdmissionDischarge::ResourceAuthority {
                resource: resource.clone(),
                mode: mode.clone(),
            },
            Self::Role { role } => RowAdmissionDischarge::RoleAuthority { role: role.clone() },
            Self::Policy { policy } => RowAdmissionDischarge::PolicyEvidence {
                policy: policy.clone(),
            },
            Self::Process { operation } => RowAdmissionDischarge::Unsupported {
                family: "process",
                description: format!(
                    "process row '{operation}' requires process runtime discharge"
                ),
            },
            Self::Failure { ty } => RowAdmissionDischarge::FailureHandler { ty: ty.clone() },
            Self::Evidence { evidence } => RowAdmissionDischarge::Evidence {
                evidence: evidence.clone(),
            },
            Self::EffectGroup { group } => RowAdmissionDischarge::EffectGroup {
                group: group.clone(),
            },
            Self::Unsupported {
                family,
                description,
            } => RowAdmissionDischarge::Unsupported {
                family,
                description: description.clone(),
            },
        }
    }
}

fn format_operation_identity(authority: &str, operation: &str) -> String {
    let separator = authority
        .rsplit('.')
        .next()
        .and_then(|last| last.chars().next())
        .filter(|first| first.is_uppercase())
        .map_or(".", |_| "::");
    format!("{authority}{separator}{operation}")
}

fn format_core_type_name(ty: &CoreType) -> String {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => name.clone(),
        CoreType::Function { .. } => "fn".to_string(),
        CoreType::Refinement { base, predicate } => {
            format!("{} where {predicate}", format_core_type_name(base))
        }
        CoreType::Cont { .. } => "cont".to_string(),
        CoreType::Tuple(items) => {
            let names: Vec<_> = items.iter().map(format_core_type_name).collect();
            format!("({})", names.join(", "))
        }
        CoreType::Record(fields) => {
            let fields: Vec<_> = fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", format_core_type_name(ty)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        CoreType::App { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                let args: Vec<_> = args.iter().map(format_core_type_name).collect();
                format!("{name}<{}>", args.join(", "))
            }
        }
        CoreType::Mode { mode, inner, .. } => {
            format!("{mode:?}<{}>", format_core_type_name(inner))
        }
    }
}

/// Outcome of checking a single row requirement against existing authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowAdmissionCheck {
    /// Requirement is satisfied by existing authority.
    Satisfied,
    /// Requirement is missing and should reject admission.
    Missing {
        /// Failure kind to surface at the workflow boundary.
        kind: WorkflowFailureKind,
        /// Diagnostic notes.
        notes: Vec<String>,
    },
    /// Requirement is not supported by the current admission substrate.
    Unsupported {
        /// Diagnostic notes.
        notes: Vec<String>,
    },
}

impl RowAdmissionCheck {
    /// Check one row requirement against the engine's existing authority.
    pub fn check(
        engine: &Engine,
        request: &WorkflowAdmissionRequest,
        requirement: &RowAdmissionRequirement,
    ) -> Self {
        match requirement {
            RowAdmissionRequirement::Operation {
                authority,
                operation,
            } => {
                let identity = format_operation_identity(authority, operation);
                if engine.has_provider(authority) {
                    Self::Satisfied
                } else {
                    Self::Missing {
                        kind: WorkflowFailureKind::CapabilityAdmissionFailure,
                        notes: vec![format!(
                            "operation authority for '{identity}' requires registered authority '{authority}'. Rows do not grant authority"
                        )],
                    }
                }
            }
            RowAdmissionRequirement::Resource { resource, mode } => {
                if engine.resource_initializer_selection(resource).is_some() {
                    Self::Satisfied
                } else {
                    Self::Missing {
                        kind: WorkflowFailureKind::CapabilityAdmissionFailure,
                        notes: vec![format!(
                            "resource row '{resource} {mode}' requires resource initializer '{resource}', which is not selected"
                        )],
                    }
                }
            }
            RowAdmissionRequirement::Role { role } => {
                if admitted_role_name(request) == Some(role.as_str()) {
                    Self::Satisfied
                } else {
                    Self::Missing {
                        kind: WorkflowFailureKind::RoleAdmissionFailure,
                        notes: vec![format!("role row '{role}' requires admitted role '{role}'")],
                    }
                }
            }
            RowAdmissionRequirement::Policy { policy } => Self::Unsupported {
                notes: vec![format!(
                    "policy row '{policy}' requires policy evidence discharge, which is not supported by the admission substrate"
                )],
            },
            RowAdmissionRequirement::Process { operation } => Self::Unsupported {
                notes: vec![format!(
                    "process row '{operation}' is not supported by the admission substrate"
                )],
            },
            RowAdmissionRequirement::Failure { ty } => Self::Unsupported {
                notes: vec![format!(
                    "failure row '{}' requires failure handler discharge, which is not supported by the admission substrate",
                    ty.as_deref().unwrap_or("<unknown>")
                )],
            },
            RowAdmissionRequirement::Evidence { evidence } => Self::Unsupported {
                notes: vec![format!(
                    "evidence row '{evidence}' requires evidence discharge, which is not supported by the admission substrate"
                )],
            },
            RowAdmissionRequirement::EffectGroup { group } => Self::Unsupported {
                notes: vec![format!(
                    "group row '{group}' is not supported by the admission substrate"
                )],
            },
            RowAdmissionRequirement::Unsupported {
                family,
                description,
            } => Self::Unsupported {
                notes: vec![format!("unsupported {family} row: {description}")],
            },
        }
    }
}

impl Engine {
    /// Admit a workflow after checking explicit row requirements from its metadata.
    ///
    /// This method derives admission requirements from `workflow.core_callable_types`
    /// and checks them against already-registered authority. If all requirements are
    /// satisfied (or if the workflow carries no explicit rows), it delegates to
    /// [`Engine::admit_workflow`] with the supplied request.
    ///
    /// # Important
    ///
    /// Row requirements are metadata only. They do not register providers, select
    /// resources, admit roles, or grant policies. Missing authority always rejects
    /// with a structured diagnostic.
    ///
    /// # Errors
    ///
    /// Returns a [`WorkflowAdmissionOutcome::Rejected`] if any explicit row
    /// requirement is missing or unsupported.
    pub async fn admit_workflow_with_explicit_rows(
        &self,
        request: WorkflowAdmissionRequest,
        workflow: &crate::Workflow,
    ) -> WorkflowAdmissionOutcome {
        let mut row_requirements: Vec<(String, RowAdmissionRequirement)> = Vec::new();
        for (name, core_type) in &workflow.core_callable_types {
            if let ash_core::core_ash::CoreType::Function { row, .. } = core_type {
                for requirement in RowAdmissionRequirement::from_core_row(row) {
                    row_requirements.push((name.clone(), requirement));
                }
            }
        }

        for (callable_name, requirement) in row_requirements {
            match RowAdmissionCheck::check(self, &request, &requirement) {
                RowAdmissionCheck::Satisfied => {}
                RowAdmissionCheck::Missing { kind, notes } => {
                    return self
                        .reject_row_requirement(&request, kind, &callable_name, &requirement, notes)
                        .await;
                }
                RowAdmissionCheck::Unsupported { notes } => {
                    return self
                        .reject_row_requirement(
                            &request,
                            WorkflowFailureKind::RequiresViolation,
                            &callable_name,
                            &requirement,
                            notes,
                        )
                        .await;
                }
            }
        }

        // All explicit row requirements are satisfied or absent; delegate to the
        // existing admission path.
        self.admit_workflow(request).await
    }

    async fn reject_row_requirement(
        &self,
        request: &WorkflowAdmissionRequest,
        kind: WorkflowFailureKind,
        callable_name: &str,
        requirement: &RowAdmissionRequirement,
        notes: Vec<String>,
    ) -> WorkflowAdmissionOutcome {
        let workflow_id = request.workflow_id.unwrap_or_default();
        let run_id = request.run_id.unwrap_or_default();
        let admitted_capability_bindings = self
            .runtime_state
            .resolve_admitted_capability_bindings(&request.required_capabilities)
            .await;
        let admission = WorkflowAdmissionContext {
            active_role: admitted_role_name(request).map(ToOwned::to_owned),
            admitted_capabilities: request.required_capabilities.clone(),
            admitted_capability_bindings,
            requires_evidence: Vec::new(),
        };
        let ensures_evidence = build_pending_ensures_evidence(&request.ensures);
        let failure = WorkflowFailure::new(workflow_id, run_id, kind, None).with_evidence(
            WorkflowFailureEvidence {
                notes: vec![format!(
                    "callable '{callable_name}' row requirement '{}' failed: {}",
                    requirement.label(),
                    notes.join("; ")
                )],
                provenance: vec![format!(
                    "explicit_row_requirement={callable_name}::{}",
                    requirement.label()
                )],
            },
        );
        let report = WorkflowReport::failed(workflow_id, run_id, failure.clone())
            .with_admission_context(admission)
            .with_requires_evidence(Vec::new())
            .with_ensures_evidence(ensures_evidence);
        WorkflowAdmissionOutcome::Rejected { failure, report }
    }
}
