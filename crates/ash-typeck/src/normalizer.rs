//! Phase 112 SPEC-D normalizer and definitional-equality core.
//!
//! The module lowers canonical type expressions into total normal-form carriers,
//! applies compiler-internal fixture equations for closed computation heads, and
//! compares canonical normal forms for definitional equality. It deliberately does
//! not expose public `type fn` syntax, source-level equation validation,
//! associated-family solving, proposition solving, recursive computation, or
//! type-function inversion.

use std::collections::{BTreeMap, BTreeSet};

use crate::type_env::TypeEnv;
use ash_core::kind::Kind;
use ash_core::semantic_summary::{DomainConstructorId, SealedDomainId};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId, TypeFunctionPattern, TypeFunctionResultExpr,
};

/// First-order fixture equation pattern used by internal normalizer tests.
///
/// These patterns are compiler-internal setup data only. They are not parsed from
/// source and are not serialized through module semantic summaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FixturePattern {
    Var(String),
    DomainConstructor(Box<FixtureDomainConstructorPattern>),
}

/// Sealed-domain constructor fixture pattern payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureDomainConstructorPattern {
    pub constructor: DomainConstructorId,
    pub domain: SealedDomainId,
    pub args: Vec<FixturePattern>,
}

/// Result expression for an internal fixture equation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FixtureResultExpr {
    BoundVar(String),
    DomainConstructor {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<FixtureResultExpr>,
        kind: Kind,
    },
    ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<FixtureResultExpr>,
        kind: Kind,
    },
}

/// Validation and registration errors for internal fixture equations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureEquationRegistryError {
    UnboundResultVariable {
        variable: String,
    },
    DuplicateEquation {
        head: TypeComputationHeadId,
    },
    ArityMismatch {
        head: TypeComputationHeadId,
        expected: usize,
        actual: usize,
    },
}

/// One internal fixture equation for a computation head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureEquation {
    head: TypeComputationHeadId,
    patterns: Vec<FixturePattern>,
    result: FixtureResultExpr,
}

impl FixtureEquation {
    pub fn new(
        head: TypeComputationHeadId,
        patterns: Vec<FixturePattern>,
        result: FixtureResultExpr,
    ) -> Result<Self, FixtureEquationRegistryError> {
        let mut bound = BTreeSet::new();
        collect_pattern_vars(&patterns, &mut bound);
        validate_result_vars(&result, &bound)?;
        Ok(Self {
            head,
            patterns,
            result,
        })
    }

    #[must_use]
    pub const fn head(&self) -> &TypeComputationHeadId {
        &self.head
    }

    #[must_use]
    pub fn patterns(&self) -> &[FixturePattern] {
        &self.patterns
    }

    #[must_use]
    pub const fn result(&self) -> &FixtureResultExpr {
        &self.result
    }

    #[must_use]
    pub fn arity(&self) -> usize {
        self.patterns.len()
    }
}

/// Deterministic registry of internal fixture equations keyed by computation head.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FixtureEquationRegistry {
    equations: Vec<FixtureEquation>,
}

impl FixtureEquationRegistry {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.equations.is_empty()
    }

    pub fn with_equation(
        mut self,
        equation: FixtureEquation,
    ) -> Result<Self, FixtureEquationRegistryError> {
        for existing in self.equations_for(equation.head()) {
            if existing.arity() != equation.arity() {
                return Err(FixtureEquationRegistryError::ArityMismatch {
                    head: equation.head().clone(),
                    expected: existing.arity(),
                    actual: equation.arity(),
                });
            }
            if existing.patterns() == equation.patterns() {
                return Err(FixtureEquationRegistryError::DuplicateEquation {
                    head: equation.head().clone(),
                });
            }
        }
        self.equations.push(equation);
        Ok(self)
    }

    pub fn equations_for<'a>(
        &'a self,
        head: &'a TypeComputationHeadId,
    ) -> impl Iterator<Item = &'a FixtureEquation> + 'a {
        self.equations
            .iter()
            .filter(move |equation| equation.head() == head)
    }

    pub fn first_match<'a>(
        &'a self,
        head: &'a TypeComputationHeadId,
        args: &[NormalTypeExpr],
    ) -> Option<FixtureEquationMatch<'a>> {
        self.equations_for(head).find_map(|equation| {
            if equation.arity() != args.len() {
                return None;
            }
            let mut bindings = BTreeMap::new();
            let matched = equation
                .patterns()
                .iter()
                .zip(args)
                .all(|(pattern, arg)| match_pattern(pattern, arg, &mut bindings));
            matched.then_some(FixtureEquationMatch { equation, bindings })
        })
    }

    fn first_match_or_blocker<'a>(
        &'a self,
        head: &'a TypeComputationHeadId,
        args: &[NormalTypeExpr],
    ) -> FixtureEquationSelection<'a> {
        for equation in self.equations_for(head) {
            if equation.arity() != args.len() {
                continue;
            }
            let mut bindings = BTreeMap::new();
            let mut matched = true;
            let mut allow_open_var_binding = equation.head().name.ends_with("Literal")
                && matches!(
                    equation.result(),
                    FixtureResultExpr::DomainConstructor { .. }
                );
            for (pattern, arg) in equation.patterns().iter().zip(args) {
                match match_pattern_open(pattern, arg, &mut bindings, allow_open_var_binding) {
                    FixturePatternMatch::Matched => {
                        if matches!(pattern, FixturePattern::DomainConstructor(_)) {
                            allow_open_var_binding = true;
                        }
                    }
                    FixturePatternMatch::NoMatch => {
                        matched = false;
                        break;
                    }
                    FixturePatternMatch::Blocked(reason) => {
                        return FixtureEquationSelection::Blocked(reason);
                    }
                }
            }
            if matched {
                return FixtureEquationSelection::Matched(FixtureEquationMatch {
                    equation,
                    bindings,
                });
            }
        }
        FixtureEquationSelection::NoMatch
    }
}

enum FixtureEquationSelection<'a> {
    Matched(FixtureEquationMatch<'a>),
    Blocked(NormalFormBlockReason),
    NoMatch,
}

/// Metadata returned by registry lookup. TASK-820 does not apply the equation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureEquationMatch<'a> {
    equation: &'a FixtureEquation,
    bindings: BTreeMap<String, NormalTypeExpr>,
}

impl<'a> FixtureEquationMatch<'a> {
    #[must_use]
    pub const fn equation(&self) -> &'a FixtureEquation {
        self.equation
    }

    #[must_use]
    pub const fn bindings(&self) -> &BTreeMap<String, NormalTypeExpr> {
        &self.bindings
    }
}

/// Normalization strategy requested by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizationMode {
    /// Reduce only as much as needed to expose the outermost head.
    ///
    /// Phase 112 keeps this as a reserved strategy flag for callers that need to
    /// record intent; the current MVP normalizer still normalizes argument spines
    /// when constructing canonical normal forms.
    WeakHead,
    /// Recursively normalize the entire reachable expression.
    Full,
    /// Use-site selected demand-driven normalization.
    ///
    /// Reserved for future mode-sensitive forcing-point control; Phase 112 uses
    /// the same total argument-spine normalization behavior as `Full` after the
    /// caller has selected a normalization boundary.
    Demand,
}

/// Robustness fuel for total normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizationFuel {
    remaining: usize,
}

impl NormalizationFuel {
    /// Creates a fuel budget with the given number of normalization steps.
    #[must_use]
    pub const fn new(remaining: usize) -> Self {
        Self { remaining }
    }

    /// Returns the remaining budget.
    #[must_use]
    pub const fn remaining(self) -> usize {
        self.remaining
    }

    fn consume(&mut self, mode: NormalizationMode) -> NormalizationResult<()> {
        if self.remaining == 0 {
            return Err(NormalizationError::FuelExhausted { mode, remaining: 0 });
        }
        self.remaining -= 1;
        Ok(())
    }
}

impl Default for NormalizationFuel {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Options controlling a normalization request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationConfig {
    pub mode: NormalizationMode,
    pub fuel: NormalizationFuel,
    pub trace: bool,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            mode: NormalizationMode::Full,
            fuel: NormalizationFuel::default(),
            trace: false,
        }
    }
}

/// A skeleton trace event emitted by normalization when tracing is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationTraceEvent {
    pub evidence: NormalizationEvidence,
    pub remaining_fuel: usize,
}

/// Structured evidence for why a normal form was produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationEvidence {
    /// The skeleton normalizer performed structural identity conversion only.
    StructuralIdentity,
    /// A computation head was preserved as neutral because no equation registry
    /// exists in TASK-819.
    NeutralUnsupportedComputation,
    /// A fixture equation reduced a closed computation-head application.
    FixtureEquationReduced,
    /// A projection was preserved without associated-family computation.
    ProjectionPreserved { rigidity: ProjectionRigidity },
}

/// Successful normalization result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationOutcome {
    pub normal: NormalTypeExpr,
    pub mode: NormalizationMode,
    pub fuel_remaining: usize,
    pub evidence: NormalizationEvidence,
    pub trace: Vec<NormalizationTraceEvent>,
}

/// Normalizer robustness errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationError {
    /// The implementation fuel budget was exhausted. This is distinct from
    /// semantic neutral/stuck normal forms.
    FuelExhausted {
        mode: NormalizationMode,
        remaining: usize,
    },
    /// Placeholder for later cycle-guard classification.
    CycleDetected,
}

/// Result alias for normalization requests.
pub type NormalizationResult<T> = Result<T, NormalizationError>;

/// Structured diagnostic classes emitted by the normalizer/definitional-equality
/// core. These are evidence carriers for compiler diagnostics and tests; they do
/// not add reduction semantics, public syntax, equation export/import, or proof
/// search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizerDiagnosticKind {
    NeutralStuckNormalizationNote,
    NeutralAssociatedProjectionNote,
    ConcreteNormalFormRequired,
    EqualityBlockedByNeutrality,
    NonInvertingEqualityNote,
    NormalizedMismatch,
    FuelOrCycleGuard,
    LegacyFallback,
}

/// One structured normalizer diagnostic/evidence item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizerDiagnostic {
    pub kind: NormalizerDiagnosticKind,
    pub message: String,
    pub normal_slice: Option<NormalTypeExpr>,
}

impl NormalizerDiagnostic {
    #[must_use]
    pub fn new(kind: NormalizerDiagnosticKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            normal_slice: None,
        }
    }

    #[must_use]
    pub fn with_normal_slice(mut self, normal_slice: NormalTypeExpr) -> Self {
        self.normal_slice = Some(normal_slice);
        self
    }
}

/// Structured normalize-and-compare definitional equality evidence.
///
/// This API is intentionally non-inverting: normal forms are compared
/// structurally, and mismatches involving neutral or rigid blockers are reported
/// as blocked evidence rather than triggering proof search or type-function
/// inversion. In particular, same-headed neutral computation apps are not treated
/// as unification problems: their normalized argument spines are compared
/// structurally, and differing spines remain blocked evidence instead of solving
/// `CanonicalTypeExpr::Var(String)` inputs from the computation head's output.
/// Current inference metas (`Type::Var(TypeVar)`) remain owned by the legacy
/// `Type` unifier until an explicit top-level bridge is introduced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionalEqualityResult {
    /// Both inputs normalize to the same canonical normal form.
    Equal,
    /// Both inputs normalized successfully, but the resulting closed/data shapes
    /// differ. The stored slices are normalized forms for diagnostics.
    NotEqual {
        lhs_norm: NormalTypeExpr,
        rhs_norm: NormalTypeExpr,
        mismatch: String,
    },
    /// Equality could not be decided without inverting or solving underneath a
    /// neutral/rigid blocker. The stored slices are normalized forms for future
    /// diagnostics.
    BlockedByNeutrality {
        lhs_norm: NormalTypeExpr,
        rhs_norm: NormalTypeExpr,
        neutral_subterms: Vec<NormalTypeExpr>,
        no_inversion_note: String,
    },
}

/// Environment-aware normalizer.
pub struct Normalizer<'env> {
    env: &'env TypeEnv,
    config: NormalizationConfig,
    fixture_registry: &'env FixtureEquationRegistry,
}

static EMPTY_FIXTURE_EQUATION_REGISTRY: FixtureEquationRegistry = FixtureEquationRegistry {
    equations: Vec::new(),
};

impl<'env> Normalizer<'env> {
    /// Creates a normalizer with default full-normalization options.
    #[must_use]
    pub fn new(env: &'env TypeEnv) -> Self {
        Self::with_config_and_registry(
            env,
            NormalizationConfig::default(),
            &EMPTY_FIXTURE_EQUATION_REGISTRY,
        )
    }

    /// Creates a normalizer with explicit configuration.
    #[must_use]
    pub fn with_config(env: &'env TypeEnv, config: NormalizationConfig) -> Self {
        Self::with_config_and_registry(env, config, &EMPTY_FIXTURE_EQUATION_REGISTRY)
    }

    /// Creates a normalizer with explicit internal fixture equations.
    ///
    /// Fixture equations are compiler-internal setup data. Closed matching and
    /// result substitution are consumed by normalization; unmatched computation
    /// heads are still preserved as neutral forms.
    #[must_use]
    pub fn with_registry(env: &'env TypeEnv, registry: &'env FixtureEquationRegistry) -> Self {
        Self::with_config_and_registry(env, NormalizationConfig::default(), registry)
    }

    /// Creates a normalizer with explicit configuration and fixture equations.
    #[must_use]
    pub fn with_config_and_registry(
        env: &'env TypeEnv,
        config: NormalizationConfig,
        registry: &'env FixtureEquationRegistry,
    ) -> Self {
        Self {
            env,
            config,
            fixture_registry: registry,
        }
    }

    /// Normalizes a canonical type expression into a normal-form carrier.
    pub fn normalize(&self, expr: &CanonicalTypeExpr) -> NormalizationResult<NormalizationOutcome> {
        let mut state = NormalizationState {
            mode: self.config.mode,
            fuel: self.config.fuel,
            trace_enabled: self.config.trace,
            trace: Vec::new(),
            fixture_registry: self.fixture_registry,
            env: self.env,
        };
        let (normal, evidence) = state.normalize_expr(expr)?;
        Ok(NormalizationOutcome {
            normal,
            mode: self.config.mode,
            fuel_remaining: state.fuel.remaining(),
            evidence,
            trace: state.trace,
        })
    }

    /// Normalize an already-known computation application spine.
    ///
    /// This TASK-838 helper is useful when callers already hold normal-form
    /// sealed-domain constructor values. It consults only internal fixtures and
    /// checked module-local source declarations registered in this `TypeEnv`.
    pub fn normalize_known_computation_app(
        &self,
        head: &TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: &Kind,
    ) -> NormalizationResult<NormalTypeExpr> {
        let mut state = NormalizationState {
            mode: self.config.mode,
            fuel: self.config.fuel,
            trace_enabled: self.config.trace,
            trace: Vec::new(),
            fixture_registry: self.fixture_registry,
            env: self.env,
        };
        state.reduce_normalized_computation_app(head, args, kind)
    }

    /// Compare two already-normal-form expressions structurally using the same
    /// non-inverting evidence contract as canonical definitional equality.
    #[must_use]
    pub fn definitional_equality_normal_forms(
        &self,
        lhs_norm: &NormalTypeExpr,
        rhs_norm: &NormalTypeExpr,
    ) -> DefinitionalEqualityResult {
        definitional_equality_for_normal_forms(lhs_norm.clone(), rhs_norm.clone())
    }

    /// Normalize both inputs in full-normalization mode and compare the canonical
    /// normal forms structurally.
    ///
    /// This method deliberately does not perform proof search, type-function
    /// inversion, associated-family computation, or any `TypeEnv` forcing-point
    /// adoption. Normalization failures such as fuel exhaustion propagate as
    /// robustness errors rather than becoming semantic stuckness evidence.
    pub fn definitional_equality(
        &self,
        lhs: &CanonicalTypeExpr,
        rhs: &CanonicalTypeExpr,
    ) -> NormalizationResult<DefinitionalEqualityResult> {
        let mut full_config = self.config.clone();
        full_config.mode = NormalizationMode::Full;
        let full_normalizer =
            Self::with_config_and_registry(self.env, full_config, self.fixture_registry);
        let lhs_norm = full_normalizer.normalize(lhs)?.normal;
        let rhs_norm = full_normalizer.normalize(rhs)?.normal;

        Ok(definitional_equality_for_normal_forms(lhs_norm, rhs_norm))
    }

    /// Boolean convenience wrapper derived only from structured equality
    /// evidence.
    pub fn definitionally_equal(
        &self,
        lhs: &CanonicalTypeExpr,
        rhs: &CanonicalTypeExpr,
    ) -> NormalizationResult<bool> {
        self.definitional_equality(lhs, rhs)
            .map(|evidence| matches!(evidence, DefinitionalEqualityResult::Equal))
    }

    /// Require a concrete, non-neutral data normal form for contexts that cannot
    /// proceed with a stuck computation head or projection. This is a diagnostic
    /// helper only; it does not force inversion or add reduction rules.
    pub fn require_concrete_normal_form(
        &self,
        expr: &CanonicalTypeExpr,
    ) -> Result<NormalTypeExpr, Box<NormalizerDiagnostic>> {
        match self.normalize(expr) {
            Ok(outcome) => match outcome.normal {
                normal @ (NormalTypeExpr::Primitive(_)
                | NormalTypeExpr::NominalApp { .. }
                | NormalTypeExpr::DomainConstructorApp { .. }) => Ok(normal),
                normal @ (NormalTypeExpr::Var(_)
                | NormalTypeExpr::NeutralComputationApp { .. }
                | NormalTypeExpr::Projection { .. }) => Err(Box::new(NormalizerDiagnostic::new(
                    NormalizerDiagnosticKind::ConcreteNormalFormRequired,
                    "concrete normal form required; normalization produced a neutral/stuck normal form and equality will not invert it",
                )
                .with_normal_slice(normal))),
            },
            Err(error) => Err(Box::new(NormalizerDiagnostic::new(
                NormalizerDiagnosticKind::FuelOrCycleGuard,
                format!(
                    "normalizer implementation guard failed while requiring a concrete normal form: {error:?}"
                ),
            ))),
        }
    }

    /// Diagnostic evidence for a single normalization request.
    pub fn diagnostics_for_normalization(
        &self,
        expr: &CanonicalTypeExpr,
    ) -> Vec<NormalizerDiagnostic> {
        match self.normalize(expr) {
            Ok(outcome) => diagnostics_for_normal_form(&outcome.normal),
            Err(error) => vec![fuel_or_cycle_guard_diagnostic(error)],
        }
    }

    /// Diagnostic evidence for normalize-and-compare equality.
    pub fn diagnostics_for_definitional_equality(
        &self,
        lhs: &CanonicalTypeExpr,
        rhs: &CanonicalTypeExpr,
    ) -> Vec<NormalizerDiagnostic> {
        match self.definitional_equality(lhs, rhs) {
            Ok(DefinitionalEqualityResult::Equal) => Vec::new(),
            Ok(DefinitionalEqualityResult::NotEqual {
                lhs_norm,
                rhs_norm,
                mismatch,
            }) => vec![NormalizerDiagnostic::new(
                NormalizerDiagnosticKind::NormalizedMismatch,
                format!(
                    "normalized mismatch at {mismatch}: left normal form {lhs_norm:?}; right normal form {rhs_norm:?}"
                ),
            )],
            Ok(DefinitionalEqualityResult::BlockedByNeutrality {
                lhs_norm,
                rhs_norm,
                neutral_subterms,
                no_inversion_note,
            }) => {
                let mut diagnostics = vec![NormalizerDiagnostic::new(
                    NormalizerDiagnosticKind::EqualityBlockedByNeutrality,
                    format!(
                        "equality blocked by neutral/stuck normal form: left {lhs_norm:?}; right {rhs_norm:?}"
                    ),
                )];
                diagnostics.extend(neutral_subterms.into_iter().map(|normal| {
                    NormalizerDiagnostic::new(
                        NormalizerDiagnosticKind::NeutralStuckNormalizationNote,
                        "neutral/stuck normalization note: blocker is preserved structurally",
                    )
                    .with_normal_slice(normal)
                }));
                diagnostics.push(NormalizerDiagnostic::new(
                    NormalizerDiagnosticKind::NonInvertingEqualityNote,
                    no_inversion_note,
                ));
                diagnostics
            }
            Err(error) => vec![fuel_or_cycle_guard_diagnostic(error)],
        }
    }

    /// Returns this normalizer's configuration.
    #[must_use]
    pub const fn config(&self) -> &NormalizationConfig {
        &self.config
    }

    /// Returns the explicit internal fixture registry attached to this normalizer.
    #[must_use]
    pub const fn fixture_registry(&self) -> &FixtureEquationRegistry {
        self.fixture_registry
    }
}

fn diagnostics_for_normal_form(normal: &NormalTypeExpr) -> Vec<NormalizerDiagnostic> {
    let mut diagnostics = Vec::new();
    collect_normal_form_diagnostics(normal, &mut diagnostics);
    diagnostics
}

fn collect_normal_form_diagnostics(
    normal: &NormalTypeExpr,
    diagnostics: &mut Vec<NormalizerDiagnostic>,
) {
    match normal {
        NormalTypeExpr::NeutralComputationApp { reason, args, .. } => {
            diagnostics.push(
                NormalizerDiagnostic::new(
                    NormalizerDiagnosticKind::NeutralStuckNormalizationNote,
                    format!(
                        "neutral/stuck normalization note: computation head is blocked by {:?}",
                        reason.clone()
                    ),
                )
                .with_normal_slice(normal.clone()),
            );
            for arg in args {
                collect_normal_form_diagnostics(arg, diagnostics);
            }
        }
        NormalTypeExpr::Projection {
            rigidity,
            reason,
            args,
            ..
        } => {
            diagnostics.push(
                NormalizerDiagnostic::new(
                    NormalizerDiagnosticKind::NeutralAssociatedProjectionNote,
                    format!(
                        "neutral associated projection note: {rigidity:?} projection is blocked by {:?} and is preserved without associated-family computation",
                        reason.clone().unwrap_or(match rigidity {
                            ProjectionRigidity::Rigid => NormalFormBlockReason::RigidProjection,
                            ProjectionRigidity::Neutral => NormalFormBlockReason::AbstractScrutinee,
                        })
                    ),
                )
                .with_normal_slice(normal.clone()),
            );
            for arg in args {
                collect_normal_form_diagnostics(arg, diagnostics);
            }
        }
        NormalTypeExpr::NominalApp { args, .. }
        | NormalTypeExpr::DomainConstructorApp { args, .. } => {
            for arg in args {
                collect_normal_form_diagnostics(arg, diagnostics);
            }
        }
        NormalTypeExpr::Primitive(_) | NormalTypeExpr::Var(_) => {}
    }
}

fn fuel_or_cycle_guard_diagnostic(error: NormalizationError) -> NormalizerDiagnostic {
    NormalizerDiagnostic::new(
        NormalizerDiagnosticKind::FuelOrCycleGuard,
        format!(
            "normalizer implementation fuel/cycle guard failed; this is not semantic stuckness: {error:?}"
        ),
    )
}

fn definitional_equality_for_normal_forms(
    lhs_norm: NormalTypeExpr,
    rhs_norm: NormalTypeExpr,
) -> DefinitionalEqualityResult {
    if normal_forms_definitionally_equal(&lhs_norm, &rhs_norm) {
        return DefinitionalEqualityResult::Equal;
    }

    if normal_forms_are_structurally_disjoint(&lhs_norm, &rhs_norm) {
        return DefinitionalEqualityResult::NotEqual {
            lhs_norm,
            rhs_norm,
            mismatch: "root".to_string(),
        };
    }

    let neutral_subterms = neutrality_blockers_for_mismatch(&lhs_norm, &rhs_norm);
    if neutral_subterms.is_empty() {
        DefinitionalEqualityResult::NotEqual {
            lhs_norm,
            rhs_norm,
            mismatch: "root".to_string(),
        }
    } else {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            rhs_norm,
            neutral_subterms,
            no_inversion_note:
                "definitional equality normalizes and compares; it does not invert neutral computation heads or projections"
                    .to_string(),
        }
    }
}

fn normal_forms_definitionally_equal(lhs: &NormalTypeExpr, rhs: &NormalTypeExpr) -> bool {
    match (lhs, rhs) {
        (NormalTypeExpr::Primitive(lhs), NormalTypeExpr::Primitive(rhs))
        | (NormalTypeExpr::Var(lhs), NormalTypeExpr::Var(rhs)) => lhs == rhs,
        (
            NormalTypeExpr::NominalApp {
                origin: lhs_origin,
                args: lhs_args,
                kind: lhs_kind,
                ..
            },
            NormalTypeExpr::NominalApp {
                origin: rhs_origin,
                args: rhs_args,
                kind: rhs_kind,
                ..
            },
        ) => {
            lhs_origin == rhs_origin
                && lhs_kind == rhs_kind
                && normal_arg_spines_definitionally_equal(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::DomainConstructorApp {
                constructor: lhs_constructor,
                domain: lhs_domain,
                args: lhs_args,
                kind: lhs_kind,
            },
            NormalTypeExpr::DomainConstructorApp {
                constructor: rhs_constructor,
                domain: rhs_domain,
                args: rhs_args,
                kind: rhs_kind,
            },
        ) => {
            lhs_constructor == rhs_constructor
                && lhs_domain == rhs_domain
                && lhs_kind == rhs_kind
                && normal_arg_spines_definitionally_equal(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::NeutralComputationApp {
                head: lhs_head,
                args: lhs_args,
                kind: lhs_kind,
                ..
            },
            NormalTypeExpr::NeutralComputationApp {
                head: rhs_head,
                args: rhs_args,
                kind: rhs_kind,
                ..
            },
        ) => {
            lhs_head == rhs_head
                && lhs_kind == rhs_kind
                && normal_arg_spines_definitionally_equal(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::Projection {
                interface: lhs_interface,
                member: lhs_member,
                args: lhs_args,
                kind: lhs_kind,
                rigidity: lhs_rigidity,
                ..
            },
            NormalTypeExpr::Projection {
                interface: rhs_interface,
                member: rhs_member,
                args: rhs_args,
                kind: rhs_kind,
                rigidity: rhs_rigidity,
                ..
            },
        ) => {
            lhs_interface == rhs_interface
                && lhs_member == rhs_member
                && lhs_kind == rhs_kind
                && lhs_rigidity == rhs_rigidity
                && normal_arg_spines_definitionally_equal(lhs_args, rhs_args)
        }
        _ => false,
    }
}

fn normal_arg_spines_definitionally_equal(
    lhs_args: &[NormalTypeExpr],
    rhs_args: &[NormalTypeExpr],
) -> bool {
    lhs_args.len() == rhs_args.len()
        && lhs_args
            .iter()
            .zip(rhs_args)
            .all(|(lhs, rhs)| normal_forms_definitionally_equal(lhs, rhs))
}

fn normal_forms_are_structurally_disjoint(lhs: &NormalTypeExpr, rhs: &NormalTypeExpr) -> bool {
    match (lhs, rhs) {
        (NormalTypeExpr::Primitive(lhs), NormalTypeExpr::Primitive(rhs))
        | (NormalTypeExpr::Var(lhs), NormalTypeExpr::Var(rhs)) => lhs != rhs,
        (
            NormalTypeExpr::NominalApp {
                origin: lhs_origin,
                args: lhs_args,
                kind: lhs_kind,
                ..
            },
            NormalTypeExpr::NominalApp {
                origin: rhs_origin,
                args: rhs_args,
                kind: rhs_kind,
                ..
            },
        ) => {
            lhs_origin != rhs_origin
                || lhs_kind != rhs_kind
                || normal_arg_spines_structurally_disjoint(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::DomainConstructorApp {
                constructor: lhs_constructor,
                domain: lhs_domain,
                args: lhs_args,
                kind: lhs_kind,
                ..
            },
            NormalTypeExpr::DomainConstructorApp {
                constructor: rhs_constructor,
                domain: rhs_domain,
                args: rhs_args,
                kind: rhs_kind,
                ..
            },
        ) => {
            lhs_constructor != rhs_constructor
                || lhs_domain != rhs_domain
                || lhs_kind != rhs_kind
                || normal_arg_spines_structurally_disjoint(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::NeutralComputationApp {
                head: lhs_head,
                args: lhs_args,
                kind: lhs_kind,
                ..
            },
            NormalTypeExpr::NeutralComputationApp {
                head: rhs_head,
                args: rhs_args,
                kind: rhs_kind,
                ..
            },
        ) => {
            lhs_head != rhs_head
                || lhs_kind != rhs_kind
                || lhs_args.len() != rhs_args.len()
                || normal_arg_spines_structurally_disjoint(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::Projection {
                interface: lhs_interface,
                member: lhs_member,
                args: lhs_args,
                kind: lhs_kind,
                rigidity: lhs_rigidity,
                ..
            },
            NormalTypeExpr::Projection {
                interface: rhs_interface,
                member: rhs_member,
                args: rhs_args,
                kind: rhs_kind,
                rigidity: rhs_rigidity,
                ..
            },
        ) => {
            lhs_interface != rhs_interface
                || lhs_member != rhs_member
                || lhs_kind != rhs_kind
                || lhs_rigidity != rhs_rigidity
                || lhs_args.len() != rhs_args.len()
                || normal_arg_spines_structurally_disjoint(lhs_args, rhs_args)
        }
        (
            NormalTypeExpr::Primitive(_)
            | NormalTypeExpr::Var(_)
            | NormalTypeExpr::NominalApp { .. }
            | NormalTypeExpr::DomainConstructorApp { .. },
            NormalTypeExpr::Primitive(_)
            | NormalTypeExpr::Var(_)
            | NormalTypeExpr::NominalApp { .. }
            | NormalTypeExpr::DomainConstructorApp { .. },
        ) => true,
        _ => false,
    }
}

fn normal_arg_spines_structurally_disjoint(
    lhs_args: &[NormalTypeExpr],
    rhs_args: &[NormalTypeExpr],
) -> bool {
    lhs_args.len() != rhs_args.len()
        || lhs_args
            .iter()
            .zip(rhs_args)
            .any(|(lhs, rhs)| normal_forms_are_structurally_disjoint_in_spine(lhs, rhs))
}

fn normal_forms_are_structurally_disjoint_in_spine(
    lhs: &NormalTypeExpr,
    rhs: &NormalTypeExpr,
) -> bool {
    normal_form_is_concrete(lhs)
        && normal_form_is_concrete(rhs)
        && normal_forms_are_structurally_disjoint(lhs, rhs)
}

fn normal_form_is_concrete(expr: &NormalTypeExpr) -> bool {
    match expr {
        NormalTypeExpr::Primitive(_) => true,
        NormalTypeExpr::NominalApp { args, .. }
        | NormalTypeExpr::DomainConstructorApp { args, .. } => {
            args.iter().all(normal_form_is_concrete)
        }
        NormalTypeExpr::Var(_)
        | NormalTypeExpr::NeutralComputationApp { .. }
        | NormalTypeExpr::Projection { .. } => false,
    }
}

fn neutrality_blockers_for_mismatch(
    lhs: &NormalTypeExpr,
    rhs: &NormalTypeExpr,
) -> Vec<NormalTypeExpr> {
    let mut blockers = Vec::new();
    collect_neutrality_blockers(lhs, &mut blockers);
    collect_neutrality_blockers(rhs, &mut blockers);
    blockers
}

fn collect_neutrality_blockers(expr: &NormalTypeExpr, blockers: &mut Vec<NormalTypeExpr>) {
    match expr {
        NormalTypeExpr::NeutralComputationApp { .. } | NormalTypeExpr::Projection { .. } => {
            blockers.push(expr.clone());
        }
        NormalTypeExpr::NominalApp { args, .. }
        | NormalTypeExpr::DomainConstructorApp { args, .. } => {
            for arg in args {
                collect_neutrality_blockers(arg, blockers);
            }
        }
        NormalTypeExpr::Primitive(_) | NormalTypeExpr::Var(_) => {}
    }
}

fn collect_pattern_vars(patterns: &[FixturePattern], bound: &mut BTreeSet<String>) {
    for pattern in patterns {
        match pattern {
            FixturePattern::Var(name) => {
                bound.insert(name.clone());
            }
            FixturePattern::DomainConstructor(pattern) => {
                collect_pattern_vars(&pattern.args, bound);
            }
        }
    }
}

fn validate_result_vars(
    result: &FixtureResultExpr,
    bound: &BTreeSet<String>,
) -> Result<(), FixtureEquationRegistryError> {
    match result {
        FixtureResultExpr::BoundVar(variable) => {
            if bound.contains(variable) {
                Ok(())
            } else {
                Err(FixtureEquationRegistryError::UnboundResultVariable {
                    variable: variable.clone(),
                })
            }
        }
        FixtureResultExpr::DomainConstructor { args, .. }
        | FixtureResultExpr::ComputationHeadApp { args, .. } => {
            for arg in args {
                validate_result_vars(arg, bound)?;
            }
            Ok(())
        }
    }
}

fn match_pattern(
    pattern: &FixturePattern,
    arg: &NormalTypeExpr,
    bindings: &mut BTreeMap<String, NormalTypeExpr>,
) -> bool {
    match pattern {
        FixturePattern::Var(name) => match bindings.get(name) {
            Some(bound) => bound == arg,
            None => {
                bindings.insert(name.clone(), arg.clone());
                true
            }
        },
        FixturePattern::DomainConstructor(pattern) => match arg {
            NormalTypeExpr::DomainConstructorApp {
                constructor: arg_constructor,
                domain: arg_domain,
                args: arg_args,
                ..
            } => {
                pattern.constructor == *arg_constructor
                    && pattern.domain == *arg_domain
                    && pattern.args.len() == arg_args.len()
                    && pattern
                        .args
                        .iter()
                        .zip(arg_args)
                        .all(|(pattern, arg)| match_pattern(pattern, arg, bindings))
            }
            _ => false,
        },
    }
}

enum FixturePatternMatch {
    Matched,
    NoMatch,
    Blocked(NormalFormBlockReason),
}

fn match_pattern_open(
    pattern: &FixturePattern,
    arg: &NormalTypeExpr,
    bindings: &mut BTreeMap<String, NormalTypeExpr>,
    allow_open_var_binding: bool,
) -> FixturePatternMatch {
    match pattern {
        FixturePattern::Var(name) => match bindings.get(name) {
            Some(bound) if bound == arg => FixturePatternMatch::Matched,
            Some(_) => FixturePatternMatch::NoMatch,
            None => match arg {
                NormalTypeExpr::Var(_)
                | NormalTypeExpr::NeutralComputationApp { .. }
                | NormalTypeExpr::Projection { .. }
                    if !allow_open_var_binding =>
                {
                    FixturePatternMatch::Blocked(block_reason_for_normal(arg))
                }
                _ => {
                    bindings.insert(name.clone(), arg.clone());
                    FixturePatternMatch::Matched
                }
            },
        },
        FixturePattern::DomainConstructor(pattern) => match arg {
            NormalTypeExpr::DomainConstructorApp {
                constructor: arg_constructor,
                domain: arg_domain,
                args: arg_args,
                ..
            } => {
                if pattern.constructor != *arg_constructor
                    || pattern.domain != *arg_domain
                    || pattern.args.len() != arg_args.len()
                {
                    return FixturePatternMatch::NoMatch;
                }
                for (pattern, arg) in pattern.args.iter().zip(arg_args) {
                    match match_pattern_open(pattern, arg, bindings, true) {
                        FixturePatternMatch::Matched => {}
                        other => return other,
                    }
                }
                FixturePatternMatch::Matched
            }
            NormalTypeExpr::Var(_)
            | NormalTypeExpr::NeutralComputationApp { .. }
            | NormalTypeExpr::Projection { .. } => {
                FixturePatternMatch::Blocked(block_reason_for_normal(arg))
            }
            _ => FixturePatternMatch::NoMatch,
        },
    }
}

fn match_source_pattern_open(
    pattern: &TypeFunctionPattern,
    arg: &NormalTypeExpr,
    bindings: &mut BTreeMap<String, NormalTypeExpr>,
    allow_open_var_binding: bool,
) -> FixturePatternMatch {
    match pattern {
        TypeFunctionPattern::Var { name, .. } => match bindings.get(name) {
            Some(bound) if bound == arg => FixturePatternMatch::Matched,
            Some(_) => FixturePatternMatch::NoMatch,
            None => match arg {
                NormalTypeExpr::Var(_)
                | NormalTypeExpr::NeutralComputationApp { .. }
                | NormalTypeExpr::Projection { .. }
                    if !allow_open_var_binding =>
                {
                    FixturePatternMatch::Blocked(block_reason_for_normal(arg))
                }
                _ => {
                    bindings.insert(name.clone(), arg.clone());
                    FixturePatternMatch::Matched
                }
            },
        },
        TypeFunctionPattern::Wildcard { .. } => match arg {
            NormalTypeExpr::Var(_)
            | NormalTypeExpr::NeutralComputationApp { .. }
            | NormalTypeExpr::Projection { .. }
                if !allow_open_var_binding =>
            {
                FixturePatternMatch::Blocked(block_reason_for_normal(arg))
            }
            _ => FixturePatternMatch::Matched,
        },
        TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } => match arg {
            NormalTypeExpr::DomainConstructorApp {
                constructor: arg_constructor,
                domain: arg_domain,
                args: arg_args,
                ..
            } => {
                if constructor != arg_constructor
                    || domain != arg_domain
                    || fields.len() != arg_args.len()
                {
                    return FixturePatternMatch::NoMatch;
                }
                for (pattern, arg) in fields.iter().zip(arg_args) {
                    match match_source_pattern_open(pattern, arg, bindings, true) {
                        FixturePatternMatch::Matched => {}
                        other => return other,
                    }
                }
                FixturePatternMatch::Matched
            }
            NormalTypeExpr::Var(_)
            | NormalTypeExpr::NeutralComputationApp { .. }
            | NormalTypeExpr::Projection { .. } => {
                FixturePatternMatch::Blocked(block_reason_for_normal(arg))
            }
            _ => FixturePatternMatch::NoMatch,
        },
    }
}

fn block_reason_for_normal(arg: &NormalTypeExpr) -> NormalFormBlockReason {
    match arg {
        NormalTypeExpr::Var(_) => NormalFormBlockReason::AbstractScrutinee,
        NormalTypeExpr::NeutralComputationApp { reason, .. } => reason.clone(),
        NormalTypeExpr::Projection {
            rigidity, reason, ..
        } => reason.clone().unwrap_or(match rigidity {
            ProjectionRigidity::Rigid => NormalFormBlockReason::RigidProjection,
            ProjectionRigidity::Neutral => NormalFormBlockReason::AbstractScrutinee,
        }),
        _ => NormalFormBlockReason::Unsupported,
    }
}

enum SourceEquationSelection {
    Matched {
        result: TypeFunctionResultExpr,
        bindings: BTreeMap<String, NormalTypeExpr>,
    },
    Blocked(NormalFormBlockReason),
    NoMatch,
}

struct NormalizationState<'env> {
    mode: NormalizationMode,
    fuel: NormalizationFuel,
    trace_enabled: bool,
    trace: Vec<NormalizationTraceEvent>,
    fixture_registry: &'env FixtureEquationRegistry,
    env: &'env TypeEnv,
}

enum ComputationReduction {
    Reduced(NormalTypeExpr),
    Neutral {
        normal: NormalTypeExpr,
        evidence: NormalizationEvidence,
    },
}

impl<'env> NormalizationState<'env> {
    fn normalize_expr(
        &mut self,
        expr: &CanonicalTypeExpr,
    ) -> NormalizationResult<(NormalTypeExpr, NormalizationEvidence)> {
        self.fuel.consume(self.mode)?;

        let (normal, evidence) = match expr {
            CanonicalTypeExpr::Primitive(name) => (
                NormalTypeExpr::Primitive(name.clone()),
                NormalizationEvidence::StructuralIdentity,
            ),
            CanonicalTypeExpr::Var(name) => (
                NormalTypeExpr::Var(name.clone()),
                NormalizationEvidence::StructuralIdentity,
            ),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
            } => {
                if let Some(target) =
                    self.env
                        .transparent_alias_canonical_target(origin, visible_name, args)
                {
                    self.normalize_expr(&target)?
                } else {
                    (
                        NormalTypeExpr::NominalApp {
                            origin: origin.clone(),
                            visible_name: visible_name.clone(),
                            args: self.normalize_args(args)?,
                            kind: kind.clone(),
                        },
                        NormalizationEvidence::StructuralIdentity,
                    )
                }
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                rigidity,
            } => {
                let reason = match rigidity {
                    ProjectionRigidity::Rigid => Some(NormalFormBlockReason::RigidProjection),
                    ProjectionRigidity::Neutral => Some(NormalFormBlockReason::AbstractScrutinee),
                };
                (
                    NormalTypeExpr::Projection {
                        interface: interface.clone(),
                        member: member.clone(),
                        args: self.normalize_args(args)?,
                        kind: kind.clone(),
                        rigidity: *rigidity,
                        reason,
                    },
                    NormalizationEvidence::ProjectionPreserved {
                        rigidity: *rigidity,
                    },
                )
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
                self.normalize_computation_app(head, args, kind)?
            }
        };

        self.record(evidence.clone());
        Ok((normal, evidence))
    }

    fn normalize_args(
        &mut self,
        args: &[CanonicalTypeExpr],
    ) -> NormalizationResult<Vec<NormalTypeExpr>> {
        args.iter()
            .map(|arg| self.normalize_expr(arg).map(|(normal, _)| normal))
            .collect()
    }

    fn normalize_computation_app(
        &mut self,
        head: &TypeComputationHeadId,
        args: &[CanonicalTypeExpr],
        kind: &Kind,
    ) -> NormalizationResult<(NormalTypeExpr, NormalizationEvidence)> {
        let normalized_args = self.normalize_args(args)?;
        match self.select_and_reduce_normalized_computation_app(head, normalized_args, kind)? {
            ComputationReduction::Reduced(normal) => {
                Ok((normal, NormalizationEvidence::FixtureEquationReduced))
            }
            ComputationReduction::Neutral { normal, evidence } => Ok((normal, evidence)),
        }
    }

    fn normalize_fixture_result(
        &mut self,
        result: &FixtureResultExpr,
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> NormalizationResult<NormalTypeExpr> {
        self.fuel.consume(self.mode)?;
        match result {
            FixtureResultExpr::BoundVar(variable) => Ok(bindings
                .get(variable)
                .cloned()
                .expect("fixture result variables are validated at equation construction")),
            FixtureResultExpr::DomainConstructor {
                constructor,
                domain,
                args,
                kind,
            } => Ok(NormalTypeExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: self.normalize_fixture_result_args(args, bindings)?,
                kind: kind.clone(),
            }),
            FixtureResultExpr::ComputationHeadApp { head, args, kind } => {
                let args = self.normalize_fixture_result_args(args, bindings)?;
                self.reduce_normalized_computation_app(head, args, kind)
            }
        }
    }

    fn normalize_fixture_result_args(
        &mut self,
        args: &[FixtureResultExpr],
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> NormalizationResult<Vec<NormalTypeExpr>> {
        args.iter()
            .map(|arg| self.normalize_fixture_result(arg, bindings))
            .collect()
    }

    fn reduce_normalized_computation_app(
        &mut self,
        head: &TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: &Kind,
    ) -> NormalizationResult<NormalTypeExpr> {
        match self.select_and_reduce_normalized_computation_app(head, args, kind)? {
            ComputationReduction::Reduced(normal)
            | ComputationReduction::Neutral { normal, .. } => Ok(normal),
        }
    }

    fn select_and_reduce_normalized_computation_app(
        &mut self,
        head: &TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: &Kind,
    ) -> NormalizationResult<ComputationReduction> {
        match self.fixture_registry.first_match_or_blocker(head, &args) {
            FixtureEquationSelection::Matched(matched) => {
                let result = matched.equation().result().clone();
                let bindings = matched.bindings().clone();
                self.fuel.consume(self.mode)?;
                let reduced = self.normalize_fixture_result(&result, &bindings)?;
                Ok(ComputationReduction::Reduced(reduced))
            }
            FixtureEquationSelection::Blocked(reason) => Ok(ComputationReduction::Neutral {
                normal: NormalTypeExpr::NeutralComputationApp {
                    head: head.clone(),
                    args,
                    kind: kind.clone(),
                    reason,
                },
                evidence: NormalizationEvidence::NeutralUnsupportedComputation,
            }),
            FixtureEquationSelection::NoMatch => {
                match self.source_first_match_or_blocker(head, &args) {
                    SourceEquationSelection::Matched { result, bindings } => {
                        self.fuel.consume(self.mode)?;
                        let reduced = self.normalize_source_result(&result, &bindings)?;
                        Ok(ComputationReduction::Reduced(reduced))
                    }
                    SourceEquationSelection::Blocked(reason) => Ok(ComputationReduction::Neutral {
                        normal: NormalTypeExpr::NeutralComputationApp {
                            head: head.clone(),
                            args,
                            kind: kind.clone(),
                            reason,
                        },
                        evidence: NormalizationEvidence::NeutralUnsupportedComputation,
                    }),
                    SourceEquationSelection::NoMatch => Ok(ComputationReduction::Neutral {
                        normal: NormalTypeExpr::NeutralComputationApp {
                            head: head.clone(),
                            args,
                            kind: kind.clone(),
                            reason: NormalFormBlockReason::Unsupported,
                        },
                        evidence: NormalizationEvidence::NeutralUnsupportedComputation,
                    }),
                }
            }
        }
    }

    fn source_first_match_or_blocker(
        &self,
        head: &TypeComputationHeadId,
        args: &[NormalTypeExpr],
    ) -> SourceEquationSelection {
        let Some(def) = self.env.lookup_local_type_function_by_head(head) else {
            return SourceEquationSelection::NoMatch;
        };
        for equation in &def.equations {
            if equation.patterns.len() != args.len() {
                continue;
            }
            let mut bindings = BTreeMap::new();
            let mut matched = true;
            let mut allow_open_var_binding = false;
            for (pattern, arg) in equation.patterns.iter().zip(args) {
                match match_source_pattern_open(pattern, arg, &mut bindings, allow_open_var_binding)
                {
                    FixturePatternMatch::Matched => {
                        if matches!(pattern, TypeFunctionPattern::DomainConstructor { .. }) {
                            allow_open_var_binding = true;
                        }
                    }
                    FixturePatternMatch::NoMatch => {
                        matched = false;
                        break;
                    }
                    FixturePatternMatch::Blocked(reason) => {
                        return SourceEquationSelection::Blocked(reason);
                    }
                }
            }
            if matched {
                return SourceEquationSelection::Matched {
                    result: equation.result.clone(),
                    bindings,
                };
            }
        }
        SourceEquationSelection::NoMatch
    }

    fn normalize_source_result(
        &mut self,
        result: &TypeFunctionResultExpr,
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> NormalizationResult<NormalTypeExpr> {
        self.fuel.consume(self.mode)?;
        match result {
            TypeFunctionResultExpr::Primitive { name, .. } => {
                Ok(NormalTypeExpr::Primitive(name.clone()))
            }
            TypeFunctionResultExpr::Var { name, .. } => Ok(bindings
                .get(name)
                .cloned()
                .expect("type-function result variables are validated during lowering")),
            TypeFunctionResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                ..
            } => Ok(NormalTypeExpr::NominalApp {
                origin: origin.clone(),
                visible_name: visible_name.clone(),
                args: self.normalize_source_result_args(args, bindings)?,
                kind: kind.clone(),
            }),
            TypeFunctionResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                ..
            } => Ok(NormalTypeExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: self.normalize_source_result_args(args, bindings)?,
                kind: kind.clone(),
            }),
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                rigidity,
                ..
            } => Ok(NormalTypeExpr::Projection {
                interface: interface.clone(),
                member: member.clone(),
                args: self.normalize_source_result_args(args, bindings)?,
                kind: kind.clone(),
                rigidity: *rigidity,
                reason: Some(match rigidity {
                    ProjectionRigidity::Rigid => NormalFormBlockReason::RigidProjection,
                    ProjectionRigidity::Neutral => NormalFormBlockReason::AbstractScrutinee,
                }),
            }),
            TypeFunctionResultExpr::ComputationHeadApp {
                head, args, kind, ..
            } => {
                let args = self.normalize_source_result_args(args, bindings)?;
                self.reduce_normalized_computation_app(head, args, kind)
            }
        }
    }

    fn normalize_source_result_args(
        &mut self,
        args: &[TypeFunctionResultExpr],
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> NormalizationResult<Vec<NormalTypeExpr>> {
        args.iter()
            .map(|arg| self.normalize_source_result(arg, bindings))
            .collect()
    }

    fn record(&mut self, evidence: NormalizationEvidence) {
        if self.trace_enabled {
            self.trace.push(NormalizationTraceEvent {
                evidence,
                remaining_fuel: self.fuel.remaining(),
            });
        }
    }
}
