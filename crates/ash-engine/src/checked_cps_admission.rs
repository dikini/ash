//! Checked Core/CPS admission evidence for the Path B execution boundary.
//!
//! This module deliberately validates evidence and frame authority without
//! executing it.  In particular, a normalized source row is retained only as
//! an admission fact: it never manufactures a provider or handler frame.

use ash_core::{
    core_ash::{CoreEffectOp, CoreExpr, CoreType},
    core_ash_typecheck::CheckedLoweredCoreProgram,
    cps::{
        Atom as CpsAtom, ContMultiplicity, EffectRow as CpsEffectRow, Term as CpsTerm,
        Value as CpsValue,
    },
    semantic_summary::SourceAnchor,
};
use ash_typeck::{DeclaredConcreteOperation, TypeCheckResult};
use std::sync::Arc;

const ENTRY_ANSWER_CONTINUATION: &str = "__answer";
const ENTRY_ANSWER_VALUE: &str = "__entry_answer_value";

/// A sealed, handler-free checked Core/CPS admission for one source entry.
///
/// Unlike [`CheckedCpsAdmissionV1`], this carrier has no source-handler,
/// provider-binding, or frame-installation component. It is limited to a
/// checked handler-free CPS term and the exact checked entry source anchor.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedCpsEntryAdmission {
    entry_id: u64,
    source_anchor: SourceAnchor,
    executable: CpsTerm,
}

impl CheckedCpsEntryAdmission {
    /// Seals a handler-free lowered CPS term with its checked entry provenance.
    #[must_use]
    pub(crate) fn new(entry_id: u64, source_anchor: SourceAnchor, lowered: CpsTerm) -> Self {
        Self {
            entry_id,
            source_anchor,
            executable: CpsTerm::LetCont {
                name: ENTRY_ANSWER_CONTINUATION.to_string(),
                param: ENTRY_ANSWER_VALUE.to_string(),
                cont_body: Box::new(CpsTerm::Return {
                    value: CpsValue::Atom(CpsAtom::Var(ENTRY_ANSWER_VALUE.to_string())),
                }),
                body: Box::new(lowered),
                row: CpsEffectRow::default(),
                multiplicity: ContMultiplicity::Affine,
            },
        }
    }

    /// Returns the exact source anchor retained from the checked entry.
    #[must_use]
    pub const fn source_anchor(&self) -> &SourceAnchor {
        &self.source_anchor
    }

    /// Returns the internal entry identity paired with the source anchor.
    #[must_use]
    pub(crate) const fn entry_id(&self) -> u64 {
        self.entry_id
    }

    /// Returns the sealed, terminalized CPS term for the engine-owned driver.
    #[must_use]
    pub(crate) const fn executable(&self) -> &CpsTerm {
        &self.executable
    }
}

/// Engine-sealed provider object paired with the exact binding that resolved
/// it.  This is deliberately crate-private: an operation row or a public V1
/// inspection artifact cannot manufacture host authority.
#[derive(Clone)]
#[allow(dead_code)] // TASK-2014 Task 2 consumes this sealed handoff.
pub(crate) struct ResolvedProviderBinding {
    binding: ProviderBindingV1,
    provider: Arc<dyn ash_core::capability::CapabilityProvider>,
}

#[allow(dead_code)] // TASK-2014 Task 2 consumes the provider handle.
impl ResolvedProviderBinding {
    /// Pairs an already-resolved provider object with its validated binding.
    #[must_use]
    pub(crate) fn new(
        binding: ProviderBindingV1,
        provider: Arc<dyn ash_core::capability::CapabilityProvider>,
    ) -> Self {
        Self { binding, provider }
    }

    /// Returns the validated binding used for the public instruction summary.
    #[must_use]
    pub(crate) const fn binding(&self) -> &ProviderBindingV1 {
        &self.binding
    }

    /// Returns the sealed provider handle for the later interpreter handoff.
    #[must_use]
    pub(crate) fn provider(&self) -> &Arc<dyn ash_core::capability::CapabilityProvider> {
        &self.provider
    }
}

/// Opaque, Engine-issued production evidence for the first provider-backed
/// checked-CPS slice.
///
/// Unlike [`CheckedCpsEntryAdmission`] and [`CheckedCpsAdmissionV1`], this
/// token retains a private issuer seal and an already-resolved provider object.
/// It has no public constructor and cannot be reconstructed from CPS, rows, or
/// public V1 inspection evidence.
#[derive(Clone)]
#[allow(dead_code)] // TASK-2014 Task 3 consumes the sealed execution fields.
pub struct CheckedCpsProductionAdmission {
    issuer_token: Arc<()>,
    /// Per-admission private seal used solely to bind a run-control envelope
    /// to this exact immutable admission artifact.
    run_control_token: Arc<()>,
    entry_id: u64,
    source_anchor: SourceAnchor,
    checked_core: CheckedLoweredCoreProgram,
    executable: CpsTerm,
    provider_bindings: Vec<ResolvedProviderBinding>,
    frame_installations: Vec<FrameInstallationInstructionV1>,
}

#[allow(dead_code)] // TASK-2014 Task 3 verifies and consumes this token.
impl CheckedCpsProductionAdmission {
    /// Seals one exact checked `time::sleep` CPS producer and its one explicit
    /// provider instruction.
    ///
    /// This boundary is intentionally narrower than V1 admission validation:
    /// it accepts no source handlers, residual facts, open tails, duplicate
    /// instructions, or row-derived provider authority.
    pub(crate) fn validate_production_time_sleep(
        issuer_token: Arc<()>,
        entry_id: u64,
        source_anchor: SourceAnchor,
        checked_core: CheckedLoweredCoreProgram,
        expected_operation: &OperationIdentityV1,
        resolved_provider: ResolvedProviderBinding,
        frame_installations: Vec<FrameInstallationInstructionV1>,
    ) -> Result<Self, CheckedCpsAdmissionError> {
        validate_exact_time_sleep_raise(checked_core.lowered(), expected_operation)?;
        validate_exact_provider_instruction(
            expected_operation,
            resolved_provider.binding(),
            &frame_installations,
        )?;

        let executable = terminalize_production_term(checked_core.lowered().clone());
        ash_interp::cps::validate::validate_cps_program(&executable).map_err(|error| {
            CheckedCpsAdmissionError::InvalidProductionCps {
                reason: error.to_string(),
            }
        })?;

        Ok(Self {
            issuer_token,
            run_control_token: Arc::new(()),
            entry_id,
            source_anchor,
            checked_core,
            executable,
            provider_bindings: vec![resolved_provider],
            frame_installations,
        })
    }

    /// Returns the checked source anchor sealed into this token.
    #[must_use]
    pub const fn source_anchor(&self) -> &SourceAnchor {
        &self.source_anchor
    }

    /// Returns the ordered explicit frame instructions for diagnostic
    /// inspection. These summaries cannot install frames by themselves.
    #[must_use]
    pub fn frame_installation_summary(&self) -> &[FrameInstallationInstructionV1] {
        &self.frame_installations
    }

    /// Verifies that an Engine is the issuer before it hands the token to a
    /// later production driver.
    #[must_use]
    pub(crate) fn is_issued_by(&self, issuer_token: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.issuer_token, issuer_token)
    }

    /// Verifies that a control envelope was minted for this exact admission.
    #[must_use]
    pub(crate) fn has_run_control_token(&self, run_control_token: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.run_control_token, run_control_token)
    }

    /// Clones the private per-admission seal into the Engine-owned control
    /// envelope. This is crate-private so callers cannot bind controls.
    #[must_use]
    pub(crate) fn run_control_token(&self) -> Arc<()> {
        Arc::clone(&self.run_control_token)
    }

    /// Returns the sealed entry identity for a later Engine-only handoff.
    #[must_use]
    pub(crate) const fn entry_id(&self) -> u64 {
        self.entry_id
    }

    /// Returns the validated terminalized CPS program for the later driver.
    #[must_use]
    pub(crate) const fn executable(&self) -> &CpsTerm {
        &self.executable
    }

    /// Returns the exact checked Core/CPS evidence from which this token was
    /// sealed. It is crate-private so public V1 evidence cannot be promoted to
    /// executable authority.
    #[must_use]
    pub(crate) const fn checked_core(&self) -> &CheckedLoweredCoreProgram {
        &self.checked_core
    }

    /// Returns the exact resolved provider for the later driver handoff.
    #[must_use]
    pub(crate) fn resolved_provider_bindings(&self) -> &[ResolvedProviderBinding] {
        &self.provider_bindings
    }
}

fn terminalize_production_term(lowered: CpsTerm) -> CpsTerm {
    CpsTerm::LetCont {
        name: ENTRY_ANSWER_CONTINUATION.to_string(),
        param: ENTRY_ANSWER_VALUE.to_string(),
        cont_body: Box::new(CpsTerm::Return {
            value: CpsValue::Atom(CpsAtom::Var(ENTRY_ANSWER_VALUE.to_string())),
        }),
        body: Box::new(lowered),
        row: CpsEffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn validate_exact_time_sleep_raise(
    lowered: &CpsTerm,
    expected_operation: &OperationIdentityV1,
) -> Result<(), CheckedCpsAdmissionError> {
    let CpsTerm::Raise {
        op,
        args,
        resume,
        row,
    } = lowered
    else {
        return Err(CheckedCpsAdmissionError::InvalidProductionCps {
            reason: "production admission requires one direct checked time::sleep Raise"
                .to_string(),
        });
    };
    let expected_item = ash_core::cps::EffectItem {
        namespace: "cap".to_string(),
        name: format!(
            "{}.{}",
            expected_operation.impl_type(),
            expected_operation.operation()
        ),
        kind: ash_core::cps::EffectItemKind::Capability,
    };
    let exact_operation = op.item == expected_item
        && op.arg_types == expected_operation.parameter_types()
        && op.result_type == expected_operation.result_type();
    let exact_argument = matches!(args.as_slice(), [CpsAtom::Int(duration)] if *duration >= 0);
    let exact_resume = matches!(resume, ash_core::cps::ContRef::Label(label) if label == ENTRY_ANSWER_CONTINUATION);
    let exact_row = row.items.as_slice() == [expected_item];
    if !(exact_operation && exact_argument && exact_resume && exact_row) {
        return Err(CheckedCpsAdmissionError::InvalidProductionCps {
            reason: "production admission requires the exact checked Core/CPS time::sleep(Int)->Null Raise"
                .to_string(),
        });
    }
    Ok(())
}

fn validate_exact_provider_instruction(
    expected_operation: &OperationIdentityV1,
    resolved_binding: &ProviderBindingV1,
    frame_installations: &[FrameInstallationInstructionV1],
) -> Result<(), CheckedCpsAdmissionError> {
    let [
        FrameInstallationInstructionV1::Provider {
            operation,
            provider_binding,
        },
    ] = frame_installations
    else {
        return Err(CheckedCpsAdmissionError::InvalidProductionFrameInstructions);
    };
    if operation != expected_operation
        || provider_binding != resolved_binding
        || provider_binding.operation() != expected_operation
    {
        return Err(CheckedCpsAdmissionError::InvalidProductionFrameInstructions);
    }
    Ok(())
}

/// Stable, engine-owned identity for one concrete declared operation.
///
/// The identity is intentionally more specific than a source spelling: its
/// implementation type, declaring interface, parameter types, and result type
/// all participate in equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationIdentityV1 {
    identity: Box<OperationIdentityData>,
}

// The public frame-summary API yields borrowed instructions. Supporting this
// comparison keeps callers from needing to clone an identity merely to compare
// the binding and the sibling instruction projected from that same summary.
impl PartialEq<&Self> for OperationIdentityV1 {
    fn eq(&self, other: &&Self) -> bool {
        self == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OperationIdentityData {
    impl_type: String,
    interface: String,
    operation: String,
    parameter_types: Vec<String>,
    result_type: String,
}

impl OperationIdentityV1 {
    /// Constructs an exact declared-operation identity.
    #[must_use]
    pub fn new<I, S>(
        impl_type: impl Into<String>,
        interface: impl Into<String>,
        operation: impl Into<String>,
        parameter_types: I,
        result_type: impl Into<String>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            identity: Box::new(OperationIdentityData {
                impl_type: impl_type.into(),
                interface: interface.into(),
                operation: operation.into(),
                parameter_types: parameter_types.into_iter().map(Into::into).collect(),
                result_type: result_type.into(),
            }),
        }
    }

    /// Converts a declaration-backed typechecker operation without recovering
    /// identity from source text or a normalized row.
    #[must_use]
    pub fn from_declared(operation: &DeclaredConcreteOperation) -> Self {
        Self::new(
            &operation.impl_type,
            &operation.interface,
            &operation.operation,
            operation.params.iter().map(ToString::to_string),
            operation.result_type.to_string(),
        )
    }

    /// Returns the implementation type that owns the concrete operation.
    #[must_use]
    pub fn impl_type(&self) -> &str {
        &self.identity.impl_type
    }

    /// Returns the interface that declares the operation signature.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.identity.interface
    }

    /// Returns the operation name.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.identity.operation
    }

    /// Returns the declared parameter type spellings.
    #[must_use]
    pub fn parameter_types(&self) -> &[String] {
        &self.identity.parameter_types
    }

    /// Returns the declared result type spelling.
    #[must_use]
    pub fn result_type(&self) -> &str {
        &self.identity.result_type
    }
}

/// Engine-owned, source-only projection of one checked handler clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedHandlerClauseV1 {
    operation: OperationIdentityV1,
    resume_name: String,
}

impl CheckedHandlerClauseV1 {
    /// Returns the declaration-backed operation handled by this clause.
    #[must_use]
    pub const fn operation(&self) -> &OperationIdentityV1 {
        &self.operation
    }

    /// Returns the checked continuation binder spelling.
    #[must_use]
    pub fn resume_name(&self) -> &str {
        &self.resume_name
    }
}

/// Canonical engine projection of a source handler residual row.
///
/// This descriptor is intentionally not the typechecker's
/// `NormalizedHandlerRow`; V1 remains an in-memory engine schema and does not
/// expose typechecker internals as its public admission representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualRowDescriptorV1 {
    requirement_keys: Vec<String>,
    open_tail: Option<String>,
}

impl ResidualRowDescriptorV1 {
    /// Returns whether this residual is the closed empty row.
    #[must_use]
    pub const fn is_closed_empty(&self) -> bool {
        self.requirement_keys.is_empty() && self.open_tail.is_none()
    }

    /// Returns canonical non-granting requirement keys in source row order.
    #[must_use]
    pub fn requirement_keys(&self) -> &[String] {
        &self.requirement_keys
    }

    /// Returns the retained open row-tail name, if any.
    #[must_use]
    pub fn open_tail(&self) -> Option<&str> {
        self.open_tail.as_deref()
    }
}

/// Source-only facts eligible to be combined with checked Core/CPS evidence.
///
/// These facts have no frame-installation authority.  The caller must supply
/// explicit [`FrameInstallationInstructionV1`] values to an admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedSourceFactsV1 {
    handler_name: String,
    operation_identities: Vec<OperationIdentityV1>,
    residual_operation_identities: Vec<OperationIdentityV1>,
    handler_clauses: Vec<CheckedHandlerClauseV1>,
    residual_rows: Vec<ResidualRowDescriptorV1>,
    source_anchors: Vec<SourceAnchor>,
}

impl CheckedSourceFactsV1 {
    /// Projects one selected checked handler and application into V1 source
    /// evidence.
    ///
    /// # Errors
    ///
    /// Returns an error when `handler_name` is not a checked handler or when
    /// the checked program has no corresponding `handle … with handler_name`
    /// application fact.
    pub fn from_type_check(
        checked: &TypeCheckResult,
        handler_name: &str,
        source_anchor: SourceAnchor,
    ) -> Result<Self, CheckedCpsAdmissionError> {
        Self::from_type_check_with_residual_operations(
            checked,
            handler_name,
            source_anchor,
            Vec::new(),
        )
    }

    /// Projects selected checked facts with resolver-produced concrete facts
    /// for each unhandled residual operation.
    ///
    /// The source checker retains normalized residual row entries but not the
    /// complete declared signature for every entry.  This boundary therefore
    /// accepts exact resolver facts only after proving their canonical row keys
    /// match the checked residual row; it never recovers identity from text.
    ///
    /// # Errors
    ///
    /// Returns [`CheckedCpsAdmissionError::ResidualOperationFactsMismatch`]
    /// when supplied operation facts differ from the selected checked residual
    /// row.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_type_check_with_residual_operations(
        checked: &TypeCheckResult,
        handler_name: &str,
        source_anchor: SourceAnchor,
        residual_operations: Vec<DeclaredConcreteOperation>,
    ) -> Result<Self, CheckedCpsAdmissionError> {
        let handler = checked.checked_handlers.get(handler_name).ok_or_else(|| {
            CheckedCpsAdmissionError::UnknownCheckedSourceHandler {
                handler_name: handler_name.to_string(),
            }
        })?;
        if !checked
            .checked_handler_applications
            .iter()
            .any(|application| application.handler_name == handler_name)
        {
            return Err(CheckedCpsAdmissionError::MissingCheckedHandlerApplication {
                handler_name: handler_name.to_string(),
            });
        }

        let handler_clauses = handler
            .clauses
            .iter()
            .map(|clause| CheckedHandlerClauseV1 {
                operation: OperationIdentityV1::from_declared(&clause.operation),
                resume_name: clause.resume_name.clone(),
            })
            .collect::<Vec<_>>();
        let operation_identities = handler_clauses
            .iter()
            .map(|clause| clause.operation.clone())
            .collect();
        let residual_operation_identities = residual_operations
            .iter()
            .map(OperationIdentityV1::from_declared)
            .collect::<Vec<_>>();
        let expected_residual_keys = handler
            .residual_row
            .items
            .iter()
            .map(ash_typeck::NormalizedHandlerRowItem::canonical_key)
            .collect::<Vec<_>>();
        let supplied_residual_keys = residual_operations
            .iter()
            .map(declared_operation_row_key)
            .collect::<Vec<_>>();
        if expected_residual_keys != supplied_residual_keys {
            return Err(CheckedCpsAdmissionError::ResidualOperationFactsMismatch {
                expected_keys: expected_residual_keys,
                actual_keys: supplied_residual_keys,
            });
        }
        let residual_rows = vec![ResidualRowDescriptorV1 {
            requirement_keys: handler
                .residual_row
                .items
                .iter()
                .map(ash_typeck::NormalizedHandlerRowItem::canonical_key)
                .collect(),
            open_tail: handler.residual_row.tail.clone(),
        }];

        Ok(Self {
            handler_name: handler_name.to_string(),
            operation_identities,
            residual_operation_identities,
            handler_clauses,
            residual_rows,
            source_anchors: vec![source_anchor],
        })
    }

    /// Returns the selected handler name.
    #[must_use]
    pub fn handler_name(&self) -> &str {
        &self.handler_name
    }

    /// Returns declared operation identities required by the selected handler.
    #[must_use]
    pub fn operation_identities(&self) -> &[OperationIdentityV1] {
        &self.operation_identities
    }

    /// Returns resolver-backed exact identities for unhandled residual
    /// operations. These require explicit provider authorization.
    #[must_use]
    pub fn residual_operation_identities(&self) -> &[OperationIdentityV1] {
        &self.residual_operation_identities
    }

    /// Returns the selected checked operation clauses.
    #[must_use]
    pub fn handler_clauses(&self) -> &[CheckedHandlerClauseV1] {
        &self.handler_clauses
    }

    /// Returns canonical residual-row descriptors.
    #[must_use]
    pub fn residual_rows(&self) -> &[ResidualRowDescriptorV1] {
        &self.residual_rows
    }

    /// Returns source anchors retained for admission diagnostics.
    #[must_use]
    pub fn source_anchors(&self) -> &[SourceAnchor] {
        &self.source_anchors
    }
}

/// Host-selected provider binding admitted for one declared operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBindingV1 {
    operation: OperationIdentityV1,
    provider_name: String,
    provider_operation: String,
}

impl ProviderBindingV1 {
    /// Constructs a host provider binding for one exact operation identity.
    #[must_use]
    pub fn new(
        operation: OperationIdentityV1,
        provider_name: impl Into<String>,
        provider_operation: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            provider_name: provider_name.into(),
            provider_operation: provider_operation.into(),
        }
    }

    /// Returns the declared operation authorized for this binding.
    #[must_use]
    pub const fn operation(&self) -> &OperationIdentityV1 {
        &self.operation
    }

    /// Returns the host provider identifier.
    #[must_use]
    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Returns the provider-local operation identifier.
    #[must_use]
    pub fn provider_operation(&self) -> &str {
        &self.provider_operation
    }
}

/// A path to one expression in a checked Core program.
///
/// Child indices follow the structural expression order.  For a `Handle`,
/// index `0` is the clause body and index `1` is the handled body.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoreHandleLocatorV1 {
    path: Vec<usize>,
}

impl CoreHandleLocatorV1 {
    /// Returns a locator for a root `CoreExpr::Handle`.
    #[must_use]
    pub const fn root() -> Self {
        Self { path: Vec::new() }
    }

    /// Constructs a locator from an expression-child path.
    #[must_use]
    pub fn at_path<I>(path: I) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        Self {
            path: path.into_iter().collect(),
        }
    }

    /// Returns the expression-child path.
    #[must_use]
    pub fn path(&self) -> &[usize] {
        &self.path
    }
}

/// Explicit authority to install one runtime frame after admission.
///
/// The order supplied by the caller is preserved verbatim because it is the
/// input to TASK-1993's innermost-first frame lookup, not an effect-row order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameInstallationInstructionV1 {
    /// Authorizes a host provider for exactly one declared operation.
    Provider {
        /// The operation the frame is authorized to serve.
        operation: OperationIdentityV1,
        /// The admitted host binding for that operation.
        provider_binding: ProviderBindingV1,
    },
    /// Authorizes a checked source handler at a concrete Core `Handle` node.
    SourceHandler {
        /// The operation clause served by the handler frame.
        operation: OperationIdentityV1,
        /// The selected checked source handler declaration.
        handler_name: String,
        /// The exact typed Core handler node.
        core_handle: CoreHandleLocatorV1,
    },
}

/// Validation failure while assembling a sealed V1 admission artifact.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CheckedCpsAdmissionError {
    /// The narrow production sealer received CPS outside its exact checked
    /// `time::sleep(Int) -> Null` producer shape.
    #[error("invalid production checked CPS: {reason}")]
    InvalidProductionCps {
        /// Stable explanation of the rejected closed shape.
        reason: String,
    },
    /// Production authority requires precisely one explicit matching Provider
    /// instruction and never derives frames from rows.
    #[error("production time::sleep admission requires one exact Provider instruction")]
    InvalidProductionFrameInstructions,
    /// The selected source handler was absent from checked source facts.
    #[error("checked source handler '{handler_name}' was not found")]
    UnknownCheckedSourceHandler {
        /// Handler name requested by the caller.
        handler_name: String,
    },
    /// The selected checked handler has no matching checked application fact.
    #[error("checked source handler '{handler_name}' has no checked application")]
    MissingCheckedHandlerApplication {
        /// Handler name requested by the caller.
        handler_name: String,
    },
    /// A checked residual row still has an open tail and therefore lacks the
    /// resolver-attested concrete operation expansion required for admission.
    #[error("open residual tail '{tail}' requires resolver-attested expansion before admission")]
    UnexpandedOpenResidualTail {
        /// The unresolved normalized row-tail name.
        tail: String,
    },
    /// Resolver-supplied residual operation facts do not exactly cover the
    /// checked normalized residual row.
    #[error("resolver-produced residual operation facts do not match the checked residual row")]
    ResidualOperationFactsMismatch {
        /// Canonical operation row keys retained by the typechecker.
        expected_keys: Vec<String>,
        /// Canonical keys derived from resolver-backed operation facts.
        actual_keys: Vec<String>,
    },
    /// No explicit provider or source-handler authorization exists for an
    /// operation required by the selected checked source facts.
    #[error("missing explicit frame-installation authorization for operation {operation:?}")]
    MissingFrameInstallationAuthorization {
        /// Required concrete operation identity.
        operation: OperationIdentityV1,
    },
    /// A provider binding names a different concrete operation than its frame
    /// instruction.
    #[error("provider binding operation identity does not match its frame instruction")]
    ProviderIdentityMismatch {
        /// Operation named by the frame instruction.
        expected: OperationIdentityV1,
        /// Operation named by the provider binding.
        actual: OperationIdentityV1,
    },
    /// A source-handler instruction does not match a checked source clause.
    #[error("source-handler instruction does not match checked source facts")]
    SourceHandlerIdentityMismatch {
        /// Handler name named by the instruction.
        handler_name: String,
        /// Operation named by the instruction.
        operation: OperationIdentityV1,
    },
    /// A source-handler instruction does not point to a matching typed Core
    /// `Handle` expression.
    #[error("source-handler instruction does not point to a matching Core Handle")]
    CoreHandleLocatorMismatch {
        /// Handler named by the instruction.
        handler_name: String,
        /// Locator rejected against the typed Core program.
        locator: CoreHandleLocatorV1,
    },
}

/// Sealed, in-memory V1 admission artifact for a checked Core/CPS program.
///
/// Construction is available only through [`Self::validate`].  The artifact
/// retains both checked Core/CPS proof and source facts but does not execute a
/// provider or install a frame.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedCpsAdmissionV1 {
    checked_core: CheckedLoweredCoreProgram,
    source_facts: CheckedSourceFactsV1,
    frame_installations: Vec<FrameInstallationInstructionV1>,
}

impl CheckedCpsAdmissionV1 {
    /// Validates and seals a V1 admission artifact.
    ///
    /// The `CheckedLoweredCoreProgram` argument is the typed/lowered Core
    /// proof.  Source rows remain descriptive only; every provider and source
    /// handler must have an explicit matching installation instruction.
    ///
    /// # Errors
    ///
    /// Returns an error when an instruction is not exactly backed by the
    /// checked source facts, names a different provider identity, locates no
    /// matching Core handler, or omits required explicit frame authority.
    pub fn validate(
        checked_core: CheckedLoweredCoreProgram,
        source_facts: CheckedSourceFactsV1,
        frame_installations: Vec<FrameInstallationInstructionV1>,
    ) -> Result<Self, CheckedCpsAdmissionError> {
        validate_closed_residuals(&source_facts)?;
        validate_instructions(&checked_core, &source_facts, &frame_installations)?;
        validate_required_authorizations(&source_facts, &frame_installations)?;

        Ok(Self {
            checked_core,
            source_facts,
            frame_installations,
        })
    }

    /// Returns the validated typed/lowered Core/CPS program.
    #[must_use]
    pub const fn checked_core(&self) -> &CheckedLoweredCoreProgram {
        &self.checked_core
    }

    /// Returns source-only facts used during admission validation.
    #[must_use]
    pub const fn source_facts(&self) -> &CheckedSourceFactsV1 {
        &self.source_facts
    }

    /// Returns exact operation identities from the checked source facts.
    #[must_use]
    pub fn operation_identities(&self) -> &[OperationIdentityV1] {
        self.source_facts.operation_identities()
    }

    /// Returns checked handler clauses from the source evidence.
    #[must_use]
    pub fn handler_clauses(&self) -> &[CheckedHandlerClauseV1] {
        self.source_facts.handler_clauses()
    }

    /// Returns canonical residual-row descriptors from source evidence.
    #[must_use]
    pub fn residual_rows(&self) -> &[ResidualRowDescriptorV1] {
        self.source_facts.residual_rows()
    }

    /// Returns source anchors retained by the admission artifact.
    #[must_use]
    pub fn source_anchors(&self) -> &[SourceAnchor] {
        self.source_facts.source_anchors()
    }

    /// Returns caller-provided frame instructions in their original order.
    #[must_use]
    pub fn frame_installations(&self) -> &[FrameInstallationInstructionV1] {
        &self.frame_installations
    }
}

fn validate_closed_residuals(
    source_facts: &CheckedSourceFactsV1,
) -> Result<(), CheckedCpsAdmissionError> {
    if let Some(tail) = source_facts
        .residual_rows()
        .iter()
        .find_map(|row| row.open_tail().map(str::to_owned))
    {
        return Err(CheckedCpsAdmissionError::UnexpandedOpenResidualTail { tail });
    }
    Ok(())
}

fn validate_instructions(
    checked_core: &CheckedLoweredCoreProgram,
    source_facts: &CheckedSourceFactsV1,
    frame_installations: &[FrameInstallationInstructionV1],
) -> Result<(), CheckedCpsAdmissionError> {
    for instruction in frame_installations {
        match instruction {
            FrameInstallationInstructionV1::Provider {
                operation,
                provider_binding,
            } => {
                if provider_binding.operation() != operation {
                    return Err(CheckedCpsAdmissionError::ProviderIdentityMismatch {
                        expected: operation.clone(),
                        actual: provider_binding.operation().clone(),
                    });
                }
            }
            FrameInstallationInstructionV1::SourceHandler {
                operation,
                handler_name,
                core_handle,
            } => {
                if handler_name != source_facts.handler_name()
                    || !source_facts
                        .handler_clauses()
                        .iter()
                        .any(|clause| clause.operation() == operation)
                {
                    return Err(CheckedCpsAdmissionError::SourceHandlerIdentityMismatch {
                        handler_name: handler_name.clone(),
                        operation: operation.clone(),
                    });
                }
                let matches_handle =
                    find_expression_at_path(checked_core.typed().expr(), core_handle.path())
                        .is_some_and(|expression| {
                            core_handle_matches_operation(expression, operation)
                        });
                if !matches_handle {
                    return Err(CheckedCpsAdmissionError::CoreHandleLocatorMismatch {
                        handler_name: handler_name.clone(),
                        locator: core_handle.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_required_authorizations(
    source_facts: &CheckedSourceFactsV1,
    frame_installations: &[FrameInstallationInstructionV1],
) -> Result<(), CheckedCpsAdmissionError> {
    for operation in source_facts.operation_identities() {
        let has_source_handler = frame_installations.iter().any(|instruction| {
            matches!(instruction, FrameInstallationInstructionV1::SourceHandler { operation: installed, handler_name, .. } if installed == operation && handler_name == source_facts.handler_name())
        });
        if !has_source_handler {
            return Err(
                CheckedCpsAdmissionError::MissingFrameInstallationAuthorization {
                    operation: operation.clone(),
                },
            );
        }
    }
    for operation in source_facts.residual_operation_identities() {
        let has_provider = frame_installations.iter().any(|instruction| {
            matches!(instruction, FrameInstallationInstructionV1::Provider { operation: installed, .. } if installed == operation)
        });
        if !has_provider {
            return Err(
                CheckedCpsAdmissionError::MissingFrameInstallationAuthorization {
                    operation: operation.clone(),
                },
            );
        }
    }
    Ok(())
}

fn declared_operation_row_key(operation: &DeclaredConcreteOperation) -> String {
    format!(
        "operation:{}::{}::{}",
        operation.impl_type, operation.interface, operation.operation
    )
}

fn core_handle_matches_operation(expression: &CoreExpr, operation: &OperationIdentityV1) -> bool {
    let CoreExpr::Handle { clause, .. } = expression else {
        return false;
    };
    let CoreEffectOp::Operation {
        path,
        operation: core_operation,
        arg_types,
        result_type,
    } = &clause.op
    else {
        return false;
    };

    // CoreEffectOp currently carries concrete impl path, operation, and full
    // signature, but no independent interface field.  The source identity
    // retains that interface and is checked exactly against the instruction.
    path.join("::") == operation.impl_type()
        && core_operation == operation.operation()
        && core_type_spellings(arg_types) == operation.parameter_types()
        && core_type_spelling(result_type) == operation.result_type()
}

fn core_type_spellings(types: &[CoreType]) -> Vec<String> {
    types.iter().map(core_type_spelling).collect()
}

fn core_type_spelling(ty: &CoreType) -> String {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => name.clone(),
        other => format!("{other:?}"),
    }
}

fn find_expression_at_path<'a>(expr: &'a CoreExpr, path: &[usize]) -> Option<&'a CoreExpr> {
    let Some((next, rest)) = path.split_first() else {
        return Some(expr);
    };
    let child = match (expr, *next) {
        (
            CoreExpr::LetVal { body, .. }
            | CoreExpr::LetRec { body, .. }
            | CoreExpr::LetPrim { body, .. }
            | CoreExpr::LetCall { body, .. }
            | CoreExpr::LetContCall { body, .. }
            | CoreExpr::RecordDischarge { body, .. }
            | CoreExpr::Force { body, .. },
            0,
        )
        | (CoreExpr::Handle { body, .. } | CoreExpr::LetMode { body, .. }, 1) => body,
        (
            CoreExpr::If {
                then_branch,
                else_branch: _,
                ..
            },
            0,
        ) => then_branch,
        (
            CoreExpr::If {
                then_branch: _,
                else_branch,
                ..
            },
            1,
        ) => else_branch,
        (CoreExpr::Handle { clause, .. }, 0) => &clause.body,
        (CoreExpr::LetMode { expr, .. }, 0) => expr,
        _ => return None,
    };
    if rest.is_empty() {
        Some(child)
    } else {
        find_expression_at_path(child, rest)
    }
}
