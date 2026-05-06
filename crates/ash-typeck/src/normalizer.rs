//! Type-expression normalizer API skeleton for Phase 112.
//!
//! This module intentionally provides only the total API surface and structural
//! identity conversion from `CanonicalTypeExpr` to `NormalTypeExpr`. It does not
//! own reduction semantics, definitional equality adoption, or associated-family
//! computation; later SPEC-060 tasks build those pieces on this boundary.

use std::collections::{BTreeMap, BTreeSet};

use crate::type_env::TypeEnv;
use ash_core::kind::Kind;
use ash_core::semantic_summary::{DomainConstructorId, SealedDomainId};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
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
    WeakHead,
    /// Recursively normalize the entire reachable expression.
    Full,
    /// Use-site selected demand-driven normalization.
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

/// Environment-aware normalizer.
pub struct Normalizer<'env> {
    _env: &'env TypeEnv,
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
            _env: env,
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

fn block_reason_for_normal(arg: &NormalTypeExpr) -> NormalFormBlockReason {
    match arg {
        NormalTypeExpr::Var(_) => NormalFormBlockReason::AbstractScrutinee,
        NormalTypeExpr::NeutralComputationApp { reason, .. } => reason
            .clone()
            .unwrap_or(NormalFormBlockReason::NeutralScrutinee),
        NormalTypeExpr::Projection {
            rigidity, reason, ..
        } => reason.clone().unwrap_or(match rigidity {
            ProjectionRigidity::Rigid => NormalFormBlockReason::RigidProjection,
            ProjectionRigidity::Neutral => NormalFormBlockReason::AbstractScrutinee,
        }),
        _ => NormalFormBlockReason::Unsupported,
    }
}

struct NormalizationState<'env> {
    mode: NormalizationMode,
    fuel: NormalizationFuel,
    trace_enabled: bool,
    trace: Vec<NormalizationTraceEvent>,
    fixture_registry: &'env FixtureEquationRegistry,
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
            } => (
                NormalTypeExpr::NominalApp {
                    origin: origin.clone(),
                    visible_name: visible_name.clone(),
                    args: self.normalize_args(args)?,
                    kind: kind.clone(),
                },
                NormalizationEvidence::StructuralIdentity,
            ),
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
                    reason: Some(reason),
                },
                evidence: NormalizationEvidence::NeutralUnsupportedComputation,
            }),
            FixtureEquationSelection::NoMatch => Ok(ComputationReduction::Neutral {
                normal: NormalTypeExpr::NeutralComputationApp {
                    head: head.clone(),
                    args,
                    kind: kind.clone(),
                    reason: Some(NormalFormBlockReason::Unsupported),
                },
                evidence: NormalizationEvidence::NeutralUnsupportedComputation,
            }),
        }
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
