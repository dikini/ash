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

pub mod checked_cps_admission;
pub mod differential;
pub mod entry;
pub mod error;
pub mod harness;
pub mod law_cache;
pub mod module_loader;
pub mod monomorphize;
pub(crate) mod operation;
mod production_cps_driver;
pub mod providers;
pub mod row_admission;
pub mod runtime_artifact;
pub mod standard_profiles;

pub use entry::{
    EntryBootstrapError, EntryBootstrapResult, EntryVerificationError, RuntimeEntryStdlibSource,
    derive_entry_exit_code, load_runtime_entry_stdlib_sources, verify_entry_definition,
};
pub use error::{EngineError, ProductionTerminalClassification};
pub use module_loader::{CallableRowRequirementSource, CallableRowRequirementSummary};
pub use production_cps_driver::{
    ProductionCancellation, ProductionCheckedCpsOutcome, ProductionRunControl,
};
// Re-export the unified CapabilityProvider trait from ash_core
pub use ash_core::capability::CapabilityProvider;

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr as CheckedCoreExpr, CoreMultiplicity, CorePrimOp,
    CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::runtime::{
    ApplicationAdmissionContext, ApplicationBoundaryOutcome, ApplicationContractCheckEvidence,
    ApplicationEvidenceStatus, ApplicationFailure, ApplicationFailureKind, ApplicationReport,
    FailureBoundary, FailureEntity, HostBoundaryEvidence, OperationalFailure, ProcessFailure,
    RunId,
};
use ash_core::runtime_kernel::CheckedFunctionArtifact;
use ash_core::semantic_summary::{ModuleSourceOrigin, SourceAnchor, SourceOrigin};
use ash_core::{
    ApplicationId, CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Expr, Role, Value,
};
use ash_interp::{EvalError, ExecError, ExecResult, ExecutionRecord, RuntimeState};
use ash_parser::surface::Type as SurfaceType;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::checked_cps_admission::{
    CheckedCpsAdmissionV1, CheckedCpsEntryAdmission, CheckedCpsProductionAdmission,
    CheckedSourceFactsV1, CoreHandleLocatorV1, FrameInstallationInstructionV1, OperationIdentityV1,
    ProviderBindingV1, ResolvedProviderBinding,
};
use crate::operation::TIME_SLEEP_OPERATION;

const CHECKED_CPS_ANSWER_CONTINUATION: &str = "__answer";
const HANDLER_INSPECTION_ANSWER_CONTINUATION: &str = "__handler_inspection_answer";
const HANDLER_INSPECTION_ANSWER_VALUE: &str = "__handler_inspection_answer_value";
const FORWARD_SLEEP_ANSWER_CONTINUATION: &str = "__forward_sleep_answer";
const FORWARD_SLEEP_ANSWER_VALUE: &str = "__forward_sleep_answer_value";
const SEALED_PRODUCTION_HANDLER_NAME: &str = "absorb_sleep";
const SEALED_TRAP_SLEEP_HANDLER_NAME: &str = "trap_sleep";
const SEALED_FORWARD_SLEEP_HANDLER_NAME: &str = "forward_sleep";
const SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME: &str = "deep_affine_clock";
const SOURCE_HANDLER_LOWERING_UNAVAILABLE: &str =
    "source handlers require typed handler lowering before Core lowering";
const SOURCE_HANDLER_LOWERING_PLACEHOLDER: &str = "__ash_source_handler_lowering_unavailable";
const CLOSED_CHECKED_CPS_ADMISSION_MESSAGE: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn closed_checked_cps_admission_error() -> ExecError {
    ExecError::ExecutionFailed(CLOSED_CHECKED_CPS_ADMISSION_MESSAGE.to_string())
}

fn missing_production_admission(error: &EngineError) -> EngineError {
    EngineError::production_terminal(
        ProductionTerminalClassification::MissingAdmission,
        error.to_string(),
    )
}

fn invalid_checked_core_cps(error: &EngineError) -> EngineError {
    EngineError::production_terminal(
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        error.to_string(),
    )
}

fn is_source_handler_lowering_unavailable(error: &ash_parser::LoweringError) -> bool {
    matches!(
        error,
        ash_parser::LoweringError::ExprNotLowerable {
            kind: SOURCE_HANDLER_LOWERING_UNAVAILABLE
        }
    )
}

fn cps_atom_to_engine_value(atom: ash_core::cps::Atom) -> ExecResult<Value> {
    match atom {
        ash_core::cps::Atom::Int(value) => Ok(Value::Int(value)),
        ash_core::cps::Atom::Float(value) => Ok(Value::Float(value)),
        ash_core::cps::Atom::String(value) => Ok(Value::String(value)),
        ash_core::cps::Atom::Bool(value) => Ok(Value::Bool(value)),
        ash_core::cps::Atom::Null => Ok(Value::Null),
        ash_core::cps::Atom::Var(name) | ash_core::cps::Atom::ConstructorName(name) => {
            Err(ExecError::ExecutionFailed(format!(
                "checked Core/CPS terminal atom '{name}' cannot cross the engine value boundary"
            )))
        }
    }
}

fn cps_value_to_engine_value(value: ash_core::cps::Value) -> ExecResult<Value> {
    match value {
        ash_core::cps::Value::Atom(atom) => cps_atom_to_engine_value(atom),
        ash_core::cps::Value::Record { fields } => fields
            .into_iter()
            .map(|(name, field)| cps_value_to_engine_value(field).map(|value| (name, value)))
            .collect::<ExecResult<HashMap<_, _>>>()
            .map(|fields| Value::Record(Box::new(fields))),
        ash_core::cps::Value::Constructor { name, fields } => fields
            .into_iter()
            .map(|(field_name, field)| {
                cps_value_to_engine_value(field).map(|value| (field_name, value))
            })
            .collect::<ExecResult<Vec<_>>>()
            .map(|fields| Value::Variant {
                name,
                fields: Box::new(fields),
            }),
        value => Err(ExecError::ExecutionFailed(format!(
            "checked Core/CPS terminal value cannot cross the engine value boundary: {value:?}"
        ))),
    }
}

fn checked_cps_term_has_handler_or_raise(term: &ash_core::cps::Term) -> bool {
    use ash_core::cps::Term;

    match term {
        Term::Raise { .. } | Term::Handle { .. } => true,
        Term::LetVal { value, body, .. } | Term::LetRec { value, body, .. } => {
            checked_cps_value_has_handler_or_raise(value)
                || checked_cps_term_has_handler_or_raise(body)
        }
        Term::LetPrim { body, .. }
        | Term::LetContCall { body, .. }
        | Term::RecordDischarge { body, .. } => checked_cps_term_has_handler_or_raise(body),
        Term::LetCont {
            cont_body, body, ..
        } => {
            checked_cps_term_has_handler_or_raise(cont_body)
                || checked_cps_term_has_handler_or_raise(body)
        }
        Term::If {
            then_branch,
            else_branch,
            ..
        } => {
            checked_cps_term_has_handler_or_raise(then_branch)
                || checked_cps_term_has_handler_or_raise(else_branch)
        }
        Term::Match { arms, default, .. } => {
            arms.iter()
                .any(|(_, body)| checked_cps_term_has_handler_or_raise(body))
                || default
                    .as_deref()
                    .is_some_and(checked_cps_term_has_handler_or_raise)
        }
        Term::Jump { .. }
        | Term::JumpValue { .. }
        | Term::Call { .. }
        | Term::Return { .. }
        | Term::Trap { .. } => false,
    }
}

fn checked_cps_value_has_handler_or_raise(value: &ash_core::cps::Value) -> bool {
    use ash_core::cps::Value as CpsValue;

    match value {
        CpsValue::Atom(_) => false,
        CpsValue::Lam { body, .. } | CpsValue::Cont { body, .. } => {
            checked_cps_term_has_handler_or_raise(body)
        }
        CpsValue::Record { fields } => fields
            .iter()
            .any(|(_, value)| checked_cps_value_has_handler_or_raise(value)),
        CpsValue::Tuple { elems } => elems.iter().any(checked_cps_value_has_handler_or_raise),
        CpsValue::Constructor { fields, .. } => fields
            .iter()
            .any(|(_, value)| checked_cps_value_has_handler_or_raise(value)),
        CpsValue::ThunkClosure { body, .. } => checked_cps_value_has_handler_or_raise(body),
    }
}

/// Identity of a checked concrete operation declaration.
///
/// This is deliberately constructed only from the typechecker's resolved
/// declaration carrier. It prevents provider dispatch from being selected by a
/// raw source name or an independently-derived row identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DeclaredOperationIdentity {
    impl_type: String,
    interface: String,
    operation: String,
    params: Vec<String>,
    result_type: String,
}

impl From<&ash_typeck::DeclaredConcreteOperation> for DeclaredOperationIdentity {
    fn from(operation: &ash_typeck::DeclaredConcreteOperation) -> Self {
        Self {
            impl_type: operation.impl_type.clone(),
            interface: operation.interface.clone(),
            operation: operation.operation.clone(),
            params: operation.params.iter().map(ToString::to_string).collect(),
            result_type: operation.result_type.to_string(),
        }
    }
}

/// The first declaration-backed provider route is intentionally fixed to the
/// `TestClock` fixture contract. Other declaration-resolved calls continue to
/// reject at production admission until they have their own checked route.
fn is_sealed_declared_production_operation(
    operation: &ash_typeck::DeclaredConcreteOperation,
) -> bool {
    operation.impl_type == "TestClock"
        && operation.interface == "Clock"
        && operation.operation == "sleep"
        && operation.params.iter().map(ToString::to_string).eq(["Int"])
        && operation.result_type.to_string() == "Null"
}

/// The first source-handler production route is intentionally fixed to the
/// closed-empty `absorb_sleep` fixture contract.  This identity is supplied
/// by the typechecker-owned clause fact, never inferred from source text or a
/// row item.
fn is_sealed_production_handler_operation(
    operation: &ash_typeck::DeclaredConcreteOperation,
) -> bool {
    operation.impl_type == "TestClock"
        && operation.interface == "Clock"
        && operation.operation == "sleep"
        && operation.params.iter().map(ToString::to_string).eq(["Int"])
        && operation.result_type.to_string() == "Int"
}

fn is_sealed_forward_sleep_operation(operation: &ash_typeck::DeclaredConcreteOperation) -> bool {
    operation.impl_type == "TestClock"
        && operation.interface == "Clock"
        && operation.operation == "sleep"
        && operation.params.iter().map(ToString::to_string).eq(["Int"])
        && operation.result_type.to_string() == "Int"
}

fn is_sealed_forward_wake_operation(operation: &ash_typeck::DeclaredConcreteOperation) -> bool {
    operation.impl_type == "TestClock"
        && operation.interface == "Clock"
        && operation.operation == "wake"
        && operation.params.iter().map(ToString::to_string).eq(["Int"])
        && operation.result_type.to_string() == "Int"
}

fn is_exact_forward_sleep_source_program(program: &ash_parser::surface::Program) -> bool {
    use ash_parser::surface::Definition;

    program.entry.function.as_ref() == "main"
        && program.definitions.len() == 5
        && matches!(program.definitions[0], Definition::Interface(_))
        && matches!(program.definitions[1], Definition::Type(_))
        && matches!(program.definitions[2], Definition::Impl(_))
        && matches!(program.definitions[3], Definition::Handler(_))
        && matches!(program.definitions[4], Definition::Function(_))
}

/// TASK-2013's first deep handler route is intentionally one closed source
/// fixture.  Its concrete operation identities come from checked facts below;
/// this structural check only preserves the source sequencing that those facts
/// cannot express.
fn is_exact_deep_affine_clock_source_program(program: &ash_parser::surface::Program) -> bool {
    use ash_parser::surface::{BlockStmt, Definition, Expr, HandlerClause, Literal, Pattern};

    let exact_call = |expression: &Expr, operation: &str, argument: i64| {
        matches!(expression,
            Expr::Call { module: Some(impl_type), func, args, .. }
                if impl_type.as_ref() == "TestClock"
                    && func.as_ref() == operation
                    && matches!(args.as_slice(), [Expr::Literal(Literal::Int(value))] if *value == argument)
        )
    };
    let direct_resume = |body: &Expr, parameter: &str| {
        matches!(body,
            Expr::Call { module: None, func, args, .. }
                if func.as_ref() == "resume"
                    && matches!(args.as_slice(), [Expr::Variable { name, .. }] if name.as_ref() == parameter)
        )
    };
    let exact_clause = |clause: &HandlerClause, operation: &str| {
        matches!(clause,
            HandlerClause::Operation { impl_type, operation: clause_operation, pattern: Pattern::Variable { name, .. }, resume, body, .. }
                if impl_type.as_ref() == "TestClock"
                    && clause_operation.as_ref() == operation
                    && name.as_ref() == "ms"
                    && resume.as_ref() == "resume"
                    && direct_resume(body, "ms")
        )
    };
    let exact_done = |clause: &HandlerClause| {
        matches!(clause,
            HandlerClause::Done { binding, body, .. }
                if binding.as_ref() == "value"
                    && matches!(body.as_ref(), Expr::Binary { op: ash_parser::surface::BinaryOp::Add, left, right, .. }
                        if matches!(left.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "value")
                            && matches!(right.as_ref(), Expr::Literal(Literal::Int(100))))
        )
    };

    let Some(Definition::Handler(handler)) = program.definitions.get(3) else {
        return false;
    };
    let Some(Definition::Function(main)) = program.definitions.get(4) else {
        return false;
    };
    let Expr::On { clauses, .. } = &handler.body else {
        return false;
    };
    let Expr::Block {
        statements: _,
        tail_expr: Some(tail),
        ..
    } = &main.body
    else {
        return false;
    };
    let Expr::HandleWith {
        expression,
        handler: applied_handler,
        ..
    } = tail.as_ref()
    else {
        return false;
    };
    let Expr::Block {
        statements: handled_statements,
        tail_expr: Some(handled_tail),
        ..
    } = expression.as_ref()
    else {
        return false;
    };

    program.entry.function.as_ref() == "main"
        && program.definitions.len() == 5
        && matches!(program.definitions[0], Definition::Interface(_))
        && matches!(program.definitions[1], Definition::Type(_))
        && matches!(program.definitions[2], Definition::Impl(_))
        && handler.name.as_ref() == SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME
        && applied_handler.as_ref() == SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME
        && main.name.as_ref() == "main"
        && main.params.is_empty()
        && matches!(clauses.as_slice(), [sleep, wake, done]
            if exact_clause(sleep, "sleep") && exact_clause(wake, "wake") && exact_done(done))
        && matches!(handled_statements.as_slice(),
            [
                BlockStmt::Expr { expr: first, .. },
                BlockStmt::Expr { expr: second, .. },
                BlockStmt::Expr { expr: third, .. },
            ] if exact_call(first, "sleep", 0)
                && exact_call(second, "wake", 1)
                && exact_call(third, "sleep", 2)
        )
        && matches!(handled_tail.as_ref(), Expr::Literal(Literal::Int(7)))
}

fn core_operation_from_declared(operation: &ash_typeck::DeclaredConcreteOperation) -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec![operation.impl_type.clone()],
        operation: operation.operation.clone(),
        arg_types: operation
            .params
            .iter()
            .map(|parameter| CoreType::Base(parameter.to_string()))
            .collect(),
        result_type: CoreType::Base(operation.result_type.to_string()),
    }
}

/// Host-selected provider target for one checked declared-operation identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclaredOperationProviderBinding {
    provider_name: String,
    provider_operation: String,
}

/// Registry entry for the sole provider-backed production slice currently
/// admitted by TASK-2014. The provider object is resolved at registration, so
/// a later row or public instruction summary cannot choose a host provider.
#[derive(Clone)]
struct RegisteredTimeSleepProviderBinding {
    binding: ProviderBindingV1,
    provider: std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
}

impl std::fmt::Debug for RegisteredTimeSleepProviderBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisteredTimeSleepProviderBinding")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

/// Engine-resolved authority for the first declaration-backed production
/// operation.  The public declared-operation binding remains useful for row
/// admission, but only this private carrier can authorize a provider frame.
#[derive(Clone)]
struct RegisteredDeclaredProductionProviderBinding {
    binding: ProviderBindingV1,
    provider: std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
}

/// Engine-resolved host authority for TASK-2026's one exact `wake` provider
/// frame.  It is separate from the generic declared-operation and single
/// provider registries because neither can authorize a handler frame chain.
#[derive(Clone)]
struct RegisteredForwardSleepWakeProviderBinding {
    binding: ProviderBindingV1,
    provider: std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
}

impl std::fmt::Debug for RegisteredForwardSleepWakeProviderBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisteredForwardSleepWakeProviderBinding")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for RegisteredDeclaredProductionProviderBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisteredDeclaredProductionProviderBinding")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

/// The implementation boundary used for production Ash execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionExecutionBoundary {
    /// Checked Core/CPS exclusively owns production execution. Source without
    /// a validated production artifact rejects at admission rather than
    /// falling back to the legacy [`Expr`] evaluator.
    CheckedCoreCpsClosedAdmission,
}

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
    /// Private identity shared only with entries parsed by this engine.
    entry_owner_token: std::sync::Arc<()>,
    /// Private identity sealing handler-inspection execution admissions issued
    /// by this engine. This is deliberately distinct from entry provenance:
    /// checked source evidence alone is not executable authority.
    handler_inspection_execution_token: std::sync::Arc<()>,
    /// Private issuer seal for provider-backed production admissions. This is
    /// distinct from both parsed-entry provenance and handler inspection.
    production_checked_cps_execution_token: std::sync::Arc<()>,
    /// Private issuer seal for the closed-empty source-handler production
    /// admission. This is distinct from inspection and provider execution.
    production_handler_execution_token: std::sync::Arc<()>,
    /// Private issuer seal for TASK-2026's ordered handler/provider route.
    production_forward_sleep_execution_token: std::sync::Arc<()>,
    /// Private issuer seal for TASK-2013's exact deep affine handler route.
    production_deep_affine_clock_execution_token: std::sync::Arc<()>,
    /// Canonical parsed source anchors keyed by Engine-issued entry identity.
    /// Public Entry sidecars are diagnostic data and never replace this record.
    canonical_entry_source_anchors: std::sync::Mutex<HashMap<u64, CanonicalEntrySourceAnchor>>,
    /// Successful checker output retained privately for checked source-fact projection.
    checked_type_results: std::sync::Mutex<HashMap<u64, CheckedEntryTypeResult>>,
    /// Narrow engine-owned registry of runtime stdlib module sources keyed by
    /// canonical module path.
    runtime_stdlib_modules: std::sync::Mutex<std::collections::HashMap<String, String>>,
    /// Counter for generating unique IDs
    next_id: std::sync::atomic::AtomicU64,
    /// Test-only observation of explicit checked Core-to-CPS bridge use.
    #[cfg(test)]
    checked_cps_inspection_calls: std::sync::atomic::AtomicU64,
    /// Runtime-owned state that persists across related executions.
    /// Providers configured via `EngineBuilder` are passed to `RuntimeState` during build.
    runtime_state: RuntimeState,
    /// Host-selected capability implementation recipes keyed by binding name.
    capability_implementation_selections: HashMap<String, String>,
    /// Host-selected resource initializers keyed by resource type/name.
    resource_initializer_selections: HashMap<String, String>,
    /// Explicit provider targets for typechecked concrete implementation operations.
    declared_operation_provider_bindings:
        std::sync::Mutex<HashMap<DeclaredOperationIdentity, DeclaredOperationProviderBinding>>,
    /// Engine-resolved authority for the sealed declaration-backed production
    /// slice. This remains narrower than the general declared-operation
    /// binding registry and is never reconstructed from public metadata.
    declared_production_provider_bindings: std::sync::Mutex<
        HashMap<DeclaredOperationIdentity, RegisteredDeclaredProductionProviderBinding>,
    >,
    /// Exact registry-backed authority for the first `time::sleep` production
    /// token. This remains separate from generic declared implementation
    /// operation bindings because `time::sleep` is a checked built-in source
    /// operation rather than an `Impl::operation` declaration.
    time_sleep_provider_binding: std::sync::Mutex<Option<RegisteredTimeSleepProviderBinding>>,
    /// Exact Engine-resolved provider authority for at most two ordered
    /// `forward_sleep` residual `TestClock::wake` frames.
    forward_sleep_wake_provider_binding:
        std::sync::Mutex<Vec<RegisteredForwardSleepWakeProviderBinding>>,
}

/// An entry handle that carries its internal ID for type checking.
///
/// This wraps the lowered target entry expression and maintains the association
/// with its surface representation needed for type checking.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The lowered target entry expression.
    pub core: Expr,
    /// Whether `core` is an executable legacy lowering or an inert placeholder
    /// retained only so source-handler facts can be checked and projected.
    core_lowering: EntryCoreLowering,
    /// Source-facing facts retained alongside the lowered entry Core term.
    ///
    /// These facts are diagnostic/audit sidecars only. They neither affect the
    /// direct evaluator nor grant authority from a callable row.
    pub lowering_sidecars: EntryLoweringSidecars,
    /// The internal ID for looking up the surface program.
    id: u64,
    /// Private engine identity assigned when this entry is parsed.
    owner_token: std::sync::Arc<()>,
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
    /// One declaration-backed concrete operation resolved for the entry body.
    ///
    /// This is checked metadata only. It does not select a provider or grant authority.
    pub declared_concrete_operation: Option<ash_typeck::DeclaredConcreteOperation>,
}

/// Private status for the legacy Core field of an [`Entry`].
///
/// A source handler that has parsed but lacks typed handler lowering retains an
/// inert placeholder solely to preserve the checked surface program. It must
/// reject at every admission boundary before the placeholder is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryCoreLowering {
    Available,
    SourceHandlerUnavailable,
}

/// Engine-retained checker result bound to its entry provenance.
#[derive(Debug, Clone)]
struct CheckedEntryTypeResult {
    owner_token: std::sync::Arc<()>,
    source_anchor: SourceAnchor,
    /// The exact post-check legacy Core derived from the Engine-retained
    /// surface program. Production declaration-backed admission uses this
    /// private snapshot rather than accepting a later public `Entry::core`
    /// mutation as a checked operation argument.
    checked_legacy_core: Expr,
    result: ash_typeck::TypeCheckResult,
    declared_concrete_operation: Option<ash_typeck::DeclaredConcreteOperation>,
}

/// Engine-retained parsed provenance for one entry identity.
#[derive(Debug, Clone)]
struct CanonicalEntrySourceAnchor {
    owner_token: std::sync::Arc<()>,
    source_anchor: SourceAnchor,
    /// The immutable legacy Core produced with this Engine-owned parsed entry.
    /// Production admission checks the public field against this exact record
    /// before invoking `check`, so a caller cannot convert a pre-check Core
    /// mutation into a checked declared-operation argument.
    parsed_legacy_core: Expr,
}

/// Opaque, engine-issued authority to execute one checked handler inspection.
///
/// This wraps a validated V1 admission artifact but intentionally does not
/// expose a public constructor. Only [`Engine::admit_checked_handler_inspection`]
/// can bind its checked evidence, exact root handler instruction, source
/// anchor, and the issuing engine's private execution seal. Effect rows and
/// generic V1 admissions therefore remain descriptive/validated evidence, not
/// standalone authority to install or execute handler frames.
#[derive(Debug, Clone)]
pub struct CheckedHandlerInspectionAdmission {
    sealed_admission: CheckedCpsAdmissionV1,
    issuer_token: std::sync::Arc<()>,
    source_anchor: SourceAnchor,
    handler_name: String,
    root_instruction: FrameInstallationInstructionV1,
}

impl CheckedHandlerInspectionAdmission {
    const fn new(
        sealed_admission: CheckedCpsAdmissionV1,
        issuer_token: std::sync::Arc<()>,
        source_anchor: SourceAnchor,
        handler_name: String,
        root_instruction: FrameInstallationInstructionV1,
    ) -> Self {
        Self {
            sealed_admission,
            issuer_token,
            source_anchor,
            handler_name,
            root_instruction,
        }
    }

    fn is_issued_by(&self, issuer_token: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.issuer_token, issuer_token)
    }

    fn has_exact_root_handler_instruction(&self) -> bool {
        matches!(
            &self.root_instruction,
            FrameInstallationInstructionV1::SourceHandler {
                handler_name,
                core_handle,
                ..
            } if handler_name == &self.handler_name && core_handle.path().is_empty()
        ) && self.sealed_admission.frame_installations().len() == 1
            && self.sealed_admission.frame_installations()[0] == self.root_instruction
            && self.sealed_admission.source_anchors().len() == 1
            && self.sealed_admission.source_anchors()[0] == self.source_anchor
    }

    /// Returns the checked Core/CPS evidence retained for diagnostics.
    ///
    /// This is evidence only; [`Engine::execute_checked_handler_inspection`]
    /// accepts this opaque admission, not a generic checked program.
    #[must_use]
    pub const fn checked_core(&self) -> &ash_core::core_ash_typecheck::CheckedLoweredCoreProgram {
        self.sealed_admission.checked_core()
    }

    /// Returns exact operation identities from the checked source facts.
    #[must_use]
    pub fn operation_identities(&self) -> &[crate::checked_cps_admission::OperationIdentityV1] {
        self.sealed_admission.operation_identities()
    }

    /// Returns source anchors retained by the sealed inspection admission.
    #[must_use]
    pub fn source_anchors(&self) -> &[SourceAnchor] {
        self.sealed_admission.source_anchors()
    }

    /// Returns the one separately authorized root handler instruction.
    #[must_use]
    pub fn frame_installations(&self) -> &[FrameInstallationInstructionV1] {
        self.sealed_admission.frame_installations()
    }
}

/// Opaque, Engine-issued authority to execute the one sealed source-handler
/// production slice.
///
/// This token is distinct from the inspection artifact and generic V1
/// evidence. It is issued only after the Engine has bound one immutable parsed
/// entry provenance record to one checked root `Handle` and one explicit
/// `SourceHandler` instruction. Rows remain descriptive evidence and cannot
/// construct this token or a frame.
#[derive(Debug, Clone)]
pub struct CheckedHandlerProductionAdmission {
    sealed_admission: CheckedCpsAdmissionV1,
    issuer_token: std::sync::Arc<()>,
    entry_id: u64,
    source_anchor: SourceAnchor,
    handler_name: String,
    root_instruction: FrameInstallationInstructionV1,
    executable: ash_core::cps::Term,
}

impl CheckedHandlerProductionAdmission {
    fn new(
        sealed_admission: CheckedCpsAdmissionV1,
        issuer_token: std::sync::Arc<()>,
        entry_id: u64,
        source_anchor: SourceAnchor,
        handler_name: String,
        root_instruction: FrameInstallationInstructionV1,
    ) -> Result<Self, EngineError> {
        let executable =
            terminalize_handler_production_term(sealed_admission.checked_core().lowered().clone());
        ash_interp::cps::validate::validate_cps_program(&executable).map_err(|error| {
            EngineError::Type(format!(
                "checked handler production CPS validation failed: {error}"
            ))
        })?;
        Ok(Self {
            sealed_admission,
            issuer_token,
            entry_id,
            source_anchor,
            handler_name,
            root_instruction,
            executable,
        })
    }

    fn is_issued_by(&self, issuer_token: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.issuer_token, issuer_token)
    }

    fn has_exact_closed_empty_handler_authority(&self) -> bool {
        matches!(
            &self.root_instruction,
            FrameInstallationInstructionV1::SourceHandler {
                operation,
                handler_name,
                core_handle,
            } if handler_name == &self.handler_name
                && is_sealed_production_handler_name(handler_name)
                && core_handle.path().is_empty()
                && self.sealed_admission.operation_identities() == [operation.clone()]
                && self.sealed_admission.residual_rows().len() == 1
                && self.sealed_admission.residual_rows()[0].requirement_keys().is_empty()
                && self.sealed_admission.residual_rows()[0].open_tail().is_none()
        ) && self.sealed_admission.frame_installations() == [self.root_instruction.clone()]
            && self.sealed_admission.source_anchors() == [self.source_anchor.clone()]
            && self.entry_id != 0
    }

    const fn executable(&self) -> &ash_core::cps::Term {
        &self.executable
    }
}

/// Private, Engine-issued authority for TASK-2013's first deep affine source
/// handler. The two ordered instructions are the sole frame authority; the
/// retained closed residual row is descriptive evidence only.
#[derive(Clone)]
struct DeepAffineClockProductionAdmission {
    issuer_token: std::sync::Arc<()>,
    entry_id: u64,
    source_anchor: SourceAnchor,
    source_facts: CheckedSourceFactsV1,
    frame_installations: [FrameInstallationInstructionV1; 2],
    executable: ash_core::cps::Term,
}

impl DeepAffineClockProductionAdmission {
    fn new(
        issuer_token: std::sync::Arc<()>,
        entry_id: u64,
        source_anchor: SourceAnchor,
        source_facts: CheckedSourceFactsV1,
        sleep_operation: &OperationIdentityV1,
        wake_operation: &OperationIdentityV1,
    ) -> Result<Self, EngineError> {
        let frame_installations = [
            FrameInstallationInstructionV1::SourceHandler {
                operation: sleep_operation.clone(),
                handler_name: SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME.to_string(),
                core_handle: CoreHandleLocatorV1::root(),
            },
            FrameInstallationInstructionV1::SourceHandler {
                operation: wake_operation.clone(),
                handler_name: SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME.to_string(),
                core_handle: CoreHandleLocatorV1::root(),
            },
        ];
        if !has_exact_deep_affine_clock_frame_authority(
            entry_id,
            &source_anchor,
            &source_facts,
            sleep_operation,
            wake_operation,
            &frame_installations,
        ) {
            return Err(EngineError::Type(
                "deep_affine_clock production admission requires its ordered checked source clauses and explicit frame instructions".to_string(),
            ));
        }
        let executable = deep_affine_clock_executable(sleep_operation, wake_operation);
        ash_interp::cps::validate::validate_cps_program(&executable).map_err(|error| {
            EngineError::Type(format!(
                "deep_affine_clock production CPS validation failed: {error}"
            ))
        })?;
        Ok(Self {
            issuer_token,
            entry_id,
            source_anchor,
            source_facts,
            frame_installations,
            executable,
        })
    }

    fn is_issued_by(&self, issuer_token: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.issuer_token, issuer_token)
    }

    fn has_exact_authority(&self) -> bool {
        let Some(sleep_operation) = self.source_facts.operation_identities().first() else {
            return false;
        };
        let Some(wake_operation) = self.source_facts.operation_identities().get(1) else {
            return false;
        };
        has_exact_deep_affine_clock_frame_authority(
            self.entry_id,
            &self.source_anchor,
            &self.source_facts,
            sleep_operation,
            wake_operation,
            &self.frame_installations,
        ) && ash_interp::cps::validate::validate_cps_program(&self.executable).is_ok()
    }
}

fn has_exact_deep_affine_clock_frame_authority(
    entry_id: u64,
    source_anchor: &SourceAnchor,
    source_facts: &CheckedSourceFactsV1,
    sleep_operation: &OperationIdentityV1,
    wake_operation: &OperationIdentityV1,
    frame_installations: &[FrameInstallationInstructionV1; 2],
) -> bool {
    entry_id != 0
        && source_facts.handler_name() == SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME
        && source_facts.operation_identities() == [sleep_operation.clone(), wake_operation.clone()]
        && matches!(source_facts.handler_clauses(),
            [sleep, wake]
                if sleep.operation() == sleep_operation
                    && sleep.resume_name() == "resume"
                    && wake.operation() == wake_operation
                    && wake.resume_name() == "resume"
        )
        && matches!(source_facts.residual_rows(), [row] if row.is_closed_empty())
        && source_facts.source_anchors() == [source_anchor.clone()]
        && matches!(&frame_installations[0], FrameInstallationInstructionV1::SourceHandler { operation, handler_name, core_handle }
            if operation == sleep_operation
                && handler_name == SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME
                && core_handle.path().is_empty())
        && matches!(&frame_installations[1], FrameInstallationInstructionV1::SourceHandler { operation, handler_name, core_handle }
            if operation == wake_operation
                && handler_name == SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME
                && core_handle.path().is_empty())
}

fn deep_affine_clock_executable(
    sleep_operation: &OperationIdentityV1,
    wake_operation: &OperationIdentityV1,
) -> ash_core::cps::Term {
    use ash_core::cps::{
        Atom, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow, PrimOp,
        Term, Value as CpsValue,
    };

    let operation = |identity: &OperationIdentityV1| EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: format!("{}.{}", identity.impl_type(), identity.operation()),
            kind: EffectItemKind::Capability,
        },
        arg_types: identity.parameter_types().to_vec(),
        result_type: identity.result_type().to_string(),
    };
    let effect_row = |identity: &OperationIdentityV1| EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: format!("{}.{}", identity.impl_type(), identity.operation()),
            kind: EffectItemKind::Capability,
        }],
    };
    let sleep_op = operation(sleep_operation);
    let wake_op = operation(wake_operation);
    let answer = "__deep_affine_answer".to_string();
    let after_sleep = "__deep_affine_after_sleep".to_string();
    let after_wake = "__deep_affine_after_wake".to_string();
    let done = "__deep_affine_done".to_string();

    Term::LetCont {
        name: answer.clone(),
        param: "__deep_affine_answer_value".to_string(),
        cont_body: Box::new(Term::Return {
            value: CpsValue::Atom(Atom::Var("__deep_affine_answer_value".to_string())),
        }),
        body: Box::new(Term::LetCont {
            name: done.clone(),
            param: "__deep_affine_second_sleep_result".to_string(),
            cont_body: Box::new(Term::LetPrim {
                name: "__deep_affine_done_value".to_string(),
                op: PrimOp::Add,
                args: vec![Atom::Int(7), Atom::Int(100)],
                body: Box::new(Term::Jump {
                    cont: ContRef::Label(answer),
                    arg: Atom::Var("__deep_affine_done_value".to_string()),
                    row: EffectRow::default(),
                }),
            }),
            body: Box::new(Term::LetCont {
                name: after_wake.clone(),
                param: "__deep_affine_wake_result".to_string(),
                cont_body: Box::new(Term::Raise {
                    op: sleep_op.clone(),
                    args: vec![Atom::Int(2)],
                    resume: ContRef::Label(done),
                    row: effect_row(sleep_operation),
                }),
                body: Box::new(Term::LetCont {
                    name: after_sleep.clone(),
                    param: "__deep_affine_sleep_result".to_string(),
                    cont_body: Box::new(Term::Raise {
                        op: wake_op,
                        args: vec![Atom::Int(1)],
                        resume: ContRef::Label(after_wake),
                        row: effect_row(wake_operation),
                    }),
                    body: Box::new(Term::Raise {
                        op: sleep_op,
                        args: vec![Atom::Int(0)],
                        resume: ContRef::Label(after_sleep),
                        row: effect_row(sleep_operation),
                    }),
                    row: effect_row(wake_operation),
                    multiplicity: ContMultiplicity::Affine,
                }),
                row: effect_row(sleep_operation),
                multiplicity: ContMultiplicity::Affine,
            }),
            row: EffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn deep_affine_resume_clause(operation: &OperationIdentityV1) -> ash_core::cps::HandlerClause {
    use ash_core::cps::{
        Atom, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow,
        ResumeRowMetadata, Term,
    };

    ash_core::cps::HandlerClause {
        op: EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: format!("{}.{}", operation.impl_type(), operation.operation()),
                kind: EffectItemKind::Capability,
            },
            arg_types: operation.parameter_types().to_vec(),
            result_type: operation.result_type().to_string(),
        },
        params: vec!["ms".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::Var("ms".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    }
}

fn is_sealed_production_handler_name(handler_name: &str) -> bool {
    matches!(
        handler_name,
        SEALED_PRODUCTION_HANDLER_NAME | SEALED_TRAP_SLEEP_HANDLER_NAME
    )
}

fn sealed_handler_structural_rejection(
    handler_name: Option<&str>,
    message: impl Into<String>,
) -> EngineError {
    let error = EngineError::Type(message.into());
    if handler_name == Some(SEALED_TRAP_SLEEP_HANDLER_NAME) {
        missing_production_admission(&error)
    } else {
        error
    }
}

fn classify_sealed_handler_structural_error(handler_name: &str, error: EngineError) -> EngineError {
    if handler_name == SEALED_TRAP_SLEEP_HANDLER_NAME {
        missing_production_admission(&error)
    } else {
        error
    }
}

/// Opaque, Engine-issued authority for TASK-2026's exact `forward_sleep`
/// handler/provider composition.
///
/// It is deliberately distinct from generic V1 evidence, the closed-empty
/// handler admission, and the single-provider production token.  The private
/// fields bind one canonical source/Core provenance record, one checked
/// `Handle`, one or two exact `wake` provider objects, and ordered frame
/// instructions; public callers can inspect only the instruction summary.
#[derive(Clone)]
pub struct ForwardSleepProductionAdmission {
    sealed_admission: CheckedCpsAdmissionV1,
    issuer_token: std::sync::Arc<()>,
    run_control_token: std::sync::Arc<()>,
    entry_id: u64,
    source_anchor: SourceAnchor,
    sleep_operation: OperationIdentityV1,
    wake_operation: OperationIdentityV1,
    resolved_wake_providers: Vec<ResolvedProviderBinding>,
    executable: ash_core::cps::Term,
}

impl ForwardSleepProductionAdmission {
    fn new(
        sealed_admission: CheckedCpsAdmissionV1,
        issuer_token: std::sync::Arc<()>,
        entry_id: u64,
        source_anchor: SourceAnchor,
        sleep_operation: OperationIdentityV1,
        wake_operation: OperationIdentityV1,
        resolved_wake_providers: Vec<ResolvedProviderBinding>,
    ) -> Result<Self, EngineError> {
        if !has_exact_forward_sleep_frame_authority(
            &sealed_admission,
            &source_anchor,
            &sleep_operation,
            &wake_operation,
            &resolved_wake_providers,
        ) {
            return Err(EngineError::Type(
                "forward_sleep production admission requires one or two sealed Provider instructions followed by its SourceHandler instruction"
                    .to_string(),
            ));
        }
        validate_exact_forward_sleep_cps(
            sealed_admission.checked_core().lowered(),
            &sleep_operation,
            &wake_operation,
            FORWARD_SLEEP_ANSWER_CONTINUATION,
        )?;
        let executable = terminalize_forward_sleep_production_term(
            sealed_admission.checked_core().lowered().clone(),
        );
        ash_interp::cps::validate::validate_cps_program(&executable).map_err(|error| {
            EngineError::Type(format!(
                "forward_sleep production CPS validation failed: {error}"
            ))
        })?;
        Ok(Self {
            sealed_admission,
            issuer_token,
            run_control_token: std::sync::Arc::new(()),
            entry_id,
            source_anchor,
            sleep_operation,
            wake_operation,
            resolved_wake_providers,
            executable,
        })
    }

    fn is_issued_by(&self, issuer_token: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.issuer_token, issuer_token)
    }

    fn has_run_control_token(&self, run_control_token: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.run_control_token, run_control_token)
    }

    fn run_control_token(&self) -> std::sync::Arc<()> {
        std::sync::Arc::clone(&self.run_control_token)
    }

    fn has_exact_authority(&self) -> bool {
        self.entry_id != 0
            && has_exact_forward_sleep_frame_authority(
                &self.sealed_admission,
                &self.source_anchor,
                &self.sleep_operation,
                &self.wake_operation,
                &self.resolved_wake_providers,
            )
            && validate_exact_forward_sleep_cps(
                self.sealed_admission.checked_core().lowered(),
                &self.sleep_operation,
                &self.wake_operation,
                FORWARD_SLEEP_ANSWER_CONTINUATION,
            )
            .is_ok()
    }

    /// Returns the two explicitly authorized installation instructions in
    /// outer-to-inner order. Rows cannot construct frames from this summary.
    #[must_use]
    pub fn frame_installation_summary(&self) -> &[FrameInstallationInstructionV1] {
        self.sealed_admission.frame_installations()
    }

    pub(crate) const fn executable(&self) -> &ash_core::cps::Term {
        &self.executable
    }

    pub(crate) const fn sleep_operation(&self) -> &OperationIdentityV1 {
        &self.sleep_operation
    }

    pub(crate) const fn wake_operation(&self) -> &OperationIdentityV1 {
        &self.wake_operation
    }

    pub(crate) fn resolved_wake_providers(&self) -> &[ResolvedProviderBinding] {
        &self.resolved_wake_providers
    }
}

fn terminalize_forward_sleep_production_term(lowered: ash_core::cps::Term) -> ash_core::cps::Term {
    ash_core::cps::Term::LetCont {
        name: FORWARD_SLEEP_ANSWER_CONTINUATION.to_string(),
        param: FORWARD_SLEEP_ANSWER_VALUE.to_string(),
        cont_body: Box::new(ash_core::cps::Term::Return {
            value: ash_core::cps::Value::Atom(ash_core::cps::Atom::Var(
                FORWARD_SLEEP_ANSWER_VALUE.to_string(),
            )),
        }),
        body: Box::new(lowered),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::Affine,
    }
}

fn has_exact_forward_sleep_frame_authority(
    admission: &CheckedCpsAdmissionV1,
    source_anchor: &SourceAnchor,
    sleep_operation: &OperationIdentityV1,
    wake_operation: &OperationIdentityV1,
    wake_bindings: &[ResolvedProviderBinding],
) -> bool {
    let Some((source_handler, providers)) = admission.frame_installations().split_last() else {
        return false;
    };
    let providers_match = providers.len() == wake_bindings.len()
        && (1..=2).contains(&providers.len())
        && providers.iter().zip(wake_bindings).all(|(instruction, binding)| {
            matches!(instruction, FrameInstallationInstructionV1::Provider { operation, provider_binding }
                if operation == wake_operation
                    && provider_binding == binding.binding()
                    && provider_binding.operation() == wake_operation)
        });
    providers_match
        && matches!(source_handler, FrameInstallationInstructionV1::SourceHandler { operation: handled, handler_name, core_handle }
            if handled == sleep_operation
                && handler_name == SEALED_FORWARD_SLEEP_HANDLER_NAME
                && core_handle.path().is_empty())
        && admission.operation_identities() == [sleep_operation.clone()]
        && admission.source_anchors() == [source_anchor.clone()]
        && admission.residual_rows().len() == 1
        && admission.residual_rows()[0].is_closed_empty()
}

fn cps_operation_matches_identity(
    operation: &ash_core::cps::EffectOp,
    identity: &OperationIdentityV1,
) -> bool {
    operation.item.namespace == "cap"
        && operation.item.name == format!("{}.{}", identity.impl_type(), identity.operation())
        && operation.arg_types == identity.parameter_types()
        && operation.result_type == identity.result_type()
}

fn validate_exact_forward_sleep_cps(
    lowered: &ash_core::cps::Term,
    sleep_operation: &OperationIdentityV1,
    wake_operation: &OperationIdentityV1,
    answer_continuation: &str,
) -> Result<(), EngineError> {
    let ash_core::cps::Term::Handle {
        clause,
        body,
        cont,
        row,
    } = lowered
    else {
        return Err(EngineError::Type(
            "forward_sleep production admission requires a root checked CPS Handle".to_string(),
        ));
    };
    let ash_core::cps::Term::Raise {
        op: sleep_op,
        args: sleep_args,
        resume: sleep_resume,
        ..
    } = body.as_ref()
    else {
        return Err(EngineError::Type(
            "forward_sleep production admission requires its checked sleep Raise".to_string(),
        ));
    };
    let ash_core::cps::Term::Raise {
        op: wake_op,
        args: wake_args,
        resume: wake_resume,
        ..
    } = clause.body.as_ref()
    else {
        return Err(EngineError::Type(
            "forward_sleep production admission requires its checked wake Raise".to_string(),
        ));
    };
    let exact_handle_row = row.items.as_slice()
        == [ash_core::cps::EffectItem {
            namespace: "cap".to_string(),
            name: "TestClock.wake".to_string(),
            kind: ash_core::cps::EffectItemKind::Capability,
        }];
    let exact = cps_operation_matches_identity(&clause.op, sleep_operation)
        && cps_operation_matches_identity(sleep_op, sleep_operation)
        && cps_operation_matches_identity(wake_op, wake_operation)
        && matches!(sleep_args.as_slice(), [ash_core::cps::Atom::Int(0)])
        && matches!(clause.params.as_slice(), [parameter] if parameter == "ms")
        && matches!(wake_args.as_slice(), [ash_core::cps::Atom::Var(parameter)] if parameter == "ms")
        && matches!(cont, ash_core::cps::ContRef::Label(label) if label == answer_continuation)
        && matches!(sleep_resume, ash_core::cps::ContRef::Label(label) if label == answer_continuation)
        && matches!(wake_resume, ash_core::cps::ContRef::Var(resume) if resume == &clause.resume)
        && exact_handle_row;
    if !exact {
        return Err(EngineError::Type(
            "forward_sleep production admission requires its exact checked Handle/sleep/wake CPS shape"
                .to_string(),
        ));
    }
    Ok(())
}

fn terminalize_handler_production_term(lowered: ash_core::cps::Term) -> ash_core::cps::Term {
    ash_core::cps::Term::LetCont {
        name: HANDLER_INSPECTION_ANSWER_CONTINUATION.to_string(),
        param: HANDLER_INSPECTION_ANSWER_VALUE.to_string(),
        cont_body: Box::new(ash_core::cps::Term::Return {
            value: ash_core::cps::Value::Atom(ash_core::cps::Atom::Var(
                HANDLER_INSPECTION_ANSWER_VALUE.to_string(),
            )),
        }),
        body: Box::new(lowered),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::Affine,
    }
}

/// Source sidecars for the lowered entry Core term.
///
/// The current source-entry bridge has a single Core expression for the entry
/// callable, so its enclosing callable anchor is the smallest origin fact that
/// can be retained without pretending that every legacy [`Expr`] node has a
/// target-Core source annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryLoweringSidecars {
    /// Origin of the callable body that produced [`Entry::core`].
    pub entry_body_origin: SourceAnchor,
    /// Expansion origins retained for diagnostic and audit use only.
    ///
    /// These records do not alter Core execution or carry runtime authority.
    pub expansion_origins: Vec<ash_parser::surface::ExpandedSurfaceOrigin>,
    /// Parser-validated identifier hygiene metadata retained for diagnostics
    /// and audit only.
    ///
    /// This is the exact expanded-surface product when that boundary was used;
    /// the legacy function-parser fallback supplies an explicit empty vector.
    pub identifier_hygiene: Vec<ash_parser::surface::IdentifierHygieneMetadata>,
    /// Fully lowered contracts for every local callable, keyed by callable name.
    ///
    /// This deterministic diagnostic/evidence artifact does not add callable
    /// rows, install runtime checks or monitors, or grant runtime authority.
    pub callable_contracts: BTreeMap<String, ash_parser::LoweredFnContract>,
}

fn entry_lowering_sidecars(
    program: &ash_parser::surface::Program,
    module_identity: Option<&ash_core::semantic_summary::ModuleIdentity>,
    expansion_origins: Vec<ash_parser::surface::ExpandedSurfaceOrigin>,
    identifier_hygiene: Vec<ash_parser::surface::IdentifierHygieneMetadata>,
    callable_contracts: BTreeMap<String, ash_parser::LoweredFnContract>,
) -> EntryLoweringSidecars {
    let origin = module_identity.map_or_else(
        || SourceOrigin::Synthetic {
            reason: "inline engine entry source".to_string(),
        },
        |identity| match &identity.source {
            ModuleSourceOrigin::File(path) => SourceOrigin::File(path.clone()),
            ModuleSourceOrigin::Inline { parent, offset } => SourceOrigin::InlineModule {
                module: *parent,
                offset: *offset,
            },
            ModuleSourceOrigin::Synthetic { reason } => SourceOrigin::Synthetic {
                reason: reason.clone(),
            },
        },
    );
    EntryLoweringSidecars {
        entry_body_origin: SourceAnchor::new(
            origin,
            Some(ash_core::Span {
                start: program.entry.span.start,
                end: program.entry.span.end,
            }),
            format!("entry callable {}", program.entry.function),
        ),
        expansion_origins,
        identifier_hygiene,
        callable_contracts,
    }
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
    BTreeMap<String, ash_parser::LoweredFnContract>,
    Expr,
    EntryCoreLowering,
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

fn lower_local_callable_contract(
    function: &ash_parser::surface::FnDef,
    source_path: Option<&str>,
) -> Result<(String, ash_parser::LoweredFnContract), EngineError> {
    let name = function.name.to_string();
    let contract =
        ash_parser::lower_fn_contract_for_function_with_source_path(function, source_path)
            .map_err(|error| {
                EngineError::Parse(format!(
                    "failed to lower fn contract for '{}': {error}",
                    function.name
                ))
            })?;

    Ok((name, contract))
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code, clippy::too_many_arguments)]
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

    /// Return the execution boundary used by this Engine's production APIs.
    ///
    /// This is intentionally a typed declaration rather than an inference from
    /// implementation details. Until a validated production checked-Core/CPS
    /// artifact is available, every source execution route rejects closed.
    #[must_use]
    pub const fn production_execution_boundary(&self) -> ProductionExecutionBoundary {
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission
    }

    #[cfg(test)]
    fn checked_cps_inspection_count(&self) -> u64 {
        self.checked_cps_inspection_calls
            .load(std::sync::atomic::Ordering::Relaxed)
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

    fn store_canonical_entry_source_anchor(
        &self,
        entry_id: u64,
        source_anchor: SourceAnchor,
        parsed_legacy_core: Expr,
    ) {
        if let Ok(mut anchors) = self.canonical_entry_source_anchors.lock() {
            anchors.insert(
                entry_id,
                CanonicalEntrySourceAnchor {
                    owner_token: self.entry_owner_token.clone(),
                    source_anchor,
                    parsed_legacy_core,
                },
            );
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

    fn clear_checked_type_result(&self, application_id: u64) {
        if let Ok(mut map) = self.checked_type_results.lock() {
            map.remove(&application_id);
        }
    }

    fn owns_entry(&self, entry: &Entry) -> bool {
        std::sync::Arc::ptr_eq(&self.entry_owner_token, &entry.owner_token)
    }

    fn canonical_entry_source_provenance(
        &self,
        entry: &Entry,
    ) -> Result<CanonicalEntrySourceAnchor, EngineError> {
        if !self.owns_entry(entry) {
            return Err(EngineError::Type(
                "production checked-CPS admission requires an entry issued by this Engine"
                    .to_string(),
            ));
        }
        let anchors = self.canonical_entry_source_anchors.lock().map_err(|_| {
            EngineError::Type(
                "production checked-CPS admission cannot read canonical entry provenance"
                    .to_string(),
            )
        })?;
        let canonical = anchors.get(&entry.id).ok_or_else(|| {
            EngineError::Type(
                "production checked-CPS admission has no canonical parsed source anchor"
                    .to_string(),
            )
        })?;
        if !std::sync::Arc::ptr_eq(&canonical.owner_token, &entry.owner_token)
            || canonical.source_anchor != entry.lowering_sidecars.entry_body_origin
        {
            return Err(EngineError::Type(
                "production checked-CPS admission source anchor does not match the canonical parsed entry provenance"
                    .to_string(),
            ));
        }
        if canonical.parsed_legacy_core != entry.core {
            return Err(EngineError::Type(
                "production checked-CPS admission Core does not match the canonical parsed entry provenance"
                    .to_string(),
            ));
        }
        let canonical = canonical.clone();
        drop(anchors);
        Ok(canonical)
    }

    fn store_checked_type_result(&self, application: &Entry, result: ash_typeck::TypeCheckResult) {
        let Ok(anchors) = self.canonical_entry_source_anchors.lock() else {
            return;
        };
        let Some(canonical) = anchors.get(&application.id) else {
            return;
        };
        if !std::sync::Arc::ptr_eq(&canonical.owner_token, &application.owner_token)
            || canonical.source_anchor != application.lowering_sidecars.entry_body_origin
        {
            return;
        }
        let source_anchor = canonical.source_anchor.clone();
        drop(anchors);
        if let Ok(mut map) = self.checked_type_results.lock() {
            map.insert(
                application.id,
                CheckedEntryTypeResult {
                    owner_token: application.owner_token.clone(),
                    source_anchor,
                    checked_legacy_core: application.core.clone(),
                    result,
                    declared_concrete_operation: application.declared_concrete_operation.clone(),
                },
            );
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

    /// Whether an entry has no Engine-retained import facts.
    ///
    /// Closed source-production routes must make this decision from the
    /// private parse cache, not public entry sidecars. A poisoned cache is an
    /// admission error rather than evidence that imports were absent.
    fn entry_has_no_retained_imported_state(&self, entry_id: u64) -> Result<bool, EngineError> {
        let imported_type_defs = self.imported_type_defs.lock().map_err(|_| {
            EngineError::Type(
                "checked Core/CPS local-call admission cannot read retained imported type state"
                    .to_string(),
            )
        })?;
        let imported_semantic_summaries =
            self.imported_semantic_summaries.lock().map_err(|_| {
                EngineError::Type(
                "checked Core/CPS local-call admission cannot read retained imported semantic state"
                    .to_string(),
            )
            })?;
        let imported_type_function_heads =
            self.imported_type_function_heads.lock().map_err(|_| {
                EngineError::Type(
                    "checked Core/CPS local-call admission cannot read retained imported type-function state"
                        .to_string(),
                )
            })?;

        Ok(imported_type_defs.get(&entry_id).is_none_or(Vec::is_empty)
            && imported_semantic_summaries
                .get(&entry_id)
                .is_none_or(Vec::is_empty)
            && imported_type_function_heads
                .get(&entry_id)
                .is_none_or(Vec::is_empty))
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

    /// Bind one typechecked concrete operation declaration to one provider operation.
    ///
    /// Binding is explicit and validates the provider's declared metadata. In
    /// particular, the provider operation must advertise the exact concrete
    /// operation row and must not claim to grant authority.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider, its metadata, or its operation row
    /// does not exactly match the checked declaration carrier.
    ///
    /// # Panics
    ///
    /// Panics if the engine's declared-operation binding registry mutex is poisoned.
    pub fn register_declared_operation_provider_binding(
        &self,
        declared_operation: &ash_typeck::DeclaredConcreteOperation,
        provider_name: &str,
        provider_operation: &str,
    ) -> Result<(), EngineError> {
        let provider = self
            .runtime_state
            .get_provider(provider_name)
            .ok_or_else(|| {
                EngineError::CapabilityNotFound(format!(
                    "declared-operation provider '{provider_name}'"
                ))
            })?;
        let metadata = provider.provider_metadata();
        ash_core::capability::validate_provider_authoring_metadata(&metadata)
            .map_err(|error| EngineError::Configuration(error.to_string()))?;
        if metadata.provider_name != provider_name {
            return Err(EngineError::Configuration(format!(
                "provider registration '{provider_name}' does not match metadata provider '{}'",
                metadata.provider_name
            )));
        }
        let Some(operation_metadata) = metadata.operation(provider_operation) else {
            return Err(EngineError::Configuration(format!(
                "provider '{provider_name}' does not declare provider operation '{provider_operation}'"
            )));
        };
        let required_row = format!(
            "{}.{}",
            declared_operation.impl_type, declared_operation.operation
        );
        if !operation_metadata.required_rows.contains(&required_row) {
            return Err(EngineError::Configuration(format!(
                "provider operation '{provider_name}.{provider_operation}' must require declared operation row '{required_row}'"
            )));
        }
        if operation_metadata.grants_authority {
            return Err(EngineError::Configuration(format!(
                "provider operation '{provider_name}.{provider_operation}' must not grant declared-operation authority"
            )));
        }

        let identity = DeclaredOperationIdentity::from(declared_operation);
        let binding = DeclaredOperationProviderBinding {
            provider_name: provider_name.to_string(),
            provider_operation: provider_operation.to_string(),
        };
        let mut bindings = self
            .declared_operation_provider_bindings
            .lock()
            .expect("declared-operation provider binding mutex poisoned");
        if let Some(existing) = bindings.get(&identity)
            && existing != &binding
        {
            return Err(EngineError::Configuration(format!(
                "declared operation '{}.{}' is already bound to '{}.{}'",
                declared_operation.impl_type,
                declared_operation.operation,
                existing.provider_name,
                existing.provider_operation
            )));
        }
        bindings.insert(identity, binding);
        drop(bindings);

        if is_sealed_declared_production_operation(declared_operation) {
            let identity = DeclaredOperationIdentity::from(declared_operation);
            let registered = RegisteredDeclaredProductionProviderBinding {
                binding: ProviderBindingV1::new(
                    OperationIdentityV1::from_declared(declared_operation),
                    provider_name,
                    provider_operation,
                ),
                provider,
            };
            let mut production_bindings = self
                .declared_production_provider_bindings
                .lock()
                .expect("declared production provider binding mutex poisoned");
            if let Some(existing) = production_bindings.get(&identity)
                && existing.binding != registered.binding
            {
                return Err(EngineError::Configuration(format!(
                    "sealed declared production operation '{}.{}' conflicts with its existing provider binding",
                    declared_operation.impl_type, declared_operation.operation
                )));
            }
            production_bindings.insert(identity, registered);
        }
        Ok(())
    }

    pub(crate) fn declared_operation_provider_binding(
        &self,
        declared_operation: &ash_typeck::DeclaredConcreteOperation,
    ) -> Option<DeclaredOperationProviderBinding> {
        self.declared_operation_provider_bindings
            .lock()
            .expect("declared-operation provider binding mutex poisoned")
            .get(&DeclaredOperationIdentity::from(declared_operation))
            .cloned()
    }

    fn registered_declared_production_provider_binding(
        &self,
        declared_operation: &ash_typeck::DeclaredConcreteOperation,
    ) -> Result<ResolvedProviderBinding, EngineError> {
        if !is_sealed_declared_production_operation(declared_operation) {
            return Err(EngineError::Type(
                "production declared-operation admission does not admit this declaration"
                    .to_string(),
            ));
        }
        let identity = DeclaredOperationIdentity::from(declared_operation);
        let registered = self
            .declared_production_provider_bindings
            .lock()
            .expect("declared production provider binding mutex poisoned")
            .get(&identity)
            .cloned()
            .ok_or_else(|| {
                EngineError::Type(
                    "production declared-operation admission requires an Engine-registered exact provider binding"
                        .to_string(),
                )
            })?;
        let current_provider = self
            .runtime_state
            .get_provider(registered.binding.provider_name())
            .ok_or_else(|| {
                EngineError::CapabilityNotFound(format!(
                    "production declared-operation provider '{}'",
                    registered.binding.provider_name()
                ))
            })?;
        if !std::sync::Arc::ptr_eq(&current_provider, &registered.provider) {
            return Err(EngineError::Configuration(
                "production declared-operation provider changed after binding registration"
                    .to_string(),
            ));
        }
        Ok(ResolvedProviderBinding::new(
            registered.binding,
            registered.provider,
        ))
    }

    fn sealed_forward_sleep_operation_facts(
        checked: &CheckedEntryTypeResult,
    ) -> Result<
        (
            ash_typeck::DeclaredConcreteOperation,
            ash_typeck::DeclaredConcreteOperation,
        ),
        EngineError,
    > {
        let handler = checked
            .result
            .checked_handlers
            .get(SEALED_FORWARD_SLEEP_HANDLER_NAME)
            .ok_or_else(|| {
                EngineError::Type(
                    "forward_sleep production admission requires its checked handler fact"
                        .to_string(),
                )
            })?;
        let [clause] = handler.clauses.as_slice() else {
            return Err(EngineError::Type(
                "forward_sleep production admission requires one checked operation clause"
                    .to_string(),
            ));
        };
        let wake = clause.local_effect.clone().ok_or_else(|| {
            EngineError::Type(
                "forward_sleep production admission requires its checked wake clause effect"
                    .to_string(),
            )
        })?;
        let output_is_exact_wake = handler.output_row.tail.is_none()
            && handler.output_row.items.len() == 1
            && handler.output_row.items[0].canonical_key() == "operation:TestClock::Clock::wake";
        if !is_sealed_forward_sleep_operation(&clause.operation)
            || !is_sealed_forward_wake_operation(&wake)
            || !output_is_exact_wake
            || handler.done_binding != "value"
            || handler.done_binding_type.to_string() != "Int"
            || handler.answer_type.to_string() != "Int"
        {
            return Err(EngineError::Type(
                "forward_sleep production admission does not admit these checked handler facts"
                    .to_string(),
            ));
        }
        Ok((clause.operation.clone(), wake))
    }

    /// Register one of at most two ordered host provider bindings eligible for
    /// TASK-2026's sealed `forward_sleep` residual `TestClock::wake` frames.
    ///
    /// The checked entry supplies the concrete declaration identity; provider
    /// metadata is only used to verify the host binding, never to infer that
    /// identity or create frame authority.
    ///
    /// # Errors
    ///
    /// Returns an error when the entry lacks canonical checked
    /// `forward_sleep` facts, or the named provider does not advertise exactly
    /// the non-authority-granting `TestClock.wake` requirement.
    ///
    /// # Panics
    ///
    /// Panics if the Engine-owned sealed wake-binding registry mutex is poisoned.
    pub fn register_sealed_forward_sleep_wake_provider_binding(
        &self,
        entry: &Entry,
        provider_name: &str,
        provider_operation: &str,
    ) -> Result<(), EngineError> {
        self.canonical_entry_source_provenance(entry)?;
        if !self.entry_has_no_retained_imported_state(entry.id)? {
            return Err(EngineError::Type(
                "forward_sleep production admission does not admit imported source state"
                    .to_string(),
            ));
        }
        let checked = self.retained_checked_entry_result(entry)?;
        let (_, wake) = Self::sealed_forward_sleep_operation_facts(&checked)?;
        let provider = self
            .runtime_state
            .get_provider(provider_name)
            .ok_or_else(|| {
                EngineError::CapabilityNotFound(format!(
                    "forward_sleep wake provider '{provider_name}'"
                ))
            })?;
        let metadata = provider.provider_metadata();
        ash_core::capability::validate_provider_authoring_metadata(&metadata)
            .map_err(|error| EngineError::Configuration(error.to_string()))?;
        if metadata.provider_name != provider_name {
            return Err(EngineError::Configuration(format!(
                "forward_sleep wake provider registration '{provider_name}' does not match metadata provider '{}'",
                metadata.provider_name
            )));
        }
        if provider_operation != "wake" {
            return Err(EngineError::Configuration(
                "forward_sleep wake provider binding requires provider operation 'wake'"
                    .to_string(),
            ));
        }
        let operation_metadata = metadata.operation(provider_operation).ok_or_else(|| {
            EngineError::Configuration(format!(
                "forward_sleep wake provider '{provider_name}' does not declare operation '{provider_operation}'"
            ))
        })?;
        if operation_metadata.required_rows.len() != 1
            || operation_metadata.required_rows[0] != "TestClock.wake"
            || operation_metadata.grants_authority
        {
            return Err(EngineError::Configuration(
                "forward_sleep wake provider operation must require exactly non-authority-granting row 'TestClock.wake'"
                    .to_string(),
            ));
        }
        let registered = RegisteredForwardSleepWakeProviderBinding {
            binding: ProviderBindingV1::new(
                OperationIdentityV1::from_declared(&wake),
                provider_name,
                provider_operation,
            ),
            provider,
        };
        let mut bindings = self
            .forward_sleep_wake_provider_binding
            .lock()
            .expect("forward_sleep wake provider binding mutex poisoned");
        if bindings
            .iter()
            .any(|existing| existing.binding == registered.binding)
        {
            return Ok(());
        }
        if bindings.len() == 2 {
            return Err(EngineError::Configuration(
                "forward_sleep production admission permits at most two ordered wake provider bindings"
                    .to_string(),
            ));
        }
        bindings.push(registered);
        drop(bindings);
        Ok(())
    }

    fn registered_forward_sleep_wake_provider_bindings(
        &self,
        wake_operation: &ash_typeck::DeclaredConcreteOperation,
    ) -> Result<Vec<ResolvedProviderBinding>, EngineError> {
        if !is_sealed_forward_wake_operation(wake_operation) {
            return Err(EngineError::Type(
                "forward_sleep production admission does not admit this wake operation".to_string(),
            ));
        }
        let registered = self
            .forward_sleep_wake_provider_binding
            .lock()
            .expect("forward_sleep wake provider binding mutex poisoned")
            .clone();
        if registered.is_empty() {
            return Err(
                EngineError::Type(
                    "forward_sleep production admission requires an Engine-registered exact wake provider binding"
                        .to_string(),
                ),
            );
        }
        if !(1..=2).contains(&registered.len()) {
            return Err(EngineError::Type(
                "forward_sleep production admission permits at most two ordered wake provider bindings"
                    .to_string(),
            ));
        }
        let expected_operation = OperationIdentityV1::from_declared(wake_operation);
        if registered
            .iter()
            .any(|binding| binding.binding.operation() != &expected_operation)
        {
            return Err(EngineError::Type(
                "forward_sleep production wake binding does not match checked declaration facts"
                    .to_string(),
            ));
        }
        registered
            .into_iter()
            .map(|registered| {
                let current_provider = self
                    .runtime_state
                    .get_provider(registered.binding.provider_name())
                    .ok_or_else(|| {
                        EngineError::CapabilityNotFound(format!(
                            "forward_sleep wake provider '{}'",
                            registered.binding.provider_name()
                        ))
                    })?;
                if !std::sync::Arc::ptr_eq(&current_provider, &registered.provider) {
                    return Err(EngineError::Configuration(
                        "forward_sleep wake provider changed after binding registration"
                            .to_string(),
                    ));
                }
                Ok(ResolvedProviderBinding::new(
                    registered.binding,
                    registered.provider,
                ))
            })
            .collect()
    }

    /// Register the sole provider binding eligible for the checked
    /// `time::sleep` production slice.
    ///
    /// The registrar resolves the concrete runtime provider named `time` and
    /// verifies its advertised `sleep` operation metadata before retaining the
    /// provider object. A requirement row, a CPS operation spelling, or a
    /// public V1 instruction cannot synthesize this binding.
    ///
    /// # Errors
    ///
    /// Returns an error when no `time` provider is registered, its metadata is
    /// invalid, or it does not advertise the exact non-authority-granting
    /// `time.sleep` operation required by this narrow slice.
    ///
    /// # Panics
    ///
    /// Panics if the Engine-owned `time::sleep` binding registry mutex is
    /// poisoned.
    pub fn register_time_sleep_provider_binding(&self) -> Result<(), EngineError> {
        let provider = self
            .runtime_state
            .get_provider(TIME_SLEEP_OPERATION.provider)
            .ok_or_else(|| {
                EngineError::CapabilityNotFound(
                    "production time::sleep provider 'time'".to_string(),
                )
            })?;
        let metadata = provider.provider_metadata();
        ash_core::capability::validate_provider_authoring_metadata(&metadata)
            .map_err(|error| EngineError::Configuration(error.to_string()))?;
        if metadata.provider_name != TIME_SLEEP_OPERATION.provider {
            return Err(EngineError::Configuration(format!(
                "production time::sleep provider must advertise name '{}', found '{}'",
                TIME_SLEEP_OPERATION.provider, metadata.provider_name
            )));
        }
        let Some(operation_metadata) = metadata.operation(TIME_SLEEP_OPERATION.name) else {
            return Err(EngineError::Configuration(
                "production time::sleep provider must advertise operation 'sleep'".to_string(),
            ));
        };
        if !operation_metadata
            .required_rows
            .contains(&"time.sleep".to_string())
        {
            return Err(EngineError::Configuration(
                "production time.sleep provider operation must require row 'time.sleep'"
                    .to_string(),
            ));
        }
        if operation_metadata.grants_authority {
            return Err(EngineError::Configuration(
                "production time.sleep provider operation must not grant authority".to_string(),
            ));
        }

        let operation = time_sleep_operation_identity();
        let binding = ProviderBindingV1::new(
            operation,
            TIME_SLEEP_OPERATION.provider,
            TIME_SLEEP_OPERATION.name,
        );
        let registered = RegisteredTimeSleepProviderBinding { binding, provider };
        let mut binding_slot = self
            .time_sleep_provider_binding
            .lock()
            .expect("time::sleep provider binding mutex poisoned");
        if let Some(existing) = binding_slot.as_ref()
            && existing.binding != registered.binding
        {
            return Err(EngineError::Configuration(
                "production time::sleep provider binding conflicts with the existing binding"
                    .to_string(),
            ));
        }
        *binding_slot = Some(registered);
        drop(binding_slot);
        Ok(())
    }

    fn registered_time_sleep_provider_binding(
        &self,
    ) -> Result<ResolvedProviderBinding, EngineError> {
        let binding_slot = self
            .time_sleep_provider_binding
            .lock()
            .expect("time::sleep provider binding mutex poisoned");
        let registered = binding_slot.as_ref().cloned().ok_or_else(|| {
            EngineError::Type(
                "production time::sleep admission requires an Engine-registered time.sleep provider binding"
                    .to_string(),
            )
        })?;
        drop(binding_slot);
        let current_provider = self
            .runtime_state
            .get_provider(registered.binding.provider_name())
            .ok_or_else(|| {
                EngineError::CapabilityNotFound(format!(
                    "production time::sleep provider '{}'",
                    registered.binding.provider_name()
                ))
            })?;
        if !std::sync::Arc::ptr_eq(&current_provider, &registered.provider) {
            return Err(EngineError::Configuration(
                "production time::sleep provider changed after binding registration".to_string(),
            ));
        }
        Ok(ResolvedProviderBinding::new(
            registered.binding,
            registered.provider,
        ))
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
    /// the engine-owned runtime stdlib registry, then masks that prelude before
    /// delegating to the ordinary single-application parser. Masking preserves
    /// the original source coordinates retained by lowering sidecars.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the leading runtime imports are not
    /// supported or not registered, or if the remaining application source
    /// contains syntax errors.
    pub fn parse_entry_source(&self, source: &str) -> Result<Entry, EngineError> {
        self.parse_runtime_entry_source_with_module_identity(source, None)
    }

    fn parse_runtime_entry_source_with_module_identity(
        &self,
        source: &str,
        module_identity: Option<&ash_core::semantic_summary::ModuleIdentity>,
    ) -> Result<Entry, EngineError> {
        entry::validate_runtime_entry_import_prelude(source, |module_path| {
            self.has_registered_runtime_module(module_path)
        })?;
        let source = entry::mask_leading_entry_use_prelude(source);
        self.parse_entry_source_with_imports(
            &source,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            &HashMap::new(),
            module_identity,
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
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)?;
        let module_identity = module_loader::module_identity_for_path(path);
        self.parse_runtime_entry_source_with_module_identity(&source, Some(&module_identity))
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
        source_path: Option<&str>,
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
        let (core, core_lowering) = match lower_expr_with_context(&entry_fn.body, &lowering_ctx) {
            Ok(core) => (core, EntryCoreLowering::Available),
            Err(error) if is_source_handler_lowering_unavailable(&error) => (
                Expr::Variable {
                    name: SOURCE_HANDLER_LOWERING_PLACEHOLDER.to_string(),
                    span: ash_core::Span { start: 0, end: 0 },
                },
                EntryCoreLowering::SourceHandlerUnavailable,
            ),
            Err(error) => return Err(EngineError::Parse(format!("lowering error: {error}"))),
        };

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
        let mut callable_contracts = BTreeMap::new();

        for def_item in &program.definitions {
            if let ash_parser::surface::Definition::Function(fn_def) = def_item {
                let (name, contract) = lower_local_callable_contract(fn_def, source_path)?;
                callable_contracts.insert(name.clone(), contract);

                let Ok(body_expr) = lower_expr_with_context(&fn_def.body, &lowering_ctx) else {
                    continue;
                };
                let slot = module_env.insert_late(name.clone());
                let closure_params: Vec<(String, Option<String>)> = fn_def
                    .params
                    .iter()
                    .map(|p| (p.name.to_string(), None))
                    .collect();
                local_param_counts.insert(name.clone(), closure_params.len());
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
                function_specs.push((name, closure_params, body_expr));
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
            callable_contracts,
            core,
            core_lowering,
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

        let module_source_path = module_identity.and_then(|identity| match &identity.source {
            ash_core::semantic_summary::ModuleSourceOrigin::File(path) => {
                Some(std::path::Path::new(path))
            }
            ash_core::semantic_summary::ModuleSourceOrigin::Inline { .. }
            | ash_core::semantic_summary::ModuleSourceOrigin::Synthetic { .. } => None,
        });
        let parsed_program =
            module_loader::parse_program_with_functions(source, module_source_path)
                .map_err(EngineError::Parse)?;
        let program = parsed_program.program;
        let source_path = parsed_program.source_path;
        let expansion_origins = parsed_program.expansion_origins;
        let identifier_hygiene = parsed_program.identifier_hygiene;

        let id = self.next_application_id();
        let (
            local_closures,
            local_param_counts,
            callable_row_requirements,
            core_callable_types,
            callable_contracts,
            core,
            core_lowering,
        ) = Self::process_program_definitions(
            &program,
            source_path.as_deref(),
            imported_closures,
            imported_param_counts,
            imported_callable_row_requirements,
            imported_core_callable_types,
        )?;

        let lowering_sidecars = entry_lowering_sidecars(
            &program,
            module_identity,
            expansion_origins,
            identifier_hygiene,
            callable_contracts,
        );
        self.store_canonical_entry_source_anchor(
            id,
            lowering_sidecars.entry_body_origin.clone(),
            core.clone(),
        );
        self.store_surface_program(id, program);
        if let Some(identity) = module_identity {
            self.store_surface_program_module_identity(id, identity.clone());
        }
        self.store_imported_semantic_summaries(id, imported_semantic_summaries);
        self.store_imported_type_function_heads(id, imported_type_function_heads);
        self.store_imported_type_defs(id, imported_type_defs);
        Ok(Entry {
            core,
            core_lowering,
            lowering_sidecars,
            id,
            owner_token: self.entry_owner_token.clone(),
            imported_closures: local_closures,
            imported_param_counts: local_param_counts,
            imported_fn_signatures,
            imported_builtin_signatures,
            callable_row_requirements,
            core_callable_types,
            declared_concrete_operation: None,
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

    /// Project checked source-handler facts for one entry retained by this engine.
    ///
    /// The projection is available only after the same entry has successfully
    /// passed [`Self::check`]. The checker result remains engine-owned, so
    /// callers cannot pair a source anchor with arbitrary typecheck output.
    ///
    /// # Errors
    ///
    /// Returns an error when the entry has not successfully passed
    /// [`Self::check`], or when `handler_name` does not identify checked
    /// source-handler facts for that entry.
    pub fn checked_source_facts_for_handler(
        &self,
        entry: &Entry,
        handler_name: &str,
    ) -> Result<CheckedSourceFactsV1, EngineError> {
        let checked = self.retained_checked_entry_result(entry)?;
        CheckedSourceFactsV1::from_type_check(
            &checked.result,
            handler_name,
            entry.lowering_sidecars.entry_body_origin.clone(),
        )
        .map_err(|error| EngineError::Type(error.to_string()))
    }

    fn retained_checked_entry_result(
        &self,
        entry: &Entry,
    ) -> Result<CheckedEntryTypeResult, EngineError> {
        if !self.owns_entry(entry) {
            return Err(EngineError::Type(
                "source facts provenance does not match this Engine entry".to_string(),
            ));
        }
        let checked_results = self
            .checked_type_results
            .lock()
            .map_err(|_| EngineError::Type("source facts require Engine::check".to_string()))?;
        let checked = checked_results
            .get(&entry.id)
            .ok_or_else(|| EngineError::Type("source facts require Engine::check".to_string()))?
            .clone();
        drop(checked_results);
        if !std::sync::Arc::ptr_eq(&checked.owner_token, &self.entry_owner_token)
            || !std::sync::Arc::ptr_eq(&checked.owner_token, &entry.owner_token)
            || checked.source_anchor != entry.lowering_sidecars.entry_body_origin
        {
            return Err(EngineError::Type(
                "source facts provenance anchor does not match the checked entry".to_string(),
            ));
        }
        Ok(checked)
    }

    fn has_retained_checked_entry_result(&self, entry: &Entry) -> Result<bool, EngineError> {
        let checked_results = self.checked_type_results.lock().map_err(|_| {
            EngineError::Type(
                "production checked-handler admission cannot read retained typechecker facts"
                    .to_string(),
            )
        })?;
        Ok(checked_results.contains_key(&entry.id))
    }

    /// Reject a mutable public operation sidecar that no longer agrees with
    /// the fact retained from the prior successful check. The next check may
    /// refresh ordinary diagnostics, but it must never turn a forged sidecar
    /// into production authority.
    fn validate_prior_declared_operation_sidecar(&self, entry: &Entry) -> Result<(), EngineError> {
        if !self.owns_entry(entry) {
            return Err(EngineError::Type(
                "production checked-CPS admission requires an entry issued by this Engine"
                    .to_string(),
            ));
        }
        let prior = self
            .checked_type_results
            .lock()
            .map_err(|_| {
                EngineError::Type(
                    "production checked-CPS admission cannot read retained typechecker facts"
                        .to_string(),
                )
            })?
            .get(&entry.id)
            .cloned();
        if let Some(prior) = prior {
            if prior.declared_concrete_operation != entry.declared_concrete_operation {
                return Err(EngineError::Type(
                    "production declared-operation sidecar does not match the retained checked declaration fact"
                        .to_string(),
                ));
            }
            if prior.declared_concrete_operation.is_some()
                && prior.checked_legacy_core != entry.core
            {
                return Err(EngineError::Type(
                    "production declared-operation Core does not match the retained checked source-derived Core"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Admit one checked source handler as a validated inspection artifact.
    ///
    /// This boundary verifies checked source facts, typed Core-to-CPS lowering,
    /// and one explicit root source-handler instruction. It does not execute
    /// the artifact, construct a frame, bind a provider, or start host work.
    ///
    /// # Errors
    ///
    /// Returns an error when the entry is unchecked, foreign, or mutated after
    /// checking; when the selected handler is outside the narrow inspection
    /// lowering subset; or when Core/CPS validation rejects its evidence.
    pub fn admit_checked_handler_inspection(
        &self,
        entry: &Entry,
        handler_name: &str,
    ) -> Result<CheckedHandlerInspectionAdmission, EngineError> {
        let checked = self.retained_checked_entry_result(entry)?;
        let source_facts = CheckedSourceFactsV1::from_type_check(
            &checked.result,
            handler_name,
            entry.lowering_sidecars.entry_body_origin.clone(),
        )
        .map_err(|error| EngineError::Type(error.to_string()))?;
        let program = self
            .get_surface_program(entry.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;
        let core = ash_typeck::lower_checked_handler_application_to_core(
            &program,
            &checked.result,
            program.entry.function.as_ref(),
        )
        .map_err(|error| EngineError::Type(error.to_string()))?;
        let CheckedCoreExpr::Handle { clause, .. } = &core else {
            return Err(EngineError::Type(
                "checked handler inspection lowering must produce a root Core Handle".to_string(),
            ));
        };
        let operation = source_facts
            .operation_identities()
            .first()
            .cloned()
            .ok_or_else(|| {
                EngineError::Type(
                    "checked handler inspection requires one operation clause".to_string(),
                )
            })?;
        let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
        type_env.operations_mut().insert(clause.op.clone());
        let validated = ash_core::core_ash_validate::validate_core_program(
            ash_core::core_ash_validate::RawCoreProgram::new(core),
        )
        .map_err(|error| EngineError::Type(format!("checked Core validation failed: {error}")))?;
        let checked_core = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
            validated,
            &type_env,
            ash_core::core_ash_lower::CoreLoweringContext::new(
                ash_core::cps::ContRef::Label("__handler_inspection_answer".to_string()),
                CoreRow::default(),
            ),
        )
        .map_err(|error| {
            EngineError::Type(format!("checked Core-to-CPS lowering failed: {error}"))
        })?;
        let root_instruction = FrameInstallationInstructionV1::SourceHandler {
            operation,
            handler_name: handler_name.to_string(),
            core_handle: CoreHandleLocatorV1::root(),
        };
        let sealed_admission = CheckedCpsAdmissionV1::validate(
            checked_core,
            source_facts,
            vec![root_instruction.clone()],
        )
        .map_err(|error| EngineError::Type(error.to_string()))?;
        Ok(CheckedHandlerInspectionAdmission::new(
            sealed_admission,
            self.handler_inspection_execution_token.clone(),
            entry.lowering_sidecars.entry_body_origin.clone(),
            handler_name.to_string(),
            root_instruction,
        ))
    }

    /// Execute one sealed checked handler-inspection admission.
    ///
    /// This is limited to a validated inspection artifact with explicit
    /// source-handler authority. It terminalizes the artifact's already
    /// checked CPS term and evaluates its handler semantics without creating
    /// provider frames, selecting providers, or using a direct evaluator.
    ///
    /// # Errors
    ///
    /// Returns an execution failure when the opaque artifact was not issued by
    /// this engine, its exact root handler authority is malformed, CPS
    /// evaluation fails, or its terminal value cannot cross the engine value
    /// boundary.
    pub fn execute_checked_handler_inspection(
        &self,
        admission: &CheckedHandlerInspectionAdmission,
    ) -> std::future::Ready<ExecResult<Value>> {
        if !admission.is_issued_by(&self.handler_inspection_execution_token)
            || !admission.has_exact_root_handler_instruction()
        {
            return std::future::ready(Err(ExecError::ExecutionFailed(
                "checked handler inspection execution requires Engine-issued inspection provenance with one exact root SourceHandler instruction".to_string(),
            )));
        }
        let executable = ash_core::cps::Term::LetCont {
            name: HANDLER_INSPECTION_ANSWER_CONTINUATION.to_string(),
            param: HANDLER_INSPECTION_ANSWER_VALUE.to_string(),
            cont_body: Box::new(ash_core::cps::Term::Return {
                value: ash_core::cps::Value::Atom(ash_core::cps::Atom::Var(
                    HANDLER_INSPECTION_ANSWER_VALUE.to_string(),
                )),
            }),
            body: Box::new(admission.checked_core().lowered().clone()),
            row: ash_core::cps::EffectRow::default(),
            multiplicity: ash_core::cps::ContMultiplicity::Affine,
        };
        let result = ash_interp::cps::eval_checked_terminal(
            &executable,
            &ash_core::cps::Env::new(),
            &ash_core::cps::HandlerChain::new(),
        )
        .map_err(|error| {
            ExecError::ExecutionFailed(format!(
                "checked handler inspection execution failed: {error}"
            ))
        })
        .and_then(|outcome| match outcome {
            ash_interp::cps::CpsTerminalOutcome::Return(value) => cps_value_to_engine_value(value),
            ash_interp::cps::CpsTerminalOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                format!("checked handler inspection terminal trap: {reason:?}"),
            )),
        });
        std::future::ready(result)
    }

    /// Admit the sole closed-empty typed source-handler production slice.
    ///
    /// This route is deliberately limited to the checked local
    /// `absorb_sleep` handler over `TestClock::sleep(Int) -> Int`. It reuses
    /// the checked handler inspection lowering and V1 evidence validation,
    /// but mints a separate Engine-issued production token. It does not grant
    /// generic handler execution or derive frames from rows.
    ///
    /// # Errors
    ///
    /// Returns an error when provenance or pre-existing checked facts fail,
    /// the handler facts do not describe the sealed contract, or Core/CPS
    /// validation rejects the one explicit root `SourceHandler` instruction.
    pub fn admit_production_checked_handler(
        &self,
        entry: &mut Entry,
    ) -> Result<CheckedHandlerProductionAdmission, EngineError> {
        if !self.owns_entry(entry) {
            return Err(EngineError::production_terminal(
                ProductionTerminalClassification::MissingAdmission,
                "production checked-handler admission requires an entry issued by this Engine",
            ));
        }
        let canonical_provenance = self
            .canonical_entry_source_provenance(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        if !self
            .has_retained_checked_entry_result(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?
        {
            return Err(EngineError::Type(
                "source facts require Engine::check".to_string(),
            ));
        }
        let checked = self
            .retained_checked_entry_result(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        let admitted_handler_names = [
            SEALED_PRODUCTION_HANDLER_NAME,
            SEALED_TRAP_SLEEP_HANDLER_NAME,
        ]
        .into_iter()
        .filter(|handler_name| checked.result.checked_handlers.contains_key(*handler_name))
        .collect::<Vec<_>>();
        let has_trap_sleep_candidate =
            admitted_handler_names.contains(&SEALED_TRAP_SLEEP_HANDLER_NAME);
        let handler_name = match admitted_handler_names.as_slice() {
            [handler_name] => *handler_name,
            [] => {
                return Err(EngineError::Type(
                    "production checked-handler admission requires the sealed absorb_sleep handler"
                        .to_string(),
                ));
            }
            _ => {
                return Err(sealed_handler_structural_rejection(
                    has_trap_sleep_candidate.then_some(SEALED_TRAP_SLEEP_HANDLER_NAME),
                    "production checked-handler admission requires exactly one sealed handler declaration",
                ));
            }
        };
        let handler = checked
            .result
            .checked_handlers
            .get(handler_name)
            .ok_or_else(|| {
                sealed_handler_structural_rejection(
                    Some(handler_name),
                    "production checked-handler admission lost its selected sealed handler fact",
                )
            })?;
        let [clause] = handler.clauses.as_slice() else {
            return Err(sealed_handler_structural_rejection(
                Some(handler_name),
                "production checked-handler admission requires exactly one sealed operation clause",
            ));
        };
        if !is_sealed_production_handler_operation(&clause.operation) {
            return Err(sealed_handler_structural_rejection(
                Some(handler_name),
                "production checked-handler admission does not admit this handler operation",
            ));
        }

        let inspection = self
            .admit_checked_handler_inspection(entry, handler_name)
            .map_err(|error| classify_sealed_handler_structural_error(handler_name, error))?;
        let admission = CheckedHandlerProductionAdmission::new(
            inspection.sealed_admission,
            self.production_handler_execution_token.clone(),
            entry.id,
            canonical_provenance.source_anchor,
            handler_name.to_string(),
            inspection.root_instruction,
        )
        .map_err(|error| classify_sealed_handler_structural_error(handler_name, error))?;
        if !admission.has_exact_closed_empty_handler_authority() {
            return Err(sealed_handler_structural_rejection(
                Some(handler_name),
                "production checked-handler admission requires one exact closed-empty root SourceHandler instruction",
            ));
        }
        Ok(admission)
    }

    /// Execute one Engine-issued sealed source-handler production admission.
    ///
    /// The private terminalized CPS term is evaluated without a legacy direct
    /// evaluator, provider selection, or generic V1 execution entrypoint.
    ///
    /// # Errors
    ///
    /// Returns an execution failure when the admission was issued by another
    /// Engine, is not the exact sealed handler contract, or CPS evaluation
    /// traps or cannot cross the engine value boundary.
    pub fn execute_production_checked_handler(
        &self,
        admission: &CheckedHandlerProductionAdmission,
    ) -> std::future::Ready<ExecResult<Value>> {
        if !admission.is_issued_by(&self.production_handler_execution_token)
            || !admission.has_exact_closed_empty_handler_authority()
        {
            return std::future::ready(Err(ExecError::ExecutionFailed(
                "production checked-handler execution requires Engine-issued sealed handler provenance"
                    .to_string(),
            )));
        }
        let result = ash_interp::cps::eval_checked_terminal(
            admission.executable(),
            &ash_core::cps::Env::new(),
            &ash_core::cps::HandlerChain::new(),
        )
        .map_err(|error| {
            ExecError::ExecutionFailed(format!(
                "production checked-handler execution failed: {error}"
            ))
        })
        .and_then(|outcome| match outcome {
            ash_interp::cps::CpsTerminalOutcome::Return(value) => cps_value_to_engine_value(value),
            ash_interp::cps::CpsTerminalOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                format!("production checked-handler terminal trap: {reason:?}"),
            )),
        });
        std::future::ready(result)
    }

    /// Admit TASK-2013's exact deep affine two-clause source handler.
    ///
    /// This is a closed production route: checked source facts preserve clause
    /// order and residual rows, while this method alone authorizes the two
    /// frame installations used by the private CPS execution artifact.
    fn admit_production_deep_affine_clock(
        &self,
        entry: &Entry,
    ) -> Result<DeepAffineClockProductionAdmission, EngineError> {
        if !self.owns_entry(entry) {
            return Err(missing_production_admission(&EngineError::Type(
                "deep_affine_clock production admission requires an entry issued by this Engine"
                    .to_string(),
            )));
        }
        let canonical_provenance = self
            .canonical_entry_source_provenance(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        let checked = self
            .retained_checked_entry_result(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        let handler = checked
            .result
            .checked_handlers
            .get(SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME)
            .ok_or_else(|| {
                missing_production_admission(&EngineError::Type(
                    "deep_affine_clock production admission requires its checked handler fact"
                        .to_string(),
                ))
            })?;
        let [sleep, wake] = handler.clauses.as_slice() else {
            return Err(missing_production_admission(&EngineError::Type(
                "deep_affine_clock production admission requires ordered sleep and wake clauses"
                    .to_string(),
            )));
        };
        if !is_sealed_production_handler_operation(&sleep.operation)
            || !is_sealed_forward_wake_operation(&wake.operation)
            || sleep.resume_name != "resume"
            || wake.resume_name != "resume"
            || handler.output_row.tail.is_some()
            || !handler.output_row.items.is_empty()
            || handler.residual_row.tail.is_some()
            || !handler.residual_row.items.is_empty()
            || handler.done_binding != "value"
            || handler.done_binding_type.to_string() != "Int"
            || handler.answer_type.to_string() != "Int"
        {
            return Err(missing_production_admission(&EngineError::Type(
                "deep_affine_clock production admission does not admit these checked handler facts"
                    .to_string(),
            )));
        }
        let program = self.get_surface_program(entry.id).ok_or_else(|| {
            invalid_checked_core_cps(&EngineError::Type(
                "deep_affine_clock production admission lost its parsed source program".to_string(),
            ))
        })?;
        if !is_exact_deep_affine_clock_source_program(&program) {
            return Err(missing_production_admission(&EngineError::Type(
                "deep_affine_clock production admission requires its exact local source sequence"
                    .to_string(),
            )));
        }
        let source_facts = CheckedSourceFactsV1::from_type_check(
            &checked.result,
            SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME,
            canonical_provenance.source_anchor.clone(),
        )
        .map_err(|error| missing_production_admission(&EngineError::Type(error.to_string())))?;
        DeepAffineClockProductionAdmission::new(
            self.production_deep_affine_clock_execution_token.clone(),
            entry.id,
            canonical_provenance.source_anchor,
            source_facts,
            &OperationIdentityV1::from_declared(&sleep.operation),
            &OperationIdentityV1::from_declared(&wake.operation),
        )
        .map_err(|error| missing_production_admission(&error))
    }

    /// Execute one Engine-issued TASK-2013 deep affine admission through the
    /// checked CPS interpreter with only the sealed, explicit deep frames.
    fn execute_production_deep_affine_clock(
        &self,
        admission: &DeepAffineClockProductionAdmission,
    ) -> std::future::Ready<ExecResult<Value>> {
        if !admission.is_issued_by(&self.production_deep_affine_clock_execution_token)
            || !admission.has_exact_authority()
        {
            return std::future::ready(Err(ExecError::ExecutionFailed(
                "deep_affine_clock execution requires Engine-issued sealed handler provenance"
                    .to_string(),
            )));
        }
        let mut chain = ash_core::cps::HandlerChain::new();
        for instruction in &admission.frame_installations {
            let FrameInstallationInstructionV1::SourceHandler { operation, .. } = instruction
            else {
                return std::future::ready(Err(ExecError::ExecutionFailed(
                    "deep_affine_clock execution received a non-handler frame instruction"
                        .to_string(),
                )));
            };
            let clause = deep_affine_resume_clause(operation);
            chain.push(ash_core::cps::HandlerFrame::Deep { clause });
        }
        let result = ash_interp::cps::eval_checked_terminal(
            &admission.executable,
            &ash_core::cps::Env::new(),
            &chain,
        )
        .map_err(|error| {
            ExecError::ExecutionFailed(format!("deep_affine_clock CPS execution failed: {error}"))
        })
        .and_then(|outcome| match outcome {
            ash_interp::cps::CpsTerminalOutcome::Return(value) => cps_value_to_engine_value(value),
            ash_interp::cps::CpsTerminalOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                format!("deep_affine_clock CPS terminal trap: {reason:?}"),
            )),
        });
        std::future::ready(result)
    }

    /// Materialize checked Core-to-CPS lowering for the bounded typed `PureAnf` entry fragment.
    ///
    /// This bridge is the checked lowering input for the narrow handler-free
    /// admission path. It accepts typed atoms, the approved integer binary
    /// primitives, recursive Boolean `Not`, variable `let` bindings, and
    /// boolean `if` matches; pure expressions are normalized to a left-to-right
    /// ANF binding spine and routed through the explicit `__answer`
    /// continuation. It does not admit provider operations or source handlers
    /// by itself.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Type`] when the legacy lowered expression cannot be
    /// represented by the currently checked Core-to-CPS prototype, or when that
    /// prototype rejects it during validation, type checking, or lowering.
    pub fn lower_entry_to_checked_cps(
        &self,
        entry: &Entry,
    ) -> Result<ash_core::cps::Term, EngineError> {
        #[cfg(test)]
        self.checked_cps_inspection_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if entry.core_lowering == EntryCoreLowering::SourceHandlerUnavailable {
            return Err(EngineError::Type(
                "checked Core/CPS entry admission requires typed handler lowering".to_string(),
            ));
        }
        if TIME_SLEEP_OPERATION.matches_legacy_call(&entry.core) {
            return checked_cps_time_sleep_raise(&entry.core);
        }
        if let Some(operation) = entry.declared_concrete_operation.as_ref() {
            return checked_cps_declared_operation_raise(&entry.core, operation);
        }
        let answer_input = self.checked_cps_answer_input_type(entry)?;
        if let Some(core) = self.checked_cps_exact_local_call_core(entry)? {
            return Self::checked_cps_lower_validated_core(core, answer_input);
        }
        if matches!(entry.core, Expr::Constructor { .. } | Expr::Record { .. }) {
            let value = checked_cps_structural_value_from_legacy_expr(&entry.core)?;
            // `__answer` is supplied by the sealed admission artifact, so the
            // term becomes a closed, validated CPS program only after that
            // artifact installs its terminal continuation.
            return Ok(ash_core::cps::Term::JumpValue {
                cont: ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
                arg: value,
                row: ash_core::cps::EffectRow::default(),
            });
        }
        let core = checked_core_expr_from_legacy_expr(&entry.core, &HashMap::new())?;
        Self::checked_cps_lower_validated_core(core, answer_input)
    }

    /// Recognize the one source-proven local-call shape admitted by TASK-2003.
    ///
    /// The general source call surface remains closed. This inspection accepts
    /// precisely one private, zero-argument `helper` returning either the
    /// literal `7` or the exact ambient `do { return 7; }` form, declared
    /// immediately before a matching zero-argument `main` that tail-calls it.
    /// It builds the existing checked Core `Lam`/`Call` form; no closure
    /// conversion, call inference, or imported callable route is enabled here.
    fn checked_cps_exact_local_call_core(
        &self,
        entry: &Entry,
    ) -> Result<Option<CheckedCoreExpr>, EngineError> {
        let program = self
            .get_surface_program(entry.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;
        let has_exact_local_call_source = checked_cps_is_exact_local_call_program(&program);
        if has_exact_local_call_source && !self.entry_has_no_retained_imported_state(entry.id)? {
            return Err(EngineError::Type(
                "checked Core/CPS local-call admission does not admit imported source state"
                    .to_string(),
            ));
        }
        if !matches!(entry.core_lowering, EntryCoreLowering::Available)
            || entry.declared_concrete_operation.is_some()
            || !has_exact_local_call_source
            || !checked_cps_is_exact_local_call_legacy_entry(&entry.core)
        {
            return Ok(None);
        }

        let helper_type = CoreType::Function {
            params: Vec::new(),
            result: Box::new(CoreType::Base("Int".to_string())),
            row: CoreRow::default(),
        };
        let main_type_matches = entry.core_callable_types.get("main") == Some(&helper_type);
        let helper_type_matches = entry.core_callable_types.get("helper") == Some(&helper_type);
        if entry.core_callable_types.len() != 2 || !main_type_matches || !helper_type_matches {
            return Ok(None);
        }

        Ok(Some(CheckedCoreExpr::LetVal {
            name: "helper".to_string(),
            ty: helper_type,
            value: CoreValue::Lam {
                params: Vec::new(),
                body: Box::new(CheckedCoreExpr::Atom(CoreAtom::LitInt(7))),
                row: CoreRow::default(),
            },
            body: Box::new(CheckedCoreExpr::Call {
                func: CoreAtom::Var("helper".to_string()),
                args: Vec::new(),
            }),
        }))
    }

    /// Validate and lower one already-selected checked Core fragment.
    fn checked_cps_lower_validated_core(
        core: CheckedCoreExpr,
        answer_input: CoreType,
    ) -> Result<ash_core::cps::Term, EngineError> {
        let validated = ash_core::core_ash_validate::validate_core_program(
            ash_core::core_ash_validate::RawCoreProgram::new(core),
        )
        .map_err(|error| EngineError::Type(format!("checked Core validation failed: {error}")))?;
        let context = ash_core::core_ash_lower::CoreLoweringContext::new(
            ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
            CoreRow::default(),
        );
        let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
        type_env.continuations_mut().insert(
            CHECKED_CPS_ANSWER_CONTINUATION,
            CoreType::Cont {
                input: Box::new(answer_input),
                answer: Box::new(CoreType::Base("Unit".to_string())),
                row: CoreRow::default(),
                multiplicity: CoreMultiplicity::Affine,
            },
        );
        let lowered = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
            validated, &type_env, context,
        )
        .map_err(|error| {
            EngineError::Type(format!("checked Core-to-CPS lowering failed: {error}"))
        })?;
        Ok(lowered.into_parts().1)
    }

    /// Check and admit one handler-free source entry to sealed checked CPS.
    ///
    /// The resulting artifact retains the exact entry source anchor and is
    /// executable only by [`Self::execute_checked_cps_admission`]. Source forms
    /// that lower to a provider raise or handler frame remain closed.
    ///
    /// # Errors
    ///
    /// Returns an error when source checking/lowering fails or when the
    /// lowered CPS term is not handler-free.
    pub fn admit_entry_to_checked_cps(
        &self,
        entry: &mut Entry,
    ) -> Result<CheckedCpsEntryAdmission, EngineError> {
        self.canonical_entry_source_provenance(entry)?;
        self.check(entry)?;
        let lowered = self
            .lower_entry_to_checked_cps(entry)
            .map_err(|error| missing_production_admission(&error))?;
        if checked_cps_term_has_handler_or_raise(&lowered) {
            return Err(EngineError::production_terminal(
                ProductionTerminalClassification::MissingAdmission,
                "checked Core/CPS entry admission currently accepts handler-free terms only",
            ));
        }
        Ok(CheckedCpsEntryAdmission::new(
            entry.id,
            entry.lowering_sidecars.entry_body_origin.clone(),
            lowered,
        ))
    }

    /// Seal the exact checked `time::sleep` source producer for later
    /// production checked-CPS execution.
    ///
    /// This is a deliberately closed admission slice. It accepts only an
    /// Engine-owned, freshly checked `fn main() -> Null { time::sleep(<literal>)
    /// }` with a non-negative integer literal, a separately registered exact
    /// `time.sleep` provider binding, and one explicit Provider installation
    /// instruction. It neither installs a frame nor executes a provider.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign or unchecked entries, source handlers,
    /// open or non-time requirements, malformed CPS, or a missing/mismatched
    /// registered provider binding.
    pub fn admit_production_checked_cps(
        &self,
        entry: &mut Entry,
    ) -> Result<CheckedCpsProductionAdmission, EngineError> {
        if !self.owns_entry(entry) {
            return Err(EngineError::production_terminal(
                ProductionTerminalClassification::MissingAdmission,
                "production checked-CPS admission requires an entry issued by this Engine",
            ));
        }
        let canonical_provenance = self
            .canonical_entry_source_provenance(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        self.validate_prior_declared_operation_sidecar(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        self.check(entry)
            .map_err(|error| missing_production_admission(&error))?;
        let checked = self
            .retained_checked_entry_result(entry)
            .map_err(|error| invalid_checked_core_cps(&error))?;
        if !checked.result.checked_handlers.is_empty()
            || !checked.result.checked_handler_applications.is_empty()
        {
            return Err(EngineError::production_terminal(
                ProductionTerminalClassification::MissingAdmission,
                "production checked-CPS time::sleep admission does not admit source handlers",
            ));
        }

        let (operation, checked_core, resolved_provider) = if let Some(builtin_operation) =
            checked.result.checked_builtin_operation.as_ref()
        {
            let (operation, checked_core) = checked_time_sleep_fact_to_checked_core(
                builtin_operation,
                &canonical_provenance.source_anchor,
            )
            .map_err(|error| invalid_checked_core_cps(&error))?;
            let resolved_provider = self
                .registered_time_sleep_provider_binding()
                .map_err(|error| missing_production_admission(&error))?;
            (operation, checked_core, resolved_provider)
        } else {
            let declared_operation =
                    checked
                        .declared_concrete_operation
                        .as_ref()
                        .ok_or_else(|| {
                            EngineError::production_terminal(
                                ProductionTerminalClassification::MissingAdmission,
                                "production admission requires an exact retained typechecker operation fact",
                            )
                        })?;
            if !is_sealed_declared_production_operation(declared_operation) {
                return Err(EngineError::production_terminal(
                    ProductionTerminalClassification::MissingAdmission,
                    "production declared-operation admission does not admit this declaration",
                ));
            }
            let (operation, checked_core) = checked_declared_operation_fact_to_checked_core(
                &canonical_provenance.parsed_legacy_core,
                declared_operation,
            )
            .map_err(|error| invalid_checked_core_cps(&error))?;
            let resolved_provider = self
                .registered_declared_production_provider_binding(declared_operation)
                .map_err(|error| missing_production_admission(&error))?;
            (operation, checked_core, resolved_provider)
        };
        let frame_installations = vec![FrameInstallationInstructionV1::Provider {
            operation: operation.clone(),
            provider_binding: resolved_provider.binding().clone(),
        }];
        CheckedCpsProductionAdmission::validate_production_single_provider_raise(
            self.production_checked_cps_execution_token.clone(),
            entry.id,
            canonical_provenance.source_anchor,
            checked_core,
            &operation,
            resolved_provider,
            frame_installations,
        )
        .map_err(|error| {
            EngineError::production_terminal(
                ProductionTerminalClassification::InvalidCheckedCoreCps,
                error.to_string(),
            )
        })
    }

    /// Creates one execution-phase-wide cooperative control envelope.
    ///
    /// The admission argument makes it impossible to create this envelope
    /// before production admission. The issuing Engine is verified here; the
    /// envelope itself contains no source, CPS, row, frame, or provider
    /// authority.
    ///
    /// # Errors
    ///
    /// Returns an error when the admission was minted by another Engine or
    /// the requested deadline cannot be represented by `tokio::time::Instant`.
    pub fn new_production_run_control(
        &self,
        admission: &CheckedCpsProductionAdmission,
        timeout: Option<std::time::Duration>,
    ) -> Result<(ProductionRunControl, ProductionCancellation), EngineError> {
        if !admission.is_issued_by(&self.production_checked_cps_execution_token) {
            return Err(EngineError::Type(
                "production run control requires an admission issued by this Engine".to_string(),
            ));
        }
        production_cps_driver::ProductionRunControl::new(admission, timeout)
    }

    /// Executes one Engine-issued production admission through the private
    /// checked-CPS provider driver.
    ///
    /// # Errors
    ///
    /// Returns an error when the token was issued by another Engine, the
    /// control was created for a different admission, its private frame
    /// handoff is inconsistent, or the sealed CPS is malformed.
    /// Timeout and cancellation are typed successful observations so the CLI
    /// can project them to its versioned terminal envelope in a later task.
    pub fn execute_production_checked_cps(
        &self,
        admission: &CheckedCpsProductionAdmission,
        control: ProductionRunControl,
    ) -> impl std::future::Future<Output = Result<ProductionCheckedCpsOutcome, EngineError>> {
        let prepared = if control.is_for_admission(admission) {
            production_cps_driver::prepare_production_checked_cps(self, admission)
        } else {
            Err(EngineError::Type(
                "production checked-CPS execution control is bound to another admission"
                    .to_string(),
            ))
        };
        async move {
            let prepared = prepared?;
            prepared.execute(control).await
        }
    }

    /// Admit TASK-2026's exact `forward_sleep` handler/provider composition.
    ///
    /// This is a closed Path-B route: it seals a canonical checked root
    /// `Handle`, the source handler facts, one Engine-registered `wake`
    /// provider object, and two explicit installation instructions.  It does
    /// not make nonempty rows, generic handlers, or public V1 evidence
    /// executable.
    ///
    /// # Errors
    ///
    /// Returns an error for any source/Core/anchor provenance mutation,
    /// imported state, unchecked or nonexact handler fact, absent/mismatched
    /// binding, or invalid Core/CPS evidence.
    pub fn admit_production_forward_sleep(
        &self,
        entry: &mut Entry,
    ) -> Result<ForwardSleepProductionAdmission, EngineError> {
        let canonical_provenance = self.canonical_entry_source_provenance(entry)?;
        if !self.entry_has_no_retained_imported_state(entry.id)? {
            return Err(EngineError::Type(
                "forward_sleep production admission does not admit imported source state"
                    .to_string(),
            ));
        }
        self.check(entry)?;
        let checked = self.retained_checked_entry_result(entry)?;
        if checked.result.checked_handlers.len() != 1
            || checked.result.checked_handler_applications.len() != 1
        {
            return Err(EngineError::Type(
                "forward_sleep production admission requires exactly one checked handler and application"
                    .to_string(),
            ));
        }
        let (sleep_declared, wake_declared) = Self::sealed_forward_sleep_operation_facts(&checked)?;
        let program = self
            .get_surface_program(entry.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;
        if !is_exact_forward_sleep_source_program(&program) {
            return Err(EngineError::Type(
                "forward_sleep production admission requires its exact local source declaration shape"
                    .to_string(),
            ));
        }
        let core = ash_typeck::lower_checked_handler_application_to_core(
            &program,
            &checked.result,
            program.entry.function.as_ref(),
        )
        .map_err(|error| EngineError::Type(error.to_string()))?;
        let CheckedCoreExpr::Handle { .. } = &core else {
            return Err(EngineError::Type(
                "forward_sleep production admission requires a root checked Core Handle"
                    .to_string(),
            ));
        };
        let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
        for operation in [&sleep_declared, &wake_declared] {
            if !type_env
                .operations_mut()
                .insert(core_operation_from_declared(operation))
            {
                return Err(EngineError::Type(
                    "forward_sleep production admission received duplicate Core operation facts"
                        .to_string(),
                ));
            }
        }
        let validated = ash_core::core_ash_validate::validate_core_program(
            ash_core::core_ash_validate::RawCoreProgram::new(core),
        )
        .map_err(|error| EngineError::Type(format!("checked Core validation failed: {error}")))?;
        let checked_core = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
            validated,
            &type_env,
            ash_core::core_ash_lower::CoreLoweringContext::new(
                ash_core::cps::ContRef::Label(FORWARD_SLEEP_ANSWER_CONTINUATION.to_string()),
                CoreRow::default(),
            ),
        )
        .map_err(|error| {
            EngineError::Type(format!("checked Core-to-CPS lowering failed: {error}"))
        })?;
        let sleep_operation = OperationIdentityV1::from_declared(&sleep_declared);
        let wake_operation = OperationIdentityV1::from_declared(&wake_declared);
        let resolved_wake_providers =
            self.registered_forward_sleep_wake_provider_bindings(&wake_declared)?;
        let mut frame_installations = resolved_wake_providers
            .iter()
            .map(|provider| FrameInstallationInstructionV1::Provider {
                operation: wake_operation.clone(),
                provider_binding: provider.binding().clone(),
            })
            .collect::<Vec<_>>();
        frame_installations.push(FrameInstallationInstructionV1::SourceHandler {
            operation: sleep_operation.clone(),
            handler_name: SEALED_FORWARD_SLEEP_HANDLER_NAME.to_string(),
            core_handle: CoreHandleLocatorV1::root(),
        });
        let source_facts = CheckedSourceFactsV1::from_type_check(
            &checked.result,
            SEALED_FORWARD_SLEEP_HANDLER_NAME,
            canonical_provenance.source_anchor.clone(),
        )
        .map_err(|error| EngineError::Type(error.to_string()))?;
        let sealed_admission =
            CheckedCpsAdmissionV1::validate(checked_core, source_facts, frame_installations)
                .map_err(|error| EngineError::Type(error.to_string()))?;
        ForwardSleepProductionAdmission::new(
            sealed_admission,
            self.production_forward_sleep_execution_token.clone(),
            entry.id,
            canonical_provenance.source_anchor,
            sleep_operation,
            wake_operation,
            resolved_wake_providers,
        )
    }

    /// Create the ordinary run-wide timeout/cancellation envelope for one
    /// Engine-issued `forward_sleep` admission.
    ///
    /// # Errors
    ///
    /// Returns an error when the admission belongs to another Engine or the
    /// requested deadline cannot be represented by `tokio::time::Instant`.
    pub fn new_forward_sleep_run_control(
        &self,
        admission: &ForwardSleepProductionAdmission,
        timeout: Option<std::time::Duration>,
    ) -> Result<(ProductionRunControl, ProductionCancellation), EngineError> {
        if !admission.is_issued_by(&self.production_forward_sleep_execution_token)
            || !admission.has_exact_authority()
        {
            return Err(EngineError::Type(
                "forward_sleep run control requires an admission issued by this Engine".to_string(),
            ));
        }
        production_cps_driver::ProductionRunControl::new_with_admission_token(
            admission.run_control_token(),
            timeout,
        )
    }

    /// Execute one Engine-issued sealed `forward_sleep` production admission
    /// through the private checked-CPS handler/provider driver.
    ///
    /// # Errors
    ///
    /// The returned future fails when the issuer/control seals disagree or the
    /// sealed checked-CPS driver detects malformed authority or provider data.
    pub fn execute_production_forward_sleep(
        &self,
        admission: &ForwardSleepProductionAdmission,
        control: ProductionRunControl,
    ) -> impl std::future::Future<Output = Result<ProductionCheckedCpsOutcome, EngineError>> + use<>
    {
        let prepared = if admission.is_issued_by(&self.production_forward_sleep_execution_token)
            && admission.has_exact_authority()
            && control.is_for_forward_sleep_admission(admission)
        {
            production_cps_driver::prepare_production_forward_sleep(self, admission)
        } else {
            Err(EngineError::Type(
                "forward_sleep production execution requires its Engine-issued sealed admission and control"
                    .to_string(),
            ))
        };
        async move {
            let prepared = prepared?;
            prepared.execute(control).await
        }
    }

    /// Execute one sealed handler-free checked Core/CPS admission.
    ///
    /// The evaluator validates CPS before running it under an empty environment
    /// and empty handler chain. No direct expression evaluator, provider, or
    /// source-handler frame is available on this path.
    ///
    /// # Errors
    ///
    /// Returns an execution failure for invalid CPS, a CPS trap, or an atom
    /// that cannot cross the engine value boundary.
    pub fn execute_checked_cps_admission(
        &self,
        admission: &CheckedCpsEntryAdmission,
    ) -> std::future::Ready<ExecResult<Value>> {
        let _ = admission.entry_id();
        let result = ash_interp::cps::eval_checked_terminal(
            admission.executable(),
            &ash_core::cps::Env::new(),
            &ash_core::cps::HandlerChain::new(),
        )
        .map_err(|error| {
            ExecError::ExecutionFailed(format!("checked Core/CPS execution failed: {error}"))
        })
        .and_then(|outcome| match outcome {
            ash_interp::cps::CpsTerminalOutcome::Return(value) => cps_value_to_engine_value(value),
            ash_interp::cps::CpsTerminalOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                format!("checked Core/CPS terminal trap: {reason:?}"),
            )),
        });
        std::future::ready(result)
    }

    fn checked_cps_answer_input_type(&self, entry: &Entry) -> Result<CoreType, EngineError> {
        let program = self
            .get_surface_program(entry.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;
        let entry_name = program.entry.function.to_string();
        let function_type = entry.core_callable_types.get(&entry_name).ok_or_else(|| {
            EngineError::Type(format!(
                "checked entry '{entry_name}' has no canonical Core function type"
            ))
        })?;
        let CoreType::Function { result, .. } = function_type else {
            return Err(EngineError::Type(format!(
                "checked entry '{entry_name}' did not lower to a Core function type"
            )));
        };
        Ok(result.as_ref().clone())
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
        if !self.owns_entry(application) {
            return Err(EngineError::Type(
                "entry provenance does not belong to this Engine".to_string(),
            ));
        }
        self.clear_checked_type_result(application.id);
        let program = self
            .get_surface_program(application.id)
            .ok_or_else(|| EngineError::Type("program metadata not found in cache".to_string()))?;

        let mut type_env = ash_typeck::type_env::TypeEnv::with_builtin_types();
        if self.has_registered_runtime_module("time") {
            type_env.bind_variable(
                "sleep",
                ash_typeck::Type::Fn(
                    vec![ash_typeck::Type::Int],
                    Box::new(ash_typeck::Type::Null),
                ),
            );
        }
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

        let declaration_module_identity = self
            .get_surface_program_module_identity(application.id)
            .unwrap_or_else(ash_typeck::standalone_program_module_identity);
        let type_check_result = ash_typeck::type_check_program_in_env_for_module_with_config(
            &type_env,
            &program,
            declaration_module_identity.clone(),
            typeck_config,
        );

        match type_check_result {
            Ok(result) => {
                if result.is_ok() {
                    let declaration_env = declaration_resolution_env(
                        &type_env,
                        &program,
                        declaration_module_identity,
                    )
                    .map_err(EngineError::Type)?;
                    monomorphize::monomorphize_expr(&mut application.core, &declaration_env)
                        .map_err(|e| EngineError::Type(e.to_string()))?;
                    attach_time_sleep_requirement_row(application);
                    attach_declared_concrete_operation(application, &declaration_env);
                    self.store_checked_type_result(application, result);
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
    /// Execute an entry asynchronously.
    ///
    /// # Errors
    ///
    /// Returns a closed-admission error until a validated production
    /// checked-Core/CPS artifact is available for `application`.
    #[allow(clippy::unused_async)]
    pub async fn execute(&self, application: &Entry) -> ExecResult<Value> {
        let _ = application;
        Err(closed_checked_cps_admission_error())
    }

    /// Reject an application until it is supplied as a validated production
    /// checked-Core/CPS admission artifact.
    pub async fn admit_application(
        &self,
        request: ApplicationAdmissionRequest,
    ) -> ApplicationAdmissionOutcome {
        let application_id = request.application_id.unwrap_or_default();
        let run_id = request.run_id.unwrap_or_default();
        let admitted_capability_bindings = self
            .runtime_state
            .resolve_admitted_capability_bindings(&request.required_capabilities)
            .await;
        let admission = ApplicationAdmissionContext {
            active_role: admitted_role_name(&request).map(ToOwned::to_owned),
            admitted_capabilities: request.required_capabilities.clone(),
            admitted_capability_bindings,
            requires_evidence: Vec::new(),
        };
        let ensures_evidence = build_pending_ensures_evidence(&request.ensures);
        reject_admission(
            application_id,
            run_id,
            ApplicationFailureKind::AdmissionFailure,
            admission,
            Vec::new(),
            ensures_evidence,
        )
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
    /// Returns a closed-admission error until a validated production
    /// checked-Core/CPS artifact is available for `application`.
    #[allow(clippy::unused_async)]
    pub async fn execute_with_input(
        &self,
        application: &Entry,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        let _ = (application, input_bindings);
        Err(closed_checked_cps_admission_error())
    }
    /// Parse, check, and execute in one call
    ///
    /// Convenience method that chains parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered at any stage.
    #[allow(clippy::unused_async)]
    pub async fn run(&self, source: &str) -> ExecResult<Value> {
        let mut application = self.parse(source)?;
        if application.core_lowering == EntryCoreLowering::SourceHandlerUnavailable {
            self.check(&mut application).map_err(|error| {
                ExecError::ExecutionFailed(format!(
                    "checked handler production admission rejected: {error}"
                ))
            })?;
            if self
                .retained_checked_entry_result(&application)
                .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?
                .result
                .checked_handlers
                .contains_key(SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME)
            {
                let admission = self
                    .admit_production_deep_affine_clock(&application)
                    .map_err(|error| {
                        ExecError::ExecutionFailed(format!(
                            "deep_affine_clock production admission rejected: {error}"
                        ))
                    })?;
                return self
                    .execute_production_deep_affine_clock(&admission)
                    .into_inner();
            }
            if self
                .retained_checked_entry_result(&application)
                .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?
                .result
                .checked_handlers
                .contains_key(SEALED_FORWARD_SLEEP_HANDLER_NAME)
            {
                let execution = {
                    let admission = self
                        .admit_production_forward_sleep(&mut application)
                        .map_err(|error| {
                            ExecError::ExecutionFailed(format!(
                                "forward_sleep production admission rejected: {error}"
                            ))
                        })?;
                    let (control, _) = self
                        .new_forward_sleep_run_control(&admission, None)
                        .map_err(|error| {
                            ExecError::ExecutionFailed(format!(
                                "forward_sleep production control rejected: {error}"
                            ))
                        })?;
                    self.execute_production_forward_sleep(&admission, control)
                };
                return match execution.await.map_err(|error| {
                    ExecError::ExecutionFailed(format!(
                        "forward_sleep production execution failed: {error}"
                    ))
                })? {
                    ProductionCheckedCpsOutcome::Return(value) => Ok(value),
                    ProductionCheckedCpsOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                        format!("forward_sleep production terminal trap: {reason:?}"),
                    )),
                    ProductionCheckedCpsOutcome::TimedOut => Err(ExecError::ExecutionFailed(
                        "forward_sleep production timed out".to_string(),
                    )),
                    ProductionCheckedCpsOutcome::Cancelled => Err(ExecError::ExecutionFailed(
                        "forward_sleep production cancelled".to_string(),
                    )),
                };
            }
            let admission = self
                .admit_production_checked_handler(&mut application)
                .map_err(|error| {
                    ExecError::ExecutionFailed(format!(
                        "checked handler production admission rejected: {error}"
                    ))
                })?;
            return self
                .execute_production_checked_handler(&admission)
                .into_inner();
        }
        let admission = self
            .admit_entry_to_checked_cps(&mut application)
            .map_err(|error| {
                ExecError::ExecutionFailed(format!("checked Core/CPS admission rejected: {error}"))
            })?;
        self.execute_checked_cps_admission(&admission).into_inner()
    }

    /// Parse, check, and execute a application from a file
    ///
    /// Convenience method that reads a file and then runs parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    #[allow(clippy::unused_async)]
    pub async fn run_file(&self, path: impl AsRef<std::path::Path> + Send) -> ExecResult<Value> {
        let mut application = self.parse_file(path)?;
        if application.core_lowering == EntryCoreLowering::SourceHandlerUnavailable {
            self.check(&mut application).map_err(|error| {
                ExecError::ExecutionFailed(format!(
                    "checked handler production admission rejected: {error}"
                ))
            })?;
            if self
                .retained_checked_entry_result(&application)
                .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?
                .result
                .checked_handlers
                .contains_key(SEALED_DEEP_AFFINE_CLOCK_HANDLER_NAME)
            {
                let admission = self
                    .admit_production_deep_affine_clock(&application)
                    .map_err(|error| {
                        ExecError::ExecutionFailed(format!(
                            "deep_affine_clock production admission rejected: {error}"
                        ))
                    })?;
                return self
                    .execute_production_deep_affine_clock(&admission)
                    .into_inner();
            }
            if self
                .retained_checked_entry_result(&application)
                .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?
                .result
                .checked_handlers
                .contains_key(SEALED_FORWARD_SLEEP_HANDLER_NAME)
            {
                let execution = {
                    let admission = self
                        .admit_production_forward_sleep(&mut application)
                        .map_err(|error| {
                            ExecError::ExecutionFailed(format!(
                                "forward_sleep production admission rejected: {error}"
                            ))
                        })?;
                    let (control, _) = self
                        .new_forward_sleep_run_control(&admission, None)
                        .map_err(|error| {
                            ExecError::ExecutionFailed(format!(
                                "forward_sleep production control rejected: {error}"
                            ))
                        })?;
                    self.execute_production_forward_sleep(&admission, control)
                };
                return match execution.await.map_err(|error| {
                    ExecError::ExecutionFailed(format!(
                        "forward_sleep production execution failed: {error}"
                    ))
                })? {
                    ProductionCheckedCpsOutcome::Return(value) => Ok(value),
                    ProductionCheckedCpsOutcome::Trap(reason) => Err(ExecError::ExecutionFailed(
                        format!("forward_sleep production terminal trap: {reason:?}"),
                    )),
                    ProductionCheckedCpsOutcome::TimedOut => Err(ExecError::ExecutionFailed(
                        "forward_sleep production timed out".to_string(),
                    )),
                    ProductionCheckedCpsOutcome::Cancelled => Err(ExecError::ExecutionFailed(
                        "forward_sleep production cancelled".to_string(),
                    )),
                };
            }
            let admission = self
                .admit_production_checked_handler(&mut application)
                .map_err(|error| {
                    ExecError::ExecutionFailed(format!(
                        "checked handler production admission rejected: {error}"
                    ))
                })?;
            return self
                .execute_production_checked_handler(&admission)
                .into_inner();
        }
        let admission = self
            .admit_entry_to_checked_cps(&mut application)
            .map_err(|error| {
                ExecError::ExecutionFailed(format!("checked Core/CPS admission rejected: {error}"))
            })?;
        self.execute_checked_cps_admission(&admission).into_inner()
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
    pub async fn bootstrap_entry_source_result(
        &self,
        source: &str,
    ) -> Result<EntryBootstrapResult, EntryBootstrapError> {
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
            let admission = self
                .admit_entry_to_checked_cps(&mut application)
                .map_err(|error| {
                    EntryBootstrapError::Execution(format!(
                        "checked Core/CPS admission rejected: {error}"
                    ))
                })?;
            self.execute_checked_cps_admission(&admission)
                .into_inner()
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        } else {
            self.execute_with_input(&application, input_bindings)
                .await
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        };

        let exit_code = entry::derive_entry_exit_code(&result).map_err(|error| *error)?;
        Ok(EntryBootstrapResult {
            terminal_value: result,
            exit_code,
        })
    }

    /// Parse, check, verify, and execute an entry source, returning its exit code.
    ///
    /// Prefer [`Self::bootstrap_entry_source_result`] at host boundaries which
    /// also need to project the terminal language result.
    ///
    /// # Errors
    ///
    /// Returns [`EntryBootstrapError`] when loading, parsing, checking,
    /// verification, execution, or exit-code derivation fails.
    pub async fn bootstrap_entry_source(&self, source: &str) -> Result<u8, EntryBootstrapError> {
        self.bootstrap_entry_source_result(source)
            .await
            .map(|result| result.exit_code)
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
            entry_owner_token: std::sync::Arc::new(()),
            handler_inspection_execution_token: std::sync::Arc::new(()),
            production_checked_cps_execution_token: std::sync::Arc::new(()),
            production_handler_execution_token: std::sync::Arc::new(()),
            production_forward_sleep_execution_token: std::sync::Arc::new(()),
            production_deep_affine_clock_execution_token: std::sync::Arc::new(()),
            canonical_entry_source_anchors: std::sync::Mutex::new(HashMap::new()),
            checked_type_results: std::sync::Mutex::new(HashMap::new()),
            runtime_stdlib_modules: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
            #[cfg(test)]
            checked_cps_inspection_calls: std::sync::atomic::AtomicU64::new(0),
            runtime_state,
            capability_implementation_selections,
            resource_initializer_selections,
            declared_operation_provider_bindings: std::sync::Mutex::new(HashMap::new()),
            declared_production_provider_bindings: std::sync::Mutex::new(HashMap::new()),
            time_sleep_provider_binding: std::sync::Mutex::new(None),
            forward_sleep_wake_provider_binding: std::sync::Mutex::new(Vec::new()),
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
        let has_operational_operation = provider
            .provider_metadata()
            .operations
            .iter()
            .any(|operation| operation.effect.at_least(ash_core::Effect::Operational));
        if !name.is_empty() && has_operational_operation {
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

/// Checks the entire source declaration shape for the one admitted local call.
///
/// Requiring exactly two private function declarations keeps this separate
/// from imported functions, overloads, recursion, forward references, rows,
/// contracts, handlers, and higher-order source forms.
fn checked_cps_is_exact_local_call_program(program: &ash_parser::surface::Program) -> bool {
    let [
        ash_parser::surface::Definition::Function(helper),
        ash_parser::surface::Definition::Function(main),
    ] = program.definitions.as_slice()
    else {
        return false;
    };
    program.entry.function.as_ref() == "main"
        && checked_cps_is_exact_local_call_function(helper, "helper")
        && checked_cps_is_exact_local_call_function(main, "main")
        && checked_cps_is_exact_local_call_helper_body(&helper.body)
        && checked_cps_is_exact_local_call_main_body(&main.body)
}

fn checked_cps_is_exact_local_call_function(
    function: &ash_parser::surface::FnDef,
    expected_name: &str,
) -> bool {
    matches!(
        function.visibility,
        ash_parser::surface::Visibility::Inherited
    ) && function.name.as_ref() == expected_name
        && function.type_params.is_empty()
        && function.params.is_empty()
        && function.proposition_tail.is_none()
        && function.contract.is_none()
        && matches!(
            function.return_type.as_ref(),
            Some(ash_parser::surface::Type::Name(name)) if name.as_ref() == "Int"
        )
}

fn checked_cps_is_exact_local_call_helper_body(body: &ash_parser::surface::Expr) -> bool {
    matches!(
        body,
        ash_parser::surface::Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty()
            && matches!(
                tail_expr.as_ref(),
                ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(7))
            )
    ) || matches!(
        body,
        ash_parser::surface::Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty()
            && matches!(
                tail_expr.as_ref(),
                ash_parser::surface::Expr::DoBlock { target, stmts, .. }
                    if target.name.as_ref() == "__ambient"
                        && target.args.is_empty()
                        && matches!(
                            stmts.as_slice(),
                            [ash_parser::surface::DoStmt::Return { value, .. }]
                                if matches!(
                                    value.as_ref(),
                                    ash_parser::surface::Expr::Literal(
                                        ash_parser::surface::Literal::Int(7)
                                    )
                                )
                        )
            )
    )
}

fn checked_cps_is_exact_local_call_main_body(body: &ash_parser::surface::Expr) -> bool {
    matches!(
        body,
        ash_parser::surface::Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty()
            && matches!(
                tail_expr.as_ref(),
                ash_parser::surface::Expr::Call { func, module: None, args, .. }
                    if func.as_ref() == "helper" && args.is_empty()
            )
    )
}

fn checked_cps_is_exact_local_call_legacy_entry(entry: &Expr) -> bool {
    matches!(
        entry,
        Expr::FnApply { func, args }
            if args.is_empty()
                && matches!(func.as_ref(), Expr::Variable { name, .. } if name == "helper")
    )
}

fn checked_core_expr_from_legacy_expr(
    expr: &Expr,
    atom_types: &HashMap<String, CoreType>,
) -> Result<CheckedCoreExpr, EngineError> {
    let mut source_names = HashSet::new();
    checked_core_collect_source_names(expr, &mut source_names);
    checked_core_expr_from_legacy_expr_with_source_names(expr, atom_types, &source_names)
}

fn checked_core_expr_from_legacy_expr_with_source_names(
    expr: &Expr,
    atom_types: &HashMap<String, CoreType>,
    source_names: &HashSet<String>,
) -> Result<CheckedCoreExpr, EngineError> {
    match expr {
        Expr::Literal(_) | Expr::Variable { .. } | Expr::Binary { .. } | Expr::Unary { .. } => {
            checked_core_pure_anf_expr(expr, atom_types, source_names)
        }
        Expr::Let {
            pattern,
            expr,
            body,
            ..
        } => {
            let ash_core::Pattern::Variable { name, .. } = pattern else {
                return Err(EngineError::Type(
                    "checked Core-to-CPS bridge accepts only variable let patterns".to_string(),
                ));
            };
            let mut body_atom_types = atom_types.clone();
            let (value, bindings) =
                checked_core_pure_anf_atom(expr, &mut body_atom_types, source_names)?;
            let ty = checked_core_atom_type(&value, &body_atom_types)?;
            body_atom_types.insert(name.clone(), ty.clone());
            let let_value = CheckedCoreExpr::LetVal {
                name: name.clone(),
                ty,
                value: CoreValue::Atom(value),
                body: Box::new(checked_core_expr_from_legacy_expr_with_source_names(
                    body,
                    &body_atom_types,
                    source_names,
                )?),
            };
            Ok(checked_core_wrap_prim_bindings(bindings, let_value))
        }
        Expr::Match { scrutinee, arms } => {
            let mut condition_atom_types = atom_types.clone();
            let (condition, bindings) = checked_core_pure_anf_atom(
                scrutinee,
                &mut condition_atom_types,
                source_names,
            )?;
            checked_core_require_boolean_atom(&condition, &condition_atom_types)?;
            let (then_branch, else_branch) = checked_core_boolean_match_branches(arms)?;
            let conditional = CheckedCoreExpr::If {
                cond: condition,
                then_branch: Box::new(checked_core_expr_from_legacy_expr_with_source_names(
                    then_branch,
                    atom_types,
                    source_names,
                )?),
                else_branch: Box::new(checked_core_expr_from_legacy_expr_with_source_names(
                    else_branch,
                    atom_types,
                    source_names,
                )?),
            };
            Ok(checked_core_wrap_prim_bindings(bindings, conditional))
        }
        _ => Err(EngineError::Type(
            "checked Core-to-CPS bridge currently accepts pure typed atoms, approved integer binary primitives, recursive Boolean Not, variable-let, and boolean-if entry results".to_string(),
        )),
    }
}

struct CheckedCorePrimBinding {
    name: String,
    op: CorePrimOp,
    args: Vec<CoreAtom>,
}

/// Normalizes the sealed, handler-free pure fragment to one left-to-right ANF
/// binding spine. Every caller supplies its own terminal context so the same
/// typed normalization is used for entry results, let RHSs, conditions, and
/// branches without granting admission to effectful forms.
fn checked_core_pure_anf_expr(
    expr: &Expr,
    atom_types: &HashMap<String, CoreType>,
    source_names: &HashSet<String>,
) -> Result<CheckedCoreExpr, EngineError> {
    let mut anf_atom_types = atom_types.clone();
    let (result, bindings) = checked_core_pure_anf_atom(expr, &mut anf_atom_types, source_names)?;
    let terminal = CheckedCoreExpr::Jump {
        cont: CoreContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
        arg: result,
    };

    Ok(checked_core_wrap_prim_bindings(bindings, terminal))
}

fn checked_core_wrap_prim_bindings(
    bindings: Vec<CheckedCorePrimBinding>,
    terminal: CheckedCoreExpr,
) -> CheckedCoreExpr {
    bindings
        .into_iter()
        .rev()
        .fold(terminal, |body, binding| CheckedCoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(body),
        })
}

fn checked_core_pure_anf_atom(
    expr: &Expr,
    atom_types: &mut HashMap<String, CoreType>,
    source_names: &HashSet<String>,
) -> Result<(CoreAtom, Vec<CheckedCorePrimBinding>), EngineError> {
    match expr {
        Expr::Literal(_) | Expr::Variable { .. } => {
            let atom = checked_core_atom_from_legacy_expr(expr)?;
            let _ = checked_core_atom_type(&atom, atom_types)?;
            Ok((atom, Vec::new()))
        }
        Expr::Binary {
            op,
            left,
            right,
            ..
        } => {
            let Some((op, result_name_base)) = checked_core_binary_primitive(*op) else {
                return Err(EngineError::Type(
                    "checked Core-to-CPS bridge currently accepts approved integer binary primitives and recursive Boolean Not in the pure ANF fragment".to_string(),
                ));
            };
            let (left, mut bindings) =
                checked_core_pure_anf_atom(left, atom_types, source_names)?;
            let (right, right_bindings) =
                checked_core_pure_anf_atom(right, atom_types, source_names)?;
            let left_type = checked_core_atom_type(&left, atom_types)?;
            let right_type = checked_core_atom_type(&right, atom_types)?;
            let result_type = checked_core_binary_primitive_result_type(&op, &left_type, &right_type)?;
            let result_name = checked_core_fresh_prim_result_name(
                atom_types,
                source_names,
                result_name_base,
            );
            atom_types.insert(result_name.clone(), result_type);
            bindings.extend(right_bindings);
            bindings.push(CheckedCorePrimBinding {
                name: result_name.clone(),
                op,
                args: vec![left, right],
            });
            Ok((CoreAtom::Var(result_name), bindings))
        }
        Expr::Unary {
            op: ash_core::UnaryOp::Not,
            expr,
            ..
        } => {
            let (operand, mut bindings) =
                checked_core_pure_anf_atom(expr, atom_types, source_names)?;
            checked_core_require_boolean_atom(&operand, atom_types)?;
            let result_name = checked_core_fresh_prim_result_name(
                atom_types,
                source_names,
                "__checked_not_result",
            );
            atom_types.insert(result_name.clone(), CoreType::Base("Bool".to_string()));
            bindings.push(CheckedCorePrimBinding {
                name: result_name.clone(),
                op: CorePrimOp::Not,
                args: vec![operand],
            });
            Ok((CoreAtom::Var(result_name), bindings))
        }
        _ => Err(EngineError::Type(
            "checked Core-to-CPS pure ANF lowering accepts only typed atoms, approved integer binary primitives, and recursive Boolean Not".to_string(),
        )),
    }
}

fn checked_core_require_boolean_atom(
    atom: &CoreAtom,
    atom_types: &HashMap<String, CoreType>,
) -> Result<(), EngineError> {
    let bool_type = CoreType::Base("Bool".to_string());
    if checked_core_atom_type(atom, atom_types)? == bool_type {
        Ok(())
    } else {
        Err(EngineError::Type(
            "checked Core-to-CPS Boolean Not operand must have type Bool".to_string(),
        ))
    }
}

fn checked_core_binary_primitive_result_type(
    op: &CorePrimOp,
    left: &CoreType,
    right: &CoreType,
) -> Result<CoreType, EngineError> {
    let int_type = CoreType::Base("Int".to_string());
    let bool_type = CoreType::Base("Bool".to_string());
    if matches!(op, CorePrimOp::Eq | CorePrimOp::Ne) && left == &bool_type && right == &bool_type {
        return Ok(bool_type);
    }
    if left != &int_type || right != &int_type {
        return Err(EngineError::Type(
            "checked Core-to-CPS binary primitive operands must both have type Int, except Eq and Ne which also accept two Bool operands".to_string(),
        ));
    }
    match op {
        CorePrimOp::Add | CorePrimOp::Sub | CorePrimOp::Mul | CorePrimOp::Div => Ok(int_type),
        CorePrimOp::Eq
        | CorePrimOp::Ne
        | CorePrimOp::Lt
        | CorePrimOp::Le
        | CorePrimOp::Gt
        | CorePrimOp::Ge => Ok(bool_type),
        _ => Err(EngineError::Type(
            "checked Core-to-CPS binary ANF lowering received a non-binary primitive".to_string(),
        )),
    }
}

const fn checked_core_binary_primitive(
    op: ash_core::BinaryOp,
) -> Option<(CorePrimOp, &'static str)> {
    match op {
        ash_core::BinaryOp::Add => Some((CorePrimOp::Add, "__checked_add_result")),
        ash_core::BinaryOp::Sub => Some((CorePrimOp::Sub, "__checked_sub_result")),
        ash_core::BinaryOp::Mul => Some((CorePrimOp::Mul, "__checked_mul_result")),
        ash_core::BinaryOp::Div => Some((CorePrimOp::Div, "__checked_div_result")),
        ash_core::BinaryOp::Eq => Some((CorePrimOp::Eq, "__checked_eq_result")),
        ash_core::BinaryOp::Ne => Some((CorePrimOp::Ne, "__checked_ne_result")),
        ash_core::BinaryOp::Lt => Some((CorePrimOp::Lt, "__checked_lt_result")),
        ash_core::BinaryOp::Le => Some((CorePrimOp::Le, "__checked_le_result")),
        ash_core::BinaryOp::Gt => Some((CorePrimOp::Gt, "__checked_gt_result")),
        ash_core::BinaryOp::Ge => Some((CorePrimOp::Ge, "__checked_ge_result")),
        ash_core::BinaryOp::Mod
        | ash_core::BinaryOp::And
        | ash_core::BinaryOp::Or
        | ash_core::BinaryOp::In
        | ash_core::BinaryOp::Pipe => None,
    }
}

fn checked_core_fresh_prim_result_name(
    atom_types: &HashMap<String, CoreType>,
    source_names: &HashSet<String>,
    base: &str,
) -> String {
    if !atom_types.contains_key(base) && !source_names.contains(base) {
        return base.to_string();
    }

    let mut suffix = 0_u32;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !atom_types.contains_key(&candidate) && !source_names.contains(&candidate) {
            return candidate;
        }
        suffix = suffix
            .checked_add(1)
            .expect("fresh primitive result suffix overflow");
    }
}

fn checked_core_collect_source_names(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Literal(_) | Expr::Variable { .. } | Expr::CheckObligation { .. } => {}
        Expr::FieldAccess { expr, .. }
        | Expr::Split(expr)
        | Expr::Fail { payload: expr }
        | Expr::Unary { expr, .. }
        | Expr::Spawn { init: expr, .. } => checked_core_collect_source_names(expr, names),
        Expr::IndexAccess { expr, index }
        | Expr::Binary {
            left: expr,
            right: index,
            ..
        } => {
            checked_core_collect_source_names(expr, names);
            checked_core_collect_source_names(index, names);
        }
        Expr::Call { arguments, .. } => {
            for argument in arguments {
                checked_core_collect_source_names(argument, names);
            }
        }
        Expr::Constructor { fields, .. } | Expr::Record { fields } => {
            for (_, field) in fields {
                checked_core_collect_source_names(field, names);
            }
        }
        Expr::Match { scrutinee, arms }
        | Expr::WithError {
            body: scrutinee,
            arms,
        } => {
            checked_core_collect_source_names(scrutinee, names);
            for arm in arms {
                checked_core_collect_source_names(&arm.body, names);
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            checked_core_collect_source_names(expr, names);
            checked_core_collect_source_names(then_branch, names);
            checked_core_collect_source_names(else_branch, names);
        }
        Expr::FnDef { body, .. } => checked_core_collect_source_names(body, names),
        Expr::Let {
            pattern,
            expr,
            body,
            ..
        } => {
            if let ash_core::Pattern::Variable { name, .. } = pattern {
                names.insert(name.clone());
            }
            checked_core_collect_source_names(expr, names);
            checked_core_collect_source_names(body, names);
        }
        Expr::FnApply { func, args } => {
            checked_core_collect_source_names(func, names);
            for argument in args {
                checked_core_collect_source_names(argument, names);
            }
        }
    }
}

fn time_sleep_operation_identity() -> OperationIdentityV1 {
    OperationIdentityV1::new(
        TIME_SLEEP_OPERATION.module,
        "builtin::time",
        TIME_SLEEP_OPERATION.name,
        ["Int"],
        "Null",
    )
}

fn checked_time_sleep_fact_to_checked_core(
    checked_operation: &ash_typeck::CheckedBuiltinOperation,
    checked_anchor: &SourceAnchor,
) -> Result<
    (
        OperationIdentityV1,
        ash_core::core_ash_typecheck::CheckedLoweredCoreProgram,
    ),
    EngineError,
> {
    let ash_typeck::CheckedBuiltinOperation::TimeSleep(time_sleep) = checked_operation;
    let Some(anchor_span) = checked_anchor.span else {
        return Err(EngineError::Type(
            "checked time::sleep source fact requires an entry source anchor span".to_string(),
        ));
    };
    if anchor_span.start != time_sleep.entry_span.start
        || anchor_span.end != time_sleep.entry_span.end
    {
        return Err(EngineError::Type(
            "checked time::sleep source fact does not match the retained entry source anchor"
                .to_string(),
        ));
    }

    let operation_identity = time_sleep_operation_identity();
    let operation = CoreEffectOp::Operation {
        path: vec![TIME_SLEEP_OPERATION.module.to_string()],
        operation: TIME_SLEEP_OPERATION.name.to_string(),
        arg_types: vec![CoreType::Base("Int".to_string())],
        result_type: CoreType::Named("Null".to_string()),
    };
    let core = CheckedCoreExpr::Raise {
        op: operation.clone(),
        args: vec![CoreAtom::LitInt(time_sleep.duration_millis)],
    };
    let validated = ash_core::core_ash_validate::validate_core_program(
        ash_core::core_ash_validate::RawCoreProgram::new(core),
    )
    .map_err(|error| {
        EngineError::Type(format!(
            "checked time::sleep Core validation failed: {error}"
        ))
    })?;
    let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
    type_env.types_mut().insert_name("Null");
    type_env.operations_mut().insert(operation);
    let context = ash_core::core_ash_lower::CoreLoweringContext::new(
        ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
        CoreRow::default(),
    );
    let checked_core = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
        validated, &type_env, context,
    )
    .map_err(|error| {
        EngineError::Type(format!(
            "checked time::sleep Core-to-CPS lowering failed: {error}"
        ))
    })?;
    Ok((operation_identity, checked_core))
}

/// Lowers the one sealed declaration-backed operation from its retained
/// typechecker fact.  Its source expression supplies only already-checked
/// literal/local argument values; the operation identity itself never comes
/// from the mutable `Entry` sidecar, row, or provider metadata.
fn checked_declared_operation_fact_to_checked_core(
    entry: &Expr,
    operation: &ash_typeck::DeclaredConcreteOperation,
) -> Result<
    (
        OperationIdentityV1,
        ash_core::core_ash_typecheck::CheckedLoweredCoreProgram,
    ),
    EngineError,
> {
    if !is_sealed_declared_production_operation(operation) {
        return Err(EngineError::Type(
            "production declared-operation admission does not admit this declaration".to_string(),
        ));
    }
    let arguments =
        evaluated_declared_operation_arguments(entry, operation).map_err(EngineError::Type)?;
    let [Value::Int(argument)] = arguments.as_slice() else {
        return Err(EngineError::Type(
            "production declared-operation admission requires one checked Int argument".to_string(),
        ));
    };
    let operation_identity = OperationIdentityV1::from_declared(operation);
    let core_operation = CoreEffectOp::Operation {
        path: vec![operation.impl_type.clone()],
        operation: operation.operation.clone(),
        arg_types: vec![CoreType::Base("Int".to_string())],
        result_type: CoreType::Named("Null".to_string()),
    };
    let core = CheckedCoreExpr::Raise {
        op: core_operation.clone(),
        args: vec![CoreAtom::LitInt(*argument)],
    };
    let validated = ash_core::core_ash_validate::validate_core_program(
        ash_core::core_ash_validate::RawCoreProgram::new(core),
    )
    .map_err(|error| {
        EngineError::Type(format!(
            "checked declared-operation Core validation failed: {error}"
        ))
    })?;
    let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
    type_env.types_mut().insert_name("Null");
    type_env.operations_mut().insert(core_operation);
    let context = ash_core::core_ash_lower::CoreLoweringContext::new(
        ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
        CoreRow::default(),
    );
    let checked_core = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
        validated, &type_env, context,
    )
    .map_err(|error| {
        EngineError::Type(format!(
            "checked declared-operation Core-to-CPS lowering failed: {error}"
        ))
    })?;
    Ok((operation_identity, checked_core))
}

fn attach_time_sleep_requirement_row(application: &mut Entry) {
    if !TIME_SLEEP_OPERATION.matches_legacy_call(&application.core) {
        return;
    }

    let Some(CoreType::Function { row, .. }) = application.core_callable_types.get_mut("main")
    else {
        return;
    };
    if !row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Operation { path, operation }
                if path == &[TIME_SLEEP_OPERATION.module.to_string()]
                    && operation == TIME_SLEEP_OPERATION.name
        )
    }) {
        row.items.push(CoreRowItem::Operation {
            path: vec![TIME_SLEEP_OPERATION.module.to_string()],
            operation: TIME_SLEEP_OPERATION.name.to_string(),
        });
    }
}

fn attach_declared_concrete_operation(
    application: &mut Entry,
    type_env: &ash_typeck::type_env::TypeEnv,
) {
    let Some(operation) = resolved_declared_operation_in_lexical_entry(&application.core, type_env)
    else {
        return;
    };
    let Some(CoreType::Function { row, .. }) = application.core_callable_types.get_mut("main")
    else {
        return;
    };
    let path = vec![operation.impl_type.clone()];
    if !row.items.iter().any(|item| {
        matches!(
            item,
            CoreRowItem::Operation {
                path: existing_path,
                operation: existing_operation,
            } if existing_path == &path && existing_operation == &operation.operation
        )
    }) {
        row.items.push(CoreRowItem::Operation {
            path,
            operation: operation.operation.clone(),
        });
    }
    application.declared_concrete_operation = Some(operation);
}

/// Resolve one concrete operation from the narrow, checked entry shape admitted by TASK-2015.
///
/// The only accepted wrapper is a lexical chain of variable `let` bindings around a tail
/// qualified call. This deliberately does not search arbitrary expression trees: a nested or
/// competing call must not silently acquire operation metadata.
fn resolved_declared_operation_in_lexical_entry(
    entry: &Expr,
    type_env: &ash_typeck::type_env::TypeEnv,
) -> Option<ash_typeck::DeclaredConcreteOperation> {
    match entry {
        Expr::Call {
            func,
            module: Some(impl_type),
            ..
        } => type_env
            .resolve_declared_concrete_operation(impl_type, func)
            .ok(),
        Expr::Let {
            pattern: ash_core::Pattern::Variable { .. },
            body,
            ..
        } => resolved_declared_operation_in_lexical_entry(body, type_env),
        _ => None,
    }
}

/// Evaluate arguments for a checked declared operation along its admitted lexical entry spine.
///
/// Values are obtained only from literal bindings or earlier local bindings; this neither
/// consults providers nor derives an operation identity from names. Other expression forms are
/// intentionally rejected until their source-to-Core value transport is explicitly implemented.
fn evaluated_declared_operation_arguments(
    entry: &Expr,
    operation: &ash_typeck::DeclaredConcreteOperation,
) -> Result<Vec<Value>, String> {
    fn evaluate_local_value(expr: &Expr, values: &HashMap<String, Value>) -> Result<Value, String> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable { name, .. } => values
                .get(name)
                .cloned()
                .ok_or_else(|| format!("declared-operation local value '{name}' is not bound")),
            _ => Err(
                "declared-operation execution accepts only literal or previously bound local values"
                    .to_string(),
            ),
        }
    }

    fn walk(
        expr: &Expr,
        operation: &ash_typeck::DeclaredConcreteOperation,
        values: &mut HashMap<String, Value>,
    ) -> Result<Vec<Value>, String> {
        match expr {
            Expr::Let {
                pattern: ash_core::Pattern::Variable { name, .. },
                expr,
                body,
                ..
            } => {
                let value = evaluate_local_value(expr, values)?;
                values.insert(name.clone(), value);
                walk(body, operation, values)
            }
            Expr::Call {
                func,
                module: Some(impl_type),
                arguments,
            } if impl_type == &operation.impl_type && func == &operation.operation => arguments
                .iter()
                .map(|argument| evaluate_local_value(argument, values))
                .collect(),
            Expr::Call { .. } => Err(
                "bound declared operation does not match its checked concrete call form"
                    .to_string(),
            ),
            _ => {
                Err("bound declared operation is missing its checked lexical call form".to_string())
            }
        }
    }

    walk(entry, operation, &mut HashMap::new())
}

fn declaration_resolution_env(
    base: &ash_typeck::type_env::TypeEnv,
    program: &ash_parser::surface::Program,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
) -> Result<ash_typeck::type_env::TypeEnv, String> {
    let mut env = base.clone();
    // The declaration resolver runs after ordinary program checking, but it
    // still needs the same local nominal identities in order to resolve an
    // `ImplType::operation` qualifier.  In particular, a local `newtype` is
    // deliberately not a transparent core type definition, so registering
    // only interfaces and impls here would make a valid `impl Fs<PosixFs>`
    // appear to target an unbound type.
    env.set_current_module_identity(module_identity);
    env.register_surface_declarations(&program.definitions)
        .map_err(|error| error.to_string())?;
    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(|error| error.to_string())?;
        }
    }
    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(env)
}

fn checked_cps_declared_operation_raise(
    entry: &Expr,
    operation: &ash_typeck::DeclaredConcreteOperation,
) -> Result<ash_core::cps::Term, EngineError> {
    let arguments =
        evaluated_declared_operation_arguments(entry, operation).map_err(EngineError::Type)?;
    let args = arguments
        .iter()
        .map(|argument| match argument {
            Value::Int(value) => Ok(ash_core::cps::Atom::Int(*value)),
            Value::String(value) => Ok(ash_core::cps::Atom::String(value.clone())),
            _ => Err(EngineError::Type(
                "checked declared-operation lowering accepts only evaluated integer or string arguments"
                    .to_string(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let item = ash_core::cps::EffectItem {
        namespace: operation.impl_type.clone(),
        name: operation.operation.clone(),
        kind: ash_core::cps::EffectItemKind::Capability,
    };
    Ok(ash_core::cps::Term::Raise {
        op: ash_core::cps::EffectOp {
            item: item.clone(),
            arg_types: operation.params.iter().map(ToString::to_string).collect(),
            result_type: operation.result_type.to_string(),
        },
        args,
        resume: ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
        row: ash_core::cps::EffectRow { items: vec![item] },
    })
}

fn checked_cps_time_sleep_raise(entry: &Expr) -> Result<ash_core::cps::Term, EngineError> {
    let Expr::Call { arguments, .. } = entry else {
        return Err(EngineError::Type(
            "checked time::sleep lowering requires its concrete call form".to_string(),
        ));
    };
    let [Expr::Literal(Value::Int(duration))] = arguments.as_slice() else {
        return Err(EngineError::Type(
            "checked time::sleep lowering requires one integer literal duration".to_string(),
        ));
    };
    if *duration < 0 {
        return Err(EngineError::Type(
            "checked time::sleep lowering requires a non-negative integer literal duration"
                .to_string(),
        ));
    }
    let item = ash_core::cps::EffectItem {
        namespace: TIME_SLEEP_OPERATION.module.to_string(),
        name: TIME_SLEEP_OPERATION.name.to_string(),
        kind: ash_core::cps::EffectItemKind::Capability,
    };
    Ok(ash_core::cps::Term::Raise {
        op: ash_core::cps::EffectOp {
            item: item.clone(),
            arg_types: vec!["Int".to_string()],
            result_type: "Null".to_string(),
        },
        args: vec![ash_core::cps::Atom::Int(*duration)],
        resume: ash_core::cps::ContRef::Label(CHECKED_CPS_ANSWER_CONTINUATION.to_string()),
        row: ash_core::cps::EffectRow { items: vec![item] },
    })
}

fn checked_core_boolean_match_branches(
    arms: &[ash_core::MatchArm],
) -> Result<(&Expr, &Expr), EngineError> {
    let Some(then_branch) = arms.iter().find_map(|arm| match arm.pattern {
        ash_core::Pattern::Literal(Value::Bool(true)) => Some(&arm.body),
        _ => None,
    }) else {
        return Err(EngineError::Type(
            "checked Core-to-CPS bridge requires a true boolean match branch".to_string(),
        ));
    };
    let Some(else_branch) = arms.iter().find_map(|arm| match arm.pattern {
        ash_core::Pattern::Literal(Value::Bool(false)) => Some(&arm.body),
        _ => None,
    }) else {
        return Err(EngineError::Type(
            "checked Core-to-CPS bridge requires a false boolean match branch".to_string(),
        ));
    };
    if arms.len() != 2 {
        return Err(EngineError::Type(
            "checked Core-to-CPS bridge accepts only two-arm boolean matches".to_string(),
        ));
    }
    Ok((then_branch, else_branch))
}

fn checked_core_atom_from_legacy_expr(expr: &Expr) -> Result<CoreAtom, EngineError> {
    match expr {
        Expr::Literal(Value::Int(value)) => Ok(CoreAtom::LitInt(*value)),
        Expr::Literal(Value::String(value)) => Ok(CoreAtom::LitString(value.clone())),
        Expr::Literal(Value::Bool(value)) => Ok(CoreAtom::LitBool(*value)),
        Expr::Literal(Value::Null) => Ok(CoreAtom::LitUnit),
        Expr::Variable { name, .. } => Ok(CoreAtom::Var(name.clone())),
        Expr::Literal(_) => Err(EngineError::Type(
            "checked Core-to-CPS bridge does not represent this literal value".to_string(),
        )),
        _ => Err(EngineError::Type(
            "checked Core-to-CPS bridge accepts only atomic let values".to_string(),
        )),
    }
}

/// Builds the bounded recursive constructor-value subset used by the checked
/// entry bridge. Source typechecking has already established the constructor
/// result type; this function only preserves its verified runtime shape in
/// CPS, rejecting every computational field form until it has a typed Core
/// lowering.
fn checked_cps_structural_value_from_legacy_expr(
    expr: &Expr,
) -> Result<ash_core::cps::Value, EngineError> {
    use ash_core::cps::{Atom as CpsAtom, Value as CpsValue};

    match expr {
        Expr::Literal(Value::Int(value)) => Ok(CpsValue::Atom(CpsAtom::Int(*value))),
        Expr::Literal(Value::String(value)) => Ok(CpsValue::Atom(CpsAtom::String(value.clone()))),
        Expr::Literal(Value::Bool(value)) => Ok(CpsValue::Atom(CpsAtom::Bool(*value))),
        Expr::Literal(Value::Null) => Ok(CpsValue::Atom(CpsAtom::Null)),
        Expr::Constructor { name, fields } => fields
            .iter()
            .map(|(field_name, field)| {
                checked_cps_structural_value_from_legacy_expr(field)
                    .map(|value| (field_name.clone(), value))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|fields| CpsValue::Constructor {
                name: name.clone(),
                fields,
            }),
        Expr::Record { fields } => fields
            .iter()
            .map(|(field_name, field)| {
                checked_cps_structural_value_from_legacy_expr(field)
                    .map(|value| (field_name.clone(), value))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|fields| CpsValue::Record { fields }),
        _ => Err(EngineError::Type(
            "checked Core/CPS structural entry lowering accepts only nested constructors, records, and primitive literal fields"
                .to_string(),
        )),
    }
}

fn checked_core_atom_type(
    atom: &CoreAtom,
    atom_types: &HashMap<String, CoreType>,
) -> Result<CoreType, EngineError> {
    match atom {
        CoreAtom::LitInt(_) => Ok(CoreType::Base("Int".to_string())),
        CoreAtom::LitString(_) => Ok(CoreType::Base("String".to_string())),
        CoreAtom::LitBool(_) => Ok(CoreType::Base("Bool".to_string())),
        CoreAtom::LitUnit => Ok(CoreType::Base("Unit".to_string())),
        CoreAtom::Var(name) => atom_types.get(name).cloned().ok_or_else(|| {
            EngineError::Type(format!(
                "checked Core-to-CPS bridge cannot resolve a type for variable `{name}`"
            ))
        }),
        CoreAtom::PrimName(_) | CoreAtom::ConstructorName(_) => Err(EngineError::Type(
            "checked Core-to-CPS bridge cannot infer a let binding type from this atom".to_string(),
        )),
    }
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

// TASK-2014 keeps production-frame construction inside the Engine crate: the
// test module may inspect the private carrier, while downstream crates cannot
// reconstruct one from public V1 inspection data or rows.
