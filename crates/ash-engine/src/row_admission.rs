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
//! - [`Engine::admit_application_with_explicit_rows`] checks those requirements
//!   against already-registered authority, then delegates to the existing
//!   [`Engine::admit_application`] path.
//!
//! Rows alone do not grant authority. A satisfied operation row means the host
//! already registered a matching provider; a satisfied resource row means the
//! host already selected a matching resource initializer; a satisfied role row
//! means the admission request already carries the matching admitted role.

use ash_core::core_ash::{CoreName, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_contract::ContractDischargeRecord;
use ash_core::runtime::{
    ApplicationAdmissionContext, ApplicationFailure, ApplicationFailureEvidence,
    ApplicationFailureKind, ApplicationReport,
};

use crate::{
    ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine, admitted_role_name,
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
        /// Evidence family (e.g. `test`, `law`, `proof`, `monitor`, `observation`).
        family: CoreName,
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
        /// Evidence family (e.g. `test`, `law`, `proof`, `monitor`, `observation`).
        family: CoreName,
        /// Evidence path.
        evidence: String,
    },
    /// Contract discharge rows require static, evidence, or dynamic discharge.
    Contract {
        /// Contract discharge record.
        discharge: ContractDischargeRecord,
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

/// One operation-discharge frame supplied by an admission/runtime environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationAdmissionFrame {
    /// Impl/type-qualified operation identity, such as `PosixFs::read`.
    pub identity: String,
    /// Frame kind used to discharge the operation.
    pub kind: OperationAdmissionFrameKind,
}

/// Operation-discharge frame kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationAdmissionFrameKind {
    /// Program handler frame.
    Handler {
        /// Handler identity for diagnostics/evidence.
        handler: String,
    },
    /// Provider authority frame.
    Provider {
        /// Provider identity for diagnostics/evidence.
        provider: String,
    },
}

/// Evidence proving an operation row requirement is discharged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowAdmissionProof {
    /// Discharged by a handler frame.
    OperationHandlerFrame {
        /// Impl/type-qualified operation identity.
        identity: String,
        /// Handler identity.
        handler: String,
    },
    /// Discharged by a provider frame.
    OperationProviderFrame {
        /// Impl/type-qualified operation identity.
        identity: String,
        /// Provider identity.
        provider: String,
    },
}

/// Admission/runtime evidence available when checking explicit row requirements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowAdmissionEnvironment {
    operation_frames: Vec<OperationAdmissionFrame>,
}

impl RowAdmissionEnvironment {
    /// Create an empty admission environment.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an operation frame, returning the updated environment.
    #[must_use]
    pub fn with_operation_frame(mut self, frame: OperationAdmissionFrame) -> Self {
        self.push_operation_frame(frame);
        self
    }

    /// Push an operation frame onto the innermost end of the frame stack.
    pub fn push_operation_frame(&mut self, frame: OperationAdmissionFrame) {
        self.operation_frames.push(frame);
    }

    /// Return operation-discharge proof using innermost-to-outermost frame order.
    #[must_use]
    pub fn prove_operation(
        &self,
        requirement: &RowAdmissionRequirement,
    ) -> Option<RowAdmissionProof> {
        let RowAdmissionRequirement::Operation {
            authority,
            operation,
        } = requirement
        else {
            return None;
        };
        let identity = format_operation_identity(authority, operation);
        self.operation_frames
            .iter()
            .rev()
            .find(|frame| frame.identity == identity)
            .map(|frame| match &frame.kind {
                OperationAdmissionFrameKind::Handler { handler } => {
                    RowAdmissionProof::OperationHandlerFrame {
                        identity,
                        handler: handler.clone(),
                    }
                }
                OperationAdmissionFrameKind::Provider { provider } => {
                    RowAdmissionProof::OperationProviderFrame {
                        identity,
                        provider: provider.clone(),
                    }
                }
            })
    }
}

impl RowAdmissionRequirement {
    /// Derive admission requirements from a Core row.
    #[must_use]
    pub fn from_core_row(row: &CoreRow) -> Vec<Self> {
        row.items.iter().map(Self::from_core_row_item).collect()
    }

    fn from_core_row_item(item: &CoreRowItem) -> Self {
        match item {
            CoreRowItem::Operation { path, operation } => Self::Operation {
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
            CoreRowItem::Evidence { path } => {
                let family = path.first().map_or("unknown", CoreName::as_str);
                Self::Evidence {
                    family: family.into(),
                    evidence: path
                        .iter()
                        .map(CoreName::as_str)
                        .collect::<Vec<_>>()
                        .join("."),
                }
            }
            CoreRowItem::EffectGroupRef { path } => Self::EffectGroup {
                group: path.join("."),
            },
            CoreRowItem::Contract { contract } => Self::Unsupported {
                family: "contract",
                description: format!(
                    "contract row item '{contract}' requires a contract-discharge record"
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
            Self::Evidence { family, evidence } => format!("evidence {family}:{evidence}"),
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
            Self::Evidence { family, evidence } => RowAdmissionDischarge::Evidence {
                family: family.clone(),
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
        /// Failure kind to surface at the application boundary.
        kind: ApplicationFailureKind,
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
        request: &ApplicationAdmissionRequest,
        requirement: &RowAdmissionRequirement,
    ) -> Self {
        Self::check_with_environment(
            engine,
            request,
            requirement,
            &RowAdmissionEnvironment::new(),
        )
    }

    /// Check one row requirement against explicit admission/runtime evidence.
    pub fn check_with_environment(
        engine: &Engine,
        request: &ApplicationAdmissionRequest,
        requirement: &RowAdmissionRequirement,
        environment: &RowAdmissionEnvironment,
    ) -> Self {
        match requirement {
            RowAdmissionRequirement::Operation {
                authority,
                operation,
            } => {
                let identity = format_operation_identity(authority, operation);
                if environment.prove_operation(requirement).is_some()
                    || engine.has_provider(authority)
                {
                    Self::Satisfied
                } else {
                    Self::Missing {
                        kind: ApplicationFailureKind::CapabilityAdmissionFailure,
                        notes: vec![format!(
                            "operation authority for '{identity}' requires a handler/provider frame or registered authority '{authority}'. Rows do not grant authority"
                        )],
                    }
                }
            }
            RowAdmissionRequirement::Resource { resource, mode } => {
                if engine.resource_initializer_selection(resource).is_some() {
                    Self::Satisfied
                } else {
                    Self::Missing {
                        kind: ApplicationFailureKind::CapabilityAdmissionFailure,
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
                        kind: ApplicationFailureKind::RoleAdmissionFailure,
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
            RowAdmissionRequirement::Evidence { family, evidence } => {
                const VALID_FAMILIES: &[&str] = &["test", "law", "proof", "monitor", "observation"];
                if !VALID_FAMILIES.contains(&family.as_str()) {
                    return Self::Missing {
                        kind: ApplicationFailureKind::RequiresViolation,
                        notes: vec![format!(
                            "evidence row '{evidence}' has invalid family '{family}'; expected test, law, proof, monitor, or observation"
                        )],
                    };
                }
                // Evidence rows are requirements/records, not authority.
                // Without a valid evidence record in the admission request, fail closed.
                Self::Missing {
                    kind: ApplicationFailureKind::RequiresViolation,
                    notes: vec![format!(
                        "evidence row '{family}:{evidence}' requires a valid evidence record and an explicit strategy allowing evidence discharge; rows do not grant authority"
                    )],
                }
            }
            RowAdmissionRequirement::Unsupported {
                family,
                description,
            } if *family == "contract" => Self::Missing {
                kind: ApplicationFailureKind::RequiresViolation,
                notes: vec![format!(
                    "contract row {description} requires a ContractDischargeRecord and does not grant authority"
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
    /// [`Engine::admit_application`] with the supplied request.
    ///
    /// # Important
    ///
    /// Row requirements are metadata only. They do not register providers, select
    /// resources, admit roles, or grant policies. Missing authority always rejects
    /// with a structured diagnostic.
    ///
    /// # Errors
    ///
    /// Returns a [`ApplicationAdmissionOutcome::Rejected`] if any explicit row
    /// requirement is missing or unsupported.
    pub async fn admit_application_with_explicit_rows(
        &self,
        request: ApplicationAdmissionRequest,
        workflow: &crate::Entry,
    ) -> ApplicationAdmissionOutcome {
        let mut row_requirements: Vec<(String, RowAdmissionRequirement)> = Vec::new();
        for (name, core_type) in &workflow.core_callable_types {
            if let ash_core::core_ash::CoreType::Function { row, .. } = core_type {
                for requirement in RowAdmissionRequirement::from_core_row(row) {
                    row_requirements.push((name.clone(), requirement));
                }
            }
        }

        if let Some(declared_operation) = &workflow.declared_concrete_operation {
            let declared_requirement = RowAdmissionRequirement::Operation {
                authority: declared_operation.impl_type.clone(),
                operation: declared_operation.operation.clone(),
            };
            if self
                .declared_operation_provider_binding(declared_operation)
                .is_none()
            {
                return self
                    .reject_row_requirement(
                        &request,
                        ApplicationFailureKind::CapabilityAdmissionFailure,
                        "main",
                        &declared_requirement,
                        vec![format!(
                            "missing declared-operation binding for '{}.{}'",
                            declared_operation.impl_type, declared_operation.operation
                        )],
                    )
                    .await;
            }
            // The binding above, not a same-spelled provider name, discharges
            // this exact declaration-backed row. Other rows retain their normal
            // handler-over-provider admission checks below.
            row_requirements.retain(|(_, requirement)| requirement != &declared_requirement);
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
                            ApplicationFailureKind::RequiresViolation,
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
        self.admit_application(request).await
    }

    /// Set a contract-discharge record on a callable's admission requirement view.
    ///
    /// This is a metadata-only hook: it records the discharge but does not install
    /// providers, resources, roles, handlers, or runtime authority. Discharge is
    /// still checked through the admission path; missing or invalid discharge rejects.
    ///
    /// # Panics
    ///
    /// Panics if the internal contract-discharge registry mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns `None` if `callable_name` has no function row; otherwise returns the
    /// previous discharge record if one was set.
    pub fn set_contract_discharge_for_callable(
        &mut self,
        callable_name: &str,
        discharge: ContractDischargeRecord,
        workflow: &crate::Entry,
    ) -> Option<ContractDischargeRecord> {
        let _ = workflow;
        // For now, the record is kept in a dedicated engine-side registry so it is
        // available for admission and later runtime check planning without mutating
        // the callable row.
        self.runtime_state
            .contract_discharge_records
            .lock()
            .expect("contract discharge registry mutex poisoned")
            .insert(callable_name.to_string(), discharge)
    }

    /// Returns the contract-discharge record registered for a callable, if any.
    pub fn contract_discharge_record_for_callable(
        &self,
        callable_name: &str,
        _workflow: &crate::Entry,
    ) -> Option<ContractDischargeRecord> {
        self.runtime_state.contract_discharge_record(callable_name)
    }

    async fn reject_row_requirement(
        &self,
        request: &ApplicationAdmissionRequest,
        kind: ApplicationFailureKind,
        callable_name: &str,
        requirement: &RowAdmissionRequirement,
        notes: Vec<String>,
    ) -> ApplicationAdmissionOutcome {
        let application_id = request.application_id.unwrap_or_default();
        let run_id = request.run_id.unwrap_or_default();
        let admitted_capability_bindings = self
            .runtime_state
            .resolve_admitted_capability_bindings(&request.required_capabilities)
            .await;
        let admission = ApplicationAdmissionContext {
            active_role: admitted_role_name(request).map(ToOwned::to_owned),
            admitted_capabilities: request.required_capabilities.clone(),
            admitted_capability_bindings,
            requires_evidence: Vec::new(),
        };
        let ensures_evidence = build_pending_ensures_evidence(&request.ensures);
        let failure = ApplicationFailure::new(application_id, run_id, kind, None).with_evidence(
            ApplicationFailureEvidence {
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
        let report = ApplicationReport::failed(application_id, run_id, failure.clone())
            .with_admission_context(admission)
            .with_requires_evidence(Vec::new())
            .with_ensures_evidence(ensures_evidence);
        ApplicationAdmissionOutcome::Rejected { failure, report }
    }
}
