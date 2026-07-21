//! Ash Engine - Unified embedding API for Ash applications
//!
//! This crate provides the central `Engine` type for integrating Ash into Rust applications.
//! It encapsulates the entire application lifecycle: Parse → Check → Execute.
//!
//! # Example
//!
//! ```
//! use ash_engine::Engine;
//!
//! # tokio_test::block_on(async {
//! let engine = Engine::new().build().expect("engine builds");
//! # });
//! ```

pub mod entry;
pub mod error;
pub mod harness;
pub mod law_cache;
pub mod module_loader;
pub mod monomorphize;
pub mod providers;
pub mod row_admission;
pub mod runtime_artifact;
pub mod standard_profiles;

pub use entry::{
    EntryBootstrapError, EntryVerificationError, RuntimeEntryStdlibSource,
    load_runtime_entry_stdlib_sources, verify_entry_definition,
};
pub use error::EngineError;
pub use module_loader::{CallableRowRequirementSource, CallableRowRequirementSummary};
// Re-export the unified CapabilityProvider trait from ash_core
pub use ash_core::capability::CapabilityProvider;

use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_core::runtime::{
    ApplicationAdmissionContext, ApplicationBoundaryOutcome, ApplicationContractCheckEvidence,
    ApplicationEvidenceStatus, ApplicationFailure, ApplicationFailureKind, ApplicationReport,
    FailureBoundary, FailureEntity, HostBoundaryEvidence, OperationalFailure, ProcessFailure,
    RunId,
};
use ash_core::runtime_kernel::CheckedFunctionArtifact;
use ash_core::semantic_summary::SourceAnchor;
use ash_core::{
    ApplicationId, CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Expr, Provenance,
    Role, Value,
};
use ash_interp::{
    Context, EvalError, ExecError, ExecResult, ExecutionRecord, PolicyEvaluator, RoleContext,
    RuntimeState, eval_expr_async,
};
use ash_parser::surface::Type as SurfaceType;
use std::collections::{HashMap, HashSet};

/// The central engine for all Ash operations
///
/// The `Engine` provides a unified interface for parsing, type checking,
/// and executing Ash applications. It is designed to be:
///
/// - **Send + Sync**: Can be shared across threads
/// - **Configurable**: Built using the builder pattern
/// - **Extensible**: Supports custom capability providers
///
/// # Example
///
/// ```
/// use ash_engine::Engine;
///
/// # tokio_test::block_on(async {
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .build()
///     .expect("engine builds");
/// # });
/// ```
#[derive(Debug)]
pub struct Engine {
    /// Imported ADT/type definitions keyed by parsed entry ID.
    imported_type_defs:
        std::sync::Mutex<std::collections::HashMap<u64, Vec<ash_core::ast::TypeDef>>>,
    /// Imported semantic summaries keyed by parsed entry ID.
    imported_semantic_summaries: std::sync::Mutex<
        std::collections::HashMap<u64, Vec<ash_core::semantic_summary::ModuleSemanticSummary>>,
    >,
    /// Source-visible imported type-function heads keyed by parsed entry ID.
    imported_type_function_heads: std::sync::Mutex<
        std::collections::HashMap<u64, Vec<(String, ash_core::type_ir::TypeComputationHeadId)>>,
    >,
    /// Parsed program metadata for applications loaded with local pure-function definitions.
    surface_programs:
        std::sync::Mutex<std::collections::HashMap<u64, ash_parser::surface::Program>>,
    /// Current source module identity for parsed programs, when the application came from a file.
    surface_program_module_identities: std::sync::Mutex<
        std::collections::HashMap<u64, ash_core::semantic_summary::ModuleIdentity>,
    >,
    /// Narrow engine-owned registry of runtime stdlib module sources keyed by
    /// canonical module path.
    runtime_stdlib_modules: std::sync::Mutex<std::collections::HashMap<String, String>>,
    /// Counter for generating unique IDs
    next_id: std::sync::atomic::AtomicU64,
    /// Runtime-owned state that persists across related executions.
    /// Providers configured via `EngineBuilder` are passed to `RuntimeState` during build.
    runtime_state: RuntimeState,
    /// Host-selected capability implementation recipes keyed by binding name.
    capability_implementation_selections: HashMap<String, String>,
    /// Host-selected resource initializers keyed by resource type/name.
    resource_initializer_selections: HashMap<String, String>,
}

/// An entry handle that carries its internal ID for type checking.
///
/// This wraps the lowered target entry expression and maintains the association
/// with its surface representation needed for type checking.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The lowered target entry expression.
    pub core: Expr,
    /// The internal ID for looking up the surface program.
    id: u64,
    /// Imported callable closures, bound into context before execution.
    /// Populated from `module_loader::InlineCallable` during parse.
    pub imported_closures: std::collections::HashMap<String, ash_core::Value>,
    /// Param counts for imported callables, used to register type signatures.
    pub imported_param_counts: std::collections::HashMap<String, usize>,
    /// Declared type signatures for imported ordinary `pub fn` callables.
    pub imported_fn_signatures: std::collections::HashMap<String, ash_parser::surface::FnDef>,
    /// Declared type signatures for imported builtin fn callables.
    ///
    /// When present, `Engine::check()` uses `builtin_fn_signature_type` to
    /// produce the proper polymorphic type instead of an arity-only synthetic.
    pub imported_builtin_signatures:
        std::collections::HashMap<String, ash_parser::surface::BuiltinFnDef>,
    /// Explicit callable row requirements for imported and local callables.
    ///
    /// These are requirement summaries only; they do not install providers,
    /// admission facts, handlers, roles, resources, or runtime authority.
    pub callable_row_requirements: std::collections::HashMap<String, CallableRowRequirementSummary>,
    /// Core Ash function types for imported and local callables.
    ///
    /// This is a metadata bridge from explicit source rows to Core requirement
    /// rows. It does not make a callable executable or grant authority.
    pub core_callable_types: std::collections::HashMap<String, CoreType>,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core && self.id == other.id
    }
}

/// Result of checking a non-application module file.
///
/// Contains counts of validated type definitions and pub fn snippets,
/// along with any warnings or errors encountered during validation.
#[derive(Debug)]
pub struct ModuleFileCheckResult {
    /// Number of `pub type` definitions that parsed and registered successfully.
    pub type_count: usize,
    /// Number of `pub fn` snippets that parsed successfully.
    pub fn_count: usize,
    /// Non-fatal warnings collected during checking.
    pub warnings: Vec<String>,
    /// Fatal errors collected during checking.
    pub errors: Vec<String>,
}

/// Admission-time application contract requirements evaluated above interpreter execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationContractRequirement {
    /// Require the admitted application role to match this role name.
    Role(String),
    /// Require the admitted capability surface to include this capability name.
    Capability(String),
    /// Host/runtime-supplied evidence result for one `requires` clause.
    Evidence {
        /// Clause label being checked at admission.
        clause: String,
        /// Whether the host/runtime considered the clause satisfied.
        passed: bool,
        /// Evidence notes explaining the admission-time check result.
        notes: Vec<String>,
    },
}

/// Request for application admission above interpreter execution.
#[derive(Debug, Clone)]
pub struct ApplicationAdmissionRequest {
    /// Human-readable application name for admission/reporting.
    pub entry_name: String,
    /// Core entry expression to execute if admission succeeds.
    pub body: Expr,
    /// Explicit application identity to preserve, if one is already allocated.
    pub application_id: Option<ApplicationId>,
    /// Explicit host/runtime run identity to preserve, if one is already allocated.
    pub run_id: Option<RunId>,
    /// Admitted active role name, if any.
    pub active_role: Option<String>,
    /// Admitted runtime role context, if the caller can supply a truthful role projection.
    pub admitted_role: Option<Role>,
    /// Capability surface admitted to the application boundary.
    pub required_capabilities: Vec<String>,
    /// Admission-time requirements to validate before body execution.
    pub requires: Vec<ApplicationContractRequirement>,
    /// Ensures clause labels carried forward for TASK-716 completion-time evaluation.
    pub ensures: Vec<String>,
}

/// Admitted application boundary carrier returned by engine admission.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmittedApplicationBoundary {
    outcome: ApplicationBoundaryOutcome,
}

impl AdmittedApplicationBoundary {
    /// Wrap one admitted application boundary outcome.
    #[must_use]
    pub const fn new(outcome: ApplicationBoundaryOutcome) -> Self {
        Self { outcome }
    }

    /// Return the admitted application identity.
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        self.outcome.application_id()
    }

    /// Return the admitted host/runtime run identity.
    #[must_use]
    pub fn run_id(&self) -> RunId {
        self.outcome.run_id()
    }

    /// Borrow the admitted application boundary report.
    #[must_use]
    pub fn report(&self) -> &ApplicationReport {
        self.outcome.report()
    }

    /// Borrow the underlying application boundary outcome.
    #[must_use]
    pub const fn outcome(&self) -> &ApplicationBoundaryOutcome {
        &self.outcome
    }
}

/// Result of application admission above interpreter execution.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationAdmissionOutcome {
    /// Admission succeeded and produced a application boundary carrier.
    Admitted {
        /// Boundary outcome and report produced for the admitted application.
        boundary: AdmittedApplicationBoundary,
    },
    /// Admission failed before or at governed execution.
    Rejected {
        /// Structured application failure describing the rejection.
        failure: ApplicationFailure,
        /// Boundary report captured at rejection time.
        report: ApplicationReport,
    },
}

/// Result of processing a multi-entry program: closures, param counts, and lowered entry.
type ProgramProcessingResult = (
    HashMap<String, Value>,
    HashMap<String, usize>,
    HashMap<String, CallableRowRequirementSummary>,
    HashMap<String, CoreType>,
    Expr,
);

fn program_entry_function(
    program: &ash_parser::surface::Program,
) -> Option<&ash_parser::surface::FnDef> {
    program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name == program.entry.function =>
            {
                Some(function)
            }
            _ => None,
        })
}

fn build_pending_ensures_evidence(ensures: &[String]) -> Vec<ApplicationContractCheckEvidence> {
    ensures
        .iter()
        .cloned()
        .map(|clause| {
            ApplicationContractCheckEvidence::pending(
                clause,
                vec!["deferred-to-task-716".to_string()],
            )
        })
        .collect()
}

fn build_requires_evidence(
    requires: &[ApplicationContractRequirement],
) -> Vec<ApplicationContractCheckEvidence> {
    requires
        .iter()
        .filter_map(|requirement| match requirement {
            ApplicationContractRequirement::Evidence {
                clause,
                passed,
                notes,
            } => Some(if *passed {
                ApplicationContractCheckEvidence::passed(clause.clone(), notes.clone())
            } else {
                ApplicationContractCheckEvidence::failed(clause.clone(), notes.clone())
            }),
            ApplicationContractRequirement::Role(_)
            | ApplicationContractRequirement::Capability(_) => None,
        })
        .collect()
}

fn admitted_role_name(request: &ApplicationAdmissionRequest) -> Option<&str> {
    request
        .admitted_role
        .as_ref()
        .map(|role| role.name.as_str())
        .or(request.active_role.as_deref())
}

fn reject_admission(
    application_id: ApplicationId,
    run_id: RunId,
    kind: ApplicationFailureKind,
    admission: ApplicationAdmissionContext,
    requires_evidence: Vec<ApplicationContractCheckEvidence>,
    ensures_evidence: Vec<ApplicationContractCheckEvidence>,
) -> ApplicationAdmissionOutcome {
    let failure = ApplicationFailure::new(application_id, run_id, kind, None);
    let report = ApplicationReport::failed(application_id, run_id, failure.clone())
        .with_admission_context(admission)
        .with_requires_evidence(requires_evidence)
        .with_ensures_evidence(ensures_evidence);
    ApplicationAdmissionOutcome::Rejected { failure, report }
}

fn failed_boundary_outcome_from_exec_error(
    application_id: ApplicationId,
    run_id: RunId,
    error: &ExecError,
    admission: ApplicationAdmissionContext,
    requires_evidence: Vec<ApplicationContractCheckEvidence>,
    ensures_evidence: Vec<ApplicationContractCheckEvidence>,
    execution_record: Option<&ExecutionRecord>,
) -> ApplicationBoundaryOutcome {
    let cause = lower_operational_cause_from_exec_error(run_id, error);
    let failure = ApplicationFailure::new(
        application_id,
        run_id,
        ApplicationFailureKind::BodyFailureEscaped,
        Some(cause.clone()),
    );
    let report = project_execution_report(
        ApplicationReport::failed(application_id, run_id, failure.clone())
            .with_admission_context(admission)
            .with_requires_evidence(requires_evidence)
            .with_ensures_evidence(ensures_evidence),
        execution_record,
        Some(&cause),
    );
    ApplicationBoundaryOutcome::failed(failure, report)
}

fn lower_operational_cause_from_exec_error(run_id: RunId, error: &ExecError) -> OperationalFailure {
    match error {
        ExecError::Eval(EvalError::OperationalFailure(failure)) => failure.as_ref().clone(),
        ExecError::Eval(eval_error) => OperationalFailure::new(
            FailureBoundary::Process,
            FailureEntity::Run(run_id),
            Value::String(eval_error.to_string()),
            "EvalError",
        ),
        _ => OperationalFailure::new(
            FailureBoundary::Application,
            FailureEntity::Run(run_id),
            Value::String(error.to_string()),
            "ExecError",
        ),
    }
}

fn report_evidence_from_execution(execution_record: &ExecutionRecord) -> Vec<String> {
    let mut evidence = vec![format!("execution_phase={:?}", execution_record.phase())];
    if let Some(completion) = execution_record.project_completion() {
        evidence.push(format!(
            "terminal_effect={:?}",
            completion.effects().terminal()
        ));
        evidence.push(format!("trace_events={}", execution_record.trace().len()));
        evidence.push(format!(
            "lower_process_summary=result={:?};pending_local={};pending_role={};reached_effects={:?}",
            completion.result(),
            completion.obligations().pending().len(),
            completion.obligations().role_pending().len(),
            completion.effects().reached(),
        ));
    }
    evidence
}

fn report_provenance_from_execution(execution_record: &ExecutionRecord) -> Vec<String> {
    let provenance = execution_record.provenance();
    let mut notes = vec![format!(
        "execution_application_id={:?}",
        provenance.application_id
    )];
    if let Some(parent) = provenance.parent {
        notes.push(format!("execution_parent_application_id={parent:?}"));
    }
    if !provenance.lineage.is_empty() {
        notes.push(format!("execution_lineage={:?}", provenance.lineage));
    }
    notes
}

fn obligation_evidence_from_execution(execution_record: &ExecutionRecord) -> Vec<String> {
    let obligations = execution_record.obligations();
    let mut evidence = obligations
        .pending()
        .iter()
        .map(|name| format!("local_pending:{name}"))
        .collect::<Vec<_>>();
    if let Some(active_role) = obligations.active_role() {
        evidence.push(format!("active_role:{active_role}"));
    }
    evidence.extend(
        obligations
            .role_pending()
            .iter()
            .map(|name| format!("role_pending:{name}")),
    );
    evidence.extend(
        obligations
            .role_discharged()
            .iter()
            .map(|name| format!("role_discharged:{name}")),
    );
    evidence
}

fn lower_process_failures_from_causes(lower_causes: &[OperationalFailure]) -> Vec<ProcessFailure> {
    lower_causes
        .iter()
        .filter_map(|cause| match cause.entity {
            FailureEntity::Process(process_id) => {
                Some(ProcessFailure::new(process_id, cause.clone()))
            }
            _ => None,
        })
        .collect()
}

fn project_execution_report(
    report: ApplicationReport,
    execution_record: Option<&ExecutionRecord>,
    lower_cause: Option<&OperationalFailure>,
) -> ApplicationReport {
    let lower_causes = lower_cause.cloned().into_iter().collect::<Vec<_>>();
    let report = report
        .with_lower_causes(lower_causes.clone())
        .with_lower_process_failures(lower_process_failures_from_causes(&lower_causes));
    match execution_record {
        Some(execution_record) => report
            .with_obligation_evidence(obligation_evidence_from_execution(execution_record))
            .with_evidence(report_evidence_from_execution(execution_record))
            .with_provenance(report_provenance_from_execution(execution_record)),
        None => report,
    }
}

fn resolve_ensures_evidence(
    ensures: &[String],
    result: &Value,
) -> Vec<ApplicationContractCheckEvidence> {
    ensures
        .iter()
        .cloned()
        .map(|clause| {
            if let Some(field) = clause.strip_prefix("result.") {
                let passed = match result {
                    Value::Record(fields) => fields.get(field).is_some_and(|value| match value {
                        Value::Bool(boolean) => *boolean,
                        Value::Null => false,
                        _ => true,
                    }),
                    _ => false,
                };
                let note =
                    format!("evaluated result field `{field}` against entry result {result}");
                if passed {
                    ApplicationContractCheckEvidence::passed(clause, vec![note])
                } else {
                    ApplicationContractCheckEvidence::failed(clause, vec![note])
                }
            } else {
                ApplicationContractCheckEvidence::failed(
                    clause,
                    vec![
                        "completion boundary has no evaluator for opaque ensures label".to_string(),
                    ],
                )
            }
        })
        .collect()
}

fn completion_failure_outcome(
    application_id: ApplicationId,
    run_id: RunId,
    kind: ApplicationFailureKind,
    admission: ApplicationAdmissionContext,
    requires_evidence: Vec<ApplicationContractCheckEvidence>,
    ensures_evidence: Vec<ApplicationContractCheckEvidence>,
    execution_record: Option<&ExecutionRecord>,
) -> ApplicationBoundaryOutcome {
    let failure = ApplicationFailure::new(application_id, run_id, kind, None);
    let report = project_execution_report(
        ApplicationReport::failed(application_id, run_id, failure.clone())
            .with_admission_context(admission)
            .with_requires_evidence(requires_evidence)
            .with_ensures_evidence(ensures_evidence),
        execution_record,
        None,
    );
    ApplicationBoundaryOutcome::failed(failure, report)
}

#[allow(clippy::too_many_arguments)]
fn admitted_completion_outcome(
    application_id: ApplicationId,
    run_id: RunId,
    value: Value,
    admission: ApplicationAdmissionContext,
    requires_evidence: Vec<ApplicationContractCheckEvidence>,
    ensures: &[String],
    execution_record: Option<&ExecutionRecord>,
) -> ApplicationBoundaryOutcome {
    let local_pending =
        execution_record.is_some_and(|record| !record.obligations().pending().is_empty());
    let role_pending =
        execution_record.is_some_and(|record| !record.obligations().role_pending().is_empty());
    let ensures_evidence = resolve_ensures_evidence(ensures, &value);
    let ensures_failed = ensures_evidence
        .iter()
        .any(|entry| entry.status == ApplicationEvidenceStatus::Failed);

    if local_pending {
        completion_failure_outcome(
            application_id,
            run_id,
            ApplicationFailureKind::LocalObligationsUndischarged,
            admission,
            requires_evidence,
            Vec::new(),
            execution_record,
        )
    } else if role_pending {
        completion_failure_outcome(
            application_id,
            run_id,
            ApplicationFailureKind::RoleObligationsUndischarged,
            admission,
            requires_evidence,
            Vec::new(),
            execution_record,
        )
    } else if ensures.is_empty() || !ensures_failed {
        ApplicationBoundaryOutcome::succeeded(
            value.clone(),
            project_execution_report(
                ApplicationReport::succeeded(application_id, run_id)
                    .with_admission_context(admission)
                    .with_requires_evidence(requires_evidence)
                    .with_ensures_evidence(ensures_evidence)
                    .with_result(value),
                execution_record,
                None,
            ),
        )
    } else {
        completion_failure_outcome(
            application_id,
            run_id,
            ApplicationFailureKind::EnsuresViolation,
            admission,
            requires_evidence,
            ensures_evidence,
            execution_record,
        )
    }
}

impl Engine {
    /// Create a new engine builder with default configuration
    ///
    /// Returns an `EngineBuilder` that can be used to configure capabilities
    /// before building the engine.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    ///
    /// let builder = Engine::new();
    /// ```
    #[allow(clippy::new_ret_no_self, clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Generate a unique ID for parsed entry handles.
    fn next_application_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Store imported type definitions for a parsed entry.
    fn store_imported_type_defs(&self, application_id: u64, defs: Vec<ash_core::ast::TypeDef>) {
        if let Ok(mut map) = self.imported_type_defs.lock() {
            map.insert(application_id, defs);
        }
    }

    /// Store imported semantic summaries for a parsed entry.
    fn store_imported_semantic_summaries(
        &self,
        application_id: u64,
        summaries: Vec<ash_core::semantic_summary::ModuleSemanticSummary>,
    ) {
        if let Ok(mut map) = self.imported_semantic_summaries.lock() {
            map.insert(application_id, summaries);
        }
    }

    /// Store source-visible imported type-function heads for a parsed entry.
    fn store_imported_type_function_heads(
        &self,
        application_id: u64,
        heads: Vec<(String, ash_core::type_ir::TypeComputationHeadId)>,
    ) {
        if let Ok(mut map) = self.imported_type_function_heads.lock() {
            map.insert(application_id, heads);
        }
    }

    fn store_surface_program(&self, application_id: u64, program: ash_parser::surface::Program) {
        if let Ok(mut map) = self.surface_programs.lock() {
            map.insert(application_id, program);
        }
    }

    fn store_surface_program_module_identity(
        &self,
        application_id: u64,
        module_identity: ash_core::semantic_summary::ModuleIdentity,
    ) {
        if let Ok(mut map) = self.surface_program_module_identities.lock() {
            map.insert(application_id, module_identity);
        }
    }

    fn get_surface_program(&self, id: u64) -> Option<ash_parser::surface::Program> {
        self.surface_programs
            .lock()
            .map_or(None, |map| map.get(&id).cloned())
    }

    fn get_surface_program_module_identity(
        &self,
        id: u64,
    ) -> Option<ash_core::semantic_summary::ModuleIdentity> {
        self.surface_program_module_identities
            .lock()
            .map_or(None, |map| map.get(&id).cloned())
    }

    /// Retrieve imported type definitions by application ID.
    fn get_imported_type_defs(&self, id: u64) -> Vec<ash_core::ast::TypeDef> {
        self.imported_type_defs.lock().map_or_else(
            |_| Vec::new(),
            |map| map.get(&id).cloned().unwrap_or_default(),
        )
    }

    /// Retrieve imported semantic summaries by application ID.
    fn get_imported_semantic_summaries(
        &self,
        id: u64,
    ) -> Vec<ash_core::semantic_summary::ModuleSemanticSummary> {
        self.imported_semantic_summaries.lock().map_or_else(
            |_| Vec::new(),
            |map| map.get(&id).cloned().unwrap_or_default(),
        )
    }

    /// Retrieve source-visible imported type-function heads by application ID.
    fn get_imported_type_function_heads(
        &self,
        id: u64,
    ) -> Vec<(String, ash_core::type_ir::TypeComputationHeadId)> {
        self.imported_type_function_heads.lock().map_or_else(
            |_| Vec::new(),
            |map| map.get(&id).cloned().unwrap_or_default(),
        )
    }

    /// Register a runtime stdlib module source under its canonical module path.
    fn register_runtime_stdlib_module(
        &self,
        module_path: &str,
        source: String,
    ) -> Result<(), EngineError> {
        self.runtime_stdlib_modules
            .lock()
            .map_err(|_| {
                EngineError::Configuration("runtime stdlib registry lock poisoned".to_string())
            })?
            .insert(module_path.to_string(), source);
        Ok(())
    }

    /// Load the narrow runtime stdlib registry owned by this engine.
    ///
    /// This only registers the canonical runtime entry modules currently needed
    /// by the Phase 57 entry path.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Io`] if a required stdlib source file cannot be read
    /// or [`EngineError::Configuration`] if the registry cannot be updated.
    pub fn load_runtime_stdlib(&self) -> Result<(), EngineError> {
        for module in load_runtime_entry_stdlib_sources()? {
            self.register_runtime_stdlib_module(module.module_path, module.source)?;
        }

        Ok(())
    }

    /// Return whether the engine has registered the canonical runtime module path.
    #[must_use]
    pub fn has_registered_runtime_module(&self, module_path: &str) -> bool {
        self.runtime_stdlib_modules
            .lock()
            .is_ok_and(|registry| registry.contains_key(module_path))
    }

    /// Check whether a capability provider with the given name is registered.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        self.runtime_state.has_provider(name)
    }

    /// Return the number of registered capability providers.
    #[must_use]
    pub fn provider_count(&self) -> usize {
        self.runtime_state.provider_count()
    }

    /// Install one standard provider/admission profile into this engine runtime.
    ///
    /// # Errors
    ///
    /// Returns runtime validation errors when profile metadata, sandbox policy, or capability
    /// binding admission is malformed or incompatible with provider metadata.
    pub async fn install_standard_profile(
        &self,
        profile: standard_profiles::StandardProviderProfile,
    ) -> ExecResult<standard_profiles::InstalledStandardProfile> {
        profile.install(&self.runtime_state).await
    }

    /// Return retained redacted host-boundary evidence for this engine runtime.
    pub async fn host_boundary_evidence(&self) -> Vec<HostBoundaryEvidence> {
        self.runtime_state.host_boundary_evidence().await
    }

    /// Return the selected Ash-defined implementation for a host binding name.
    #[must_use]
    pub fn capability_implementation_selection(&self, binding: &str) -> Option<&str> {
        self.capability_implementation_selections
            .get(binding)
            .map(String::as_str)
    }

    /// Return the number of configured capability implementation selections.
    #[must_use]
    pub fn capability_implementation_selection_count(&self) -> usize {
        self.capability_implementation_selections.len()
    }

    /// Return the selected initializer for a resource type/name.
    #[must_use]
    pub fn resource_initializer_selection(&self, resource: &str) -> Option<&str> {
        self.resource_initializer_selections
            .get(resource)
            .map(String::as_str)
    }

    /// Return the number of configured resource initializer selections.
    #[must_use]
    pub fn resource_initializer_selection_count(&self) -> usize {
        self.resource_initializer_selections.len()
    }

    /// Validate host-facing capability/resource configuration against a source module.
    ///
    /// This is intentionally validation-only: TASK-743 exposes names selected by a
    /// host and rejects unknown source names without lowering source declarations
    /// into runtime admissions.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Configuration` when a selected capability implementation
    /// or resource initializer target is not declared by the provided source.
    pub fn validate_configuration_for_source(&self, source: &str) -> Result<(), EngineError> {
        let implementations = declared_capability_implementation_names(source);
        let resources = declared_resource_type_names(source);

        for implementation in self.capability_implementation_selections.values() {
            if !implementations.contains(implementation) {
                return Err(EngineError::Configuration(format!(
                    "unknown capability implementation '{implementation}'"
                )));
            }
        }

        for resource in self.resource_initializer_selections.keys() {
            if !resources.contains(resource) {
                return Err(EngineError::Configuration(format!(
                    "unknown resource initializer target '{resource}'"
                )));
            }
        }

        Ok(())
    }

    fn runtime_stdlib_type_defs(&self) -> Result<Vec<ash_core::ast::TypeDef>, EngineError> {
        let sources = self
            .runtime_stdlib_modules
            .lock()
            .map_err(|_| {
                EngineError::Configuration("runtime stdlib registry lock poisoned".to_string())
            })?
            .iter()
            .map(|(module_path, source)| (module_path.clone(), source.clone()))
            .collect::<Vec<_>>();

        let mut type_defs = Vec::new();
        for (module_path, source) in &sources {
            type_defs.extend(
                module_loader::collect_runtime_stdlib_type_defs_from_module_file(
                    module_path,
                    source,
                )?,
            );
        }
        Ok(type_defs)
    }

    /// Parse source code into an Entry
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the source contains syntax errors.
    pub fn parse(&self, source: &str) -> Result<Entry, EngineError> {
        let imported_callables = HashMap::new();
        self.parse_entry_source_with_imports(
            source,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            &imported_callables,
            None,
        )
    }

    /// Parse entry source into a [`Entry`], tolerating a leading `use` prelude.
    ///
    /// This helper is intentionally narrow and only exists for the runtime entry
    /// path. It validates contiguous leading runtime `use` declarations against
    /// the engine-owned runtime stdlib registry before stripping them and
    /// delegating to the ordinary single-application parser.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the leading runtime imports are not
    /// supported or not registered, or if the remaining application source
    /// contains syntax errors.
    pub fn parse_entry_source(&self, source: &str) -> Result<Entry, EngineError> {
        entry::validate_runtime_entry_import_prelude(source, |module_path| {
            self.has_registered_runtime_module(module_path)
        })?;
        self.parse_entry_source_with_imports(
            entry::strip_leading_entry_use_lines(source),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            &HashMap::new(),
            None,
        )
    }

    /// Parse entry source from a file, tolerating the narrow leading `use` prelude.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read and `EngineError::Parse`
    /// if the entry source is invalid.
    pub fn parse_entry_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Entry, EngineError> {
        let source = std::fs::read_to_string(path)?;
        self.parse_entry_source(&source)
    }

    #[allow(dead_code)]
    /// Parse an ordinary Ash file from disk.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns `EngineError::Parse` if the file contains syntax errors.
    pub fn parse_file(&self, path: impl AsRef<std::path::Path>) -> Result<Entry, EngineError> {
        let path = path.as_ref();
        let module_identity = module_loader::module_identity_for_path(path);
        let loaded = module_loader::load_ordinary_file(path)?;
        self.parse_loaded_ordinary_file(&loaded, &module_identity)
    }

    /// Parse already-read ordinary Ash file source.
    ///
    /// `path` supplies only module identity and import-resolution context; the
    /// entry source is taken from `source` without re-reading `path`.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the supplied source contains syntax
    /// errors or `EngineError::Configuration` if path context is invalid.
    pub fn parse_file_source(
        &self,
        path: impl AsRef<std::path::Path>,
        source: &str,
    ) -> Result<Entry, EngineError> {
        let path = path.as_ref();
        let module_identity = module_loader::module_identity_for_path(path);
        let loaded = module_loader::load_ordinary_source(path, source)?;
        self.parse_loaded_ordinary_file(&loaded, &module_identity)
    }

    fn parse_loaded_ordinary_file(
        &self,
        loaded: &module_loader::LoadedOrdinaryFile,
        module_identity: &ash_core::semantic_summary::ModuleIdentity,
    ) -> Result<Entry, EngineError> {
        self.parse_entry_source_with_imports(
            &loaded.ordinary_source,
            loaded.imported_type_defs.clone(),
            loaded.imported_semantic_summaries.clone(),
            loaded.imported_type_function_heads.clone(),
            &loaded.imported_callables,
            Some(module_identity),
        )
    }
    /// Extract local function definitions as closures and register helper applications.
    ///
    /// Returns `(local_closures, local_param_counts)` with both imported and
    /// locally-defined entries.
    fn process_program_definitions(
        program: &ash_parser::surface::Program,
        imported_closures: HashMap<String, Value>,
        imported_param_counts: HashMap<String, usize>,
        imported_callable_row_requirements: HashMap<String, CallableRowRequirementSummary>,
        imported_core_callable_types: HashMap<String, CoreType>,
    ) -> Result<ProgramProcessingResult, EngineError> {
        use ash_core::env_frame::EnvFrame;
        use ash_parser::{
            LoweringContext, effectful_names_from_definitions, lower_expr_with_context,
        };

        let entry_fn = program_entry_function(program)
            .ok_or_else(|| EngineError::Parse("expected fn main entry".to_string()))?;
        let lowering_ctx = LoweringContext::with_effectful_names(effectful_names_from_definitions(
            &program.definitions,
        ));
        let core = lower_expr_with_context(&entry_fn.body, &lowering_ctx)
            .map_err(|e| EngineError::Parse(format!("lowering error: {e}")))?;

        let (
            mut local_closures,
            mut local_param_counts,
            mut callable_row_requirements,
            mut core_callable_types,
        ) = (
            imported_closures,
            imported_param_counts,
            imported_callable_row_requirements,
            imported_core_callable_types,
        );

        let mut imported_env = EnvFrame::new();
        for (name, value) in &local_closures {
            imported_env.insert(name.clone(), value.clone());
        }
        let mut module_env = EnvFrame::with_parent(std::sync::Arc::new(imported_env));
        let mut late_slots = HashMap::new();
        let mut function_specs = Vec::new();

        for def_item in &program.definitions {
            if let ash_parser::surface::Definition::Function(fn_def) = def_item
                && let Ok(body_expr) = lower_expr_with_context(&fn_def.body, &lowering_ctx)
            {
                let name = fn_def.name.to_string();
                let slot = module_env.insert_late(name.clone());
                let params: Vec<(String, Option<String>)> = fn_def
                    .params
                    .iter()
                    .map(|p| (p.name.to_string(), None))
                    .collect();
                local_param_counts.insert(name.clone(), params.len());
                if let Some(row_requirement) =
                    module_loader::callable_row_requirement_from_fn_def(fn_def)
                {
                    callable_row_requirements.insert(name.clone(), row_requirement);
                }
                let core_type = core_callable_type_from_fn_def(fn_def).map_err(|error| {
                    EngineError::Parse(format!(
                        "failed to lower callable row for '{}': {error}",
                        fn_def.name
                    ))
                })?;
                core_callable_types.insert(name.clone(), core_type);
                late_slots.insert(name.clone(), slot);
                function_specs.push((name, params, body_expr));
            }
        }

        let module_env = std::sync::Arc::new(module_env);
        for (name, params, body_expr) in function_specs {
            let closure = Value::Closure {
                params,
                body: Box::new(body_expr),
                env: module_env.clone(),
            };
            if let Some(slot) = late_slots.get(&name) {
                slot.set_late(closure.clone());
            }
            local_closures.insert(name, closure);
        }

        Ok((
            local_closures,
            local_param_counts,
            callable_row_requirements,
            core_callable_types,
            core,
        ))
    }

    #[allow(clippy::too_many_lines)]
    fn parse_entry_source_with_imports(
        &self,
        source: &str,
        imported_type_defs: Vec<ash_core::ast::TypeDef>,
        imported_semantic_summaries: Vec<ash_core::semantic_summary::ModuleSemanticSummary>,
        imported_type_function_heads: Vec<(String, ash_core::type_ir::TypeComputationHeadId)>,
        imported_callables: &HashMap<String, module_loader::InlineCallable>,
        module_identity: Option<&ash_core::semantic_summary::ModuleIdentity>,
    ) -> Result<Entry, EngineError> {
        let (
            imported_closures,
            imported_param_counts,
            imported_callable_row_requirements,
            imported_core_callable_types,
            imported_fn_signatures,
            imported_builtin_signatures,
        ) = build_imported_closures(imported_callables)?;

        let parsed_program =
            module_loader::parse_program_with_functions(source).map_err(EngineError::Parse)?;
        let program = parsed_program.program;

        let id = self.next_application_id();
        let (
            local_closures,
            local_param_counts,
            callable_row_requirements,
            core_callable_types,
            core,
        ) = Self::process_program_definitions(
            &program,
            imported_closures,
            imported_param_counts,
            imported_callable_row_requirements,
            imported_core_callable_types,
        )?;

        self.store_surface_program(id, program);
        if let Some(identity) = module_identity {
            self.store_surface_program_module_identity(id, identity.clone());
        }
        self.store_imported_semantic_summaries(id, imported_semantic_summaries);
        self.store_imported_type_function_heads(id, imported_type_function_heads);
        self.store_imported_type_defs(id, imported_type_defs);
        Ok(Entry {
            core,
            id,
            imported_closures: local_closures,
            imported_param_counts: local_param_counts,
            imported_fn_signatures,
            imported_builtin_signatures,
            callable_row_requirements,
            core_callable_types,
        })
    }
    /// Infer the canonical Ash type name for an expression.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the expression does not parse and
    /// `EngineError::Type` if the inferred type is not concrete enough to report.
    pub fn infer_expression_type(&self, source: &str) -> Result<String, EngineError> {
        use ash_parser::parse_expr::expr;
        use ash_typeck::type_env::TypeEnv;
        use winnow::prelude::*;

        let mut input = ash_parser::new_input(source);
        let expr = expr
            .parse_next(&mut input)
            .map_err(|e| EngineError::Parse(format!("{e}")))?;

        let ty = ash_typeck::check_expr::infer_type(&TypeEnv::with_builtin_types(), &expr);
        match ty {
            ash_typeck::Type::Var(_) => Err(EngineError::Type(
                "could not infer a canonical type for expression".to_string(),
            )),
            other => Ok(other.to_string()),
        }
    }

    /// Type check an entry
    ///
    /// On success, this also monomorphizes any generic interface method calls
    /// in the application core so that the interpreter never sees unresolved
    /// dispatch at runtime.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Type` if type checking or monomorphization fails.
    #[allow(clippy::too_many_lines)]
    pub fn check(&self, application: &mut Entry) -> Result<(), EngineError> {
        self.check_with_typeck_config(application, &ash_typeck::TypeCheckConfig::default())
    }

    /// Check an entry and return the core-owned function artifact needed by `RuntimeKernel`.
    ///
    /// The returned body is the same lowered expression that was checked and then
    /// monomorphized for execution. Its row and result type come from the canonical
    /// Core function type recorded while lowering the selected source function.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Type`] when checking fails or the selected entry has
    /// no canonical Core function type.
    pub fn check_entry_artifact(
        &self,
        application: &mut Entry,
        function_identity: impl Into<String>,
        source_anchor: SourceAnchor,
    ) -> Result<CheckedFunctionArtifact, EngineError> {
        self.check(application)?;
        let program = self
            .get_surface_program(application.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;
        let entry_name = program.entry.function.to_string();
        let function_type = application
            .core_callable_types
            .get(&entry_name)
            .ok_or_else(|| {
                EngineError::Type(format!(
                    "checked entry '{entry_name}' has no canonical Core function type"
                ))
            })?;
        let CoreType::Function { row, result, .. } = function_type else {
            return Err(EngineError::Type(format!(
                "checked entry '{entry_name}' did not lower to a Core function type"
            )));
        };

        Ok(CheckedFunctionArtifact {
            function_identity: function_identity.into(),
            effect_row: row.clone(),
            body: application.core.clone(),
            source_anchor,
            result_type: result.as_ref().clone(),
        })
    }

    /// Type check a parsed entry using explicit typechecker configuration.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Type` if type checking or monomorphization fails,
    /// and propagates imported-summary/type metadata errors from the existing
    /// engine check path.
    #[allow(clippy::too_many_lines)]
    pub fn check_with_typeck_config(
        &self,
        application: &mut Entry,
        typeck_config: &ash_typeck::TypeCheckConfig,
    ) -> Result<(), EngineError> {
        let program = self
            .get_surface_program(application.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;

        let mut type_env = ash_typeck::type_env::TypeEnv::with_builtin_types();
        let imported_summaries = self.get_imported_semantic_summaries(application.id);
        register_imported_semantic_summaries(&mut type_env, &imported_summaries)?;
        expose_imported_type_function_heads(
            &mut type_env,
            self.get_imported_type_function_heads(application.id),
        )?;
        let mut imported_type_defs = self.get_imported_type_defs(application.id);
        imported_type_defs.extend(self.runtime_stdlib_type_defs()?);
        let local_type_defs = module_loader::core_type_defs_from_definitions(&program.definitions)?;
        imported_type_defs.extend(local_type_defs.clone());
        register_imported_type_defs(&mut type_env, imported_type_defs)?;
        for local_type in &local_type_defs {
            type_env
                .expose_type_representation(&local_type.name)
                .map_err(|error| EngineError::Type(error.to_string()))?;
        }
        bind_imported_callable_types(&mut type_env, application)?;

        let type_check_result = self
            .get_surface_program_module_identity(application.id)
            .map_or_else(
                || {
                    ash_typeck::type_check_program_in_env_with_config(
                        &type_env,
                        &program,
                        typeck_config,
                    )
                },
                |module_identity| {
                    ash_typeck::type_check_program_in_env_for_module_with_config(
                        &type_env,
                        &program,
                        module_identity,
                        typeck_config,
                    )
                },
            );

        match type_check_result {
            Ok(result) => {
                if result.is_ok() {
                    monomorphize::monomorphize_expr(&mut application.core, &type_env)
                        .map_err(|e| EngineError::Type(e.to_string()))?;
                    Ok(())
                } else {
                    // Collect type errors into a message
                    let errors: Vec<String> =
                        result.errors.iter().map(|e| format!("{e:?}")).collect();
                    Err(EngineError::Type(errors.join("; ")))
                }
            }
            Err(e) => Err(EngineError::Type(format!("{e}"))),
        }
    }

    /// Verify that a parsed entry artifact matches the canonical entry contract.
    ///
    /// This is a pure metadata validation over the cached parsed entry definition.
    /// It does not load the standard library, resolve imports, or perform bootstrap.
    ///
    /// # Errors
    ///
    /// Returns [`EntryVerificationError`] if the cached surface metadata is missing
    /// or if the entry signature does not match the canonical `main` contract.
    pub fn verify_entry_definition(&self, entry: &Entry) -> Result<(), EntryVerificationError> {
        let program = self
            .get_surface_program(entry.id)
            .ok_or(EntryVerificationError::MissingApplicationMetadata)?;
        let def = program_entry_function(&program).ok_or(EntryVerificationError::MissingMain)?;

        verify_entry_definition(def)
    }

    /// Check a non-application module file for validity.
    ///
    /// Reads the file as an authoritative `ModuleFile`, lowers ordinary type
    /// metadata, validates the public semantic surface in a fresh `TypeEnv`,
    /// and counts parseable `pub fn` exports. Returns a `ModuleFileCheckResult`
    /// with type/fn counts and any non-fatal validation diagnostics.
    ///
    /// # Error model
    ///
    /// This method uses a **dual error path**:
    /// - **Hard errors** (file I/O failure or authoritative `ModuleFile` parse
    ///   failure) propagate via `Result::Err` and abort early.
    /// - **Soft errors** (public API private-type exposure or semantic-summary
    ///   registration failures) accumulate in `result.errors` so callers can
    ///   report all export-surface issues together.
    /// - **Warnings** are reserved for non-fatal public function export diagnostics
    ///   that do not invalidate the parsed `ModuleFile`.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read, or
    /// `EngineError::Parse` if the module file cannot be parsed for type metadata.
    #[allow(clippy::too_many_lines)]
    pub fn check_module_file(
        &self,
        path: &std::path::Path,
    ) -> Result<ModuleFileCheckResult, EngineError> {
        let source = std::fs::read_to_string(path)?;

        let type_metadata =
            module_loader::collect_module_type_metadata_from_module_file(path, &source)?;
        module_loader::validate_expanded_surface_module_file(path, &source)?;
        let type_count = type_metadata
            .type_defs
            .iter()
            .filter(|type_def| matches!(type_def.visibility, ash_core::ast::Visibility::Public))
            .count();
        let (fn_count, fn_diagnostics) = module_loader::count_pub_fn_snippets(&source);

        let warnings: Vec<String> = fn_diagnostics
            .iter()
            .map(|d| {
                d.name.as_ref().map_or_else(
                    || format!("pub fn: {}", d.reason),
                    |name| format!("pub fn '{name}': {}", d.reason),
                )
            })
            .collect();
        let mut errors = module_loader::public_callable_signature_resolution_errors(
            path,
            &source,
            &type_metadata.type_defs,
        );
        errors.extend(module_loader::public_imported_type_visibility_errors(
            path, &source,
        ));
        errors.extend(module_loader::public_interface_constraint_visibility_errors(path, &source));
        errors.extend(module_loader::public_opaque_import_constructor_errors(
            path, &source,
        ));
        errors.extend(module_loader::public_representation_visibility_errors(
            path,
            &source,
            &type_metadata.type_defs,
        ));
        errors.extend(
            module_loader::public_representation_type_function_leak_errors(
                &type_metadata.type_defs,
                &type_metadata
                    .type_function_defs
                    .iter()
                    .map(|type_fn| type_fn.name.to_string())
                    .collect(),
            ),
        );

        // Build a TypeEnv and register the public lowered semantic summary so
        // ordinary type identities and exposed representations take the same
        // path as imports without colliding with hidden local builtin/private
        // implementation details such as std::act's ActEnv.
        let mut type_env = ash_typeck::TypeEnv::with_builtin_types();

        // Process imports and register imported types so local type definitions
        // can reference them (e.g., Strategy<T> referencing imported GenContext).
        let module_root = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let crate_root = module_loader::import_resolution::discover_crate_root(module_root);
        let imports = module_loader::parse_module_imports(&source)?;
        let mut imported_type_defs = Vec::new();
        let mut imported_type_names = HashSet::new();
        let mut module_cache = HashMap::new();
        let mut visiting = HashSet::new();

        for import in imports {
            let (module_segments, search_roots) =
                module_loader::import_resolution::import_resolution_roots(
                    &import.module_segments,
                    module_root,
                    crate_root.as_deref(),
                )?;
            if let Ok(Some(module_path)) = module_loader::import_resolution::resolve_module_path(
                &module_segments,
                &search_roots,
            ) {
                let exports = module_loader::collect_module_exports(
                    &module_path,
                    &mut module_cache,
                    &mut visiting,
                )?;
                for selection in import.selections {
                    match selection {
                        module_loader::ImportSelection::Glob => {
                            for (name, type_def) in &exports.type_defs {
                                let imported_type =
                                    module_loader::type_def_with_visible_name(type_def, name);
                                if imported_type_names.insert(imported_type.name.clone()) {
                                    imported_type_defs.push(imported_type);
                                }
                            }
                        }
                        module_loader::ImportSelection::Named { name, alias } => {
                            let exported_name =
                                alias.as_ref().map_or_else(|| name.clone(), Clone::clone);
                            if let Some(type_def) = exports.type_defs.get(&name) {
                                // Use type_def_with_visible_name instead of selected_type_def_with_import_visibility
                                // to avoid dependency metadata aliasing ($ash_dependency$...) which breaks
                                // type registration when the dependency is also imported separately.
                                let imported_type = module_loader::type_def_with_visible_name(
                                    type_def,
                                    &exported_name,
                                );
                                let imported_type_name = imported_type.name.clone();
                                // Collect dependencies before moving imported_type
                                let mut dependency_names = Vec::new();
                                module_loader::collect_core_type_body_names(
                                    &imported_type.body,
                                    &mut dependency_names,
                                );
                                if imported_type_names.insert(imported_type_name.clone()) {
                                    imported_type_defs.push(imported_type);
                                }
                                for dep_name in dependency_names {
                                    if dep_name == imported_type_name
                                        || imported_type_names.contains(&dep_name)
                                    {
                                        continue;
                                    }
                                    if let Some(dep_type_def) = exports.type_defs.get(&dep_name) {
                                        let dep_imported =
                                            module_loader::type_def_with_visible_name(
                                                dep_type_def,
                                                &dep_name,
                                            );
                                        if imported_type_names.insert(dep_imported.name.clone()) {
                                            imported_type_defs.push(dep_imported);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Register imported type names as placeholders so local type definitions can reference them
        for imported_type in &imported_type_defs {
            if !type_env.has_type(&imported_type.name) {
                type_env.declare_type_name(&imported_type.name);
            }
        }
        // Register imported type identities
        for imported_type in imported_type_defs {
            if type_env.has_full_type(&imported_type.name)
                || type_env
                    .type_identity_for_name(&imported_type.name)
                    .is_some()
            {
                continue;
            }
            if let Err(e) = type_env.register_type_identity(&imported_type) {
                errors.push(format!("imported type '{}': {e}", imported_type.name));
            }
        }

        let mut public_summary = type_metadata.summary;
        public_summary
            .exported_types
            .retain(|summary| matches!(summary.visibility, ash_core::ast::Visibility::Public));
        public_summary
            .exported_constructors
            .retain(|summary| matches!(summary.visibility, ash_core::ast::Visibility::Public));
        if let Err(e) = type_env.register_module_semantic_summary(&public_summary) {
            errors.push(format!("{e}"));
        }

        Ok(ModuleFileCheckResult {
            type_count,
            fn_count,
            warnings,
            errors,
        })
    }
    async fn execute_expr_with_bindings(
        &self,
        expr: &Expr,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        let policy_eval = PolicyEvaluator::new();
        let admitted_bindings = self.runtime_state.admitted_capability_binding_ids().await;
        let act_cap_ctx = self
            .runtime_state
            .create_capability_context_for_bindings(&admitted_bindings)
            .await?;
        let ctx = Context::with_bindings(input_bindings)
            .with_admitted_capability_bindings(admitted_bindings)
            .with_runtime_state(self.runtime_state.clone())
            .with_act_env(ash_interp::act_env::ActEnv::new(
                act_cap_ctx,
                policy_eval,
                Provenance::new(),
            ));
        eval_expr_async(expr, &ctx).await.map_err(ExecError::from)
    }

    /// Execute an entry asynchronously.
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute(&self, application: &Entry) -> ExecResult<Value> {
        self.execute_expr_with_bindings(&application.core, application.imported_closures.clone())
            .await
    }

    /// Admit and execute a application through the application-boundary carrier substrate.
    #[allow(clippy::too_many_lines)]
    pub async fn admit_application(
        &self,
        request: ApplicationAdmissionRequest,
    ) -> ApplicationAdmissionOutcome {
        if let Some(active_role) = request.active_role.as_deref() {
            let application_id = request.application_id.unwrap_or_default();
            let run_id = request.run_id.unwrap_or_default();
            let admitted_capability_bindings = self
                .runtime_state
                .resolve_admitted_capability_bindings(&request.required_capabilities)
                .await;
            let admission = ApplicationAdmissionContext {
                active_role: Some(active_role.to_string()),
                admitted_capabilities: request.required_capabilities.clone(),
                admitted_capability_bindings,
                requires_evidence: Vec::new(),
            };
            let ensures_evidence = build_pending_ensures_evidence(&request.ensures);
            let Some(admitted_role) = request.admitted_role.as_ref() else {
                return reject_admission(
                    application_id,
                    run_id,
                    ApplicationFailureKind::RoleAdmissionFailure,
                    admission,
                    Vec::new(),
                    ensures_evidence,
                );
            };
            if admitted_role.name != active_role {
                return reject_admission(
                    application_id,
                    run_id,
                    ApplicationFailureKind::RoleAdmissionFailure,
                    admission,
                    Vec::new(),
                    ensures_evidence,
                );
            }
        }

        let application_id = request.application_id.unwrap_or_default();
        let run_id = request.run_id.unwrap_or_default();
        let admitted_capability_bindings = self
            .runtime_state
            .resolve_admitted_capability_bindings(&request.required_capabilities)
            .await;
        let admission = ApplicationAdmissionContext {
            active_role: admitted_role_name(&request).map(ToOwned::to_owned),
            admitted_capabilities: request.required_capabilities.clone(),
            admitted_capability_bindings: admitted_capability_bindings.clone(),
            requires_evidence: Vec::new(),
        };
        let ensures_evidence = build_pending_ensures_evidence(&request.ensures);

        for requirement in &request.requires {
            match requirement {
                ApplicationContractRequirement::Role(required_role)
                    if admitted_role_name(&request) != Some(required_role.as_str()) =>
                {
                    return reject_admission(
                        application_id,
                        run_id,
                        ApplicationFailureKind::RoleAdmissionFailure,
                        admission.clone(),
                        Vec::new(),
                        ensures_evidence.clone(),
                    );
                }
                ApplicationContractRequirement::Capability(required_capability)
                    if !request.required_capabilities.contains(required_capability) =>
                {
                    return reject_admission(
                        application_id,
                        run_id,
                        ApplicationFailureKind::CapabilityAdmissionFailure,
                        admission.clone(),
                        Vec::new(),
                        ensures_evidence.clone(),
                    );
                }
                ApplicationContractRequirement::Role(_)
                | ApplicationContractRequirement::Capability(_)
                | ApplicationContractRequirement::Evidence { .. } => {}
            }
        }

        let requires_evidence = build_requires_evidence(&request.requires);
        if requires_evidence
            .iter()
            .any(|entry| entry.status == ApplicationEvidenceStatus::Failed)
        {
            return reject_admission(
                application_id,
                run_id,
                ApplicationFailureKind::RequiresViolation,
                admission,
                requires_evidence,
                ensures_evidence,
            );
        }

        let mut ctx = Context::new();
        if let Some(admitted_role) = request.admitted_role.clone() {
            ctx = ctx.with_role_context(RoleContext::new(admitted_role));
        }
        ctx = ctx.with_admitted_capability_bindings(admitted_capability_bindings.clone());
        let act_cap_ctx = self
            .runtime_state
            .create_capability_context_for_bindings(&admitted_capability_bindings)
            .await
            .unwrap_or_else(|_| ash_interp::capability::CapabilityContext::new());
        ctx = ctx.with_act_env(ash_interp::act_env::ActEnv::new(
            act_cap_ctx,
            PolicyEvaluator::new(),
            Provenance::new(),
        ));
        let execution_result = eval_expr_async(&request.body, &ctx)
            .await
            .map_err(ExecError::from);
        let execution_record = self.runtime_state.last_execution_record().await;
        let execution_record_ref = execution_record.as_ref();
        let outcome = match execution_result {
            Ok(value) => admitted_completion_outcome(
                application_id,
                run_id,
                value,
                admission,
                requires_evidence,
                &request.ensures,
                execution_record_ref,
            ),
            Err(error) => failed_boundary_outcome_from_exec_error(
                application_id,
                run_id,
                &error,
                admission,
                requires_evidence,
                ensures_evidence,
                execution_record_ref,
            ),
        };

        ApplicationAdmissionOutcome::Admitted {
            boundary: AdmittedApplicationBoundary::new(outcome),
        }
    }

    /// Execute an entry asynchronously with input bindings
    ///
    /// The input bindings are injected into the application's execution context
    /// as initial variable bindings. This is useful for passing CLI arguments
    /// or other external inputs to the application.
    ///
    /// # Arguments
    /// * `application` - The application to execute
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute_with_input(
        &self,
        application: &Entry,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        // Merge imported closures with input bindings (input takes precedence)
        let mut bindings = application.imported_closures.clone();
        bindings.extend(input_bindings);
        self.execute_expr_with_bindings(&application.core, bindings)
            .await
    }
    /// Parse, check, and execute in one call
    ///
    /// Convenience method that chains parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered at any stage.
    pub async fn run(&self, source: &str) -> ExecResult<Value> {
        let mut application = self.parse(source)?;
        self.check(&mut application)?;
        self.execute(&application).await
    }

    /// Parse, check, and execute a application from a file
    ///
    /// Convenience method that reads a file and then runs parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file(&self, path: impl AsRef<std::path::Path>) -> ExecResult<Value> {
        let mut application = self.parse_file(path)?;
        self.check(&mut application)?;
        self.execute(&application).await
    }

    /// Parse, check, and execute an entry source file with input bindings
    ///
    /// Convenience method that reads a file and then runs parse → check → execute
    /// with the provided input bindings injected into the execution context.
    ///
    /// # Arguments
    /// * `path` - Path to the entry source file
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file_with_input(
        &self,
        path: impl AsRef<std::path::Path>,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        let mut application = self.parse_file(path)?;
        self.check(&mut application)?;
        self.execute_with_input(&application, input_bindings).await
    }

    /// Parse, check, verify, and execute an entry source, returning its exit code.
    ///
    /// This is the narrow Phase 57 runtime entry path. It loads the engine-owned runtime
    /// stdlib registry, parses entry source with leading-`use` tolerance, checks the
    /// entry, validates the canonical `main` signature, executes it, and derives the
    /// observable process exit code from the terminal result payload.
    ///
    /// # Errors
    ///
    /// Returns [`EntryBootstrapError`] if stdlib loading fails, the entry source does not
    /// parse or type-check, the `main` contract is invalid, execution fails, or the runtime
    /// error payload carries an out-of-range exit code.
    pub async fn bootstrap_entry_source(&self, source: &str) -> Result<u8, EntryBootstrapError> {
        self.load_runtime_stdlib()?;
        let mut application = self.parse_entry_source(source)?;
        self.verify_entry_definition(&application)?;
        self.check(&mut application)?;
        let program = self
            .get_surface_program(application.id)
            .ok_or(EntryVerificationError::MissingApplicationMetadata)?;
        let def = program_entry_function(&program).ok_or(EntryVerificationError::MissingMain)?;
        let input_bindings = entry::entry_input_bindings(def);

        let result = if input_bindings.is_empty() {
            self.execute(&application)
                .await
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        } else {
            self.execute_with_input(&application, input_bindings)
                .await
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        };

        entry::derive_entry_exit_code(&result)
    }

    /// Parse, check, verify, and execute an entry file, returning its exit code.
    ///
    /// # Errors
    ///
    /// Returns [`EntryBootstrapError`] for any stdlib, I/O, parse, type, verification,
    /// execution, or exit-code derivation failure.
    pub async fn bootstrap_entry_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<u8, EntryBootstrapError> {
        let source = std::fs::read_to_string(path).map_err(EngineError::Io)?;
        self.bootstrap_entry_source(&source).await
    }
}

fn register_imported_semantic_summaries(
    type_env: &mut ash_typeck::TypeEnv,
    summaries: &[ash_core::semantic_summary::ModuleSemanticSummary],
) -> Result<(), EngineError> {
    type_env
        .register_module_semantic_summaries_and_discharge_required_propositions(summaries)
        .map(|_| ())
        .map_err(|error| {
            EngineError::Type(format!("imported proposition summary error: {error}"))
        })?;
    Ok(())
}

fn expose_imported_type_function_heads(
    type_env: &mut ash_typeck::TypeEnv,
    heads: Vec<(String, ash_core::type_ir::TypeComputationHeadId)>,
) -> Result<(), EngineError> {
    for (name, head) in heads {
        type_env
            .expose_imported_type_function_name(name, head)
            .map_err(|error| {
                EngineError::Type(format!("imported type function visibility error: {error}"))
            })?;
    }
    Ok(())
}

fn register_imported_type_defs(
    type_env: &mut ash_typeck::TypeEnv,
    imported_type_defs: Vec<ash_core::ast::TypeDef>,
) -> Result<(), EngineError> {
    for imported_type in &imported_type_defs {
        if !type_env.has_type(&imported_type.name) {
            type_env.declare_type_name(&imported_type.name);
        }
    }
    for imported_type in imported_type_defs {
        if type_env.has_full_type(&imported_type.name)
            || type_env
                .type_identity_for_name(&imported_type.name)
                .is_some()
        {
            continue;
        }

        type_env
            .register_type_identity(&imported_type)
            .map_err(|error| EngineError::Type(error.to_string()))?;
        if matches!(imported_type.visibility, ash_core::ast::Visibility::Public) {
            type_env
                .expose_type_representation(&imported_type.name)
                .map_err(|error| EngineError::Type(error.to_string()))?;
        }
    }
    Ok(())
}

fn surface_path_to_core_path(path: &[ash_parser::surface::Name]) -> Vec<String> {
    path.iter().map(ToString::to_string).collect()
}

fn surface_type_to_core_type(ty: &SurfaceType) -> Result<CoreType, String> {
    match ty {
        SurfaceType::Hole { span } => Err(format!(
            "type holes cannot lower to Core callable type metadata at {span:?}"
        )),
        SurfaceType::Name(name) => Ok(match name.as_ref() {
            "Int" | "String" | "Bool" | "Time" | "Ref" | "Unit" => CoreType::Base(name.to_string()),
            "()" => CoreType::Base("Unit".to_string()),
            other => CoreType::Named(other.to_string()),
        }),
        SurfaceType::List(item) => Ok(CoreType::App {
            name: "List".to_string(),
            args: vec![surface_type_to_core_type(item)?],
        }),
        SurfaceType::Tuple(items) => items
            .iter()
            .map(surface_type_to_core_type)
            .collect::<Result<Vec<_>, _>>()
            .map(CoreType::Tuple),
        SurfaceType::Record(fields) => fields
            .iter()
            .map(|(name, ty)| surface_type_to_core_type(ty).map(|ty| (name.to_string(), ty)))
            .collect::<Result<Vec<_>, _>>()
            .map(CoreType::Record),
        SurfaceType::Capability(name) => Ok(CoreType::Named(name.to_string())),
        SurfaceType::Constructor { name, args } => {
            let args = args
                .iter()
                .map(surface_type_to_core_type)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CoreType::App {
                name: name.to_string(),
                args,
            })
        }
        SurfaceType::Fn(params, row, ret) => {
            let params = params
                .iter()
                .map(surface_type_to_core_type)
                .collect::<Result<Vec<_>, _>>()?;
            let result = Box::new(surface_type_to_core_type(ret)?);
            let row = row.as_ref().map_or_else(
                || Ok(CoreRow::default()),
                surface_computation_row_to_core_row,
            )?;
            Ok(CoreType::Function {
                params,
                result,
                row,
            })
        }
        SurfaceType::Associated { .. } => {
            Err("associated types cannot lower to Core callable type metadata yet".to_string())
        }
        SurfaceType::AssociatedFamilyProjection { .. } => Err(
            "associated family projections cannot lower to Core callable type metadata yet"
                .to_string(),
        ),
    }
}

fn surface_type_to_core_type_lossy(ty: &SurfaceType) -> CoreType {
    match ty {
        SurfaceType::Hole { .. } => CoreType::Var("_".to_string()),
        SurfaceType::Name(name) => match name.as_ref() {
            "Int" | "String" | "Bool" | "Time" | "Ref" | "Unit" => CoreType::Base(name.to_string()),
            "()" => CoreType::Base("Unit".to_string()),
            other => CoreType::Named(other.to_string()),
        },
        SurfaceType::List(item) => CoreType::App {
            name: "List".to_string(),
            args: vec![surface_type_to_core_type_lossy(item)],
        },
        SurfaceType::Tuple(items) => {
            CoreType::Tuple(items.iter().map(surface_type_to_core_type_lossy).collect())
        }
        SurfaceType::Record(fields) => CoreType::Record(
            fields
                .iter()
                .map(|(name, ty)| (name.to_string(), surface_type_to_core_type_lossy(ty)))
                .collect(),
        ),
        SurfaceType::Capability(name) => CoreType::Named(name.to_string()),
        SurfaceType::Constructor { name, args } => CoreType::App {
            name: name.to_string(),
            args: args.iter().map(surface_type_to_core_type_lossy).collect(),
        },
        SurfaceType::Fn(params, row, ret) => CoreType::Function {
            params: params.iter().map(surface_type_to_core_type_lossy).collect(),
            result: Box::new(surface_type_to_core_type_lossy(ret)),
            row: row
                .as_ref()
                .and_then(|row| surface_computation_row_to_core_row(row).ok())
                .unwrap_or_default(),
        },
        SurfaceType::Associated { name, .. } => CoreType::Named(format!("Associated::{name}")),
        SurfaceType::AssociatedFamilyProjection {
            interface, member, ..
        } => CoreType::Named(format!("<{interface}>::{member}")),
    }
}

fn surface_computation_row_to_core_row(
    row: &ash_parser::surface::ComputationRow,
) -> Result<CoreRow, String> {
    let mut items = Vec::new();
    let mut tail = None;

    for item in &row.items {
        match item {
            ash_parser::surface::ComputationRowItem::Operation { path, .. } => {
                let (operation, path) = path
                    .split_last()
                    .ok_or_else(|| "operation row item has no operation name".to_string())?;
                items.push(CoreRowItem::operation(
                    surface_path_to_core_path(path),
                    operation.to_string(),
                ));
            }
            ash_parser::surface::ComputationRowItem::WholeRow { variable, .. }
            | ash_parser::surface::ComputationRowItem::Tail { variable, .. } => {
                if tail.replace(variable.to_string()).is_some() {
                    return Err(format!("duplicate Core row tail '{variable}'"));
                }
            }
            ash_parser::surface::ComputationRowItem::Resource { path, mode, .. } => {
                items.push(CoreRowItem::Resource {
                    path: surface_path_to_core_path(path),
                    mode: mode
                        .as_ref()
                        .map_or_else(|| "use".to_string(), ToString::to_string),
                });
            }
            ash_parser::surface::ComputationRowItem::Role { path, .. } => {
                items.push(CoreRowItem::Role {
                    path: surface_path_to_core_path(path),
                });
            }
            ash_parser::surface::ComputationRowItem::Policy { path, .. } => {
                items.push(CoreRowItem::Policy {
                    path: surface_path_to_core_path(path),
                });
            }
            ash_parser::surface::ComputationRowItem::Channel {
                mode,
                path,
                payload,
                ..
            } => {
                items.push(CoreRowItem::Channel {
                    path: surface_path_to_core_path(path),
                    mode: mode
                        .as_ref()
                        .map_or_else(|| "send".to_string(), ToString::to_string),
                    payload_type: Box::new(payload.as_ref().map_or_else(
                        || Ok(CoreType::Base("Unit".to_string())),
                        surface_type_to_core_type,
                    )?),
                });
            }
            ash_parser::surface::ComputationRowItem::Process { operation, .. } => {
                items.push(CoreRowItem::Process {
                    operation: operation
                        .as_ref()
                        .map_or_else(|| "spawn".to_string(), ToString::to_string),
                });
            }
            ash_parser::surface::ComputationRowItem::Fail { path, .. } => {
                items.push(CoreRowItem::Failure {
                    ty: path.as_ref().map(|path| {
                        Box::new(CoreType::Named(
                            path.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join("."),
                        ))
                    }),
                });
            }
            ash_parser::surface::ComputationRowItem::Evidence { path, .. } => {
                items.push(CoreRowItem::Evidence {
                    path: surface_path_to_core_path(path),
                });
            }
            ash_parser::surface::ComputationRowItem::Group { path, .. } => {
                items.push(CoreRowItem::EffectGroupRef {
                    path: surface_path_to_core_path(path),
                });
            }
        }
    }

    Ok(CoreRow { items, tail })
}

fn callable_row_for_core_type(
    return_type: &SurfaceType,
    proposition_tail: Option<&ash_parser::surface::PropositionTail>,
) -> Result<CoreRow, String> {
    match (
        module_loader::callable_inline_return_row(Some(return_type)),
        proposition_tail.and_then(|tail| tail.row.as_ref()),
    ) {
        (Some(_), Some(_)) => {
            Err("duplicate inline and expanded callable rows cannot lower to Core".to_string())
        }
        (Some(row), None) => surface_computation_row_to_core_row(row),
        (None, Some(row)) => surface_computation_row_to_core_row(&row.row),
        (None, None) => Ok(CoreRow::default()),
    }
}

fn core_callable_type_from_fn_def(
    function: &ash_parser::surface::FnDef,
) -> Result<CoreType, String> {
    let has_explicit_row = module_loader::callable_row_requirement_from_fn_def(function).is_some();
    let params = function
        .params
        .iter()
        .map(|param| {
            if has_explicit_row {
                surface_type_to_core_type(&param.ty)
            } else {
                Ok(surface_type_to_core_type_lossy(&param.ty))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let Some(return_type) = function.return_type.as_ref() else {
        return Ok(CoreType::Function {
            params,
            result: Box::new(CoreType::Var(format!("{}_return", function.name))),
            row: CoreRow::default(),
        });
    };
    let result = Box::new(match return_type {
        SurfaceType::Fn(params, row, ret) if params.is_empty() && row.is_some() => {
            surface_type_to_core_type(ret)?
        }
        other if has_explicit_row => surface_type_to_core_type(other)?,
        other => surface_type_to_core_type_lossy(other),
    });
    let row = callable_row_for_core_type(return_type, function.proposition_tail.as_ref())?;
    Ok(CoreType::Function {
        params,
        result,
        row,
    })
}

fn core_callable_type_from_builtin(
    builtin: &ash_parser::surface::BuiltinFnDef,
) -> Result<CoreType, String> {
    let params = builtin
        .params
        .iter()
        .map(|param| surface_type_to_core_type(&param.ty))
        .collect::<Result<Vec<_>, _>>()?;
    let result = Box::new(match &builtin.return_type {
        SurfaceType::Fn(params, row, ret) if params.is_empty() && row.is_some() => {
            surface_type_to_core_type(ret)?
        }
        other => surface_type_to_core_type(other)?,
    });
    let row = callable_row_for_core_type(&builtin.return_type, builtin.proposition_tail.as_ref())?;
    Ok(CoreType::Function {
        params,
        result,
        row,
    })
}
impl Default for Engine {
    /// Creates a default engine with standard configuration.
    ///
    /// # Panics
    ///
    /// Panics if the engine cannot be built (e.g., out of memory). This is
    /// extremely unlikely in practice since the default configuration requires
    /// no external resources.
    fn default() -> Self {
        // SAFETY: The default EngineBuilder configuration is infallible.
        // It only allocates memory and performs simple initializations.
        Self::new().build().expect("default engine builds")
    }
}

/// Configuration for HTTP capabilities
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    /// Timeout for HTTP requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl HttpConfig {
    /// Create a new HTTP config with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout_seconds: 30,
            max_redirects: 10,
            verify_ssl: true,
        }
    }
}

/// Builder for configuring and constructing an Engine
///
/// The builder pattern allows for fluent configuration of capabilities:
///
/// ```
/// use ash_engine::Engine;
///
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .with_fs_capabilities()
///     .build()
///     .expect("engine builds");
/// ```
#[derive(Debug, Default)]
pub struct EngineBuilder {
    /// Whether to enable stdio capabilities
    enable_stdio: bool,
    /// Whether to enable filesystem capabilities
    enable_fs: bool,
    /// HTTP configuration if enabled
    http_config: Option<HttpConfig>,
    /// Custom providers to register (using the unified `CapabilityProvider` trait)
    custom_providers: std::collections::HashMap<
        String,
        std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
    >,
    /// `RuntimeKernel` capability bindings to admit for custom providers.
    custom_provider_bindings: Vec<CapabilityBinding>,
    /// Host-selected capability implementation recipes keyed by binding name.
    capability_implementation_selections: Vec<(String, String)>,
    /// Host-selected resource initializers keyed by resource type/name.
    resource_initializer_selections: Vec<(String, String)>,
}

fn custom_provider_admitted_capabilities(
    registration_name: &str,
    provider: &dyn ash_core::capability::CapabilityProvider,
) -> Vec<String> {
    let metadata = provider.provider_metadata();
    if ash_core::capability::validate_provider_authoring_metadata(&metadata).is_err() {
        return Vec::new();
    }

    let mut capabilities = metadata
        .operations
        .iter()
        .flat_map(|operation| operation.required_rows.iter())
        .map(|row| {
            if metadata.provider_name != registration_name
                && let Some((_, operation_name)) = row.split_once('.')
            {
                format!("{registration_name}.{operation_name}")
            } else {
                row.clone()
            }
        })
        .collect::<Vec<_>>();
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

impl EngineBuilder {
    /// Create a new engine builder
    ///
    /// Prefer using `Engine::new()` instead of this method directly.
    fn new() -> Self {
        Self::default()
    }

    /// Build the configured engine
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the engine cannot be constructed
    /// (e.g., missing required capabilities or invalid configuration).
    pub fn build(self) -> Result<Engine, EngineError> {
        use providers::{
            FsProvider, HttpConfig as ProviderHttpConfig, HttpProvider, StdioProvider,
        };
        use std::sync::Arc;

        // Providers are stored as the unified trait type from ash_core
        let mut providers: std::collections::HashMap<
            String,
            Arc<dyn ash_core::capability::CapabilityProvider>,
        > = std::collections::HashMap::new();

        // Register stdio provider if enabled
        if self.enable_stdio {
            let provider = StdioProvider::new();
            providers.insert(provider.name().to_string(), Arc::new(provider));
        }

        // Register filesystem provider if enabled
        if self.enable_fs {
            let provider: Arc<dyn ash_core::capability::CapabilityProvider> =
                Arc::new(FsProvider::new());
            providers.insert("fs".to_string(), Arc::clone(&provider));
            // Also register under stdlib capability names for directory and metadata operations
            providers.insert("dir".to_string(), Arc::clone(&provider));
            providers.insert("meta".to_string(), provider);
        }

        // Register HTTP provider if configured
        if let Some(config) = self.http_config {
            let provider = HttpProvider::with_config(
                ProviderHttpConfig::new().with_timeout(config.timeout_seconds),
            );
            providers.insert("http".to_string(), Arc::new(provider));
        }

        // Register custom providers (these can override built-ins)
        for (name, provider) in self.custom_providers {
            providers.insert(name, provider);
        }

        // Build the RuntimeState with the unified providers
        let runtime_state = RuntimeState::new().with_providers(providers);
        for binding in self.custom_provider_bindings {
            futures::executor::block_on(runtime_state.admit_capability_binding(binding))
                .map_err(|error| EngineError::Configuration(error.to_string()))?;
        }
        let capability_implementation_selections = selections_to_map(
            self.capability_implementation_selections,
            "capability implementation selection",
        )?;
        let resource_initializer_selections = selections_to_map(
            self.resource_initializer_selections,
            "resource initializer selection",
        )?;

        Ok(Engine {
            imported_type_defs: std::sync::Mutex::new(std::collections::HashMap::new()),
            imported_semantic_summaries: std::sync::Mutex::new(std::collections::HashMap::new()),
            imported_type_function_heads: std::sync::Mutex::new(std::collections::HashMap::new()),
            surface_programs: std::sync::Mutex::new(std::collections::HashMap::new()),
            surface_program_module_identities: std::sync::Mutex::new(
                std::collections::HashMap::new(),
            ),
            runtime_stdlib_modules: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
            runtime_state,
            capability_implementation_selections,
            resource_initializer_selections,
        })
    }

    /// Add standard I/O capabilities (print, println, `read_line`)
    ///
    /// These are operational-effect capabilities for console I/O.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_stdio_capabilities(mut self) -> Self {
        self.enable_stdio = true;
        self
    }

    /// Add filesystem capabilities (`read_file`, `write_file`)
    ///
    /// These are operational-effect capabilities for file operations.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_fs_capabilities(mut self) -> Self {
        self.enable_fs = true;
        self
    }

    /// Configure HTTP capabilities
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::{Engine, HttpConfig};
    ///
    /// let result = Engine::new()
    ///     .with_http_capabilities(HttpConfig::new())
    ///     .build();
    /// assert!(result.is_ok());
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_http_capabilities(mut self, config: HttpConfig) -> Self {
        self.http_config = Some(config);
        self
    }

    /// Add a custom capability provider
    ///
    /// Custom providers can be used to extend the engine with application-specific
    /// capabilities. They can also override built-in providers by using the same name.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    /// use ash_core::{Constraint, Effect, Value};
    /// use ash_core::capability::{
    ///     CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
    /// };
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    ///
    /// #[derive(Debug)]
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl CapabilityProvider for MyProvider {
    ///     fn name(&self) -> &str { "my_provider" }
    ///     fn effect(&self) -> Effect { Effect::Operational }
    ///     fn provider_metadata(&self) -> ProviderAuthoringMetadata {
    ///         ProviderAuthoringMetadata::new("my_provider").with_operation(
    ///             ProviderOperationMetadata::new("*", Effect::Operational)
    ///                 .with_required_row("my_provider.*")
    ///                 .with_sandbox_policy("host.my_provider.test")
    ///                 .with_provenance_policy("host.my_provider.test.redacted"),
    ///         )
    ///     }
    ///     async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
    ///         Ok(Value::Null)
    ///     }
    ///     async fn execute(
    ///         &self,
    ///         _action_name: &str,
    ///         _args: &[Value],
    ///     ) -> Result<Value, CapabilityError> {
    ///         Ok(Value::Null)
    ///     }
    /// }
    ///
    /// let engine = Engine::new()
    ///     .with_custom_provider("custom", Arc::new(MyProvider))
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_custom_provider(
        mut self,
        name: &str,
        provider: std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
    ) -> Self {
        self.custom_providers
            .insert(name.to_string(), provider.clone());
        if !name.is_empty() && provider.effect().at_least(ash_core::Effect::Operational) {
            let admitted_capabilities = custom_provider_admitted_capabilities(name, &*provider);
            self.custom_provider_bindings
                .retain(|binding| binding.name != name);
            self.custom_provider_bindings
                .push(CapabilityBinding::host_provider(
                    CapabilityBindingId::new(),
                    name,
                    CapabilityInterfaceId::new(name),
                    name,
                    admitted_capabilities,
                ));
        }
        self
    }

    /// Select an Ash-defined capability implementation recipe for a host binding name.
    #[must_use]
    pub fn with_capability_implementation(
        mut self,
        binding: impl Into<String>,
        implementation: impl Into<String>,
    ) -> Self {
        self.capability_implementation_selections
            .push((binding.into(), implementation.into()));
        self
    }

    /// Select a host resource initializer for a named resource type.
    #[must_use]
    pub fn with_resource_initializer(
        mut self,
        resource: impl Into<String>,
        initializer: impl Into<String>,
    ) -> Self {
        self.resource_initializer_selections
            .push((resource.into(), initializer.into()));
        self
    }

    /// Configure LLM capabilities with the given provider configurations
    ///
    /// Registers an `LlmProvider` that supports multi-provider routing for OpenAI-compatible APIs.
    ///
    /// # Arguments
    /// * `configs` - Map of provider name to `LlmConfig` (e.g., "openai" -> config, "ollama" -> config)
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    /// use ash_engine::providers::llm::LlmConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut configs = HashMap::new();
    /// configs.insert("openai".to_string(), LlmConfig::openai("sk-xxx"));
    /// configs.insert("ollama".to_string(), LlmConfig::ollama());
    ///
    /// let engine = Engine::new()
    ///     .with_llm_capabilities(configs)
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    pub fn with_llm_capabilities(
        mut self,
        configs: std::collections::HashMap<String, crate::providers::llm::LlmConfig>,
    ) -> Self {
        match crate::providers::llm::LlmProvider::new(configs) {
            Ok(provider) => {
                self.custom_providers
                    .insert("llm".to_string(), std::sync::Arc::new(provider));
            }
            Err(e) => {
                // Log warning and skip registration
                eprintln!("Warning: Failed to create LLM provider: {e}");
            }
        }
        self
    }
}

fn selections_to_map(
    selections: Vec<(String, String)>,
    label: &str,
) -> Result<HashMap<String, String>, EngineError> {
    let mut map = HashMap::new();
    for (key, value) in selections {
        if key.trim().is_empty() || value.trim().is_empty() {
            return Err(EngineError::Configuration(format!(
                "invalid {label}: names must be non-empty"
            )));
        }
        if map.insert(key.clone(), value).is_some() {
            return Err(EngineError::Configuration(format!(
                "duplicate {label} for '{key}'"
            )));
        }
    }
    Ok(map)
}

fn declared_capability_implementation_names(source: &str) -> HashSet<String> {
    declared_names_after_marker(source, &["pub", "capability", "impl"])
}

fn declared_resource_type_names(source: &str) -> HashSet<String> {
    declared_names_after_marker(source, &["pub", "resource", "type"])
}

fn declared_names_after_marker(source: &str, marker: &[&str]) -> HashSet<String> {
    let words: Vec<&str> = source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|word| !word.is_empty())
        .collect();
    let mut names = HashSet::new();
    for window in words.windows(marker.len() + 1) {
        if window.starts_with(marker) {
            names.insert(window[marker.len()].to_string());
        }
    }
    names
}

/// Bind imported callable type signatures into a type environment.
///
/// For each entry in `application.imported_param_counts`, checks whether a
/// declared builtin signature is available in `application.imported_builtin_signatures`.
/// If so, uses `builtin_fn_signature_type` to produce the precise polymorphic type.
/// If a declared signature exists, signature conversion errors are hard type
/// errors rather than silently falling back to arity-only types.
#[allow(clippy::unnecessary_wraps)]
fn bind_imported_callable_types(
    type_env: &mut ash_typeck::type_env::TypeEnv,
    application: &Entry,
) -> Result<(), EngineError> {
    for (name, &param_count) in &application.imported_param_counts {
        if let Some(sig) = application.imported_fn_signatures.get(name) {
            let ty = ash_typeck::fn_signature_type(type_env, sig).map_err(|error| {
                EngineError::Type(format!(
                    "failed to resolve imported function signature for '{name}': {error}"
                ))
            })?;
            type_env.bind_variable(name, ty);
            continue;
        }
        if let Some(sig) = application.imported_builtin_signatures.get(name) {
            let ty = ash_typeck::builtin_fn_signature_type(type_env, sig).map_err(|error| {
                EngineError::Type(format!(
                    "failed to resolve imported builtin signature for '{name}': {error}"
                ))
            })?;
            type_env.bind_variable(name, ty);
            continue;
        }
        // Arity-only synthetic type (fresh type variables)
        let param_types: Vec<ash_typeck::Type> = (0..param_count)
            .map(|_| ash_typeck::Type::Var(ash_typeck::types::TypeVar::fresh()))
            .collect();
        let ret_type = ash_typeck::Type::Var(ash_typeck::types::TypeVar::fresh());
        type_env.bind_variable(name, ash_typeck::Type::Fn(param_types, Box::new(ret_type)));
    }
    Ok(())
}

/// Imported callable bindings built for runtime and type-checker integration.
type ImportedClosureBindings = (
    HashMap<String, Value>,
    HashMap<String, usize>,
    HashMap<String, CallableRowRequirementSummary>,
    HashMap<String, CoreType>,
    HashMap<String, ash_parser::surface::FnDef>,
    HashMap<String, ash_parser::surface::BuiltinFnDef>,
);

/// Convert imported callables to `Value::Closure` for runtime binding.
/// Each callable body is lowered from surface to core Expr, then wrapped in a closure.
///
/// Returns `(closures, param_counts, fn_signatures, builtin_signatures)` where
/// the signature maps carry declared type signatures for imported callables so
/// that `Engine::check()` can bind precise types instead of arity-only synthetics.
#[allow(clippy::too_many_lines)]
fn build_imported_closures(
    imported_callables: &HashMap<String, module_loader::InlineCallable>,
) -> Result<ImportedClosureBindings, EngineError> {
    use module_loader::CallableKind;

    struct ClosureSpec {
        name: String,
        params: Vec<(String, Option<String>)>,
        body: ash_core::Expr,
    }

    fn user_closure_spec(
        name: String,
        callable: &module_loader::InlineCallable,
        body: &ash_parser::surface::Expr,
    ) -> Option<ClosureSpec> {
        let lowering_ctx =
            ash_parser::LoweringContext::with_effectful_names(callable.effectful_names.clone());
        let body = match ash_parser::lower_expr_with_context(body, &lowering_ctx) {
            Ok(expr) => expr,
            Err(e) => {
                eprintln!("warning: failed to lower imported callable '{name}': {e}");
                return None;
            }
        };
        Some(ClosureSpec {
            name,
            params: callable.params.iter().map(|p| (p.clone(), None)).collect(),
            body,
        })
    }

    fn original_callable_name(name: &str, callable: &module_loader::InlineCallable) -> String {
        match callable.signature.as_ref() {
            Some(module_loader::CallableSignature::Function(function)) => function.name.to_string(),
            _ => name.to_string(),
        }
    }

    fn closure_with_module_family(
        export_spec: ClosureSpec,
        mut family_specs: HashMap<String, ClosureSpec>,
    ) -> Value {
        family_specs
            .entry(export_spec.name.clone())
            .or_insert_with(|| ClosureSpec {
                name: export_spec.name.clone(),
                params: export_spec.params.clone(),
                body: export_spec.body.clone(),
            });

        let mut module_env = ash_core::env_frame::EnvFrame::new();
        let mut late_slots = HashMap::new();
        for spec in family_specs.values() {
            let slot = module_env.insert_late(spec.name.clone());
            late_slots.insert(spec.name.clone(), slot);
        }
        let module_env = std::sync::Arc::new(module_env);

        for spec in family_specs.into_values() {
            let closure = Value::Closure {
                params: spec.params,
                body: Box::new(spec.body),
                env: module_env.clone(),
            };
            if let Some(slot) = late_slots.get(&spec.name) {
                slot.set_late(closure);
            }
        }

        Value::Closure {
            params: export_spec.params,
            body: Box::new(export_spec.body),
            env: module_env,
        }
    }

    // Resolved once; builtin_dispatch_table() uses OnceLock so this is a
    // single atomic load on subsequent calls, but hoisting it avoids calling
    // the function N times inside the loop.
    let dispatch_table = ash_interp::eval::builtin_dispatch_table();

    let mut closures = HashMap::new();
    let mut param_counts = HashMap::new();
    let mut callable_row_requirements = HashMap::new();
    let mut core_callable_types = HashMap::new();
    let mut fn_signatures = HashMap::new();
    let mut builtin_signatures = HashMap::new();
    for (name, callable) in imported_callables {
        let params: Vec<(String, Option<String>)> =
            callable.params.iter().map(|p| (p.clone(), None)).collect();
        param_counts.insert(name.clone(), params.len());
        if let Some(row_requirement) = &callable.row_requirement {
            callable_row_requirements.insert(name.clone(), row_requirement.clone());
        }

        if let Some(sig) = &callable.signature {
            match sig {
                module_loader::CallableSignature::Function(fn_def) => {
                    let core_type = core_callable_type_from_fn_def(fn_def).map_err(|error| {
                        EngineError::Parse(format!(
                            "failed to lower imported callable row for '{name}': {error}"
                        ))
                    })?;
                    core_callable_types.insert(name.clone(), core_type);
                    fn_signatures.insert(name.clone(), fn_def.clone());
                }
                module_loader::CallableSignature::Builtin(builtin) => {
                    let core_type = core_callable_type_from_builtin(builtin).map_err(|error| {
                        EngineError::Parse(format!(
                            "failed to lower imported builtin callable row for '{name}': {error}"
                        ))
                    })?;
                    core_callable_types.insert(name.clone(), core_type);
                    builtin_signatures.insert(name.clone(), builtin.clone());
                }
            }
        }

        let body_expr = match &callable.kind {
            CallableKind::User { body } => {
                let Some(export_spec) = user_closure_spec(name.clone(), callable, body) else {
                    continue;
                };
                let original_name = original_callable_name(name, callable);
                let mut family_specs = HashMap::new();
                for (module_name, module_callable) in &callable.module_runtime_callables {
                    let CallableKind::User { body } = &module_callable.kind else {
                        continue;
                    };
                    if let Some(spec) =
                        user_closure_spec(module_name.clone(), module_callable, body)
                    {
                        family_specs.insert(module_name.clone(), spec);
                    }
                }
                if !family_specs.contains_key(&original_name) {
                    family_specs.insert(
                        original_name.clone(),
                        ClosureSpec {
                            name: original_name,
                            params: export_spec.params.clone(),
                            body: export_spec.body.clone(),
                        },
                    );
                }
                closures.insert(
                    name.clone(),
                    closure_with_module_family(export_spec, family_specs),
                );
                continue;
            }
            CallableKind::Builtin { module } => {
                let dispatch_name = match callable.signature.as_ref() {
                    Some(module_loader::CallableSignature::Builtin(builtin)) => {
                        builtin.name.to_string()
                    }
                    _ => callable.exported_name.clone(),
                };
                let qualified = format!("{module}::{dispatch_name}");
                let call_module = if dispatch_table.contains_key(qualified.as_str()) {
                    Some(module.clone())
                } else {
                    None
                };

                let unqualified_entry = dispatch_table.get(dispatch_name.as_str());
                if callable.params.is_empty()
                    && let Some(entry) = unqualified_entry
                    && entry.variadic
                {
                    param_counts.insert(name.clone(), 0);
                    continue;
                }

                let param_exprs: Vec<ash_core::Expr> = callable
                    .params
                    .iter()
                    .map(|p| ash_core::Expr::Variable {
                        name: p.clone(),
                        span: ash_core::ast::Span::default(),
                    })
                    .collect();
                ash_core::Expr::Call {
                    func: dispatch_name,
                    module: call_module,
                    arguments: param_exprs,
                }
            }
        };
        let closure = Value::Closure {
            params,
            body: Box::new(body_expr),
            env: std::sync::Arc::new(ash_core::env_frame::EnvFrame::new()),
        };
        closures.insert(name.clone(), closure);
    }

    Ok((
        closures,
        param_counts,
        callable_row_requirements,
        core_callable_types,
        fn_signatures,
        builtin_signatures,
    ))
}

#[cfg(test)]
mod tests;
